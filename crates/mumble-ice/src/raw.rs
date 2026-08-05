//! Escape hatch к сгенерированному слою.
//!
//! Обёрнута основная часть операций; длинный хвост (редкие геттеры, будущие
//! операции Murmur 1.6) доступен здесь — с уже готовым Ice-контекстом, чтобы
//! секрет не приходилось собирать руками.

use crate::client::Shared;
use murmur_slice::mumble_server::{MetaPrx, ServerPrx};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::MutexGuard;

/// Одолженный прокси `Server`.
///
/// ```ignore
/// use murmur_slice::mumble_server::Server as _;
///
/// let mut raw = server.raw().await;
/// let ctx = raw.ctx();
/// let checksums = raw.get_all_conf(ctx).await?;
/// ```
pub struct RawServer<'a> {
    guard: MutexGuard<'a, ServerPrx>,
    shared: Arc<Shared>,
}

impl<'a> RawServer<'a> {
    pub(crate) fn new(guard: MutexGuard<'a, ServerPrx>, shared: Arc<Shared>) -> RawServer<'a> {
        RawServer { guard, shared }
    }

    /// Ice-контекст (секрет и прочее) — последний аргумент любой операции.
    pub fn ctx(&self) -> Option<HashMap<String, String>> {
        self.shared.ctx()
    }
}

impl<'a> Deref for RawServer<'a> {
    type Target = ServerPrx;
    fn deref(&self) -> &ServerPrx {
        &self.guard
    }
}

impl<'a> DerefMut for RawServer<'a> {
    fn deref_mut(&mut self) -> &mut ServerPrx {
        &mut self.guard
    }
}

/// Одолженный прокси `Meta`.
pub struct RawMeta<'a> {
    guard: MutexGuard<'a, MetaPrx>,
    shared: Arc<Shared>,
}

impl<'a> RawMeta<'a> {
    pub(crate) fn new(guard: MutexGuard<'a, MetaPrx>, shared: Arc<Shared>) -> RawMeta<'a> {
        RawMeta { guard, shared }
    }

    pub fn ctx(&self) -> Option<HashMap<String, String>> {
        self.shared.ctx()
    }
}

impl<'a> Deref for RawMeta<'a> {
    type Target = MetaPrx;
    fn deref(&self) -> &MetaPrx {
        &self.guard
    }
}

impl<'a> DerefMut for RawMeta<'a> {
    fn deref_mut(&mut self) -> &mut MetaPrx {
        &mut self.guard
    }
}
