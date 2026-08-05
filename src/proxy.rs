use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use pest_derive::Parser;

use crate::connection::Connection;
use crate::encoding::{FromBytes, IceSize, SliceFlags, SliceFlagsTypeEncoding, ToBytes};
use crate::errors::{
    ProtocolError, RemoteException, RemoteUserException, RequestFailedException, RequestFailedKind,
};
use crate::protocol::{
    Encapsulation, EndPointType, EndpointData, Identity, ProxyData, RawReply, ReplyData,
    RequestData, Version,
};

#[derive(Parser)]
#[grammar = "proxystring.pest"]
pub struct ProxyParser;

/// Верхняя граница размера кадра. Оставлено здесь для совместимости; живёт в
/// [`crate::connection`].
pub use crate::connection::MAX_MESSAGE_SIZE;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Куда ведёт прокси.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Target {
    pub ident: String,
    pub host: String,
    pub port: i32,
    pub secure: bool,
}

impl Target {
    pub fn proxy_string(&self) -> String {
        format!(
            "{}:{} -h {} -p {}",
            self.ident,
                if self.secure { "ssl" } else { "tcp" },
            self.host,
            self.port
        )
    }
}

/// Кэш соединений: несколько прокси на один и тот же эндпоинт делят сокет.
///
/// `Weak`, чтобы мёртвые соединения не удерживались. Кэш глобальный сознательно:
/// это пул ресурсов, а не конфигурация, и без него `FromBytes for Proxy` не смог
/// бы обойтись без дозвона (см. ниже).
mod cache {
    use super::*;
    use std::sync::Weak;

    lazy_static::lazy_static! {
        static ref CONNECTIONS: std::sync::Mutex<HashMap<ConnKey, Weak<Connection>>> =
            std::sync::Mutex::new(HashMap::new());
    }

    #[derive(PartialEq, Eq, Hash, Clone)]
    pub(super) struct ConnKey {
        host: String,
        port: i32,
        secure: bool,
    }

    impl From<&Target> for ConnKey {
        fn from(t: &Target) -> ConnKey {
            ConnKey {
                host: t.host.clone(),
                port: t.port,
                secure: t.secure,
            }
        }
    }

    pub(super) fn get(target: &Target) -> Option<Arc<Connection>> {
        let key = ConnKey::from(target);
        let guard = CONNECTIONS.lock().unwrap();
        guard
            .get(&key)
            .and_then(|w| w.upgrade())
            .filter(|c| c.is_alive())
    }

    pub(super) fn put(target: &Target, conn: &Arc<Connection>) {
        let mut guard = CONNECTIONS.lock().unwrap();
        guard.retain(|_, w| w.upgrade().is_some());
        guard.insert(ConnKey::from(target), Arc::downgrade(conn));
    }
}

/// Прокси на удалённый Ice-объект.
///
/// Дешёвый в клонировании и разделяемый: соединение живёт отдельно, за `Arc`.
/// Соединение открывается **лениво**, при первом вызове — поэтому построить
/// прокси можно синхронно, в том числе при десериализации.
#[derive(Clone)]
pub struct Proxy {
    target: Target,
    /// Разрешённое соединение. `None`, пока не позвали.
    conn: Arc<tokio::sync::Mutex<Option<Arc<Connection>>>>,
    pub context: Option<HashMap<String, String>>,
    timeout: Duration,
}

impl Proxy {
    /// Прокси на уже открытое соединение.
    pub fn with_connection(conn: Arc<Connection>, target: Target, context: Option<HashMap<String, String>>) -> Proxy {
        Proxy {
            target,
            conn: Arc::new(tokio::sync::Mutex::new(Some(conn))),
            context,
            timeout: Duration::from_secs(30),
        }
    }

    /// Прокси без соединения: откроется при первом вызове.
    ///
    /// Именно это позволило убрать `futures::executor::block_on` с TCP-дозвоном
    /// из `FromBytes` — там дозвон блокировал реактор, которого сам же и ждал,
    /// а внутри обработчика колбека делал сетевой вызов, пока пир ждёт ответа.
    pub fn unresolved(target: Target, context: Option<HashMap<String, String>>) -> Proxy {
        Proxy {
            target,
            conn: Arc::new(tokio::sync::Mutex::new(None)),
            context,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn target(&self) -> &Target {
        &self.target
    }

    pub fn ident(&self) -> &str {
        &self.target.ident
    }

    pub fn host(&self) -> &str {
        &self.target.host
    }

    pub fn port(&self) -> i32 {
        self.target.port
    }

    pub fn is_secure(&self) -> bool {
        self.target.secure
    }

    /// Таймаут одного вызова.
    pub fn set_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Копия прокси с другим контекстом.
    ///
    /// Раньше это открывало **новое соединение** только чтобы приложить
    /// контекст; теперь соединение то же.
    pub fn with_context(&self, context: HashMap<String, String>) -> Proxy {
        Proxy {
            target: self.target.clone(),
            conn: self.conn.clone(),
            context: Some(context),
            timeout: self.timeout,
        }
    }

    /// Тот же объект, но другая идентичность на том же эндпоинте.
    pub fn with_ident(&self, ident: &str) -> Proxy {
        Proxy {
            target: Target {
                ident: String::from(ident),
                ..self.target.clone()
            },
            conn: self.conn.clone(),
            context: self.context.clone(),
            timeout: self.timeout,
        }
    }

    /// Возвращает соединение, открывая его при необходимости.
    pub async fn connection(&self) -> Result<Arc<Connection>, BoxError> {
        let mut guard = self.conn.lock().await;
        if let Some(c) = guard.as_ref() {
            if c.is_alive() {
                return Ok(c.clone());
            }
        }
        // Переиспользуем сокет, если его уже открыл другой прокси.
        if let Some(c) = cache::get(&self.target) {
            *guard = Some(c.clone());
            return Ok(c);
        }
        let c = self.dial().await?;
        cache::put(&self.target, &c);
        *guard = Some(c.clone());
        Ok(c)
    }

    async fn dial(&self) -> Result<Arc<Connection>, BoxError> {
        let props = {
            let g = crate::communicator::INITDATA
                .lock()
                .map_err(|_| ProtocolError::new("Ice properties poisoned by a panic"))?;
            g.properties().clone()
        };
        let addr = format!("{}:{}", self.target.host, self.target.port);
        let stream: Box<dyn crate::transport::Transport + Send + Sync + Unpin> =
            if self.target.secure {
                Box::new(crate::ssl::SslTransport::new(&addr, &props).await?)
            } else {
                Box::new(crate::tcp::TcpTransport::new(&addr).await?)
            };
        Connection::connect(stream, &addr).await
    }

    /// Выполняет вызов.
    ///
    /// Берёт `&self`: несколько вызовов могут идти по одному соединению
    /// одновременно. Тип-параметр остался для совместимости со сгенерированным
    /// кодом и не используется — тип исключения приходит с провода.
    pub async fn dispatch<T: 'static + std::fmt::Debug + std::fmt::Display + FromBytes + Send + Sync>(
        &self,
        op: &str,
        mode: u8,
        params: &Encapsulation,
        context: Option<HashMap<String, String>>,
    ) -> Result<ReplyData, BoxError> {
        let conn = self.connection().await?;
        let request = RequestData {
            request_id: conn.next_request_id(),
            id: Identity::new(&self.target.ident),
            facet: Vec::new(),
            operation: String::from(op),
            mode,
            context: context
                .or_else(|| self.context.clone())
                .unwrap_or_default(),
            params: params.clone(),
        };
        let raw = conn.invoke(&request, self.timeout).await?;
        Self::interpret_reply(raw)
    }

    /// Превращает сырой ответ в результат либо в типизированную ошибку.
    ///
    /// Раньше статусы 2..=7 возвращали `Err` из декодера **внутри** reader-таска
    /// и убивали соединение, а статус 1 разворачивался в единственный тип,
    /// захардкоженный кодогеном для операции: прочитанный Slice type-id
    /// выбрасывался, и два разных исключения Murmur'а были неразличимы.
    pub fn interpret_reply(reply: RawReply) -> Result<ReplyData, BoxError> {
        use crate::protocol::reply_status as st;
        match reply.status {
            st::OK => {
                let mut read = 0;
                let body = Encapsulation::from_bytes(&reply.payload, &mut read)?;
                Ok(ReplyData {
                    request_id: reply.request_id,
                    status: reply.status,
                    body,
                })
            }
            st::USER_EXCEPTION => {
                let mut read = 0;
                let body = Encapsulation::from_bytes(&reply.payload, &mut read)?;
                // Исключение в кодировке 1.1 — это цепочка слайсов от самого
                // производного к базовому; у каждого свой байт SliceFlags, а
                // последний помечен FLAG_IS_LAST_SLICE.
                //
                // Type-id читается как строка ВСЕГДА, не глядя на биты типа во
                // флагах. Проверено дампом ответа настоящего Murmur:
                //   00 27 "::MumbleServer::InvalidChannelException"
                //   20 1f "::MumbleServer::ServerException"
                // то есть биты типа нулевые (NoTypeId), а строка при этом есть.
                // Ветвление по битам давало type_id == "<NoTypeId>" и ломало
                // определение того, какое именно исключение приехало.
                let mut r = 0i32;
                let flags = SliceFlags::from_bytes(&body.data, &mut r)?;
                let type_id = match flags.type_id {
                    SliceFlagsTypeEncoding::CompactTypeId | SliceFlagsTypeEncoding::IndexTypeId => {
                        // Компактный/индексный id: сохраняем как число —
                        // сопоставить с именем без реестра типов нельзя.
                        let id = IceSize::from_bytes(&body.data[r as usize..], &mut r)?;
                        format!("<compact:{}>", id.size)
                    }
                    _ => String::from_bytes(&body.data[r as usize..], &mut r)?,
                };
                Err(Box::new(RemoteUserException {
                    type_id,
                    payload: body.data[r as usize..].to_vec(),
                }))
            }
            st::OBJECT_NOT_EXIST | st::FACET_NOT_EXIST | st::OPERATION_NOT_EXIST => {
                let kind = match reply.status {
                    st::OBJECT_NOT_EXIST => RequestFailedKind::ObjectNotExist,
                    st::FACET_NOT_EXIST => RequestFailedKind::FacetNotExist,
                    _ => RequestFailedKind::OperationNotExist,
                };
                let mut r = 0i32;
                let identity = Identity::from_bytes(&reply.payload, &mut r)
                    .map(|i| {
                        if i.category.is_empty() {
                            i.name
                        } else {
                            format!("{}/{}", i.category, i.name)
                        }
                    })
                    .unwrap_or_default();
                let facet = Vec::<String>::from_bytes(&reply.payload[r as usize..], &mut r)
                    .unwrap_or_default();
                let operation =
                    String::from_bytes(&reply.payload[r as usize..], &mut r).unwrap_or_default();
                Err(Box::new(RequestFailedException {
                    kind,
                    identity,
                    facet,
                    operation,
                }))
            }
            st::UNKNOWN_LOCAL_EXCEPTION | st::UNKNOWN_USER_EXCEPTION | st::UNKNOWN_EXCEPTION => {
                let mut r = 0i32;
                let cause = String::from_bytes(&reply.payload, &mut r)
                    .unwrap_or_else(|_| format!("reply status {}", reply.status));
                Err(Box::new(RemoteException { cause }))
            }
            other => Err(Box::new(ProtocolError::new(&format!(
                "Unknown reply status {}",
                other
            )))),
        }
    }
}

impl std::fmt::Debug for Proxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Proxy")
            .field("target", &self.target.proxy_string())
            .finish()
    }
}

impl ToBytes for Proxy {
    fn to_bytes(&self) -> Result<Vec<u8>, BoxError> {
        let id = Identity::new(&self.target.ident);
        let proxy_data = ProxyData {
            name: id.name,
            category: id.category,
            facet: vec![],
            // Ice `Reference::Mode`: Twoway=0, Oneway=1, BatchOneway=2,
            // Datagram=3, BatchDatagram=4. Раньше здесь была захардкожена 2,
            // то есть каждый наш прокси представлялся batch-oneway.
            mode: 0,
            secure: self.target.secure,
            protocol: Version { major: 1, minor: 0 },
            encoding: Version { major: 1, minor: 1 },
        };
        let mut out = proxy_data.to_bytes()?;
        // Ice пишет здесь КОЛИЧЕСТВО эндпоинтов (`writeSize(endpoints.size())`),
        // а не длину их байтового представления. Раньше сюда попадала
        // `inner.len()`, и настоящий Ice читал «27 эндпоинтов», после чего
        // разбирал мусор — из-за этого любой прокси, отданный Murmur'у
        // (addCallback, setAuthenticator, addContextCallback), был непригоден.
        out.extend(IceSize { size: 1 }.to_bytes()?);
        let ep_type: i16 = if self.target.secure { 2 } else { 1 };
        out.extend(ep_type.to_bytes()?);
        let ep = EndpointData {
            host: self.target.host.clone(),
            port: self.target.port,
            timeout: -1,
            compress: false,
        };
        out.extend(Encapsulation::from(ep.to_bytes()?).to_bytes()?);
        Ok(out)
    }
}

impl FromBytes for Proxy {
    /// Разбирает прокси **без** открытия соединения.
    ///
    /// Раньше здесь был `futures::executor::block_on` с TCP-дозвоном прямо
    /// внутри десериализации: на current_thread-рантайме это дедлок (декодер
    /// блокирует реактор, которого ждёт), а внутри обработчика колбека —
    /// синхронный сетевой вызов, пока пир ждёт ответа. Теперь соединение
    /// открывается лениво, при первом обращении к прокси.
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, BoxError>
    where
        Self: Sized,
    {
        let mut read = 0i32;
        let proxy_data = ProxyData::from_bytes(bytes, &mut read)?;

        // Счётчик эндпоинтов. Раньше это поле читалось и выбрасывалось,
        // что и маскировало симметричный баг в `to_bytes`.
        let count = IceSize::from_bytes(&bytes[read as usize..], &mut read)?.size;
        if count <= 0 {
            // count == 0 означает косвенный (well-known) прокси: дальше идёт
            // строка adapter-id, которую надо резолвить через локатор.
            return Err(Box::new(ProtocolError::new(
                "Indirect proxy (0 endpoints) received; locator resolution is not supported here",
            )));
        }

        // Берём первый эндпоинт, который умеем, но разбираем все, чтобы курсор
        // остался согласованным для полей, идущих после прокси.
        let mut chosen: Option<EndPointType> = None;
        for _ in 0..count {
            let ep_disc = i16::from_bytes(&bytes[read as usize..], &mut read)?;
            let enc = Encapsulation::from_bytes(&bytes[read as usize..], &mut read)?;
            let mut er = 0i32;
            let parsed = match ep_disc {
                1 => Some(EndPointType::TCP(EndpointData::from_bytes(&enc.data, &mut er)?)),
                2 => Some(EndPointType::SSL(EndpointData::from_bytes(&enc.data, &mut er)?)),
                // Прочие транспорты (udp=3, ws=4, wss=5) пропускаем, а не роняем
                // весь прокси: Ice-пир вправе публиковать несколько эндпоинтов.
                _ => None,
            };
            if chosen.is_none() {
                chosen = parsed;
            }
        }

        let target = match chosen {
            Some(EndPointType::TCP(ep)) => Target {
                ident: proxy_data.identity_string(),
                host: ep.host,
                port: ep.port,
                secure: false,
            },
            Some(EndPointType::SSL(ep)) => Target {
                ident: proxy_data.identity_string(),
                host: ep.host,
                port: ep.port,
                secure: true,
            },
            _ => {
                return Err(Box::new(ProtocolError::new(
                    "Proxy carries no tcp/ssl endpoint we can use",
                )))
            }
        };

        *read_bytes += read;
        Ok(Proxy::unresolved(target, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> Target {
        Target {
            ident: String::from("Meta"),
            host: String::from("127.0.0.1"),
            port: 6502,
            secure: false,
        }
    }

    /// Прокси должен строиться синхронно, без дозвона: это то, что позволило
    /// убрать `block_on` из декодера.
    #[test]
    fn unresolved_proxy_needs_no_runtime() {
        let p = Proxy::unresolved(target(), None);
        assert_eq!("Meta", p.ident());
        assert_eq!("127.0.0.1", p.host());
        assert_eq!(6502, p.port());
        assert!(!p.is_secure());
    }

    /// Прокси круглым рейсом через провод: количество эндпоинтов, режим, адрес.
    #[test]
    fn proxy_round_trips_through_the_wire() {
        let p = Proxy::unresolved(target(), None);
        let bytes = p.to_bytes().unwrap();
        let mut read = 0i32;
        let back = Proxy::from_bytes(&bytes, &mut read).unwrap();
        assert_eq!(p.ident(), back.ident());
        assert_eq!(p.host(), back.host());
        assert_eq!(p.port(), back.port());
        assert_eq!(read as usize, bytes.len(), "курсор должен пройти всё");
    }

    #[test]
    fn ssl_flag_survives_the_round_trip() {
        let mut t = target();
        t.secure = true;
        let p = Proxy::unresolved(t, None);
        let bytes = p.to_bytes().unwrap();
        let mut read = 0i32;
        let back = Proxy::from_bytes(&bytes, &mut read).unwrap();
        assert!(back.is_secure());
    }

    /// Смена контекста не должна открывать новое соединение — раньше открывала.
    #[tokio::test]
    async fn with_context_shares_the_connection_slot() {
        let p = Proxy::unresolved(target(), None);
        let mut m = HashMap::new();
        m.insert(String::from("secret"), String::from("s"));
        let q = p.with_context(m);
        assert_eq!(p.host(), q.host());
        assert!(
            Arc::ptr_eq(&p.conn, &q.conn),
            "слот соединения должен быть тем же"
        );
        assert!(q.context.is_some());
    }

    #[test]
    fn with_ident_keeps_the_endpoint() {
        let p = Proxy::unresolved(target(), None);
        let q = p.with_ident("s1");
        assert_eq!("s1", q.ident());
        assert_eq!(p.host(), q.host());
        assert_eq!(p.port(), q.port());
    }

    #[test]
    fn target_renders_a_proxy_string() {
        assert_eq!("Meta:tcp -h 127.0.0.1 -p 6502", target().proxy_string());
        let mut t = target();
        t.secure = true;
        assert_eq!("Meta:ssl -h 127.0.0.1 -p 6502", t.proxy_string());
    }

    /// Murmur присылает строковый type-id, но с НУЛЕВЫМИ битами типа во флагах
    /// слайса. Ветвление по битам давало `<NoTypeId>` и теряло тип исключения.
    ///
    /// Регрессия: этот обход однажды был потерян при переписывании слоя.
    /// Байты взяты из дампа настоящего ответа Murmur.
    #[test]
    fn exception_type_id_is_read_even_with_zero_type_bits() {
        let type_id = "::MumbleServer::InvalidChannelException";
        // flags = 0x00: last_slice не выставлен, биты типа нулевые.
        let mut slice_body = vec![0x00u8];
        slice_body.extend(String::from(type_id).to_bytes().unwrap());
        slice_body.extend([1u8, 2, 3]);

        let payload = Encapsulation::from(slice_body).to_bytes().unwrap();
        let raw = RawReply {
            request_id: 1,
            status: crate::protocol::reply_status::USER_EXCEPTION,
            payload,
        };
        let err = Proxy::interpret_reply(raw).unwrap_err();
        let ux = err
            .downcast_ref::<RemoteUserException>()
            .expect("должно быть RemoteUserException");
        assert_eq!(type_id, ux.type_id, "тип исключения потерян");
        assert_eq!(vec![1u8, 2, 3], ux.payload);
    }

    /// Компактный type-id сохраняем как число: сопоставить с именем без реестра
    /// типов нельзя, но и молчать об этом не надо.
    #[test]
    fn compact_type_id_is_preserved_as_a_number() {
        // Биты типа: 0=NoTypeId, 1=StringTypeId, 2=IndexTypeId, 3=CompactTypeId.
        let mut slice_body = vec![0x03u8];
        slice_body.extend(IceSize { size: 7 }.to_bytes().unwrap());
        let payload = Encapsulation::from(slice_body).to_bytes().unwrap();
        let raw = RawReply {
            request_id: 1,
            status: crate::protocol::reply_status::USER_EXCEPTION,
            payload,
        };
        let err = Proxy::interpret_reply(raw).unwrap_err();
        let ux = err.downcast_ref::<RemoteUserException>().unwrap();
        assert_eq!("<compact:7>", ux.type_id);
    }

    /// Косвенный прокси мы не резолвим, но и молча мусор разбирать не должны.
    #[test]
    fn indirect_proxy_is_rejected_clearly() {
        let id = Identity::new("Meta");
        let data = ProxyData {
            name: id.name,
            category: id.category,
            facet: vec![],
            mode: 0,
            secure: false,
            protocol: Version { major: 1, minor: 0 },
            encoding: Version { major: 1, minor: 1 },
        };
        let mut bytes = data.to_bytes().unwrap();
        bytes.extend(IceSize { size: 0 }.to_bytes().unwrap());
        let mut read = 0i32;
        let err = Proxy::from_bytes(&bytes, &mut read).unwrap_err().to_string();
        assert!(err.contains("Indirect"), "{}", err);
    }
}
