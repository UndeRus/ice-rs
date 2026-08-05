use crate::errors::*;
use crate::proxy_parser::*;
use crate::iceobject::*;
use crate::protocol::*;
use crate::encoding::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::{Notify, RwLock};

/// Ключ реестра servant'ов: полная Ice-идентичность плюс facet.
///
/// Раньше поиск шёл только по `req.id.name`, из-за чего `cat/name` и `name`
/// сталкивались, а facet игнорировался целиком.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ServantKey {
    pub category: String,
    pub name: String,
    pub facet: String,
}

impl ServantKey {
    /// Из строки идентичности (`name` либо `category/name`) без facet'а.
    pub fn new(ident: &str) -> ServantKey {
        let id = Identity::new(ident);
        ServantKey {
            category: id.category,
            name: id.name,
            facet: String::new(),
        }
    }

    pub fn with_facet(mut self, facet: &str) -> ServantKey {
        self.facet = String::from(facet);
        self
    }

    fn from_request(req: &RequestData) -> ServantKey {
        ServantKey {
            category: req.id.category.clone(),
            name: req.id.name.clone(),
            // Ice передаёт facet как StringSeq, но фактически это одно значение.
            facet: req.facet.first().cloned().unwrap_or_default(),
        }
    }

    /// Строка идентичности для сообщений об ошибках.
    #[allow(dead_code)]
    fn identity_string(&self) -> String {
        if self.category.is_empty() {
            self.name.clone()
        } else {
            format!("{}/{}", self.category, self.name)
        }
    }
}

type Registry = Arc<RwLock<HashMap<ServantKey, Arc<dyn Servant>>>>;

pub struct Adapter {
    endpoint: DirectProxyData,
    /// Адрес, который объявляется пирам. Отличается от адреса прослушивания под
    /// NAT/Docker: Murmur звонит на него наружу, поэтому `0.0.0.0` объявлять
    /// нельзя.
    advertise: Option<(String, i32)>,
    servants: Registry,
    shutdown: Arc<Notify>,
}

/// Работающий адаптер: слушает в отдельном таске, умеет гаситься.
pub struct AdapterHandle {
    local_addr: std::net::SocketAddr,
    shutdown: Arc<Notify>,
    join: tokio::task::JoinHandle<()>,
}

impl AdapterHandle {
    /// Реальный адрес прослушивания. Нужен, когда порт запрошен как `0`.
    pub fn local_addr(&self) -> std::net::SocketAddr {
        self.local_addr
    }

    /// Просит accept-цикл остановиться и ждёт его завершения.
    pub async fn shutdown(self) {
        self.shutdown.notify_waiters();
        let _ = self.join.await;
    }
}

/// Reads one full Ice TCP frame (header + payload; length from `Header.message_size`).
async fn read_ice_frame(
    stream: &mut TcpStream,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
    let mut hdr = [0u8; 14];
    stream.read_exact(&mut hdr).await?;
    let mut hdr_done = 0i32;
    let header = Header::from_bytes(&hdr, &mut hdr_done)?;
    // Проверять ДО приведения к usize: отрицательный i32 превращается в ~1.8e19,
    // проходит проверку `< 14` и уходит в аллокацию.
    if header.message_size < 14 || header.message_size > crate::proxy::MAX_MESSAGE_SIZE {
        return Err(Box::new(ProtocolError::new(&format!(
            "Ice: implausible message_size {} (allowed 14..={})",
            header.message_size,
            crate::proxy::MAX_MESSAGE_SIZE
        ))));
    }
    let total = header.message_size as usize;
    let mut buf = vec![0u8; total];
    buf[..14].copy_from_slice(&hdr);
    if total > 14 {
        stream.read_exact(&mut buf[14..]).await?;
    }
    Ok(buf)
}

impl Adapter {
    pub fn with_endpoint(name: &str, endpoint: &str) -> Result<Adapter, Box<dyn std::error::Error + Sync + Send>> {
        let endpoint = parse_proxy_string(&format!("{}:{}", name, endpoint))?;
        let endpoint = match endpoint {
            ProxyStringType::DirectProxy(endpoint) => {
                endpoint
            }
            _ => {
                return Err(Box::new(ProtocolError::new("Direct proxy required for endpoint")))
            }
        };

        Ok(Adapter{
            endpoint,
            advertise: None,
            servants: Arc::new(RwLock::new(HashMap::new())),
            shutdown: Arc::new(Notify::new()),
        })
    }

    /// Задаёт адрес, который попадёт в прокси, отдаваемые пирам.
    ///
    /// Обязателен, если слушаем на `0.0.0.0`: объявить wildcard нельзя — пир не
    /// сможет по нему дозвониться.
    pub fn advertise(&mut self, host: &str, port: i32) -> &mut Self {
        self.advertise = Some((String::from(host), port));
        self
    }

    /// Регистрирует servant (новый интерфейс с `&self`).
    pub async fn register(&self, key: ServantKey, servant: Arc<dyn Servant>) {
        self.servants.write().await.insert(key, servant);
    }

    /// Регистрирует servant под простой идентичностью, без facet'а.
    ///
    /// Сгенерированный `*Server` превращается в `Arc<dyn Servant>` методом
    /// `into_servant()`:
    /// ```ignore
    /// adapter.add("hello", HelloServer::new(Box::new(HelloImpl {})).into_servant());
    /// ```
    pub fn add(&mut self, ident: &str, servant: Arc<dyn Servant>) -> &mut Self {
        self.register_blocking(ServantKey::new(ident), servant)
    }

    /// Синхронная регистрация до запуска — удобно в конструкторах.
    pub fn register_blocking(&mut self, key: ServantKey, servant: Arc<dyn Servant>) -> &mut Self {
        // До `serve()` реестр ни с кем не разделён, так что блокирующий захват
        // здесь не может ни с чем состязаться.
        self.servants
            .try_write()
            .expect("registry is not shared before serve()")
            .insert(key, servant);
        self
    }

    pub async fn unregister(&self, key: &ServantKey) -> Option<Arc<dyn Servant>> {
        self.servants.write().await.remove(key)
    }

    /// Прокси-байты для локального объекта в кодировке Ice — то, что отдаётся
    /// пиру, чтобы он смог позвонить нам обратно.
    pub fn proxy_bytes(&self, ident: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        let (host, port, secure) = self.advertised_target()?;
        crate::protocol::marshal_proxy_ref(ident, &host, port, secure)
    }

    /// Прокси-строка для локального объекта (`ident:tcp -h host -p port`).
    pub fn proxy_string(&self, ident: &str) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let (host, port, secure) = self.advertised_target()?;
        Ok(format!(
            "{}:{} -h {} -p {}",
            ident,
            if secure { "ssl" } else { "tcp" },
            host,
            port
        ))
    }

    fn advertised_target(&self) -> Result<(String, i32, bool), Box<dyn std::error::Error + Sync + Send>> {
        let (bind_host, bind_port, secure) = match &self.endpoint.endpoint {
            EndPointType::TCP(d) => (d.host.clone(), d.port, false),
            EndPointType::SSL(d) => (d.host.clone(), d.port, true),
            _ => {
                return Err(Box::new(ProtocolError::new(
                    "Direct proxy required for endpoint",
                )))
            }
        };
        if let Some((host, port)) = &self.advertise {
            return Ok((host.clone(), *port, secure));
        }
        // Wildcard объявлять нельзя — пир по нему не дозвонится. Раньше такой
        // адрес молча уезжал в прокси, и колбеки просто не приходили.
        if bind_host == "0.0.0.0" || bind_host == "::" || bind_host.is_empty() {
            return Err(Box::new(ProtocolError::new(
                "Listening on a wildcard address: call advertise(host, port) with an address the peer can reach",
            )));
        }
        Ok((bind_host, bind_port, secure))
    }

    /// Привязывается к эндпоинту, поднимает accept-цикл в отдельном таске и
    /// возвращает хендл.
    ///
    /// В отличие от [`Adapter::activate`] не блокируется и позволяет погасить
    /// адаптер.
    pub async fn serve(&self) -> Result<AdapterHandle, Box<dyn std::error::Error + Sync + Send>> {
        let listener = self.bind().await?;
        let local_addr = listener.local_addr()?;
        let servants = self.servants.clone();
        let shutdown = self.shutdown.clone();
        let stop = shutdown.clone();
        let join = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = stop.notified() => return,
                    accepted = listener.accept() => {
                        match accepted {
                            Ok((socket, _peer)) => {
                                // Таск на соединение: медленный или сломанный пир
                                // больше не задерживает остальных и не роняет
                                // listener.
                                let servants = servants.clone();
                                tokio::spawn(async move {
                                    let mut socket = socket;
                                    let _ = Adapter::serve_connection(&servants, &mut socket).await;
                                });
                            }
                            // Ошибка accept'а (например, исчерпание дескрипторов)
                            // раньше выходила наружу и убивала весь listener.
                            Err(_) => continue,
                        }
                    }
                }
            }
        });
        Ok(AdapterHandle {
            local_addr,
            shutdown,
            join,
        })
    }

    async fn bind(&self) -> Result<TcpListener, Box<dyn std::error::Error + Sync + Send>> {
        match &self.endpoint.endpoint {
            EndPointType::TCP(data) => {
                Ok(TcpListener::bind(format!("{}:{}", data.host, data.port)).await?)
            }
            EndPointType::SSL(_) => Err(Box::new(ProtocolError::new(
                "SSL object adapter is not implemented",
            ))),
            _ => Err(Box::new(ProtocolError::new(
                "Direct proxy required for endpoint",
            ))),
        }
    }

    /// Обслуживает адаптер до остановки. Не возвращается при нормальной работе.
    pub async fn activate(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let handle = self.serve().await?;
        let _ = handle.join.await;
        Ok(())
    }

    /// Обслуживает одно уже принятое соединение.
    ///
    /// Берёт `&self`, поэтому одним адаптером можно обслуживать сколько угодно
    /// соединений одновременно.
    pub async fn handle_socket(&self, stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        Adapter::serve_connection(&self.servants, stream).await
    }

    async fn serve_connection(
        servants: &Registry,
        stream: &mut TcpStream,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        // Ice требует, чтобы серверная сторона поздоровалась первой. Если этого
        // не сделать сразу, Murmur присылает CloseConnection и рвёт соединение.
        let hello = Header::new(3, 14).to_bytes()?;
        stream.write_all(&hello).await?;
        stream.flush().await?;

        loop {
            let buffer = read_ice_frame(stream).await?;
            let mut read = 0;
            let header = Header::from_bytes(&buffer, &mut read)?;
            match header.message_type {
                0 => {
                    let mut req_read = 0i32;
                    let req = match RequestData::from_bytes(&buffer[read as usize..], &mut req_read) {
                        Ok(req) => req,
                        // Битый запрос роняет только этот кадр: соединение может
                        // нести другие валидные запросы.
                        Err(_) => continue,
                    };

                    // `request_id == 0` — oneway: ответа не ждут и присылать его
                    // нельзя. Murmur рассылает колбеки именно так (в его логе
                    // прокси помечен `-o`).
                    let oneway = req.request_id == 0;

                    let key = ServantKey::from_request(&req);
                    let servant = servants.read().await.get(&key).cloned();
                    let outcome = match servant {
                        Some(servant) => Adapter::dispatch_to(servant, &req).await,
                        None => {
                            // Различаем «нет объекта» и «нет facet'а»: если такая
                            // идентичность есть с другим facet'ом, это status 3.
                            let facet_mismatch = {
                                let guard = servants.read().await;
                                guard
                                    .keys()
                                    .any(|k| k.category == key.category && k.name == key.name)
                            };
                            if facet_mismatch {
                                DispatchResult::FacetNotExist
                            } else {
                                DispatchResult::ObjectNotExist
                            }
                        }
                    };

                    if !oneway {
                        let reply = encode_reply(&req, &key, outcome)?;
                        stream.write_all(&reply).await?;
                        stream.flush().await?;
                    }
                }
                4 => return Ok(()),
                // ValidateConnection от пира и прочие служебные кадры просто
                // игнорируем: раньше любой неожидаемый тип убивал соединение.
                _ => continue,
            }
        }
    }

    async fn dispatch_to(servant: Arc<dyn Servant>, req: &RequestData) -> DispatchResult {
        // Операции `::Ice::Object` закрываем централизованно, чтобы каждый
        // servant не переписывал их заново (и не отвечал на `ice_isA` только
        // собственным type-id, игнорируя наследование).
        match req.operation.as_str() {
            "ice_ping" => return DispatchResult::Ok(Encapsulation::empty()),
            "ice_id" => {
                return match servant.type_id().to_bytes() {
                    Ok(b) => DispatchResult::Ok(Encapsulation::from(b)),
                    Err(e) => DispatchResult::Failed(e.to_string()),
                }
            }
            "ice_ids" => {
                return match servant.type_ids().to_bytes() {
                    Ok(b) => DispatchResult::Ok(Encapsulation::from(b)),
                    Err(e) => DispatchResult::Failed(e.to_string()),
                }
            }
            "ice_isA" => {
                let buf = crate::protocol::peel_slice_param_payload(&req.params.data);
                let mut read = 0;
                return match String::from_bytes(&buf, &mut read) {
                    Ok(param) => match servant.is_a(&param).to_bytes() {
                        Ok(b) => DispatchResult::Ok(Encapsulation::from(b)),
                        Err(e) => DispatchResult::Failed(e.to_string()),
                    },
                    Err(e) => DispatchResult::Failed(e.to_string()),
                };
            }
            _ => {}
        }
        servant.dispatch(req).await
    }
}

/// Собирает байты ответа Ice для исхода диспатча.
///
/// Раньше все ошибки уезжали как `status: 1` с сырой UTF-8 строкой в теле — это
/// не валидное Ice-исключение, и настоящий пир на таком ответе рвал соединение
/// и молча снимал регистрацию колбека.
fn encode_reply(
    req: &RequestData,
    key: &ServantKey,
    outcome: DispatchResult,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
    use crate::protocol::reply_status as st;

    let (status, body): (u8, Vec<u8>) = match outcome {
        DispatchResult::Ok(enc) => (st::OK, enc.to_bytes()?),
        DispatchResult::UserException { type_id, body } => {
            let mut payload = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            }
            .to_bytes()?;
            payload.extend(type_id.to_bytes()?);
            payload.extend(body);
            (st::USER_EXCEPTION, Encapsulation::from(payload).to_bytes()?)
        }
        DispatchResult::ObjectNotExist => (
            st::OBJECT_NOT_EXIST,
            request_failed_body(req, key)?,
        ),
        DispatchResult::FacetNotExist => (st::FACET_NOT_EXIST, request_failed_body(req, key)?),
        DispatchResult::OperationNotExist => (
            st::OPERATION_NOT_EXIST,
            request_failed_body(req, key)?,
        ),
        DispatchResult::Failed(msg) => (st::UNKNOWN_LOCAL_EXCEPTION, msg.to_bytes()?),
    };

    // request_id (4) + status (1) + тело
    let mut payload = req.request_id.to_bytes()?;
    payload.extend(status.to_bytes()?);
    payload.extend(body);

    let mut out = Header::new(2, 14 + payload.len() as i32).to_bytes()?;
    out.extend(payload);
    Ok(out)
}

/// Тело Ice `RequestFailedException`: Identity, StringSeq facet, string operation.
fn request_failed_body(
    req: &RequestData,
    key: &ServantKey,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
    let mut out = Identity {
        name: key.name.clone(),
        category: key.category.clone(),
    }
    .to_bytes()?;
    out.extend(req.facet.to_bytes()?);
    out.extend(req.operation.to_bytes()?);
    Ok(out)
}

/// Мост от старого `IceObjectServer` (с `&mut self`) к новому [`Servant`].
///
/// Сгенерированный код пока реализует старый интерфейс; обёртка держит свой
/// мьютекс, поэтому разные servant'ы больше не состязаются друг с другом — а
/// именно это и подвешивало доставку колбеков.
pub struct LegacyServant {
    inner: tokio::sync::Mutex<Box<dyn IceObjectServer + Send + Sync>>,
    type_ids: Vec<String>,
}

impl LegacyServant {
    pub fn new(
        inner: Box<dyn IceObjectServer + Send + Sync>,
        type_ids: Vec<String>,
    ) -> Arc<LegacyServant> {
        Arc::new(LegacyServant {
            inner: tokio::sync::Mutex::new(inner),
            type_ids,
        })
    }
}

#[async_trait::async_trait]
impl Servant for LegacyServant {
    fn type_ids(&self) -> Vec<String> {
        self.type_ids.clone()
    }


    async fn dispatch(&self, request: &RequestData) -> DispatchResult {
        let mut guard = self.inner.lock().await;
        match guard.handle_request(request).await {
            Ok(reply) => match reply.status {
                0 => DispatchResult::Ok(reply.body),
                _ => DispatchResult::Failed(format!(
                    "legacy servant returned status {}",
                    reply.status
                )),
            },
            Err(e) => {
                let msg = e.to_string();
                // Старый сгенерированный код сообщает «операции нет» только
                // текстом ошибки.
                if msg.contains("Operation not found") {
                    DispatchResult::OperationNotExist
                } else {
                    DispatchResult::Failed(msg)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Echo {
        ids: Vec<String>,
    }

    #[async_trait::async_trait]
    impl Servant for Echo {
        fn type_ids(&self) -> Vec<String> {
            self.ids.clone()
        }
        async fn dispatch(&self, request: &RequestData) -> DispatchResult {
            match request.operation.as_str() {
                "ok" => DispatchResult::Ok(Encapsulation::empty()),
                "boom" => DispatchResult::UserException {
                    type_id: String::from("::Demo::BoomException"),
                    body: vec![1, 2, 3],
                },
                _ => DispatchResult::OperationNotExist,
            }
        }
    }

    fn req(ident: &str, facet: Vec<String>, op: &str, request_id: i32) -> RequestData {
        let id = Identity::new(ident);
        RequestData {
            request_id,
            id,
            facet,
            operation: String::from(op),
            mode: 0,
            context: std::collections::HashMap::new(),
            params: Encapsulation::empty(),
        }
    }

    fn decode_status(bytes: &[u8]) -> (i32, u8) {
        // header(14) + request_id(4) + status(1)
        let mut r = 0i32;
        let request_id = i32::from_bytes(&bytes[14..18], &mut r).unwrap();
        (request_id, bytes[18])
    }

    /// Раньше «объекта нет» уезжало статусом 1 с сырой строкой в теле, что не
    /// является валидным Ice-исключением.
    #[test]
    fn object_not_exist_uses_status_two_with_request_failed_body() {
        let r = req("nope", vec![], "ok", 5);
        let key = ServantKey::from_request(&r);
        let bytes = encode_reply(&r, &key, DispatchResult::ObjectNotExist).unwrap();
        let (id, status) = decode_status(&bytes);
        assert_eq!(5, id);
        assert_eq!(reply_status::OBJECT_NOT_EXIST, status);
        // Тело: Identity, StringSeq facet, string operation.
        let mut rd = 0i32;
        let ident = Identity::from_bytes(&bytes[19..], &mut rd).unwrap();
        assert_eq!("nope", ident.name);
        let facet = Vec::<String>::from_bytes(&bytes[19 + rd as usize..], &mut rd).unwrap();
        assert!(facet.is_empty());
        let op = String::from_bytes(&bytes[19 + rd as usize..], &mut rd).unwrap();
        assert_eq!("ok", op);
    }

    #[test]
    fn operation_not_exist_uses_status_four() {
        let r = req("obj", vec![], "missing", 9);
        let key = ServantKey::from_request(&r);
        let bytes = encode_reply(&r, &key, DispatchResult::OperationNotExist).unwrap();
        assert_eq!(
            (9, reply_status::OPERATION_NOT_EXIST),
            decode_status(&bytes)
        );
    }

    #[test]
    fn facet_not_exist_uses_status_three() {
        let r = req("obj", vec![String::from("f")], "ok", 1);
        let key = ServantKey::from_request(&r);
        let bytes = encode_reply(&r, &key, DispatchResult::FacetNotExist).unwrap();
        assert_eq!((1, reply_status::FACET_NOT_EXIST), decode_status(&bytes));
    }

    /// Пользовательское исключение должно уезжать как SliceFlags + type-id +
    /// члены внутри Encapsulation — ровно то, что читает `Proxy::interpret_reply`.
    #[test]
    fn user_exception_round_trips_through_client_decoder() {
        let r = req("obj", vec![], "boom", 3);
        let key = ServantKey::from_request(&r);
        let bytes = encode_reply(
            &r,
            &key,
            DispatchResult::UserException {
                type_id: String::from("::Demo::BoomException"),
                body: vec![1, 2, 3],
            },
        )
        .unwrap();
        assert_eq!((3, reply_status::USER_EXCEPTION), decode_status(&bytes));

        let mut rd = 0i32;
        let raw = RawReply::from_bytes(&bytes[14..], &mut rd).unwrap();
        let err = crate::proxy::Proxy::interpret_reply(raw).unwrap_err();
        let ux = err
            .downcast_ref::<RemoteUserException>()
            .expect("должно быть RemoteUserException");
        assert_eq!("::Demo::BoomException", ux.type_id);
        assert_eq!("BoomException", ux.short_name());
        assert_eq!(vec![1u8, 2, 3], ux.payload);
    }

    #[test]
    fn internal_failure_uses_status_five() {
        let r = req("obj", vec![], "ok", 2);
        let key = ServantKey::from_request(&r);
        let bytes = encode_reply(&r, &key, DispatchResult::Failed(String::from("nope"))).unwrap();
        assert_eq!(
            (2, reply_status::UNKNOWN_LOCAL_EXCEPTION),
            decode_status(&bytes)
        );
    }

    /// Реестр ключуется по полной идентичности: `cat/name` и `name` — разные
    /// объекты. Раньше поиск шёл только по `name` и они сталкивались.
    #[test]
    fn registry_distinguishes_category_and_facet() {
        let plain = ServantKey::new("obj");
        let with_cat = ServantKey::new("cat/obj");
        let with_facet = ServantKey::new("obj").with_facet("f");
        assert_ne!(plain, with_cat);
        assert_ne!(plain, with_facet);
        assert_eq!("cat/obj", with_cat.identity_string());
        assert_eq!("obj", plain.identity_string());
    }

    /// `ice_isA` должен отвечать по всей цепочке type-id, а не только по
    /// собственному типу — иначе `checkedCast` к базовому интерфейсу провалится.
    #[tokio::test]
    async fn ice_is_a_honours_the_whole_type_id_chain() {
        let servant: Arc<dyn Servant> = Arc::new(Echo {
            ids: vec![
                String::from("::Demo::Derived"),
                String::from("::Demo::Base"),
                String::from("::Ice::Object"),
            ],
        });
        assert!(servant.is_a("::Demo::Derived"));
        assert!(servant.is_a("::Demo::Base"));
        assert!(!servant.is_a("::Demo::Other"));
        assert_eq!("::Demo::Derived", servant.type_id());

        // И через диспатч адаптера.
        let mut r = req("obj", vec![], "ice_isA", 1);
        r.params = Encapsulation::from(String::from("::Demo::Base").to_bytes().unwrap());
        match Adapter::dispatch_to(servant.clone(), &r).await {
            DispatchResult::Ok(enc) => {
                let mut rd = 0i32;
                assert!(bool::from_bytes(&enc.data, &mut rd).unwrap());
            }
            other => panic!("ожидали Ok, получили {:?}", other),
        }
    }

    #[tokio::test]
    async fn ice_ping_and_ice_ids_are_handled_centrally() {
        let servant: Arc<dyn Servant> = Arc::new(Echo {
            ids: vec![String::from("::Demo::X"), String::from("::Ice::Object")],
        });
        // Echo::dispatch не знает про ice_ping — их закрывает адаптер.
        match Adapter::dispatch_to(servant.clone(), &req("obj", vec![], "ice_ping", 1)).await {
            DispatchResult::Ok(_) => {}
            other => panic!("ice_ping: {:?}", other),
        }
        match Adapter::dispatch_to(servant, &req("obj", vec![], "ice_ids", 1)).await {
            DispatchResult::Ok(enc) => {
                let mut rd = 0i32;
                let ids = Vec::<String>::from_bytes(&enc.data, &mut rd).unwrap();
                assert_eq!(vec!["::Demo::X", "::Ice::Object"], ids);
            }
            other => panic!("ice_ids: {:?}", other),
        }
    }

    /// Объявлять wildcard-адрес нельзя: пир по нему не дозвонится. Раньше он
    /// молча уезжал в прокси, и колбеки просто не приходили.
    #[test]
    fn wildcard_bind_requires_explicit_advertise() {
        let a = Adapter::with_endpoint("cb", "tcp -h 0.0.0.0 -p 7100").unwrap();
        let err = a.proxy_string("cb").unwrap_err().to_string();
        assert!(err.contains("advertise"), "неинформативная ошибка: {}", err);

        let mut a = Adapter::with_endpoint("cb", "tcp -h 0.0.0.0 -p 7100").unwrap();
        a.advertise("bot.internal", 7100);
        assert_eq!(
            "cb:tcp -h bot.internal -p 7100",
            a.proxy_string("cb").unwrap()
        );
        // И байты прокси должны собираться под объявленный адрес.
        let want = crate::protocol::marshal_proxy_ref("cb", "bot.internal", 7100, false).unwrap();
        assert_eq!(want, a.proxy_bytes("cb").unwrap());
    }

    /// `serve()` не блокируется, отдаёт реальный порт при запросе `0` и гасится.
    /// Раньше `activate()` был бесконечным циклом без пути выхода.
    #[tokio::test]
    async fn serve_resolves_ephemeral_port_and_shuts_down() {
        let mut adapter = Adapter::with_endpoint("obj", "tcp -h 127.0.0.1 -p 0").unwrap();
        adapter.add(
            "obj",
            Arc::new(Echo {
                ids: vec![String::from("::Demo::X"), String::from("::Ice::Object")],
            }),
        );
        let handle = adapter.serve().await.expect("serve");
        let addr = handle.local_addr();
        assert_ne!(0, addr.port(), "порт 0 должен разрешиться в настоящий");

        // Соединение принимается и получает ValidateConnection.
        let mut sock = tokio::net::TcpStream::connect(addr).await.expect("connect");
        let mut hdr = [0u8; 14];
        tokio::io::AsyncReadExt::read_exact(&mut sock, &mut hdr)
            .await
            .expect("ValidateConnection");
        let mut r = 0i32;
        let h = Header::from_bytes(&hdr, &mut r).unwrap();
        assert_eq!(3, h.message_type, "серверная сторона здоровается первой");

        handle.shutdown().await;
    }

    /// Два соединения обслуживаются одновременно одним адаптером. До S3 это было
    /// невозможно: `handle_socket` держал `&mut self` на всё время соединения.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn two_connections_are_served_concurrently() {
        let mut adapter = Adapter::with_endpoint("obj", "tcp -h 127.0.0.1 -p 0").unwrap();
        adapter.add(
            "obj",
            Arc::new(Echo {
                ids: vec![String::from("::Demo::X"), String::from("::Ice::Object")],
            }),
        );
        let handle = adapter.serve().await.expect("serve");
        let addr = handle.local_addr();

        // Первое соединение открываем и НЕ закрываем — раньше именно это
        // навсегда занимало адаптер.
        let mut first = tokio::net::TcpStream::connect(addr).await.expect("first");
        let mut hdr = [0u8; 14];
        tokio::io::AsyncReadExt::read_exact(&mut first, &mut hdr)
            .await
            .expect("hello 1");

        // Второе должно обслужиться, пока первое живо.
        let mut second = tokio::net::TcpStream::connect(addr).await.expect("second");
        let mut hdr2 = [0u8; 14];
        tokio::time::timeout(
            std::time::Duration::from_secs(3),
            tokio::io::AsyncReadExt::read_exact(&mut second, &mut hdr2),
        )
        .await
        .expect("второе соединение голодает: адаптер сериализован")
        .expect("hello 2");

        handle.shutdown().await;
    }

    #[test]
    fn concrete_bind_address_needs_no_advertise() {
        let a = Adapter::with_endpoint("cb", "tcp -h 127.0.0.1 -p 7100").unwrap();
        assert_eq!("cb:tcp -h 127.0.0.1 -p 7100", a.proxy_string("cb").unwrap());
    }
}
