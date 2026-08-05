//! Мост между трейтом [`Authenticator`] и сгенерированным servant'ом.
//!
//! Здесь живёт трансляция сентинелов, и это единственное место, где числа
//! `-1`/`-2`/`-3` вообще появляются.
//!
//! Два отличия от моста колбеков:
//!
//! - Обработчик **ждётся**, а не спавнится: Murmur стоит и ждёт ответа, ответ —
//!   это и есть результат.
//! - При панике на провод уходит **безопасное** значение (fall-through для
//!   поисков, `Unavailable` для `authenticate`), а не ошибка. Сломанный
//!   аутентификатор должен деградировать до «Murmur смотрит свою базу», а не до
//!   «никто не может войти».

use super::{AuthRequest, AuthResult, Authenticator, CertificateDer, Lookup, RegisterResult, UpdateResult};
use crate::error::{Error, Result};
use crate::events::{Registration, Registry, Subscription, SubscriptionState};
use crate::ids::UserId;
use crate::model::UserInfo;
use crate::server::VirtualServer;
use async_trait::async_trait;
use murmur_slice::mumble_server::{
    self as slice, ServerUpdatingAuthenticatorI, ServerUpdatingAuthenticatorServer,
};
use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

type IceCtx = Option<HashMap<String, String>>;

// ── трансляция сентинелов ─────────────────────────────────────────────────

/// `authenticate`: id / `-1` отказ / `-2` не моё / `-3` недоступно.
pub(crate) fn encode_auth(r: AuthResult) -> (i32, String, Vec<String>) {
    match r {
        AuthResult::Ok(ok) => (
            ok.user_id.get(),
            ok.rename.unwrap_or_default(),
            ok.groups,
        ),
        AuthResult::Denied => (-1, String::new(), Vec::new()),
        AuthResult::FallThrough => (-2, String::new(), Vec::new()),
        AuthResult::Unavailable => (-3, String::new(), Vec::new()),
    }
}

/// `nameToId`: id либо `-2`.
pub(crate) fn encode_name_to_id(r: Lookup<UserId>) -> i32 {
    match r {
        Lookup::Found(id) => id.get(),
        Lookup::Unknown => -2,
    }
}

/// `idToName`: имя либо пустая строка.
pub(crate) fn encode_id_to_name(r: Lookup<String>) -> String {
    match r {
        Lookup::Found(n) => n,
        Lookup::Unknown => String::new(),
    }
}

/// `idToTexture`: аватар либо пустая текстура.
pub(crate) fn encode_texture(r: Lookup<Vec<u8>>) -> Vec<u8> {
    match r {
        Lookup::Found(t) => t,
        Lookup::Unknown => Vec::new(),
    }
}

/// `getInfo`: `true` + out-параметр либо `false`.
pub(crate) fn encode_user_info(r: Lookup<UserInfo>) -> (bool, slice::UserInfoMap) {
    match r {
        Lookup::Found(info) => (true, info.to_slice()),
        Lookup::Unknown => (false, HashMap::new()),
    }
}

/// `registerUser`: id / `-1` / `-2`.
pub(crate) fn encode_register(r: RegisterResult) -> i32 {
    match r {
        RegisterResult::Ok(id) => id.get(),
        RegisterResult::Failed => -1,
        RegisterResult::FallThrough => -2,
    }
}

/// `unregisterUser`/`setInfo`/`setTexture`: `1` / `0` / `-1`.
pub(crate) fn encode_update(r: UpdateResult) -> i32 {
    match r {
        UpdateResult::Ok => 1,
        UpdateResult::Failed => 0,
        UpdateResult::FallThrough => -1,
    }
}

// ── изоляция паники ───────────────────────────────────────────────────────

/// Ждёт обработчик, а при панике отдаёт безопасное значение.
///
/// Панику ловит `tokio::spawn`: она приходит как `Err(JoinError)`. Из
/// `&mut self`-метода сгенерированного трейта паника иначе улетела бы в задачу
/// соединения и оборвала его.
async fn guarded<F, T>(
    op: &'static str,
    auth: &Arc<dyn Authenticator>,
    safe: T,
    fut: F,
) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    match tokio::spawn(fut).await {
        Ok(v) => v,
        Err(join) => {
            let msg = if join.is_panic() {
                format!("паника в аутентификаторе ({})", op)
            } else {
                format!("обработчик аутентификатора отменён ({})", op)
            };
            auth.on_error(Error::Protocol(msg)).await;
            safe
        }
    }
}

/// Мост к сгенерированному servant'у.
pub(crate) struct AuthShim {
    auth: Arc<dyn Authenticator>,
}

#[async_trait]
impl ServerUpdatingAuthenticatorI for AuthShim {
    async fn authenticate(
        &mut self,
        name: &String,
        pw: &String,
        certificates: &slice::CertificateList,
        certhash: &String,
        certstrong: bool,
        newname: &mut String,
        groups: &mut slice::GroupNameList,
        _ctx: IceCtx,
    ) -> i32 {
        let req = AuthRequest {
            name: name.clone(),
            password: pw.clone(),
            certificates: certificates
                .iter()
                .map(|c| CertificateDer(c.clone()))
                .collect(),
            cert_hash: certhash.clone(),
            cert_strong: certstrong,
        };
        let auth = self.auth.clone();
        let a2 = auth.clone();
        // Безопасное значение — Unavailable: Murmur скажет клиенту повторить, а не
        // «неверный пароль».
        let result = guarded("authenticate", &a2, AuthResult::Unavailable, async move {
            auth.authenticate(req).await
        })
        .await;

        let (id, rename, gs) = encode_auth(result);
        *newname = rename;
        *groups = gs;
        id
    }

    async fn get_info(&mut self, id: i32, info: &mut slice::UserInfoMap, _ctx: IceCtx) -> bool {
        let auth = self.auth.clone();
        let a2 = auth.clone();
        let looked = guarded("get_info", &a2, Lookup::Unknown, async move {
            auth.user_info(UserId(id)).await
        })
        .await;
        let (found, map) = encode_user_info(looked);
        *info = map;
        found
    }

    async fn name_to_id(&mut self, name: &String, _ctx: IceCtx) -> i32 {
        let auth = self.auth.clone();
        let a2 = auth.clone();
        let name = name.clone();
        let looked = guarded("name_to_id", &a2, Lookup::Unknown, async move {
            auth.name_to_id(&name).await
        })
        .await;
        encode_name_to_id(looked)
    }

    async fn id_to_name(&mut self, id: i32, _ctx: IceCtx) -> String {
        let auth = self.auth.clone();
        let a2 = auth.clone();
        let looked = guarded("id_to_name", &a2, Lookup::Unknown, async move {
            auth.id_to_name(UserId(id)).await
        })
        .await;
        encode_id_to_name(looked)
    }

    async fn id_to_texture(&mut self, id: i32, _ctx: IceCtx) -> slice::Texture {
        let auth = self.auth.clone();
        let a2 = auth.clone();
        let looked = guarded("id_to_texture", &a2, Lookup::Unknown, async move {
            auth.id_to_texture(UserId(id)).await
        })
        .await;
        encode_texture(looked)
    }

    async fn register_user(&mut self, info: &slice::UserInfoMap, _ctx: IceCtx) -> i32 {
        let auth = self.auth.clone();
        let a2 = auth.clone();
        let parsed = UserInfo::from_slice(info);
        let r = guarded(
            "register_user",
            &a2,
            RegisterResult::FallThrough,
            async move { auth.register_user(&parsed).await },
        )
        .await;
        encode_register(r)
    }

    async fn unregister_user(&mut self, id: i32, _ctx: IceCtx) -> i32 {
        let auth = self.auth.clone();
        let a2 = auth.clone();
        let r = guarded(
            "unregister_user",
            &a2,
            UpdateResult::FallThrough,
            async move { auth.unregister_user(UserId(id)).await },
        )
        .await;
        encode_update(r)
    }

    async fn get_registered_users(&mut self, filter: &String, _ctx: IceCtx) -> slice::NameMap {
        let auth = self.auth.clone();
        let a2 = auth.clone();
        let filter = filter.clone();
        let looked = guarded(
            "get_registered_users",
            &a2,
            Lookup::Unknown,
            async move { auth.registered_users(&filter).await },
        )
        .await;
        match looked {
            Lookup::Found(m) => m.into_iter().map(|(k, v)| (k.get(), v)).collect(),
            // В Slice здесь нет fall-through: пустая карта — единственный способ
            // сказать «не моё». Документировано в трейте.
            Lookup::Unknown => HashMap::new(),
        }
    }

    async fn set_info(&mut self, id: i32, info: &slice::UserInfoMap, _ctx: IceCtx) -> i32 {
        let auth = self.auth.clone();
        let a2 = auth.clone();
        let parsed = UserInfo::from_slice(info);
        let r = guarded("set_info", &a2, UpdateResult::FallThrough, async move {
            auth.set_user_info(UserId(id), &parsed).await
        })
        .await;
        encode_update(r)
    }

    async fn set_texture(&mut self, id: i32, tex: &slice::Texture, _ctx: IceCtx) -> i32 {
        let auth = self.auth.clone();
        let a2 = auth.clone();
        let tex = tex.clone();
        let r = guarded("set_texture", &a2, UpdateResult::FallThrough, async move {
            auth.set_texture(UserId(id), &tex).await
        })
        .await;
        encode_update(r)
    }
}

/// Ставит аутентификатор: servant в адаптер, прокси в Murmur.
pub(crate) async fn make_authenticator_subscription(
    registry: &Arc<Registry>,
    server: &VirtualServer,
    auth: Arc<dyn Authenticator>,
) -> Result<Subscription> {
    let adapter = registry.ensure_adapter().await?;
    let ident = registry.next_ident("auth");

    // Сгенерированный servant, а не рукописный: после починки кодогена у него
    // есть все десять операций, правильная цепочка type-id (Murmur делает
    // checkedCast к базовому `ServerAuthenticator`) и верный порядок полей в
    // ответе. Маршалить `UserInfoMap`/`Texture`/`NameMap` руками было бы
    // ошибкоопаснее.
    let servant = ServerUpdatingAuthenticatorServer::new(Box::new(AuthShim { auth })).into_servant();
    adapter
        .adapter
        .register(ice_rs::adapter::ServantKey::new(&ident), servant)
        .await;

    let proxy_string = format!(
        "{}:tcp -h {} -p {}",
        ident, adapter.advertise.0, adapter.advertise.1
    );

    let state = Arc::new(SubscriptionState {
        id: registry.take_id(),
        ident,
        proxy_string,
        server: server.clone(),
        registration: Registration::Authenticator,
        dead: AtomicBool::new(false),
        death: Notify::new(),
        death_reason: Mutex::new(None),
    });

    crate::events::register_subscription_in_murmur(&state).await?;
    registry.track(state.clone()).await;
    crate::events::install_meta_callback(registry, server).await;

    Ok(Subscription::new(state, registry.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Смысл всего модуля: сентинелы наружу не выходят, а внутрь переводятся
    /// однозначно.
    #[test]
    fn authenticate_sentinels() {
        assert_eq!(
            (7, String::new(), Vec::<String>::new()),
            encode_auth(AuthResult::Ok(super::super::AuthOk::new(UserId(7))))
        );
        assert_eq!(
            (-1, String::new(), vec![]),
            encode_auth(AuthResult::Denied)
        );
        assert_eq!(
            (-2, String::new(), vec![]),
            encode_auth(AuthResult::FallThrough)
        );
        assert_eq!(
            (-3, String::new(), vec![]),
            encode_auth(AuthResult::Unavailable)
        );
    }

    /// Переименование и группы уезжают в out-параметры.
    #[test]
    fn authenticate_carries_rename_and_groups() {
        let ok = super::super::AuthOk::new(UserId(9))
            .rename("Alice [staff]")
            .group("admin");
        let (id, rename, groups) = encode_auth(AuthResult::Ok(ok));
        assert_eq!(9, id);
        assert_eq!("Alice [staff]", rename);
        assert_eq!(vec!["admin"], groups);
    }

    #[test]
    fn lookup_sentinels() {
        assert_eq!(3, encode_name_to_id(Lookup::Found(UserId(3))));
        assert_eq!(-2, encode_name_to_id(Lookup::Unknown));

        assert_eq!("bob", encode_id_to_name(Lookup::Found(String::from("bob"))));
        assert_eq!("", encode_id_to_name(Lookup::Unknown));

        assert_eq!(vec![1u8, 2], encode_texture(Lookup::Found(vec![1, 2])));
        assert!(encode_texture(Lookup::Unknown).is_empty());
    }

    #[test]
    fn get_info_sentinels() {
        let (found, map) = encode_user_info(Lookup::Found(UserInfo::new("alice")));
        assert!(found);
        assert!(!map.is_empty());

        let (found, map) = encode_user_info(Lookup::Unknown);
        assert!(!found, "Unknown должен давать false, а не пустую карту с true");
        assert!(map.is_empty());
    }

    #[test]
    fn update_and_register_sentinels() {
        assert_eq!(1, encode_update(UpdateResult::Ok));
        assert_eq!(0, encode_update(UpdateResult::Failed));
        assert_eq!(-1, encode_update(UpdateResult::FallThrough));

        assert_eq!(5, encode_register(RegisterResult::Ok(UserId(5))));
        assert_eq!(-1, encode_register(RegisterResult::Failed));
        assert_eq!(-2, encode_register(RegisterResult::FallThrough));
    }

    /// `Denied` и `FallThrough` обязаны различаться на проводе: перепутать их
    /// значит заблокировать всех пользователей базы Murmur'а.
    #[test]
    fn denied_and_fallthrough_are_distinct_on_the_wire() {
        let denied = encode_auth(AuthResult::Denied).0;
        let fallthrough = encode_auth(AuthResult::FallThrough).0;
        assert_ne!(denied, fallthrough);
        assert_eq!(-1, denied);
        assert_eq!(-2, fallthrough);
    }
}
