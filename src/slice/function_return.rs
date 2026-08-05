use quote::{__private::TokenStream, quote};

use super::types::IceType;

#[derive(Clone, Debug)]
pub struct FunctionReturn {
    pub r#type: IceType,
}

impl FunctionReturn {
    pub fn new(r#type: IceType) -> FunctionReturn {
        FunctionReturn { r#type }
    }

    pub fn empty() -> FunctionReturn {
        FunctionReturn {
            r#type: IceType::VoidType,
        }
    }

    pub fn token(&self) -> TokenStream {
        self.r#type.token()
    }

    pub fn return_token(&self) -> TokenStream {
        let return_token = self.token();
        match &self.r#type {
            IceType::VoidType => quote! {
                Ok(())
            },
            IceType::Optional(type_name, _) => {
                let option_token = type_name.token();
                quote! {
                    Option::<#option_token>::from_bytes(&reply.body.data[read_bytes as usize..reply.body.data.len()], &mut read_bytes)
                }
            }
            IceType::Proxy(_) => {
                // Возвращённый прокси делит соединение с родительским: та же
                // идентичность на том же эндпоинте.
                //
                // Раньше здесь на КАЖДЫЙ вызов поднимался новый `Communicator`
                // и открывался новый сокет, а эндпоинт из ответа выбрасывался.
                quote! {
                    let proxy_data = ProxyData::from_bytes(&reply.body.data[read_bytes as usize..reply.body.data.len()], &mut read_bytes)?;
                    let proxy = self.proxy.with_ident(&proxy_data.identity_string());
                    #return_token::unchecked_cast(proxy).await
                }
            }
            IceType::CustomType(_) => {
                quote! {
                    #return_token::from_bytes(&reply.body.data[read_bytes as usize..reply.body.data.len()], &mut read_bytes)
                }
            }
            _ => {
                quote! {
                    #return_token::from_bytes(&reply.body.data[read_bytes as usize..reply.body.data.len()], &mut read_bytes)
                }
            }
        }
    }
}
