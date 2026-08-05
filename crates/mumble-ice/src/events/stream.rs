//! Поток событий — вторичная форма API поверх [`ServerEvents`].
//!
//! Нужна ботам, у которых главный цикл уже `select!`-ится на чём-то ещё. Это
//! именно **производная** от трейта, а не вторая реализация: под капотом тот же
//! мост.

use super::{ContextInvocation, ServerEvents, Subscription};
use crate::error::{Error, Result};
use crate::ids::ServerId;
use crate::model::{Channel, TextMessage, User};
use crate::server::VirtualServer;
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Событие сервера.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Event {
    UserConnected(User),
    UserDisconnected(User),
    UserStateChanged(User),
    UserTextMessage { user: User, message: TextMessage },
    ChannelCreated(Channel),
    ChannelRemoved(Channel),
    ChannelStateChanged(Channel),
    ContextAction(ContextInvocation),
    ServerStarted(ServerId),
    ServerStopped(ServerId),
    /// Колбеки переподписаны после перезапуска виртуального сервера.
    ///
    /// **Все закэшированные `SessionId` после этого мусор** — их выдаёт
    /// соединение, а соединения оборвались. Отдельный вариант, а не строчка в
    /// логе, именно поэтому.
    Reattached,
    /// Канал был переполнен и `n` событий потеряно. Сообщаем, а не молчим.
    Lagged { dropped: usize },
    /// Неисправимая ошибка в моте или обработчике.
    Error(String),
}

/// Что делать при переполнении канала.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    /// Выбрасывать новые события и считать потери (по умолчанию).
    DropNewest,
    /// Ждать место в канале.
    ///
    /// Для колбеков это допустимо (Murmur их не ждёт), но затормозит наш
    /// servant.
    Backpressure,
}

/// Приёмник событий.
///
/// Пока значение живо — подписка жива.
pub struct EventStream {
    rx: mpsc::Receiver<Event>,
    _sub: Subscription,
    bridge: Arc<StreamBridge>,
}

impl EventStream {
    pub async fn recv(&mut self) -> Option<Event> {
        // Сначала отдаём накопленную потерю, чтобы бот узнал о ней вовремя.
        let dropped = self.bridge.dropped.swap(0, Ordering::AcqRel);
        if dropped > 0 {
            return Some(Event::Lagged { dropped });
        }
        self.rx.recv().await
    }

    /// Забрать событие, если оно уже есть. `None` — пока ничего.
    ///
    /// В tokio 1.1 `Receiver::try_recv` не публичен, поэтому реализовано через
    /// `recv()` с нулевым таймаутом.
    pub async fn try_recv(&mut self) -> Option<Event> {
        let dropped = self.bridge.dropped.swap(0, Ordering::AcqRel);
        if dropped > 0 {
            return Some(Event::Lagged { dropped });
        }
        match tokio::time::timeout(std::time::Duration::ZERO, self.rx.recv()).await {
            Ok(v) => v,
            Err(_) => None,
        }
    }

    /// Снять подписку явно, с отчётом об ошибке.
    pub async fn unsubscribe(self) -> Result<()> {
        self._sub.unsubscribe().await
    }
}

impl std::fmt::Debug for EventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventStream").finish()
    }
}

pub(crate) struct StreamBridge {
    tx: mpsc::Sender<Event>,
    dropped: AtomicUsize,
    policy: Overflow,
}

impl StreamBridge {
    pub(crate) fn new(capacity: usize, policy: Overflow) -> (Arc<StreamBridge>, mpsc::Receiver<Event>) {
        let (tx, rx) = mpsc::channel(capacity.max(1));
        (
            Arc::new(StreamBridge {
                tx,
                dropped: AtomicUsize::new(0),
                policy,
            }),
            rx,
        )
    }

    pub(crate) fn into_stream(self: Arc<Self>, rx: mpsc::Receiver<Event>, sub: Subscription) -> EventStream {
        EventStream {
            rx,
            _sub: sub,
            bridge: self,
        }
    }

    async fn emit(&self, ev: Event) {
        match self.policy {
            Overflow::Backpressure => {
                let _ = self.tx.send(ev).await;
            }
            Overflow::DropNewest => {
                if self.tx.try_send(ev).is_err() {
                    self.dropped.fetch_add(1, Ordering::AcqRel);
                }
            }
        }
    }
}

#[async_trait]
impl ServerEvents for StreamBridge {
    async fn user_connected(&self, _srv: &VirtualServer, user: User) -> Result<()> {
        self.emit(Event::UserConnected(user)).await;
        Ok(())
    }
    async fn user_disconnected(&self, _srv: &VirtualServer, user: User) -> Result<()> {
        self.emit(Event::UserDisconnected(user)).await;
        Ok(())
    }
    async fn user_state_changed(&self, _srv: &VirtualServer, user: User) -> Result<()> {
        self.emit(Event::UserStateChanged(user)).await;
        Ok(())
    }
    async fn user_text_message(
        &self,
        _srv: &VirtualServer,
        user: User,
        message: TextMessage,
    ) -> Result<()> {
        self.emit(Event::UserTextMessage { user, message }).await;
        Ok(())
    }
    async fn channel_created(&self, _srv: &VirtualServer, channel: Channel) -> Result<()> {
        self.emit(Event::ChannelCreated(channel)).await;
        Ok(())
    }
    async fn channel_removed(&self, _srv: &VirtualServer, channel: Channel) -> Result<()> {
        self.emit(Event::ChannelRemoved(channel)).await;
        Ok(())
    }
    async fn channel_state_changed(&self, _srv: &VirtualServer, channel: Channel) -> Result<()> {
        self.emit(Event::ChannelStateChanged(channel)).await;
        Ok(())
    }
    async fn reattached(&self, _srv: &VirtualServer) -> Result<()> {
        self.emit(Event::Reattached).await;
        Ok(())
    }
    async fn on_error(&self, err: Error) {
        self.emit(Event::Error(err.to_string())).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(name: &str) -> User {
        User {
            session: crate::ids::SessionId(1),
            id: None,
            name: String::from(name),
            channel: crate::ids::ChannelId::ROOT,
            mute: false,
            deaf: false,
            suppress: false,
            priority_speaker: false,
            self_mute: false,
            self_deaf: false,
            recording: false,
            online: std::time::Duration::ZERO,
            idle: std::time::Duration::ZERO,
            bytes_per_sec: 0,
            client: Default::default(),
            release: String::new(),
            os: String::new(),
            os_version: String::new(),
            identity: String::new(),
            plugin_context: String::new(),
            comment: String::new(),
            address: None,
            udp: true,
            udp_ping: 0.0,
            tcp_ping: 0.0,
        }
    }

    #[tokio::test]
    async fn events_reach_the_channel() {
        let (bridge, mut rx) = StreamBridge::new(4, Overflow::DropNewest);
        bridge.emit(Event::UserConnected(user("alice"))).await;
        match rx.recv().await.unwrap() {
            Event::UserConnected(u) => assert_eq!("alice", u.name),
            other => panic!("не то событие: {:?}", other),
        }
    }

    /// Потери должны быть видны боту, а не проглатываться.
    #[tokio::test]
    async fn overflow_is_counted_and_reported() {
        let (bridge, rx) = StreamBridge::new(1, Overflow::DropNewest);
        bridge.emit(Event::Reattached).await; // занимает единственное место
        bridge.emit(Event::Reattached).await; // теряется
        bridge.emit(Event::Reattached).await; // теряется
        assert_eq!(2, bridge.dropped.load(Ordering::Acquire));

        // Первым делом поток обязан сообщить о потере.
        let sub_dropped = bridge.dropped.swap(0, Ordering::AcqRel);
        assert_eq!(2, sub_dropped);
        drop(rx);
    }

    #[tokio::test]
    async fn error_becomes_an_event_not_a_silence() {
        let (bridge, mut rx) = StreamBridge::new(4, Overflow::DropNewest);
        ServerEvents::on_error(&*bridge, Error::ReadOnlyMode).await;
        match rx.recv().await.unwrap() {
            Event::Error(msg) => assert!(msg.contains("только для чтения"), "{}", msg),
            other => panic!("не то событие: {:?}", other),
        }
    }
}
