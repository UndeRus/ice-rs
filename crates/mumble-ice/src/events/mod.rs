//! Колбеки: подписка на события Murmur.
//!
//! Первичная форма — трейт с дефолтными реализациями, а не набор замыканий.
//! Причины конкретные: хендлер бота почти всегда хочет `&VirtualServer` (чтобы
//! ответить, переместить, проверить права), состояние бота — одна структура, а
//! `ServerCallback` в Slice и так один Ice-объект с семью операциями. С
//! замыканиями каждый из семи обработчиков независимо клонировал бы `Arc` на
//! сервер и на состояние — тот самый бойлерплейт, от которого этот крейт
//! избавляет.
//!
//! Дополнительно есть [`EventStream`] — тот же трейт, обёрнутый в канал, для
//! ботов, у которых главный цикл уже `select!`-ится.

mod meta;
mod shim;
mod stream;

pub use stream::{Event, EventStream, Overflow};

use crate::error::{Error, Result};
use crate::ids::{ChannelId, ServerId, SessionId};
use crate::model::{Channel, TextMessage, User};
use crate::perm::ContextFlags;
use crate::server::VirtualServer;
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

/// События виртуального сервера.
///
/// У всех методов есть дефолтная реализация, поэтому бот, которому нужны только
/// текстовые сообщения, пишет ровно один метод.
///
/// # Ошибки не глотаются
///
/// Исключение из колбека заставляет Murmur **молча снять регистрацию целиком** —
/// худший из возможных отказов. Поэтому `Err` уходит в [`on_error`], а Murmur'у
/// всё равно отвечаем «ок»; паника хендлера тоже перехватывается.
///
/// [`on_error`]: Self::on_error
#[async_trait]
pub trait ServerEvents: Send + Sync + 'static {
    async fn user_connected(&self, _srv: &VirtualServer, _user: User) -> Result<()> {
        Ok(())
    }
    async fn user_disconnected(&self, _srv: &VirtualServer, _user: User) -> Result<()> {
        Ok(())
    }
    async fn user_state_changed(&self, _srv: &VirtualServer, _user: User) -> Result<()> {
        Ok(())
    }
    async fn user_text_message(
        &self,
        _srv: &VirtualServer,
        _user: User,
        _message: TextMessage,
    ) -> Result<()> {
        Ok(())
    }
    async fn channel_created(&self, _srv: &VirtualServer, _channel: Channel) -> Result<()> {
        Ok(())
    }
    async fn channel_removed(&self, _srv: &VirtualServer, _channel: Channel) -> Result<()> {
        Ok(())
    }
    async fn channel_state_changed(&self, _srv: &VirtualServer, _channel: Channel) -> Result<()> {
        Ok(())
    }

    /// Murmur перезапустил виртуальный сервер и снял наши колбеки, а фасад их
    /// уже вернул.
    ///
    /// К этому моменту **все закэшированные `SessionId` — мусор**: их выдаёт
    /// соединение, а соединения все оборвались.
    async fn reattached(&self, _srv: &VirtualServer) -> Result<()> {
        Ok(())
    }

    /// Сюда попадает всё, что вернули методы выше, плюс паники и ошибки
    /// декодирования.
    async fn on_error(&self, err: Error) {
        eprintln!("mumble-ice: ошибка в обработчике события: {}", err);
    }
}

/// События уровня `Meta`: запуск и остановка виртуальных серверов.
#[async_trait]
pub trait MetaEvents: Send + Sync + 'static {
    async fn server_started(&self, _id: ServerId) -> Result<()> {
        Ok(())
    }
    async fn server_stopped(&self, _id: ServerId) -> Result<()> {
        Ok(())
    }
    async fn on_error(&self, err: Error) {
        eprintln!("mumble-ice: ошибка в обработчике Meta-события: {}", err);
    }
}

/// Контекстное действие, вызванное пользователем из меню Mumble.
///
/// Не `Eq`: внутри `User` есть пинги (`f32`).
#[derive(Debug, Clone, PartialEq)]
pub struct ContextInvocation {
    pub action: String,
    /// Кто вызвал.
    pub by: User,
    /// Цель-пользователь, если действие было над пользователем.
    pub target_user: Option<SessionId>,
    /// Цель-канал, если действие было над каналом.
    pub target_channel: Option<ChannelId>,
}

#[async_trait]
pub trait ContextHandler: Send + Sync + 'static {
    async fn invoked(&self, srv: &VirtualServer, ev: ContextInvocation) -> Result<()>;
    async fn on_error(&self, err: Error) {
        eprintln!("mumble-ice: ошибка в обработчике контекстного действия: {}", err);
    }
}

/// Описание контекстного действия для меню Mumble.
#[derive(Debug, Clone)]
pub struct ContextAction {
    /// Ключ, который вернётся в обработчик.
    pub action: String,
    /// Подпись, которую видит пользователь.
    pub text: String,
    pub contexts: ContextFlags,
}

impl ContextAction {
    pub fn new(action: impl Into<String>, text: impl Into<String>, contexts: ContextFlags) -> Self {
        ContextAction {
            action: action.into(),
            text: text.into(),
            contexts,
        }
    }
}

/// Что именно зарегистрировано в Murmur — нужно, чтобы переподписать это же
/// после перезапуска виртуального сервера.
pub(crate) enum Registration {
    ServerEvents(Arc<dyn ServerEvents>),
    Context {
        session: SessionId,
        action: ContextAction,
        handler: Arc<dyn ContextHandler>,
    },
    /// Аутентификатор. Переподписывается так же, как колбеки: перезапуск
    /// виртуального сервера снимает и его, а это значит, что все пользователи
    /// внезапно начинают проверяться по локальной базе Murmur'а.
    Authenticator,
}

pub(crate) struct SubscriptionState {
    pub(crate) id: u64,
    pub(crate) ident: String,
    /// Прокси-строка, отданная Murmur'у. Ice сравнивает прокси целиком, включая
    /// эндпоинт, поэтому для снятия регистрации нужна ровно та же строка.
    pub(crate) proxy_string: String,
    pub(crate) server: VirtualServer,
    pub(crate) registration: Registration,
    /// Подписка снята (пользователем или безвозвратно умерла).
    pub(crate) dead: AtomicBool,
    pub(crate) death: Notify,
    pub(crate) death_reason: Mutex<Option<String>>,
}

/// Живая подписка.
///
/// Пока значение живо — колбек зарегистрирован; при `drop` регистрация снимается.
#[must_use = "если бросить Subscription, колбек будет снят"]
pub struct Subscription {
    pub(crate) state: Arc<SubscriptionState>,
    pub(crate) registry: Arc<Registry>,
    /// `forget()` выставляет это, чтобы `drop` не снимал регистрацию.
    forgotten: bool,
}

impl Subscription {
    pub(crate) fn new(state: Arc<SubscriptionState>, registry: Arc<Registry>) -> Subscription {
        Subscription {
            state,
            registry,
            forgotten: false,
        }
    }

    /// Оставить подписку на весь срок жизни процесса, не держа значение.
    pub fn forget(mut self) {
        self.forgotten = true;
    }

    /// Снять подписку, сообщив об ошибке. `drop` делает это молча.
    pub async fn unsubscribe(mut self) -> Result<()> {
        self.forgotten = true;
        self.registry.remove(&self.state).await
    }

    /// Разрешается, когда подписка умерла безвозвратно.
    ///
    /// Позволяет боту `select!`-иться на этом вместо бесконечного сна и не
    /// работать вглухую.
    pub async fn closed(&self) -> Error {
        loop {
            if self.state.dead.load(Ordering::Acquire) {
                let reason = self.state.death_reason.lock().await.clone();
                return Error::Protocol(
                    reason.unwrap_or_else(|| String::from("подписка снята")),
                );
            }
            self.state.death.notified().await;
        }
    }

    pub fn is_alive(&self) -> bool {
        !self.state.dead.load(Ordering::Acquire)
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if self.forgotten || self.state.dead.load(Ordering::Acquire) {
            return;
        }
        // Снятие регистрации — сетевой вызов, а `drop` синхронный. Отправляем в
        // фон: это best-effort, для явного снятия с отчётом есть
        // `unsubscribe()`.
        let state = self.state.clone();
        let registry = self.registry.clone();
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::spawn(async move {
                let _ = registry.remove(&state).await;
            });
        }
    }
}

impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription")
            .field("ident", &self.state.ident)
            .field("alive", &self.is_alive())
            .finish()
    }
}

/// Реестр подписок и владелец Ice-адаптера.
///
/// Адаптер поднимается лениво — на первой подписке, — но адрес проверяется ещё
/// на `connect()`: иначе плохая конфигурация всплывёт секундами позже как
/// невнятный `InvalidCallback` от Murmur'а.
pub(crate) struct Registry {
    shared: Arc<crate::client::Shared>,
    adapter: Mutex<Option<Arc<AdapterState>>>,
    subs: Mutex<Vec<Arc<SubscriptionState>>>,
    next_id: AtomicU64,
    /// Зарегистрирован ли наш внутренний `MetaCallback` (нужен для переподписки).
    meta_hooked: AtomicBool,
    meta_handlers: Mutex<Vec<Arc<dyn MetaEvents>>>,
}

pub(crate) struct AdapterState {
    pub(crate) adapter: Arc<ice_rs::adapter::Adapter>,
    handle: Mutex<Option<ice_rs::adapter::AdapterHandle>>,
    /// Адрес, который объявляется Murmur'у.
    pub(crate) advertise: (String, u16),
}

impl Registry {
    pub(crate) fn new(shared: Arc<crate::client::Shared>) -> Registry {
        Registry {
            shared,
            adapter: Mutex::new(None),
            subs: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
            meta_hooked: AtomicBool::new(false),
            meta_handlers: Mutex::new(Vec::new()),
        }
    }

    /// Поднимает адаптер, если он ещё не поднят.
    pub(crate) async fn ensure_adapter(&self) -> Result<Arc<AdapterState>> {
        let mut guard = self.adapter.lock().await;
        if let Some(a) = guard.as_ref() {
            return Ok(a.clone());
        }

        let listen = self.shared.callback_listen;

        // Эфемерный порт разрешаем ДО создания адаптера: прокси, который уедет
        // Murmur'у, должен нести настоящий порт, а `advertise()` требует `&mut`,
        // то есть задать его после упаковки в `Arc` уже нельзя.
        let port = if listen.port() == 0 {
            let probe = tokio::net::TcpListener::bind((listen.ip(), 0))
                .await
                .map_err(|e| Error::config(format!("не смог занять порт на {}: {}", listen.ip(), e)))?;
            let p = probe
                .local_addr()
                .map_err(|e| Error::config(format!("local_addr: {}", e)))?
                .port();
            drop(probe);
            p
        } else {
            listen.port()
        };

        let mut adapter = ice_rs::adapter::Adapter::with_endpoint(
            "mumble-ice",
            &format!("tcp -h {} -p {}", listen.ip(), port),
        )
        .map_err(|e| Error::config(format!("не смог создать адаптер на {}: {}", listen, e)))?;

        // Адрес объявления. Wildcard адаптер объявлять откажется сам — на
        // `connect()` мы это уже проверили и подсказали, что делать.
        let advertise = match &self.shared.callback_advertise {
            Some((h, p)) => {
                adapter.advertise(h, *p as i32);
                (h.clone(), *p)
            }
            None => (listen.ip().to_string(), port),
        };

        let adapter = Arc::new(adapter);
        let handle = adapter
            .serve()
            .await
            .map_err(|e| Error::config(format!("не смог слушать на {}:{}: {}", listen.ip(), port, e)))?;

        let state = Arc::new(AdapterState {
            adapter,
            handle: Mutex::new(Some(handle)),
            advertise,
        });
        *guard = Some(state.clone());
        Ok(state)
    }

    /// Следующий внутренний номер подписки.
    pub(crate) fn take_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) fn next_ident(&self, kind: &str) -> String {
        let n = self.next_id.fetch_add(1, Ordering::Relaxed);
        format!("{}-{}-{}", self.shared.callback_identity_prefix, kind, n)
    }

    pub(crate) async fn track(&self, state: Arc<SubscriptionState>) {
        self.subs.lock().await.push(state);
    }

    pub(crate) async fn add_meta_handler(&self, h: Arc<dyn MetaEvents>) {
        self.meta_handlers.lock().await.push(h);
    }

    pub(crate) async fn meta_handlers(&self) -> Vec<Arc<dyn MetaEvents>> {
        self.meta_handlers.lock().await.clone()
    }

    pub(crate) fn meta_hooked(&self) -> bool {
        self.meta_hooked.load(Ordering::Acquire)
    }

    pub(crate) fn set_meta_hooked(&self) {
        self.meta_hooked.store(true, Ordering::Release);
    }

    /// Снимает регистрацию в Murmur и убирает подписку из реестра.
    pub(crate) async fn remove(&self, state: &Arc<SubscriptionState>) -> Result<()> {
        if state.dead.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        state.death.notify_waiters();
        self.subs.lock().await.retain(|s| s.id != state.id);

        let adapter = self.adapter.lock().await.clone();
        if let Some(a) = adapter {
            let _ = a
                .adapter
                .unregister(&ice_rs::adapter::ServantKey::new(&state.ident))
                .await;
        }
        // Просим Murmur забыть прокси. Если сервер уже перезапустился, он и так
        // всё снял — ошибку здесь глотаем осознанно.
        let _ = shim::remove_registration(state).await;
        Ok(())
    }

    /// Все живые подписки на указанном сервере.
    pub(crate) async fn live_for(&self, id: ServerId) -> Vec<Arc<SubscriptionState>> {
        self.subs
            .lock()
            .await
            .iter()
            .filter(|s| !s.dead.load(Ordering::Acquire) && s.server.id() == id)
            .cloned()
            .collect()
    }

    /// Переподписывает всё живое на сервере — вызывается из `MetaCallback::started`.
    ///
    /// `MumbleServer.ice` прямо предупреждает: при остановке виртуального сервера
    /// колбеки снимаются, и вернуть их — забота клиента. Каждый Python-скрипт для
    /// Murmur переписывает это заново; здесь это делается один раз.
    pub(crate) async fn reattach(&self, id: ServerId) {
        let subs = self.live_for(id).await;
        if subs.is_empty() {
            return;
        }
        // Адаптер уже поднят (без него подписок бы не было), servant'ы в нём
        // остались — заново нужен только addCallback на стороне Murmur'а.
        for s in subs {
            // Идентичность сохраняется, поэтому повторный addCallback не
            // удваивает события: Murmur ключует колбеки по прокси.
            if let Err(e) = shim::register_in_murmur(&s).await {
                let mut reason = s.death_reason.lock().await;
                *reason = Some(format!("не удалось переподписаться: {}", e));
                drop(reason);
                s.dead.store(true, Ordering::Release);
                s.death.notify_waiters();
                continue;
            }
            shim::notify_reattached(&s).await;
        }
    }

    pub(crate) async fn shutdown(&self) {
        let subs: Vec<_> = self.subs.lock().await.drain(..).collect();
        for s in subs {
            let _ = self.remove(&s).await;
        }
        if let Some(a) = self.adapter.lock().await.take() {
            if let Some(h) = a.handle.lock().await.take() {
                h.shutdown().await;
            }
        }
    }
}

pub(crate) use meta::install_meta_callback;
pub(crate) use shim::{make_subscription, register_in_murmur as register_subscription_in_murmur};
pub(crate) use stream::StreamBridge;
