use quote::{__private::TokenStream, quote};

use super::types::IceType;

#[derive(Clone, Debug)]
pub struct FunctionThrows {
    /// All declared Slice exceptions (first is used for reply error decoding).
    pub types: Vec<IceType>,
}

impl FunctionThrows {
    pub fn new(types: Vec<IceType>) -> FunctionThrows {
        FunctionThrows { types }
    }

    pub fn empty() -> FunctionThrows {
        FunctionThrows { types: Vec::new() }
    }

    pub fn token(&self) -> TokenStream {
        match self.types.first() {
            Some(throw) => throw.token(),
            _ => quote! {
                ProtocolError
            },
        }
    }
}
