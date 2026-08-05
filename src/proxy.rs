use std::{collections::HashMap, hash::Hash};
use std::sync::Arc;

use task::JoinHandle;
use tokio::{io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf}, sync::Mutex, task};

use crate::{errors::{ProtocolError, RemoteException, RemoteUserException, RequestFailedException, RequestFailedKind}, protocol::{Header, MessageType, RawReply}, proxy_factory::ProxyFactory, proxy_parser::{DirectProxyData, ProxyStringType, parse_proxy_string}, transport::Transport};
use crate::protocol::{EndPointType, ReplyData, RequestData, Identity, Encapsulation, EndpointData, ProxyData};
use crate::encoding::{ToBytes, FromBytes, IceSize, SliceFlags, SliceFlagsTypeEncoding};
use crate::communicator::INITDATA;
use futures::executor::block_on;

#[derive(Parser)]
#[grammar = "proxystring.pest"]
pub struct ProxyParser;

/// Верхняя граница размера одного Ice-кадра (аналог `Ice.MessageSizeMax`, 2 МиБ
/// по умолчанию у ZeroC). Без неё `message_size` с провода — просто `i32`:
/// отрицательное значение при приведении к `usize` превращается в ~1.8e19,
/// проходит проверку `< 14` и уходит в `vec![0u8; total]`.
pub const MAX_MESSAGE_SIZE: i32 = 2 * 1024 * 1024;

pub struct Proxy {
    pub write: WriteHalf<Box<dyn Transport + Send + Sync + Unpin>>,
    pub request_id: i32,
    pub ident: String,
    pub host: String,
    pub port: i32,
    pub context: Option<HashMap<String, String>>,
    pub handle: Option<JoinHandle<Result<(), Box<dyn std::error::Error + Sync + Send>>>>,
    pub message_queue: Arc<Mutex<Vec<MessageType>>>,
    /// Причина смерти соединения, если оно уже мёртво.
    ///
    /// Раньше reader-таск умирал молча (его результат никто не читал), и все
    /// последующие вызовы честно ждали ответа 30 секунд, чтобы упасть с
    /// «Timeout waiting for response». Теперь ожидающие видят настоящую причину
    /// сразу.
    pub dead: Arc<Mutex<Option<String>>>,
    pub stream_type: String
}


impl Drop for Proxy {
    fn drop(&mut self) {
        if std::thread::panicking() {
            if let Some(h) = self.handle.take() {
                h.abort();
            }
            return;
        }
        let _ = tokio::task::block_in_place(|| {
            futures::executor::block_on(async { self.close_connection().await })
        });
        if let Some(h) = self.handle.take() {
            h.abort();
        }
    }
}

impl Proxy {
    /// Один полный TCP-кадр Ice (как в `adapter::read_ice_frame`): длина из `Header.message_size`.
    async fn read_ice_message(
        rx: &mut ReadHalf<Box<dyn Transport + Send + Sync + Unpin>>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        let mut hdr = [0u8; 14];
        rx.read_exact(&mut hdr).await?;
        let mut hdr_done = 0i32;
        let header = Header::from_bytes(&hdr, &mut hdr_done)?;
        // Проверять ДО приведения к usize и до аллокации.
        if header.message_size < 14 || header.message_size > MAX_MESSAGE_SIZE {
            return Err(Box::new(ProtocolError::new(&format!(
                "Ice: implausible message_size {} (allowed 14..={})",
                header.message_size, MAX_MESSAGE_SIZE
            ))));
        }
        let total = header.message_size as usize;
        let mut buf = vec![0u8; total];
        buf[..14].copy_from_slice(&hdr);
        if total > 14 {
            rx.read_exact(&mut buf[14..]).await?;
        }
        Ok(buf)
    }

    async fn read_thread(
        mut rx: ReadHalf<Box<dyn Transport + Send + Sync + Unpin>>,
        message_queue: Arc<Mutex<Vec<MessageType>>>,
        dead: Arc<Mutex<Option<String>>>,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let reason = Self::read_loop(&mut rx, &message_queue).await;
        // Что бы ни случилось, зафиксировать причину, чтобы ожидающие не висели
        // до таймаута.
        let mut lock = dead.lock().await;
        if lock.is_none() {
            *lock = Some(reason.clone());
        }
        Ok(())
    }

    /// Возвращает причину завершения — соединение после этого мёртво.
    async fn read_loop(
        rx: &mut ReadHalf<Box<dyn Transport + Send + Sync + Unpin>>,
        message_queue: &Arc<Mutex<Vec<MessageType>>>,
    ) -> String {
        loop {
            let buffer = match Self::read_ice_message(rx).await {
                Ok(b) => b,
                Err(e) => return format!("connection closed: {}", e),
            };
            let mut read: i32 = 0;
            let header = match Header::from_bytes(&buffer[..], &mut read) {
                Ok(h) => h,
                Err(e) => return format!("bad Ice header: {}", e),
            };

            let message = match header.message_type {
                0 => {
                    // Входящий запрос на исходящем соединении. Bidirectional не
                    // поддерживается, но раньше этот кадр возвращал Err и убивал
                    // reader, после чего все вызовы на этом прокси зависали.
                    MessageType::Request(header)
                }
                2 => match RawReply::from_bytes(&buffer[read as usize..], &mut read) {
                    Ok(reply) => MessageType::Reply(header, reply),
                    // Битый ответ роняет только этот кадр, а не соединение.
                    Err(_) => continue,
                },
                3 => MessageType::ValidateConnection(header),
                4 => {
                    // Штатное закрытие со стороны пира.
                    let mut lock = message_queue.lock().await;
                    lock.push(MessageType::CloseConnection(header));
                    return String::from("peer closed the connection");
                }
                other => {
                    // 1/6/7 = batch/compressed — не поддерживаем, но кадр уже
                    // считан целиком, поэтому просто идём дальше.
                    let _ = other;
                    continue;
                }
            };

            let mut lock = message_queue.lock().await;
            lock.push(message);
        }
    }

    pub fn new(stream: Box<dyn Transport + Send + Sync + Unpin>, ident: &str, host: &str, port: i32, context: Option<HashMap<String, String>>) -> Proxy {
        let stream_type = stream.transport_type();
        let (rx, tx) = tokio::io::split(stream);
        let mut proxy = Proxy {
            write: tx,
            request_id: 0,
            ident: String::from(ident),
            host: String::from(host),
            port,
            context: context,
            handle: None,
            message_queue: Arc::new(Mutex::new(Vec::new())),
            dead: Arc::new(Mutex::new(None)),
            stream_type
        };
        let message_queue = proxy.message_queue.clone();
        let dead = proxy.dead.clone();
        proxy.handle = Some(task::spawn(async move {
            Proxy::read_thread(rx, message_queue, dead).await
        }));

        proxy
    }

    async fn close_connection(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    {
        let header = Header::new(4, 14);
        let bytes = header.to_bytes()?;
        // `write` мог записать частично и считался ошибкой; `write_all` дописывает.
        self.write.write_all(&bytes).await?;
        self.write.flush().await?;
        Ok(())
    }

    pub async fn ice_context(&mut self, context: HashMap<String, String>) -> Result<Proxy, Box<dyn std::error::Error + Send + Sync>> {
        let init_data = crate::communicator::INITDATA.lock().unwrap();
        let proxy_string = format!("{}:{} -h {} -p {}", self.ident, self.stream_type, self.host, self.port);
        match parse_proxy_string(&proxy_string)? {
            ProxyStringType::DirectProxy(data) => {
                ProxyFactory::create_proxy(data, init_data.properties(), Some(context)).await
            }
            _ => {
                Err(Box::new(ProtocolError::new("ice_context() - could not create proxy")))
            }
        }
    }

    pub async fn dispatch<
        T: 'static + std::fmt::Debug + std::fmt::Display + FromBytes + Send + Sync,
    >(
        &mut self,
        op: &str,
        mode: u8,
        params: &Encapsulation,
        context: Option<HashMap<String, String>>,
    ) -> Result<ReplyData, Box<dyn std::error::Error + Send + Sync>> {
        let id = String::from(self.ident.clone());
        let req = self.create_request(&id, op, mode, params, context);
        self.make_request::<T>(&req).await
    }

    pub fn create_request(&mut self, identity_name: &str, operation: &str, mode: u8, params: &Encapsulation, context: Option<HashMap<String, String>>) -> RequestData {
        let context = match context {
            Some(context) => context,
            None => {
                match self.context.as_ref() {
                    Some(context) => context.clone(),
                    None => HashMap::new()
                }
            }
        };
        self.request_id = self.request_id + 1;
        RequestData {
            request_id: self.request_id,
            id: Identity::new(identity_name),
            facet: Vec::new(),
            operation: String::from(operation),
            mode: mode,
            context: context,
            params: params.clone()
        }
    }

    async fn send_request(&mut self, request: &RequestData) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let req_bytes = request.to_bytes()?;
        let header = Header::new(0, 14 + req_bytes.len() as i32);
        let mut bytes = header.to_bytes()?;
        bytes.extend(req_bytes);

        self.write.write_all(&bytes).await?;
        self.write.flush().await?;
        Ok(())
    }

    pub async fn await_validate_connection_message(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let timeout = std::time::Duration::from_secs(30); // TODO: read from ice config
        let now = std::time::Instant::now();

        loop {
            {
                let mut lock = self.message_queue.lock().await;
                let index = lock.iter().position(|i| {
                    match i {
                        MessageType::ValidateConnection(_) => true,
                        _ => false
                    }
                });
                match index {
                    Some(index) => {
                        lock.swap_remove(index);
                        break;
                    },
                    None => {}
                }
            }

            if let Some(reason) = self.dead.lock().await.as_ref() {
                return Err(Box::new(ProtocolError::new(&format!(
                    "Connection died while waiting for connection validation: {}",
                    reason
                ))));
            }

            if now.elapsed() >= timeout {
                return Err(Box::new(ProtocolError::new("Timeout waiting for response")));
            }

            // `std::thread::sleep` здесь блокировал воркер tokio на всё время
            // ожидания и делал невозможной работу на current_thread-рантайме.
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        Ok(())
    }

    pub async fn await_reply_message(&mut self, request_id: i32) -> Result<MessageType, Box<dyn std::error::Error + Sync + Send>> {
        let timeout = std::time::Duration::from_secs(30); // TODO: read from ice config
        let now = std::time::Instant::now();

        loop {
            {
                let mut lock = self.message_queue.lock().await;
                let index = lock.iter().position(|i| {
                    match i {
                        MessageType::Reply(_, data) => {
                            if data.request_id == request_id {
                                true
                            } else {
                                false
                            }
                        },
                        _ => false
                    }
                });
                match index {
                    Some(index) => {
                        let result = lock.swap_remove(index);
                        return Ok(result)
                    },
                    None => {}
                }
            }

            if let Some(reason) = self.dead.lock().await.as_ref() {
                return Err(Box::new(ProtocolError::new(&format!(
                    "Connection died while waiting for reply to request {}: {}",
                    request_id, reason
                ))));
            }

            if now.elapsed() >= timeout {
                return Err(Box::new(ProtocolError::new("Timeout waiting for response")));
            }

            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }

    async fn read_response<T: 'static + std::fmt::Debug + std::fmt::Display + FromBytes + Send + Sync>(&mut self, request_id: i32) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>> {
        let message = self.await_reply_message(request_id).await?;
        match message {
            MessageType::Reply(_header, reply) => Self::interpret_reply(reply),
            _ => Err(Box::new(ProtocolError::new(&format!("Unsupported message type: {:?}", message))))
        }
    }

    /// Превращает сырой ответ в результат либо в типизированную ошибку.
    ///
    /// Раньше это делалось в двух местах и обоими неправильно: статусы 2..=7
    /// возвращали `Err` из декодера внутри reader-таска (убивая соединение), а
    /// статус 1 сразу разворачивался в единственный тип, захардкоженный
    /// кодогеном для операции, — прочитанный Slice type-id выбрасывался.
    pub fn interpret_reply(
        reply: RawReply,
    ) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>> {
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
                        // Компактный/индексный id: сохраняем как число — сопоставить
                        // с именем без реестра типов нельзя.
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
                // Тело RequestFailedException: Identity, StringSeq facet, string operation.
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
                let facet =
                    Vec::<String>::from_bytes(&reply.payload[r as usize..], &mut r).unwrap_or_default();
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

    pub async fn make_request<T: 'static + std::fmt::Debug + std::fmt::Display + FromBytes + Send + Sync>(&mut self, request: &RequestData) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>>
    {
        self.send_request(request).await?;
        self.read_response::<T>(request.request_id).await
    }
}

impl ToBytes for Proxy {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        crate::protocol::marshal_proxy_ref(
            &self.ident,
            &self.host,
            self.port,
            self.stream_type == "ssl",
        )
    }
}

impl FromBytes for Proxy {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let mut read = 0i32;
        let proxy_data = ProxyData::from_bytes(bytes, &mut read)?;
        let props = {
            let g = INITDATA.lock().unwrap();
            g.properties().clone()
        };
        // Счётчик эндпоинтов. Раньше это поле читалось и выбрасывалось (`let _sz`),
        // что и маскировало симметричный баг в `to_bytes`.
        let count = IceSize::from_bytes(&bytes[read as usize..], &mut read)?.size;
        if count <= 0 {
            // count == 0 означает косвенный (well-known) прокси: дальше по проводу
            // идёт строка adapter-id, которую надо резолвить через локатор.
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
        let endpoint = chosen.ok_or_else(|| {
            ProtocolError::new("Proxy carries no tcp/ssl endpoint we can use")
        })?;
        let direct = DirectProxyData {
            ident: proxy_data.identity_string(),
            endpoint,
        };
        let proxy = block_on(async { ProxyFactory::create_proxy(direct, &props, None).await })?;
        *read_bytes += read;
        Ok(proxy)
    }
}