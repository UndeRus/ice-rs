use std::{collections::HashMap, hash::Hash};
use std::sync::Arc;

use task::JoinHandle;
use tokio::{io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf}, sync::Mutex, task};

use crate::{errors::{ProtocolError, UserError}, protocol::{Header, MessageType}, proxy_factory::ProxyFactory, proxy_parser::{DirectProxyData, ProxyStringType, parse_proxy_string}, transport::Transport};
use crate::protocol::{EndPointType, ReplyData, RequestData, Identity, Encapsulation, EndpointData, ProxyData, Version};
use crate::encoding::{ToBytes, FromBytes, IceSize};
use crate::communicator::INITDATA;
use futures::executor::block_on;

#[derive(Parser)]
#[grammar = "proxystring.pest"]
pub struct ProxyParser;

pub struct Proxy {
    pub write: WriteHalf<Box<dyn Transport + Send + Sync + Unpin>>,
    pub request_id: i32,
    pub ident: String,
    pub host: String,
    pub port: i32,
    pub context: Option<HashMap<String, String>>,
    pub handle: Option<JoinHandle<Result<(), Box<dyn std::error::Error + Sync + Send>>>>,
    pub message_queue: Arc<Mutex<Vec<MessageType>>>,
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
        let total = header.message_size as usize;
        if total < 14 {
            return Err(Box::new(ProtocolError::new("Ice: message_size < 14")));
        }
        let mut buf = vec![0u8; total];
        buf[..14].copy_from_slice(&hdr);
        if total > 14 {
            rx.read_exact(&mut buf[14..]).await?;
        }
        Ok(buf)
    }

    async fn read_thread(mut rx: ReadHalf<Box<dyn Transport + Send + Sync + Unpin>>, message_queue: Arc<Mutex<Vec<MessageType>>>) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        loop {
            let buffer = Self::read_ice_message(&mut rx).await?;
            let mut read: i32 = 0;
            let header = Header::from_bytes(&buffer[read as usize..buffer.len()], &mut read)?;

            let message = match header.message_type {
                2 => {
                    let reply =
                        ReplyData::from_bytes(&buffer[read as usize..buffer.len()], &mut read)?;
                    MessageType::Reply(header, reply)
                }
                3 => MessageType::ValidateConnection(header),
                _ => {
                    return Err(Box::new(ProtocolError::new(&format!(
                        "TCP: Unsuppored reply message type: {}",
                        header.message_type
                    ))))
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
            stream_type
        };
        let message_queue = proxy.message_queue.clone();
        proxy.handle = Some(task::spawn(async move {
            Proxy::read_thread(rx, message_queue).await
        }));

        proxy
    }

    async fn close_connection(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    {
        let header = Header::new(4, 14);
        let mut bytes = header.to_bytes()?;
        let written = self.write.write(&mut bytes).await?;
        if written != header.message_size as usize {
            return Err(Box::new(ProtocolError::new("TCP: Could not validate connection")))
        }

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

        let written = self.write.write(&mut bytes).await?;
        if written != header.message_size as usize {
            return Err(Box::new(ProtocolError::new(&format!("TCP: Error writing request {}", request.request_id))))
        }
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

            if now.elapsed() >= timeout {
                return Err(Box::new(ProtocolError::new("Timeout waiting for response")));
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
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

            if now.elapsed() >= timeout {
                return Err(Box::new(ProtocolError::new("Timeout waiting for response")));
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    async fn read_response<T: 'static + std::fmt::Debug + std::fmt::Display + FromBytes + Send + Sync>(&mut self, request_id: i32) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>> {
        let message = self.await_reply_message(request_id).await?;
        match message {
            MessageType::Reply(_header, reply) => {
                match reply.status {
                    1 => {
                        let mut read = 0;
                        Err(Box::new(UserError {
                            exception: T::from_bytes(&reply.body.data, &mut read)?
                        }))
                    }
                    _ => Ok(reply),
                }
            },
            _ => Err(Box::new(ProtocolError::new(&format!("Unsupported message type: {:?}", message))))
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
        let id = Identity::new(&self.ident);
        let proxy_data = ProxyData {
            name: id.name,
            category: id.category,
            facet: vec![],
            mode: 2,
            secure: self.stream_type == "ssl",
            protocol: Version { major: 1, minor: 0 },
            encoding: Version { major: 1, minor: 1 },
        };
        let mut out = proxy_data.to_bytes()?;
        let ep_type: i16 = if self.stream_type == "ssl" { 2 } else { 1 };
        let mut inner = Vec::new();
        inner.extend(ep_type.to_bytes()?);
        let ep = EndpointData {
            host: self.host.clone(),
            port: self.port,
            timeout: -1,
            compress: false,
        };
        let enc = Encapsulation::from(ep.to_bytes()?);
        inner.extend(enc.to_bytes()?);
        out.extend(IceSize { size: inner.len() as i32 }.to_bytes()?);
        out.extend(inner);
        Ok(out)
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
        let _sz = IceSize::from_bytes(&bytes[read as usize..], &mut read)?;
        let ep_disc = i16::from_bytes(&bytes[read as usize..], &mut read)?;
        let enc = Encapsulation::from_bytes(&bytes[read as usize..], &mut read)?;
        let mut er = 0i32;
        let endpoint = EndpointData::from_bytes(&enc.data, &mut er)?;
        let direct = DirectProxyData {
            ident: proxy_data.identity_string(),
            endpoint: match ep_disc {
                1 => EndPointType::TCP(endpoint),
                2 => EndPointType::SSL(endpoint),
                x => {
                    return Err(Box::new(ProtocolError::new(&format!(
                        "Unsupported proxy endpoint discriminator {}",
                        x
                    ))))
                }
            },
        };
        let proxy = block_on(async { ProxyFactory::create_proxy(direct, &props, None).await })?;
        *read_bytes += read;
        Ok(proxy)
    }
}