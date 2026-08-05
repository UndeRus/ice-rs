use crate::slice::function::Function;
use quote::{__private::TokenStream, format_ident, quote};


#[derive(Clone, Debug)]
pub struct Interface {
    pub id: TokenStream,
    pub ice_id: String,
    /// Slice-имя базового интерфейса из `extends`, если он есть.
    ///
    /// Раньше `extends` парсер молча выбрасывал (`Rule::extends => {}`), поэтому
    /// `ServerUpdatingAuthenticator extends ServerAuthenticator` генерировался
    /// без пяти унаследованных операций: Murmur звал `authenticate`, а диспатчер
    /// отвечал «операции нет», и `checkedCast` к базовому типу тоже проваливался.
    pub extends: Option<String>,
    pub functions: Vec<Function>
}

impl Interface {
    pub fn empty() -> Interface {
        Interface {
            id: TokenStream::new(),
            ice_id: String::from(""),
            extends: None,
            functions: Vec::new()
        }
    }

    pub fn add_function(&mut self, function: Function) {
        self.functions.push(function);
    }

    /// Генерирует код интерфейса.
    ///
    /// `functions` — уже сплющенный список операций (свои плюс унаследованные),
    /// `type_ids` — цепочка Slice type-id от самого производного к
    /// `::Ice::Object`. Оба готовит `Module::resolve_interface`, потому что база
    /// ищется среди соседей по модулю, а самому интерфейсу они не видны.
    pub fn generate(
        &self,
        mod_path: &str,
        functions: &[Function],
        type_ids: &[String],
    ) -> Result<TokenStream, Box<dyn std::error::Error>> {
        let mut decl_tokens = TokenStream::new();
        for function in functions {
            let token = function.generate_decl()?;
            decl_tokens = quote! {
                #decl_tokens
                #token
            };
        }
        let mut impl_tokens = TokenStream::new();
        for function in functions {
            let token = function.generate_impl()?;
            impl_tokens = quote! {
                #impl_tokens
                #token
            };
        }
        let mut server_decl_tokens = TokenStream::new();
        for function in functions {
            let token = function.generate_server_decl()?;
            server_decl_tokens = quote! {
                #server_decl_tokens
                #token
            };
        }
        let mut server_handler_tokens = TokenStream::new();
        for function in functions {
            let token = function.generate_server_handler()?;
            server_handler_tokens = quote! {
                #server_handler_tokens
                #token
            };
        }

        let id_token = &self.id;
        let id_proxy_token = format_ident!("{}Prx", self.id.to_string());
        let id_server_trait_token = format_ident!("{}I", self.id.to_string());
        let id_server_token = format_ident!("{}Server", self.id.to_string());
        let type_id_token = format!("{}::{}", mod_path, self.ice_id);
        Ok(quote! {
            #[async_trait]
            pub trait #id_token : IceObject {
                #decl_tokens
            }

            #[async_trait]
            pub trait #id_server_trait_token {
                #server_decl_tokens
            }

            pub struct #id_server_token {
                server_impl: Box<dyn #id_server_trait_token + Send + Sync>
            }

            impl #id_server_token {
                #[allow(dead_code)]
                pub fn new(server_impl: Box<dyn #id_server_trait_token + Send + Sync>) -> #id_server_token {
                    #id_server_token {
                        server_impl
                    }
                }

                /// Отвечает по всей цепочке наследования, а не только по
                /// собственному type-id: иначе `checkedCast` к базовому
                /// интерфейсу со стороны пира проваливается.
                async fn ice_is_a(&self, param: &str) -> bool {
                    Self::ice_type_ids().iter().any(|t| t == param)
                }

                /// Slice type-id'ы объекта, от самого производного к
                /// `::Ice::Object`.
                #[allow(dead_code)]
                pub fn ice_type_ids() -> Vec<String> {
                    vec![#(String::from(#type_ids)),*]
                }

                /// Оборачивает в `Servant`, пригодный для регистрации в адаптере.
                #[allow(dead_code)]
                pub fn into_servant(self) -> std::sync::Arc<dyn ice_rs::iceobject::Servant> {
                    ice_rs::adapter::LegacyServant::new(Box::new(self), Self::ice_type_ids())
                }
            }

            #[async_trait]
            impl IceObjectServer for #id_server_token {
                async fn handle_request(&mut self, request: &RequestData) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>> {
                    match request.operation.as_ref() {
                        "ice_ping" => Ok(ReplyData {
                            request_id: request.request_id,
                            status: 0,
                            body: Encapsulation::empty(),
                        }),
                        "ice_id" => Ok(ReplyData {
                            request_id: request.request_id,
                            status: 0,
                            body: Encapsulation::from(String::from(#type_id_token).to_bytes()?),
                        }),
                        "ice_ids" => Ok(ReplyData {
                            request_id: request.request_id,
                            status: 0,
                            body: Encapsulation::from(Self::ice_type_ids().to_bytes()?),
                        }),
                        "ice_isA" => {
                            let buf = ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                            let mut read = 0;
                            let param = String::from_bytes(&buf, &mut read)?;
                            Ok(ReplyData {
                                request_id: request.request_id,
                                status: 0,
                                body: Encapsulation::from(self.ice_is_a(&param).await.to_bytes()?)
                            })
                        },
                        #server_handler_tokens
                        _ => Err(Box::new(ProtocolError::new("Operation not found")))
                    }
                }
            }

            /// `Clone` дешёвый: внутри `Proxy`, а соединение живёт за `Arc`.
            #[derive(Clone)]
            pub struct #id_proxy_token {
                pub proxy: Proxy
            }

            #[async_trait]
            impl IceObject for #id_proxy_token {
                async fn ice_ping(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>>
                {
                    self.proxy.dispatch::<ProtocolError>(&String::from("ice_ping"), 1, &Encapsulation::empty(), None).await?;
                    Ok(())
                }

                async fn ice_is_a(&self) -> Result<bool, Box<dyn std::error::Error + Sync + Send>> {
                    let reply = self.proxy.dispatch::<ProtocolError>(&String::from("ice_isA"), 1, &Encapsulation::from(String::from(#type_id_token).to_bytes()?), None).await?;
                    let mut read_bytes: i32 = 0;
                    bool::from_bytes(&reply.body.data, &mut read_bytes)
                }

                async fn ice_id(&self) -> Result<String, Box<dyn std::error::Error + Sync + Send>>
                {
                    let reply = self.proxy.dispatch::<ProtocolError>(&String::from("ice_id"), 1, &Encapsulation::empty(), None).await?;
                    let mut read_bytes: i32 = 0;
                    String::from_bytes(&reply.body.data, &mut read_bytes)
                }

                async fn ice_ids(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Sync + Send>>
                {
                    let reply = self.proxy.dispatch::<ProtocolError>(&String::from("ice_ids"), 1, &Encapsulation::empty(), None).await?;
                    let mut read_bytes: i32 = 0;
                    Vec::from_bytes(&reply.body.data, &mut read_bytes)
                }
            }

            #[async_trait]
            impl #id_token for #id_proxy_token {
                #impl_tokens
            }

            impl #id_proxy_token {
                #[allow(dead_code)]
                pub async fn unchecked_cast(proxy: Proxy) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(Self {
                        proxy: proxy,
                    })
                }

                #[allow(dead_code)]
                pub async fn checked_cast(proxy: Proxy) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                    let mut my_proxy = Self::unchecked_cast(proxy).await?;
            
                    if !my_proxy.ice_is_a().await? {
                        return Err(Box::new(ProtocolError::new("ice_is_a() failed")));
                    }
                    Ok(my_proxy)
                }
            }

            impl ice_rs::encoding::ToBytes for #id_proxy_token {
                fn to_bytes(
                    &self,
                ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
                    self.proxy.to_bytes()
                }
            }

            impl ice_rs::encoding::FromBytes for #id_proxy_token {
                fn from_bytes(
                    bytes: &[u8],
                    read_bytes: &mut i32,
                ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
                where
                    Self: Sized,
                {
                    Ok(#id_proxy_token {
                        proxy: ice_rs::proxy::Proxy::from_bytes(bytes, read_bytes)?,
                    })
                }
            }
        })
    }
}
