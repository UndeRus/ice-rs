//! Escape hatch к сгенерированному слою.
//!
//! Обёрнута основная часть операций; длинный хвост (редкие геттеры, будущие
//! операции Murmur 1.6) доступен здесь — с уже готовым Ice-контекстом, чтобы
//! секрет не приходилось собирать руками.

use crate::client::Shared;
use murmur_slice::mumble_server::{MetaPrx, ServerPrx};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

/// Одолженный прокси `Server`.
///
/// ```ignore
/// use murmur_slice::mumble_server::Server as _;
///
/// let raw = server.raw().await;
/// let conf = raw.get_all_conf(raw.ctx()).await?;
/// ```
///
/// `&mut` больше не нужен: методы прокси берут `&self`.
pub struct RawServer {
    prx: ServerPrx,
    shared: Arc<Shared>,
}

impl RawServer {
    pub(crate) fn new(prx: ServerPrx, shared: Arc<Shared>) -> RawServer {
        RawServer { prx, shared }
    }

    /// Ice-контекст (секрет и прочее) — последний аргумент любой операции.
    pub fn ctx(&self) -> Option<HashMap<String, String>> {
        self.shared.ctx()
    }
}

impl Deref for RawServer {
    type Target = ServerPrx;
    fn deref(&self) -> &ServerPrx {
        &self.prx
    }
}

/// Одолженный прокси `Meta`.
pub struct RawMeta {
    prx: MetaPrx,
    shared: Arc<Shared>,
}

impl RawMeta {
    pub(crate) fn new(prx: MetaPrx, shared: Arc<Shared>) -> RawMeta {
        RawMeta { prx, shared }
    }

    pub fn ctx(&self) -> Option<HashMap<String, String>> {
        self.shared.ctx()
    }
}

impl Deref for RawMeta {
    type Target = MetaPrx;
    fn deref(&self) -> &MetaPrx {
        &self.prx
    }
}
