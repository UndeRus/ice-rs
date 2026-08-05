//! Человеческий API к Murmur (Mumble server) через ZeroC Ice.
//!
//! # Быстрый старт
//!
//! ```no_run
//! use mumble_ice::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> mumble_ice::Result<()> {
//!     let client = MurmurClient::connect("127.0.0.1:6502").await?;
//!     let srv = client.only_server().await?;
//!
//!     println!("Murmur {}", client.version().await?);
//!     for user in srv.users().await? {
//!         println!("{} в канале {}", user.name, user.channel);
//!     }
//!     srv.broadcast("привет").await?;
//!     Ok(())
//! }
//! ```
//!
//! С секретом и нестандартным адресом:
//!
//! ```no_run
//! # use mumble_ice::prelude::*;
//! # async fn f() -> mumble_ice::Result<()> {
//! let client = MurmurClient::builder()
//!     .host_port("murmur.example.com", 6502)
//!     .secret(std::env::var("MUMBLE_SECRET").unwrap_or_default())
//!     .connect()
//!     .await?;
//! let srv = client.server(ServerId(1)).await?;
//! # Ok(()) }
//! ```
//!
//! # Что этот слой убирает
//!
//! Работать с сгенерированным Slice-кодом напрямую значит:
//!
//! - тащить `Option<HashMap<String, String>>` с секретом в **каждый** вызов;
//! - брать `&mut self` на каждом методе, то есть не мочь поделить хендл между
//!   тасками без своего `Arc<Mutex<_>>`;
//! - принимать out-параметры как `&mut`:
//!   `meta.get_version(&mut major, &mut minor, &mut patch, &mut text, ctx)`;
//! - различать `session` и `userid` на глаз — оба `i32`, а операции делятся
//!   ровно по этой границе;
//! - разбирать сентинелы: `userid == -1` это анонимный, `-1`/`-2` из
//!   `verifyPassword` — разные отказы, пустая строка из `getConf` — «нет
//!   значения»;
//! - ловить все ошибки как `Box<dyn Error>` и различать их по тексту;
//! - хардкодить `0x01` вместо констант прав.
//!
//! Здесь вместо этого: секрет **один раз** на билдере, всё на `&self` и `Clone`,
//! именованные структуры вместо out-параметров, типизированные идентификаторы,
//! `Option` вместо сентинелов, [`Error`] с вариантами и [`Permission`] как
//! `bitflags`.
//!
//! # Конкурентность
//!
//! Вызовы к одному серверу идут параллельно по одному соединению —
//! мультиплексирование по `request_id`. Хендлы `Clone` и `&self`, поэтому
//! раздаются по таскам без внешнего мьютекса, и однопоточный рантайм тоже
//! работает.
//!
//! ```no_run
//! # use mumble_ice::prelude::*;
//! # async fn f(srv: VirtualServer) -> mumble_ice::Result<()> {
//! let (users, channels) = tokio::join!(srv.users(), srv.channels());
//! let (users, channels) = (users?, channels?);
//! # Ok(()) }
//! ```
//!
//! # Колбеки
//!
//! ```no_run
//! use mumble_ice::prelude::*;
//! use std::sync::Arc;
//!
//! struct Greeter;
//!
//! #[async_trait::async_trait]
//! impl ServerEvents for Greeter {
//!     async fn user_connected(&self, srv: &VirtualServer, u: User) -> mumble_ice::Result<()> {
//!         srv.message_user(u.session, &format!("привет, {}!", u.name)).await
//!     }
//! }
//!
//! # async fn f(srv: VirtualServer) -> mumble_ice::Result<()> {
//! let sub = srv.on_events(Arc::new(Greeter)).await?;
//! // Пока `sub` жив, колбек зарегистрирован.
//! sub.closed().await;
//! # Ok(()) }
//! ```
//!
//! У всех методов трейта есть дефолтная реализация, так что боту, которому нужны
//! только текстовые сообщения, писать надо один метод. Есть и вариант потоком:
//! [`VirtualServer::events`].
//!
//! Три вещи, которые фасад берёт на себя:
//!
//! - **Переподписка.** Murmur снимает колбеки, когда виртуальный сервер
//!   останавливается; фасад ставит внутренний `MetaCallback`, возвращает
//!   подписки и сообщает об этом через `reattached()`. После него **все
//!   закэшированные `SessionId` — мусор**.
//! - **Ошибки не глотаются.** Исключение из колбека заставляет Murmur молча
//!   снять регистрацию целиком, поэтому Murmur'у мы отвечаем «ок» всегда, а
//!   `Err` и паника обработчика уходят в `on_error()`.
//! - **Адрес обратного вызова.** Murmur звонит **наружу**, поэтому под
//!   Docker/NAT адрес прослушивания и объявляемый адрес разные:
//!   `.callback_listen(...)` и `.callback_advertise(...)`. Wildcard без явного
//!   `advertise` — ошибка ещё на `connect()`, а не невнятный отказ позже.
//!
//! # Аутентификатор
//!
//! Один трейт на **оба** Slice-интерфейса, у всех методов дефолт «не моё», так
//! что минимальный аутентификатор — одна реализация `authenticate`. Сентинелы
//! Murmur'а (`-1`, `-2`, `-3`, `1`/`0`/`-1`) наружу не выходят.
//!
//! Главное, что здесь спрятано: разница между `AuthResult::Denied` («пароль
//! неверный») и `AuthResult::FallThrough` («имени не знаю, спроси свою базу»).
//! Перепутать их значит заблокировать всех пользователей из базы Murmur'а.
//!
//! ```no_run
//! use mumble_ice::prelude::*;
//! use std::sync::Arc;
//!
//! struct Auth;
//!
//! #[async_trait::async_trait]
//! impl Authenticator for Auth {
//!     async fn authenticate(&self, req: AuthRequest) -> AuthResult {
//!         if req.name_ci() == "alice" && req.password == "pw" {
//!             AuthResult::Ok(AuthOk::new(UserId(1001)).group("admin"))
//!         } else if req.name_ci() == "alice" {
//!             AuthResult::Denied
//!         } else {
//!             AuthResult::FallThrough
//!         }
//!     }
//! }
//!
//! # async fn f(srv: VirtualServer) -> mumble_ice::Result<()> {
//! let sub = srv.set_authenticator(Arc::new(Auth)).await?;
//! # Ok(()) }
//! ```
//!
//! `VirtualServer` в трейт не передаётся намеренно: обратный вызов в
//! `Server`/`Meta` отсюда вешает Murmur. Паника обработчика деградирует до
//! безопасного значения, а не блокирует вход всем.

pub mod auth;
pub mod client;
pub mod endpoint;
pub mod events;
pub mod error;
pub mod ids;
pub mod model;
pub mod perm;
pub mod raw;
pub mod server;

pub use client::{MurmurClient, MurmurClientBuilder};
pub use endpoint::{Endpoint, TlsConfig};
pub use auth::{
    AuthOk, AuthRequest, AuthResult, Authenticator, CertificateDer, Lookup, RegisterResult,
    UpdateResult,
};
pub use error::{Error, Result};
pub use events::{
    ContextAction, ContextHandler, ContextInvocation, Event, EventStream, MetaEvents, Overflow,
    ServerEvents, Subscription,
};
pub use ids::{ChannelId, ServerId, SessionId, UserId};
pub use perm::{ContextFlags, Permission};
pub use server::VirtualServer;

/// Сырой сгенерированный слой — чтобы escape hatch не требовал второй
/// зависимости.
pub use murmur_slice as slice;

pub mod prelude {
    pub use crate::client::MurmurClient;
    pub use crate::endpoint::{Endpoint, TlsConfig};
    pub use crate::auth::{
        AuthOk, AuthRequest, AuthResult, Authenticator, CertificateDer, Lookup, RegisterResult,
        UpdateResult,
    };
    pub use crate::error::{Error, Result};
    pub use crate::events::{
        ContextAction, ContextHandler, ContextInvocation, Event, EventStream, MetaEvents, Overflow,
        ServerEvents, Subscription,
    };
    pub use crate::ids::{ChannelId, ServerId, SessionId, UserId};
    pub use crate::model::{
        Acl, AclSnapshot, AclSubject, Ban, Channel, ChannelTree, ClientVersion, DbState, Group,
        LogEntry, PasswordCheck, TextMessage, User, UserField, UserInfo, Version,
    };
    pub use crate::perm::{ContextFlags, Permission};
    pub use crate::server::VirtualServer;
}
