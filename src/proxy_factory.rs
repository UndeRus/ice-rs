use std::collections::HashMap;

use crate::{errors::ProtocolError, locator::Locator, properties::Properties, protocol::EndPointType, proxy::{Proxy, Target}, proxy_parser::{DirectProxyData, ProxyStringType, parse_proxy_string}};

pub struct ProxyFactory {
    locator: Option<Locator>
}

impl ProxyFactory {
    /// Собирает прокси. Соединение открывается лениво, при первом вызове,
    /// поэтому `properties` здесь больше не нужны — их читает сам прокси в
    /// момент дозвона.
    pub async fn create_proxy(proxy_data: DirectProxyData, _properties: &Properties, context: Option<HashMap<String, String>>) -> Result<Proxy, Box<dyn std::error::Error + Sync + Send>> {
        let target = match proxy_data.endpoint {
            EndPointType::TCP(endpoint) => Target {
                ident: proxy_data.ident,
                host: endpoint.host,
                port: endpoint.port,
                secure: false,
            },
            EndPointType::SSL(endpoint) => Target {
                ident: proxy_data.ident,
                host: endpoint.host,
                port: endpoint.port,
                secure: true,
            },
            _ => return Err(Box::new(ProtocolError::new("Error creating proxy")))
        };
        Ok(Proxy::unresolved(target, context))
    }

    pub async fn new(properties: &Properties) -> Result<ProxyFactory, Box<dyn std::error::Error + Sync + Send>> {
        Ok(ProxyFactory {
            locator: match properties.get("Ice.Default.Locator") {
                Some(locator_proxy) => {
                    match parse_proxy_string(locator_proxy) {
                        Ok(proxy_type) => {
                            match proxy_type {
                                ProxyStringType::DirectProxy(data) => {
                                    Some(Locator::from(ProxyFactory::create_proxy(data, properties, None).await?))
                                }
                                _ => None
                            }
                        },
                        _ => None
                    }
                },
                _ => None
            }
        })
    }

    pub async fn create(&mut self, proxy_string: &str, properties: &Properties) -> Result<Proxy, Box<dyn std::error::Error + Sync + Send>> {
        match parse_proxy_string(proxy_string)? {
            ProxyStringType::DirectProxy(data) => {
                ProxyFactory::create_proxy(data, properties, None).await
            }
            ProxyStringType::IndirectProxy(data) => {
                match self.locator.as_mut() {
                    Some(locator) => {
                        let data = locator.locate(data).await?;
                        ProxyFactory::create_proxy(data, properties, None).await
                    }
                    _ => Err(Box::new(ProtocolError::new(&format!("No locator set up to resolve indirect proxy"))))
                }
            }
        }
    }
}