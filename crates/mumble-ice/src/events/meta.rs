//! Внутренний `MetaCallback` — руками, а не через сгенерированный servant.
//!
//! `MetaCallback::started(Server *srv)` несёт прокси, а сгенерированный декодер
//! прокси в нижнем слое вызывает `futures::executor::block_on` и **открывает TCP-
//! соединение прямо внутри десериализации**. Внутри обработчика колбека это
//! означает синхронный дозвон на воркере, пока Murmur ждёт ответа.
//!
//! Свой servant разбирает только `ProxyData` (то есть идентичность) и ищет
//! виртуальный сервер в кэше клиента — ни блокировки, ни дозвона.

use super::{Registry, ServerEvents};
use crate::error::Error;
use crate::ids::ServerId;
use crate::server::VirtualServer;
use async_trait::async_trait;
use ice_rs::encoding::FromBytes;
use ice_rs::iceobject::{DispatchResult, Servant};
use ice_rs::protocol::{Encapsulation, ProxyData, RequestData};
use std::sync::Arc;

const META_CALLBACK_TYPE_ID: &str = "::MumbleServer::MetaCallback";

struct MetaShim {
    registry: Arc<Registry>,
    server: VirtualServer,
}

impl MetaShim {
    /// Достаёт идентичность виртуального сервера из прокси в параметрах.
    ///
    /// Murmur кодирует идентичность как `s{n}` — например `s1` для сервера 1.
    fn server_id_from_params(params: &Encapsulation) -> Option<ServerId> {
        let buf = ice_rs::protocol::peel_slice_param_payload(&params.data);
        let mut read = 0i32;
        let data = ProxyData::from_bytes(&buf, &mut read).ok()?;
        let name = data.name.trim();
        let digits = name.trim_start_matches(|c: char| !c.is_ascii_digit());
        digits.parse::<i32>().ok().map(ServerId)
    }
}

#[async_trait]
impl Servant for MetaShim {
    fn type_ids(&self) -> Vec<String> {
        vec![
            String::from(META_CALLBACK_TYPE_ID),
            String::from("::Ice::Object"),
        ]
    }

    async fn dispatch(&self, request: &RequestData) -> DispatchResult {
        let id = MetaShim::server_id_from_params(&request.params);
        match request.operation.as_str() {
            "started" => {
                let id = id.unwrap_or_else(|| self.server.id());
                // Виртуальный сервер поднялся заново, значит наши колбеки он
                // снял: возвращаем их.
                let registry = self.registry.clone();
                tokio::spawn(async move { registry.reattach(id).await });

                let handlers = self.registry.meta_handlers().await;
                for h in handlers {
                    let h2 = h.clone();
                    tokio::spawn(async move {
                        if let Err(e) = h.server_started(id).await {
                            h2.on_error(e).await;
                        }
                    });
                }
                DispatchResult::Ok(Encapsulation::empty())
            }
            "stopped" => {
                let id = id.unwrap_or_else(|| self.server.id());
                let handlers = self.registry.meta_handlers().await;
                for h in handlers {
                    let h2 = h.clone();
                    tokio::spawn(async move {
                        if let Err(e) = h.server_stopped(id).await {
                            h2.on_error(e).await;
                        }
                    });
                }
                DispatchResult::Ok(Encapsulation::empty())
            }
            _ => DispatchResult::OperationNotExist,
        }
    }
}

/// Ставит внутренний `MetaCallback`, если его ещё нет.
///
/// Вызывается на каждой подписке, но регистрирует ровно один раз.
pub(crate) async fn install_meta_callback(registry: &Arc<Registry>, server: &VirtualServer) {
    if registry.meta_hooked() {
        return;
    }
    let adapter = match registry.ensure_adapter().await {
        Ok(a) => a,
        Err(_) => return,
    };
    let ident = registry.next_ident("meta");
    let servant: Arc<dyn Servant> = Arc::new(MetaShim {
        registry: registry.clone(),
        server: server.clone(),
    });
    adapter
        .adapter
        .register(ice_rs::adapter::ServantKey::new(&ident), servant)
        .await;

    let proxy_string = format!(
        "{}:tcp -h {} -p {}",
        ident, adapter.advertise.0, adapter.advertise.1
    );
    // Не смогли — не страшно: переподписка после перезапуска не заработает, но
    // сами колбеки будут жить. Сообщаем через `on_error`, если есть кому.
    if let Err(e) = server.add_meta_callback(&proxy_string).await {
        for h in registry.meta_handlers().await {
            h.on_error(Error::Protocol(format!(
                "не удалось поставить MetaCallback (переподписка после перезапуска не сработает): {}",
                e
            )))
            .await;
        }
        let _ = adapter
            .adapter
            .unregister(&ice_rs::adapter::ServantKey::new(&ident))
            .await;
        return;
    }
    registry.set_meta_hooked();
}

/// Нужен, чтобы `ServerEvents` не считался неиспользованным в этом модуле.
#[allow(dead_code)]
fn _uses_server_events(_: &dyn ServerEvents) {}

#[cfg(test)]
mod tests {
    use super::*;
    use ice_rs::encoding::ToBytes;
    use ice_rs::protocol::Version;

    fn proxy_params(name: &str) -> Encapsulation {
        let data = ProxyData {
            name: String::from(name),
            category: String::new(),
            facet: vec![],
            mode: 0,
            secure: false,
            protocol: Version { major: 1, minor: 0 },
            encoding: Version { major: 1, minor: 1 },
        };
        Encapsulation::from(data.to_bytes().unwrap())
    }

    /// Murmur называет виртуальные серверы `s1`, `s2`, …
    #[test]
    fn extracts_server_id_from_proxy_identity() {
        assert_eq!(
            Some(ServerId(1)),
            MetaShim::server_id_from_params(&proxy_params("s1"))
        );
        assert_eq!(
            Some(ServerId(42)),
            MetaShim::server_id_from_params(&proxy_params("s42"))
        );
    }

    #[test]
    fn unparseable_identity_yields_none() {
        assert_eq!(None, MetaShim::server_id_from_params(&proxy_params("Meta")));
    }
}
