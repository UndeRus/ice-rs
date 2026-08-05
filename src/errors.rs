use std::fmt::Display;
use crate::encoding::FromBytes;

/// A `ProtocolError` indicates a problem related to the
/// ice protocol. It may be unexpected messages or problems
/// while encoding or decoding objects.
#[derive(Debug)]
pub struct ProtocolError {
    detail: String
}

impl ProtocolError {
    pub fn new(detail: &str) -> ProtocolError {
        ProtocolError {
            detail: String::from(detail)
        }
    }
}

/// A `ParsingError` appears when a problem occurs parsing ice
/// files.
#[derive(Debug)]
pub struct ParsingError {
    detail: String
}

impl ParsingError {
    pub fn new(detail: &str) -> ParsingError {
        ParsingError {
            detail: String::from(detail)
        }
    }
}

/// A `PropertyError` appears when a requested property is not
/// existing.
#[derive(Debug)]
pub struct PropertyError {
    missing_key: String
}

impl PropertyError {
    pub fn new(missing_key: &str) -> PropertyError {
        PropertyError {
            missing_key: String::from(missing_key)
        }
    }
}

/// A `RemoteException` is raised when the remote application
/// raises any error that is not an `UserError`.
#[derive(Debug)]
pub struct RemoteException {
    pub cause: String
}

/// Пользовательское исключение, пришедшее с провода (reply status 1), вместе с
/// его Slice type-id.
///
/// Раньше статус 1 сразу декодировался в единственный тип, захардкоженный
/// кодогеном для этой операции, а прочитанный type-id выбрасывался — поэтому
/// `InvalidChannelException` приезжал как `ServerBootedException`, и два разных
/// исключения Murmur'а были неразличимы. Теперь type-id сохраняется, а тело
/// остаётся сырым: решение, во что его разворачивать, принимает вызывающий.
#[derive(Debug)]
pub struct RemoteUserException {
    /// Например `::MumbleServer::InvalidChannelException`.
    pub type_id: String,
    /// Тело слайса после type-id, для дальнейшего декодирования.
    pub payload: Vec<u8>,
}

impl RemoteUserException {
    /// Type-id без ведущего пути модулей: `InvalidChannelException`.
    pub fn short_name(&self) -> &str {
        self.type_id.rsplit("::").next().unwrap_or(&self.type_id)
    }
}

/// Какая именно операция не нашлась — reply statuses 2, 3 и 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestFailedKind {
    ObjectNotExist,
    FacetNotExist,
    OperationNotExist,
}

impl RequestFailedKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestFailedKind::ObjectNotExist => "ObjectNotExist",
            RequestFailedKind::FacetNotExist => "FacetNotExist",
            RequestFailedKind::OperationNotExist => "OperationNotExist",
        }
    }
}

/// Ice `RequestFailedException` (статусы 2..=4). Раньше эти статусы возвращали
/// `Err` из декодера внутри reader-таска и убивали соединение.
#[derive(Debug)]
pub struct RequestFailedException {
    pub kind: RequestFailedKind,
    pub identity: String,
    pub facet: Vec<String>,
    pub operation: String,
}

/// A `UserError` is an error that is defined in ice files.
/// The generic type will be the defined error struct.
#[derive(Debug)]
pub struct UserError<T: std::fmt::Display> {
    pub exception: T
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProtocolError: {}", self.detail)
    }
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParsingError: {}", self.detail)
    }
}

impl std::fmt::Display for PropertyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PropertyError: missing key '{}'", self.missing_key)
    }
}

impl std::fmt::Display for RemoteUserException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({} bytes)", self.type_id, self.payload.len())
    }
}

impl std::fmt::Display for RequestFailedException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: identity '{}', facet {:?}, operation '{}'",
            self.kind.as_str(),
            self.identity,
            self.facet,
            self.operation
        )
    }
}

impl std::fmt::Display for RemoteException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RemoteException: {}", self.cause)
    }
}

impl<T: Display> std::fmt::Display for UserError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.exception)
    }
}


impl std::error::Error for ProtocolError {}
impl std::error::Error for ParsingError {}
impl std::error::Error for RemoteException {}
impl std::error::Error for PropertyError {}
impl std::error::Error for RemoteUserException {}
impl std::error::Error for RequestFailedException {}
impl<T: std::fmt::Debug + Display + FromBytes> std::error::Error for UserError<T> {}

// dummy needed, but should not get called
impl FromBytes for ProtocolError {
    fn from_bytes(bytes: &[u8], _read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> where Self: Sized {
        Ok(Self {
            detail: String::from_utf8(bytes.to_vec())?
        })
    }
}