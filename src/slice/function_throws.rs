use quote::{__private::TokenStream, quote};

use super::types::IceType;

#[derive(Clone, Debug)]
pub struct FunctionThrows {
    /// Все объявленные в `throws` исключения.
    pub types: Vec<IceType>,
}

impl FunctionThrows {
    pub fn new(types: Vec<IceType>) -> FunctionThrows {
        FunctionThrows { types }
    }

    pub fn empty() -> FunctionThrows {
        FunctionThrows { types: Vec::new() }
    }

    /// Тип-параметр для `Proxy::dispatch::<T>`.
    ///
    /// Раньше это был единственный способ узнать, какое исключение приехало, и
    /// выбор «первого из `throws`» означал, что, например,
    /// `InvalidChannelException` разворачивался в `ServerBootedException`.
    ///
    /// Теперь `Proxy::read_response` при статусе 1 читает Slice type-id и
    /// возвращает `RemoteUserException { type_id, payload }`, не глядя на `T`, —
    /// то есть тип больше не теряется, и разбирать `throws` целиком здесь не
    /// нужно. Параметр оставлен, чтобы не менять сигнатуру сгенерированного кода.
    pub fn token(&self) -> TokenStream {
        match self.types.first() {
            Some(throw) => throw.token(),
            _ => quote! {
                ProtocolError
            },
        }
    }
}
