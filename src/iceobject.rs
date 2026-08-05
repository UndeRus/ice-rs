use async_trait::async_trait;
use crate::protocol::*;

/// The `IceObject` trait is a base trait for all
/// ice interfaces. It implements functions that
/// are equal to all ice interfaces.
#[async_trait]
pub trait IceObject {
    async fn ice_ping(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send>>;
    async fn ice_is_a(&mut self) -> Result<bool, Box<dyn std::error::Error + Sync + Send>>;
    async fn ice_id(&mut self) -> Result<String, Box<dyn std::error::Error + Sync + Send>>;
    async fn ice_ids(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error + Sync + Send>>;
}

/// Чем servant отвечает на входящий запрос.
///
/// Раньше servant возвращал уже собранный `ReplyData`, и адаптер на любую ошибку
/// клал `status: 1` с сырой Rust-строкой в теле — это не валидное Ice-исключение,
/// и настоящий пир на таком ответе рвёт соединение. Теперь исход описывается
/// явно, а корректные байты собирает адаптер.
#[derive(Debug)]
pub enum DispatchResult {
    /// Успех: закодированные out-параметры и возвращаемое значение.
    Ok(Encapsulation),
    /// Пользовательское исключение из Slice — Ice status 1.
    UserException {
        /// Полный Slice type-id, например `::MumbleServer::InvalidChannelException`.
        type_id: String,
        /// Члены исключения после type-id.
        body: Vec<u8>,
    },
    /// Такого объекта нет — Ice status 2.
    ObjectNotExist,
    /// Такого facet'а нет — Ice status 3.
    FacetNotExist,
    /// Объект есть, операции нет — Ice status 4.
    OperationNotExist,
    /// Внутренний сбой servant'а — Ice status 5 (UnknownLocalException).
    Failed(String),
}

/// Объект, обслуживающий входящие Ice-запросы.
///
/// `dispatch` берёт `&self`, а не `&mut self`. Это принципиально: раньше и
/// servant, и `Adapter::handle_socket` требовали `&mut self`, поэтому один
/// адаптер невозможно было обслуживать больше чем одним соединением
/// одновременно. Murmur открывает отдельное соединение под каждую доставку, и
/// второе навсегда вставало в очередь за блокировкой — не получая даже
/// ValidateConnection, на отсутствие которого Murmur отвечает CloseConnection.
/// Внутреннюю синхронизацию, если она нужна, реализация выбирает сама.
#[async_trait]
pub trait Servant: Send + Sync {
    /// Slice type-id'ы объекта, от самого производного к базовому; последним
    /// должен идти `::Ice::Object`. Используется для `ice_ids`/`ice_isA`.
    fn type_ids(&self) -> Vec<String>;

    /// Обрабатывает запрос. Операции `ice_ping`/`ice_id`/`ice_ids`/`ice_isA`
    /// адаптер закрывает сам, если реализация вернула `OperationNotExist`.
    async fn dispatch(&self, request: &RequestData) -> DispatchResult;

    /// Самый производный type-id.
    fn type_id(&self) -> String {
        self.type_ids()
            .first()
            .cloned()
            .unwrap_or_else(|| String::from("::Ice::Object"))
    }

    /// Отвечает ли объект за указанный type-id (учитывая наследование).
    fn is_a(&self, type_id: &str) -> bool {
        self.type_ids().iter().any(|t| t == type_id)
    }
}

/// Прежний интерфейс servant'а: `&mut self` и самостоятельно собранный
/// `ReplyData`.
///
/// Оставлен для совместимости с уже существующим сгенерированным кодом. Новый
/// код должен реализовывать [`Servant`].
#[async_trait]
pub trait IceObjectServer {
    async fn handle_request(&mut self, request: &RequestData) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>>;
}
