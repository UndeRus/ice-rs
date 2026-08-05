//! ZeroC Ice для Rust: рантайм протокола Ice 1.1 и компилятор Slice.
//!
//! # Что выбрать
//!
//! **Пишете бота для Mumble?** Берите крейт `mumble-ice` — человеческий API с
//! типизированными идентификаторами, колбеками и аутентификатором. Этот крейт
//! тогда нужен только как транспорт под ним.
//!
//! **Нужен generic Ice?** Тогда вам сюда. Учтите ограничения: поддержаны TCP и
//! SSL, кодировка 1.1, twoway и oneway. Нет batch-запросов, сжатия, UDP,
//! Glacier2, ACM/heartbeat, а из Slice не поддержаны вложенные генерики,
//! `optional` в структурах, операции в классах и compact type-id у классов.
//! Практически покрыто то, что использует `MumbleServer.ice`.
//!
//! # Клиент
//!
//! ```no_run
//! use ice_rs::communicator::Communicator;
//! # mod gen { pub mod demo {
//! #   pub struct HelloPrx; impl HelloPrx {
//! #     pub async fn checked_cast(_p: ice_rs::proxy::Proxy) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> { Ok(HelloPrx) }
//! #     pub async fn say_hello(&mut self, _c: Option<std::collections::HashMap<String,String>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }
//! #   } } }
//! use gen::demo::HelloPrx;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!     let mut comm = Communicator::new().await?;
//!     let proxy = comm.string_to_proxy("hello:tcp -h localhost -p 10000").await?;
//!     let mut hello = HelloPrx::checked_cast(proxy).await?;
//!     hello.say_hello(None).await
//! }
//! ```
//!
//! # Сервер
//!
//! Реализуете сгенерированный трейт `*I`, оборачиваете в `*Server` и кладёте в
//! адаптер. `into_servant()` генерируется вместе с остальным кодом:
//!
//! ```no_run
//! # use ice_rs::adapter::Adapter;
//! # async fn f(servant: std::sync::Arc<dyn ice_rs::iceobject::Servant>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! let mut adapter = Adapter::with_endpoint("hello", "tcp -h 127.0.0.1 -p 10000")?;
//! adapter.add("hello", servant);
//! // Если слушаете на 0.0.0.0, обязательно объявите достижимый адрес: пир звонит
//! // на него сам, и wildcard в прокси ему не поможет.
//! // adapter.advertise("bot.internal", 10000);
//! let handle = adapter.serve().await?;   // не блокирует
//! println!("слушаем на {}", handle.local_addr());
//! handle.shutdown().await;
//! # Ok(()) }
//! ```
//!
//! Адаптер обслуживает соединения конкурентно (таск на соединение), ключует
//! servant'ов по полной Ice-идентичности с facet'ом и отвечает корректными
//! статусами Ice: 2/3/4 с телом `RequestFailedException`, 1 с type-id
//! исключения, 5 при внутреннем сбое.
//!
//! # Кодогенерация
//!
//! Из `.ice` в Rust — через `build.rs` либо разово бинарём `slice2rs`. Разовая
//! генерация с закоммиченным выводом предпочтительнее: потребителю не нужен
//! `rustfmt` на `PATH`, а диффы биндингов видны в ревью (так сделан крейт
//! `murmur-slice`).
//!
//! ```no_run
//! use ice_rs::slice::parser;
//! use std::path::Path;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     println!("cargo:rerun-if-changed=Hello.ice");
//!     let ice_files = vec![String::from("Hello.ice")];
//!     // Второй аргумент — каталог для `#include <...>`.
//!     let root = parser::parse_ice_files(&ice_files, ".")?;
//!     // Второй аргумент — префикс супермодуля для `use`-путей.
//!     root.generate(Path::new("./src/gen"), "")
//! }
//! ```
//!
//! Ошибка в `.ice` приходит с файлом, строкой и столбцом, а не паникой внутри
//! build.rs.
//! ```

#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate ice_derive;

pub use ice_derive::IceDerive;

pub mod errors;
pub mod protocol;
pub mod encoding;
pub mod tcp;
pub mod ssl;
pub mod ssltools;
pub mod transport;
pub mod proxy;
pub mod proxy_parser;
pub mod proxy_factory;
pub mod communicator;
pub mod iceobject;
pub mod slice;
pub mod initdata;
pub mod properties;
pub mod locator;
pub mod adapter;