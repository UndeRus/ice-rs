//! Ошибки фасада.
//!
//! Сгенерированный слой отдаёт всё как `Box<dyn Error + Send + Sync>`, так что
//! отличить «Murmur не запущен» от «нет такой сессии» можно было только
//! сравнением строк. Здесь это разложено по вариантам, по которым бот может
//! ветвиться.

use crate::ids::{ChannelId, ServerId, SessionId, UserId};
use std::time::Duration;

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Контекст вызова: подставляет в вариант ошибки тот идентификатор, который
/// вызывающий только что передал.
///
/// Нужен потому, что Ice-исключения Murmur'а не несут в себе, о каком объекте
/// речь: `InvalidSessionException` — это просто пустая структура.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FaultContext {
    pub server: Option<ServerId>,
    pub session: Option<SessionId>,
    pub channel: Option<ChannelId>,
    pub user: Option<UserId>,
}

impl FaultContext {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub(crate) fn server(mut self, v: ServerId) -> Self {
        self.server = Some(v);
        self
    }
    pub(crate) fn session(mut self, v: SessionId) -> Self {
        self.session = Some(v);
        self
    }
    pub(crate) fn channel(mut self, v: ChannelId) -> Self {
        self.channel = Some(v);
        self
    }
    pub(crate) fn user(mut self, v: UserId) -> Self {
        self.user = Some(v);
        self
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    // ── Murmur ответил отказом ────────────────────────────────────────────
    /// Секрет Ice отклонён: проверьте `icesecretread`/`icesecretwrite`.
    InvalidSecret,
    ServerNotRunning(Option<ServerId>),
    ServerFailure,
    InvalidSession(Option<SessionId>),
    InvalidChannel(Option<ChannelId>),
    InvalidUser(Option<UserId>),
    InvalidServer(Option<ServerId>),
    ReadOnlyMode,
    NestingLimit,
    InvalidTexture,
    /// Murmur не смог использовать наш callback-прокси. Почти всегда это значит,
    /// что он не может дозвониться до объявленного адреса.
    InvalidCallback {
        advertised: Option<String>,
    },
    /// Значение только для записи (например, пароль в конфиге).
    WriteOnly,
    InvalidInputData,
    InvalidListener,
    MurmurInternal,
    /// Исключение Murmur'а, которое мы не моделируем (например, из будущей версии).
    UnknownFault {
        type_id: String,
    },
    /// Операция объявлена в Slice, но этот сервер её не реализует.
    ///
    /// Так бывает: `MumbleServer.ice` описывает больше, чем умеет конкретный
    /// билд. Например, Murmur 1.5.857 отвечает `OperationNotExist` на
    /// `getAssumedDatabaseState`. Отдельный вариант нужен, чтобы бот мог
    /// деградировать, а не падать.
    OperationNotSupported {
        operation: String,
    },

    // ── наша сторона ──────────────────────────────────────────────────────
    /// Ни один виртуальный сервер не запущен.
    NoBootedServer,
    Transport(BoxError),
    Timeout {
        op: &'static str,
        after: Duration,
    },
    Protocol(String),
    Config(String),
    /// Сюда попадает всё, что не удалось разложить по вариантам выше.
    Other(BoxError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidSecret => write!(
                f,
                "Ice-секрет отклонён Murmur'ом (проверьте icesecretread/icesecretwrite)"
            ),
            Error::ServerNotRunning(id) => match id {
                Some(id) => write!(f, "виртуальный сервер {} не запущен", id),
                None => write!(f, "виртуальный сервер не запущен"),
            },
            Error::ServerFailure => write!(f, "виртуальный сервер не смог запуститься"),
            Error::InvalidSession(s) => match s {
                Some(s) => write!(f, "сессия {} больше не подключена", s),
                None => write!(f, "сессия больше не подключена"),
            },
            Error::InvalidChannel(c) => match c {
                Some(c) => write!(f, "канала {} не существует", c),
                None => write!(f, "канала не существует"),
            },
            Error::InvalidUser(u) => match u {
                Some(u) => write!(f, "пользователь {} не зарегистрирован", u),
                None => write!(f, "пользователь не зарегистрирован"),
            },
            Error::InvalidServer(id) => match id {
                Some(id) => write!(f, "виртуального сервера {} не существует", id),
                None => write!(f, "виртуального сервера не существует"),
            },
            Error::ReadOnlyMode => write!(f, "база Murmur в режиме только для чтения"),
            Error::NestingLimit => write!(f, "превышен предел вложенности каналов"),
            Error::InvalidTexture => write!(f, "некорректная текстура (аватар)"),
            Error::InvalidCallback { advertised } => match advertised {
                Some(a) => write!(
                    f,
                    "Murmur отклонил callback-прокси — проверьте, что он может дозвониться до {}",
                    a
                ),
                None => write!(f, "Murmur отклонил callback-прокси"),
            },
            Error::WriteOnly => write!(f, "Murmur не раскрывает это значение (только запись)"),
            Error::InvalidInputData => write!(f, "некорректные входные данные"),
            Error::InvalidListener => write!(f, "такого слушателя канала нет"),
            Error::MurmurInternal => write!(f, "внутренняя ошибка Murmur"),
            Error::UnknownFault { type_id } => {
                write!(f, "неизвестное исключение Murmur: {}", type_id)
            }
            Error::OperationNotSupported { operation } => write!(
                f,
                "этот Murmur не реализует операцию '{}' (она есть в Slice, но не в сборке)",
                operation
            ),
            Error::NoBootedServer => write!(f, "нет ни одного запущенного виртуального сервера"),
            Error::Transport(e) => write!(f, "ошибка транспорта: {}", e),
            Error::Timeout { op, after } => write!(f, "{} не ответил за {:?}", op, after),
            Error::Protocol(m) => write!(f, "ошибка протокола: {}", m),
            Error::Config(m) => write!(f, "ошибка конфигурации: {}", m),
            Error::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Transport(e) | Error::Other(e) => Some(&**e),
            _ => None,
        }
    }
}

impl Error {
    pub fn config(msg: impl Into<String>) -> Error {
        Error::Config(msg.into())
    }

    /// Имеет смысл повторить тот же вызов.
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Error::Transport(_) | Error::Timeout { .. } | Error::ServerNotRunning(_)
        )
    }

    /// Закэшированный идентификатор устарел — надо перезапросить состояние.
    ///
    /// После перезапуска виртуального сервера все `SessionId` становятся мусором,
    /// и это самый частый источник таких ошибок.
    pub fn is_stale_handle(&self) -> bool {
        matches!(
            self,
            Error::InvalidSession(_) | Error::InvalidChannel(_) | Error::InvalidUser(_)
        )
    }

    /// Проблема на уровне соединения: соединение стоит считать испорченным.
    ///
    /// Пока фасад держит одно соединение и переиспользует его; станет
    /// востребованным, когда появится пул.
    pub fn is_connection_broken(&self) -> bool {
        matches!(self, Error::Transport(_) | Error::Timeout { .. })
    }
}

/// Разбирает ошибку нижнего слоя в вариант фасада.
///
/// Опирается на то, что после починки провода `Proxy::read_response` сохраняет
/// Slice type-id исключения в `RemoteUserException`. До этого сгенерированный код
/// разворачивал ответ в единственный захардкоженный тип, и, например,
/// `InvalidChannelException` приезжал как `ServerBootedException` — то есть два
/// разных отказа Murmur'а были в принципе неразличимы.
pub(crate) fn from_wire(e: BoxError, cx: FaultContext) -> Error {
    use ice_rs::errors::{RemoteUserException, RequestFailedException};

    if let Some(ux) = e.downcast_ref::<RemoteUserException>() {
        return map_fault(ux.short_name(), &ux.type_id, cx);
    }
    if let Some(rf) = e.downcast_ref::<RequestFailedException>() {
        use ice_rs::errors::RequestFailedKind;
        return match rf.kind {
            // Murmur описывает в Slice больше операций, чем реализует.
            RequestFailedKind::OperationNotExist => Error::OperationNotSupported {
                operation: rf.operation.clone(),
            },
            _ => Error::Protocol(format!("{}", rf)),
        };
    }
    if e.downcast_ref::<std::io::Error>().is_some() {
        return Error::Transport(e);
    }
    let msg = e.to_string();
    if msg.contains("Timeout waiting for response") {
        return Error::Timeout {
            op: "request",
            after: Duration::from_secs(30),
        };
    }
    if msg.contains("Connection died") || msg.contains("connection closed") {
        return Error::Transport(e);
    }
    Error::Other(e)
}

fn map_fault(short: &str, type_id: &str, cx: FaultContext) -> Error {
    match short {
        "InvalidSecretException" => Error::InvalidSecret,
        "ServerBootedException" => Error::ServerNotRunning(cx.server),
        "ServerFailureException" => Error::ServerFailure,
        "InvalidSessionException" => Error::InvalidSession(cx.session),
        "InvalidChannelException" => Error::InvalidChannel(cx.channel),
        "InvalidUserException" => Error::InvalidUser(cx.user),
        "InvalidServerException" => Error::InvalidServer(cx.server),
        "ReadOnlyModeException" => Error::ReadOnlyMode,
        "NestingLimitException" => Error::NestingLimit,
        "InvalidTextureException" => Error::InvalidTexture,
        "InvalidCallbackException" => Error::InvalidCallback { advertised: None },
        "WriteOnlyException" => Error::WriteOnly,
        "InvalidInputDataException" => Error::InvalidInputData,
        "InvalidListenerException" => Error::InvalidListener,
        "InternalErrorException" => Error::MurmurInternal,
        _ => Error::UnknownFault {
            type_id: String::from(type_id),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ice_rs::errors::RemoteUserException;

    fn ux(type_id: &str) -> BoxError {
        Box::new(RemoteUserException {
            type_id: String::from(type_id),
            payload: Vec::new(),
        })
    }

    #[test]
    fn maps_murmur_faults_by_type_id() {
        let cx = FaultContext::new().session(SessionId(42));
        assert!(matches!(
            from_wire(ux("::MumbleServer::InvalidSessionException"), cx),
            Error::InvalidSession(Some(SessionId(42)))
        ));
        assert!(matches!(
            from_wire(ux("::MumbleServer::InvalidSecretException"), FaultContext::new()),
            Error::InvalidSecret
        ));
        assert!(matches!(
            from_wire(ux("::MumbleServer::ReadOnlyModeException"), FaultContext::new()),
            Error::ReadOnlyMode
        ));
    }

    /// Раньше два разных исключения были неразличимы — теперь нет.
    #[test]
    fn different_faults_map_to_different_variants() {
        let a = from_wire(ux("::MumbleServer::InvalidChannelException"), FaultContext::new());
        let b = from_wire(ux("::MumbleServer::ServerBootedException"), FaultContext::new());
        assert!(matches!(a, Error::InvalidChannel(_)));
        assert!(matches!(b, Error::ServerNotRunning(_)));
    }

    #[test]
    fn unmodelled_fault_keeps_its_type_id() {
        match from_wire(ux("::MumbleServer::SomeFutureException"), FaultContext::new()) {
            Error::UnknownFault { type_id } => {
                assert_eq!("::MumbleServer::SomeFutureException", type_id)
            }
            other => panic!("ожидали UnknownFault, получили {:?}", other),
        }
    }

    /// Операция из Slice, которой нет в сборке сервера, должна давать отдельный
    /// вариант, а не «ошибку протокола».
    #[test]
    fn operation_not_exist_becomes_operation_not_supported() {
        use ice_rs::errors::{RequestFailedException, RequestFailedKind};
        let e: BoxError = Box::new(RequestFailedException {
            kind: RequestFailedKind::OperationNotExist,
            identity: String::from("Meta"),
            facet: vec![],
            operation: String::from("getAssumedDatabaseState"),
        });
        match from_wire(e, FaultContext::new()) {
            Error::OperationNotSupported { operation } => {
                assert_eq!("getAssumedDatabaseState", operation)
            }
            other => panic!("ожидали OperationNotSupported, получили {:?}", other),
        }
    }

    #[test]
    fn classification_helpers() {
        assert!(Error::ServerNotRunning(None).is_transient());
        assert!(Error::InvalidSession(None).is_stale_handle());
        assert!(!Error::InvalidSession(None).is_transient());
        assert!(!Error::ReadOnlyMode.is_stale_handle());
    }

    #[test]
    fn display_includes_the_identifier_when_known() {
        let e = Error::InvalidSession(Some(SessionId(7)));
        assert!(e.to_string().contains('7'), "{}", e);
        let e = Error::InvalidSession(None);
        assert!(!e.to_string().contains('7'));
    }
}
