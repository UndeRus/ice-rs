use crate::{errors::ProtocolError, protocol::{Encapsulation, EndPointType, LocatorResult}, proxy_parser::{DirectProxyData, IndirectProxyData}};
use crate::encoding::{ToBytes,FromBytes};
use crate::proxy::Proxy;

pub struct Locator {
    proxy: Proxy,
}

impl Locator {
    pub fn from(proxy: Proxy) -> Locator {
        Locator { proxy }
    }

    pub async fn locate(&mut self, proxy_data: IndirectProxyData) -> Result<DirectProxyData, Box<dyn std::error::Error + Sync + Send>> {
        match proxy_data.adapter {
            Some(adapter) => {
                let result = self.find_adapter_by_id(&adapter).await?;
                Ok(DirectProxyData {
                    ident: result.proxy_data.identity_string(),
                    endpoint: result.endpoint
                })
            }
            None => {
                let obj_result = self.find_object_by_id(&proxy_data.ident).await?;
                match obj_result.endpoint {
                    EndPointType::WellKnownObject(object) => {
                        let adapter_result = self.find_adapter_by_id(&object).await?;
                        Ok(DirectProxyData {
                            ident: obj_result.proxy_data.identity_string(),
                            endpoint: adapter_result.endpoint
                        })
                    }
                    _ => Ok(DirectProxyData {
                        ident: obj_result.proxy_data.identity_string(),
                        endpoint: obj_result.endpoint
                    })
                }

            }
        }
    }

    /// `Ice::Locator::findObjectById`.
    ///
    /// Раньше запрос собирался здесь вручную, со своим счётчиком request_id.
    /// Теперь id выдаёт соединение — иначе несколько прокси на одном сокете
    /// пересекались бы по номерам.
    pub async fn find_object_by_id(&mut self, req: &str) -> Result<LocatorResult, Box<dyn std::error::Error + Sync + Send>> {
        let mut bytes = req.to_bytes()?;
        bytes.push(0);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>("findObjectById", 1, &Encapsulation::from(bytes), None)
            .await?;
        let mut read = 0;
        LocatorResult::from_bytes(&reply.body.data[read as usize..reply.body.data.len()], &mut read)
    }

    /// `Ice::Locator::findAdapterById`.
    pub async fn find_adapter_by_id(&mut self, req: &str) -> Result<LocatorResult, Box<dyn std::error::Error + Sync + Send>> {
        let bytes = req.to_bytes()?;
        let reply = self
            .proxy
            .dispatch::<ProtocolError>("findAdapterById", 1, &Encapsulation::from(bytes), None)
            .await?;
        let mut read = 0;
        LocatorResult::from_bytes(&reply.body.data[read as usize..reply.body.data.len()], &mut read)
    }
}
