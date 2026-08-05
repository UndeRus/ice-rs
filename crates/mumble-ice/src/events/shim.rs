//! Мосты между сгенерированными servant-трейтами и трейтами фасада.
//!
//! Сгенерированные `*I` берут `&mut self` и возвращают голые значения. Мост
//! реализует их и **спавнит** пользовательскую future, возвращаясь сразу: так
//! мьютекс вокруг servant'а не удерживается на время работы обработчика, а
//! `MumbleServer.ice` прямо говорит, что колбеки асинхронные и сервер ответа не
//! ждёт.

use super::{
    ContextHandler, ContextInvocation, Registration, Registry, ServerEvents, Subscription,
    SubscriptionState,
};
use crate::error::{Error, Result};
use crate::ids::{ChannelId, SessionId};
use crate::model::{Channel, TextMessage, User};
use crate::server::VirtualServer;
use async_trait::async_trait;
use murmur_slice::mumble_server::{
    self as slice, ServerCallbackI, ServerCallbackServer, ServerContextCallbackI,
    ServerContextCallbackServer,
};
use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

type IceCtx = Option<HashMap<String, String>>;

/// Запускает обработчик так, чтобы ни ошибка, ни паника не утекли.
///
/// Murmur снимает регистрацию колбека, если тот бросил исключение, — то есть
/// одна паника в обработчике оставила бы бота глухим навсегда. Поэтому Murmur'у
/// мы отвечаем «ок» всегда, а о проблеме сообщаем через `on_error`.
///
/// Панику ловит `tokio::spawn`: она приходит как `Err(JoinError)`, поэтому свой
/// `catch_unwind` не нужен.
fn spawn_guarded<F, R>(op: &'static str, reporter: Arc<R>, fut: F)
where
    F: Future<Output = Result<()>> + Send + 'static,
    R: ErrorSink + ?Sized + 'static,
{
    tokio::spawn(async move {
        match tokio::spawn(fut).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => reporter.report(e).await,
            Err(join) => {
                let msg = if join.is_panic() {
                    format!("паника в обработчике {}", op)
                } else {
                    format!("обработчик {} отменён", op)
                };
                reporter.report(Error::Protocol(msg)).await;
            }
        }
    });
}

/// Куда сообщать об ошибке обработчика.
#[async_trait]
pub(crate) trait ErrorSink: Send + Sync {
    async fn report(&self, err: Error);
}

#[async_trait]
impl ErrorSink for dyn ServerEvents {
    async fn report(&self, err: Error) {
        self.on_error(err).await
    }
}

#[async_trait]
impl ErrorSink for dyn ContextHandler {
    async fn report(&self, err: Error) {
        self.on_error(err).await
    }
}

#[async_trait]
impl ErrorSink for dyn super::MetaEvents {
    async fn report(&self, err: Error) {
        self.on_error(err).await
    }
}

/// Мост `ServerCallback`.
pub(crate) struct EventShim {
    pub(crate) server: VirtualServer,
    pub(crate) handler: Arc<dyn ServerEvents>,
}

macro_rules! fanout {
    ($self:expr, $op:literal, $method:ident, $arg:expr) => {{
        let srv = $self.server.clone();
        let h = $self.handler.clone();
        let arg = $arg;
        spawn_guarded($op, h.clone(), async move { h.$method(&srv, arg).await });
    }};
}

#[async_trait]
impl ServerCallbackI for EventShim {
    async fn user_connected(&mut self, state: &slice::User, _ctx: IceCtx) {
        fanout!(self, "user_connected", user_connected, User::from(state))
    }
    async fn user_disconnected(&mut self, state: &slice::User, _ctx: IceCtx) {
        fanout!(self, "user_disconnected", user_disconnected, User::from(state))
    }
    async fn user_state_changed(&mut self, state: &slice::User, _ctx: IceCtx) {
        fanout!(
            self,
            "user_state_changed",
            user_state_changed,
            User::from(state)
        )
    }
    async fn user_text_message(
        &mut self,
        state: &slice::User,
        message: &slice::TextMessage,
        _ctx: IceCtx,
    ) {
        let srv = self.server.clone();
        let h = self.handler.clone();
        let user = User::from(state);
        let msg = TextMessage::from(message);
        spawn_guarded("user_text_message", h.clone(), async move {
            h.user_text_message(&srv, user, msg).await
        });
    }
    async fn channel_created(&mut self, state: &slice::Channel, _ctx: IceCtx) {
        fanout!(self, "channel_created", channel_created, Channel::from(state))
    }
    async fn channel_removed(&mut self, state: &slice::Channel, _ctx: IceCtx) {
        fanout!(self, "channel_removed", channel_removed, Channel::from(state))
    }
    async fn channel_state_changed(&mut self, state: &slice::Channel, _ctx: IceCtx) {
        fanout!(
            self,
            "channel_state_changed",
            channel_state_changed,
            Channel::from(state)
        )
    }
}

/// Мост `ServerContextCallback`.
pub(crate) struct ContextShim {
    pub(crate) server: VirtualServer,
    pub(crate) handler: Arc<dyn ContextHandler>,
}

#[async_trait]
impl ServerContextCallbackI for ContextShim {
    async fn context_action(
        &mut self,
        action: &String,
        usr: &slice::User,
        session: i32,
        channelid: i32,
        _ctx: IceCtx,
    ) {
        let srv = self.server.clone();
        let h = self.handler.clone();
        // Сентинелы из Slice: `session == 0` — «действие не над пользователем»,
        // `channelid == -1` — «не над каналом».
        let ev = ContextInvocation {
            action: action.clone(),
            by: User::from(usr),
            target_user: if session == 0 {
                None
            } else {
                Some(SessionId(session))
            },
            target_channel: if channelid < 0 {
                None
            } else {
                Some(ChannelId(channelid))
            },
        };
        spawn_guarded("context_action", h.clone(), async move {
            h.invoked(&srv, ev).await
        });
    }
}

/// Создаёт подписку: ставит servant в адаптер и отдаёт прокси Murmur'у.
pub(crate) async fn make_subscription(
    registry: &Arc<Registry>,
    server: &VirtualServer,
    registration: Registration,
) -> Result<Subscription> {
    let adapter = registry.ensure_adapter().await?;

    let kind = match &registration {
        Registration::ServerEvents(_) => "cb",
        Registration::Context { .. } => "ctx",
        Registration::Authenticator => "auth",
    };
    let ident = registry.next_ident(kind);

    let servant: Arc<dyn ice_rs::iceobject::Servant> = match &registration {
        Registration::ServerEvents(h) => ServerCallbackServer::new(Box::new(EventShim {
            server: server.clone(),
            handler: h.clone(),
        }))
        .into_servant(),
        Registration::Context { handler, .. } => {
            ServerContextCallbackServer::new(Box::new(ContextShim {
                server: server.clone(),
                handler: handler.clone(),
            }))
            .into_servant()
        }
        // Аутентификатор ставится своим путём (`auth::make_authenticator_subscription`).
        Registration::Authenticator => {
            return Err(crate::error::Error::config(
                "аутентификатор регистрируется через set_authenticator()",
            ))
        }
    };
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
        registration,
        dead: AtomicBool::new(false),
        death: Notify::new(),
        death_reason: Mutex::new(None),
    });

    register_in_murmur(&state).await?;
    registry.track(state.clone()).await;

    // Внутренний MetaCallback — чтобы вернуть подписки после перезапуска
    // виртуального сервера.
    super::install_meta_callback(registry, server).await;

    Ok(Subscription::new(state, registry.clone()))
}

/// Отдаёт Murmur'у прокси на наш servant.
pub(crate) async fn register_in_murmur(state: &Arc<SubscriptionState>) -> Result<()> {
    match &state.registration {
        Registration::ServerEvents(_) => {
            state
                .server
                .add_server_callback(&state.proxy_string)
                .await
        }
        Registration::Context { session, action, .. } => {
            state
                .server
                .add_context_callback(*session, action, &state.proxy_string)
                .await
        }
        Registration::Authenticator => {
            state.server.set_authenticator_proxy(&state.proxy_string).await
        }
    }
}

/// Просит Murmur забыть прокси.
pub(crate) async fn remove_registration(state: &Arc<SubscriptionState>) -> Result<()> {
    match &state.registration {
        Registration::ServerEvents(_) => {
            state
                .server
                .remove_server_callback(&state.proxy_string)
                .await
        }
        Registration::Context { .. } => {
            state
                .server
                .remove_context_callback(&state.proxy_string)
                .await
        }
        // «Снять аутентификатор» в Slice отсутствует: есть только «поставить».
        // Отдаём аутентификацию обратно Murmur'у, поставив пустой.
        Registration::Authenticator => state.server.clear_authenticator().await,
    }
}

/// Сообщает обработчику, что подписка восстановлена после перезапуска.
pub(crate) async fn notify_reattached(state: &Arc<SubscriptionState>) {
    if let Registration::ServerEvents(h) = &state.registration {
        let srv = state.server.clone();
        let h = h.clone();
        spawn_guarded("reattached", h.clone(), async move {
            h.reattached(&srv).await
        });
    }
}
