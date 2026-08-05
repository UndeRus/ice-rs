use crate::slice::types::IceType;
use quote::{__private::TokenStream, quote};

use super::struct_member::StructMember;


#[derive(Clone, Debug)]
pub struct Exception {
    pub id: TokenStream,
    pub ice_id: String,
    pub members: Vec<StructMember>,
    pub extends: Option<IceType>,
}

impl Exception {
    pub fn empty() -> Exception {
        Exception {
            id: TokenStream::new(),
            ice_id: String::new(),
            members: Vec::new(),
            extends: None
        }
    }

    pub fn add_member(&mut self, member: StructMember) {
        self.members.push(member);
    }

    // pub fn class_name(&self) -> String {
    //     pascalcase::to_pascal_case(&self.name)
    // }

    pub fn generate(&self, mod_path: &str) -> Result<TokenStream, Box<dyn std::error::Error>> {
        let id_token = &self.id;
        let ice_id = &self.ice_id;
        let mut member_tokens = self.members.iter().map(|member| {
            member.declare()
        }).collect::<Vec<_>>();
        let mut member_to_bytes_tokens = self.members.iter().map(|member| {
            member.to_bytes()
        }).collect::<Vec<_>>();
        let mut member_from_bytes_tokens = self.members.iter().map(|member| {
            member.from_bytes()
        }).collect::<Vec<_>>();

        if self.extends.is_some() {
            let extends = self.extends.as_ref().unwrap();
            let token = extends.token();
            // Поле публичное: иначе исключение с базой невозможно построить
            // снаружи сгенерированного модуля.
            member_tokens.push(quote!{
                pub extends: #token
            });
            member_from_bytes_tokens.push(quote!{
                extends: #token::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?
            });
            // Базовый слайс обязан попасть и в `to_bytes`. Раньше он был только в
            // структуре и в `from_bytes`, поэтому все производные исключения
            // сериализовались без базовой части и не читались обратно.
            let base_type_id = match extends {
                IceType::CustomType(name) => format!("{}::{}", mod_path, name),
                other => format!("{}::{}", mod_path, other.token()),
            };
            member_to_bytes_tokens.push(quote!{
                {
                    let base_flags = SliceFlags {
                        type_id: SliceFlagsTypeEncoding::StringTypeId,
                        optional_members: false,
                        indirection_table: false,
                        slice_size: false,
                        last_slice: true,
                    };
                    bytes.extend(base_flags.to_bytes()?);
                    bytes.extend(String::from(#base_type_id).to_bytes()?);
                    bytes.extend(self.extends.to_bytes()?)
                }
            });
        }

        Ok(quote! {
            #[derive(Debug)]
            pub struct #id_token {
                #(#member_tokens),*
            }

            impl std::fmt::Display for #id_token {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, #ice_id)
                }
            }

            impl std::error::Error for #id_token {}

            impl ToBytes for #id_token {
                fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
                    let mut bytes = Vec::new();
                    #(#member_to_bytes_tokens);*;
                    Ok(bytes)
                }
            }

            impl FromBytes for #id_token {
                fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
                where Self: Sized {
                    let mut read = 0;
                    let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
                    let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
                    let obj = Self {
                        #(#member_from_bytes_tokens);*
                    };
                    *read_bytes = *read_bytes + read;
                    Ok(obj)
                }
            }
        })
    }
}