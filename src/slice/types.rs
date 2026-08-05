use quote::__private::TokenStream;
use quote::*;
use regex::Regex;
use inflector::cases::{pascalcase, snakecase};

#[derive(Clone, Debug)]
pub enum IceType {
    VoidType,
    BoolType,
    ByteType,
    ShortType,
    IntType,
    LongType,
    FloatType,
    DoubleType,
    StringType,
    SequenceType(Box<IceType>),
    DictType(Box<IceType>, Box<IceType>),
    Optional(Box<IceType>, u8),
    /// Slice proxy type (Interface*).
    Proxy(Box<IceType>),
    CustomType(String),
}

fn scoped_tokens(name: &str) -> TokenStream {
    let parts: Vec<&str> = name.split("::").collect();
    // Sibling submodules under `gen/` (e.g. super::ice from mumble_server)
    let mut ts = quote! { super };
    for (i, part) in parts.iter().enumerate() {
        let id = if i < parts.len() - 1 {
            format_ident!("{}", snakecase::to_snake_case(part))
        } else {
            format_ident!("{}", part)
        };
        ts = quote! { #ts :: #id };
    }
    ts
}

impl IceType {
    pub fn from(text: &str) -> Result<IceType, Box<dyn std::error::Error>> {
        let text = text.trim();
        if text.ends_with('*') {
            let base = text[..text.len() - 1].trim_end();
            if !base.is_empty() && base != "void" {
                return Ok(IceType::Proxy(Box::new(IceType::from(base)?)));
            }
        }

        // Компилируется один раз, а не на каждый вызов `IceType::from` — а он
        // вызывается на каждый член, параметр, возвращаемый тип и typedef, то
        // есть сотни раз за генерацию.
        lazy_static::lazy_static! {
            static ref TYPE_RE: Regex = Regex::new(
                r#"(?x)^
                (void)$ |
                (bool)$ |
                (byte)$ |
                (short)$ |
                (int)$ |
                (long)$ |
                (float)$ |
                (double)$ |
                (string)$ |
                (sequence)<(.+)>$ |
                (dictionary)<(.+),\s*(.+)>$ |
                "#,
            )
            .expect("built-in Slice type regex must compile");
        }
        let type_re = &*TYPE_RE;

        let captures = type_re.captures(text).map(|captures| {
            captures
                .iter()
                .skip(1)
                .flat_map(|c| c)
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
        });

        match captures.as_ref().map(|c| c.as_slice()) {
            Some(["void"]) => Ok(IceType::VoidType),
            Some(["bool"]) => Ok(IceType::BoolType),
            Some(["byte"]) => Ok(IceType::ByteType),
            Some(["short"]) => Ok(IceType::ShortType),
            Some(["int"]) => Ok(IceType::IntType),
            Some(["long"]) => Ok(IceType::LongType),
            Some(["float"]) => Ok(IceType::FloatType),
            Some(["double"]) => Ok(IceType::DoubleType),
            Some(["string"]) => Ok(IceType::StringType),
            Some(["sequence", x]) => Ok(IceType::SequenceType(Box::new(IceType::from(x.trim())?))),
            Some(["dictionary", x, y]) => Ok(IceType::DictType(
                Box::new(IceType::from(x.trim())?),
                Box::new(IceType::from(y.trim())?),
            )),
            _ => Ok(IceType::CustomType(text.to_string())),
        }
    }

    pub fn rust_type(&self) -> String {
        match self {
            IceType::VoidType => String::from("()"),
            IceType::BoolType => String::from("bool"),
            IceType::ByteType => String::from("u8"),
            IceType::ShortType => String::from("i16"),
            IceType::IntType => String::from("i32"),
            IceType::LongType => String::from("i64"),
            IceType::FloatType => String::from("f32"),
            IceType::DoubleType => String::from("f64"),
            IceType::StringType => String::from("String"),
            IceType::SequenceType(type_name) => format!("Vec<{}>", type_name.rust_type()),
            IceType::DictType(key_type, value_type) => {
                format!(
                    "HashMap<{}, {}>",
                    key_type.rust_type(),
                    value_type.rust_type()
                )
            }
            IceType::Optional(type_name, _) => format!("Option<{}>", type_name.rust_type()),
            IceType::Proxy(inner) => match &**inner {
                IceType::CustomType(n) => format!("{}Prx", pascalcase::to_pascal_case(n)),
                _ => format!("{}Prx", inner.rust_type()),
            },
            IceType::CustomType(type_name) => type_name.clone(),
        }
    }

    pub fn token_from(&self) -> TokenStream {
        match self {
            IceType::Optional(type_name, _) => {
                let sub_type = type_name.token();
                quote! { Option::<#sub_type> }
            }
            _ => self.token(),
        }
    }

    pub fn token(&self) -> TokenStream {
        match self {
            IceType::VoidType => quote! { () },
            IceType::BoolType => quote! { bool },
            IceType::ByteType => quote! { u8 },
            IceType::ShortType => quote! { i16 },
            IceType::IntType => quote! { i32 },
            IceType::LongType => quote! { i64 },
            IceType::FloatType => quote! { f32 },
            IceType::DoubleType => quote! { f64 },
            IceType::StringType => quote! { String },
            IceType::SequenceType(type_name) => {
                let sub_type = type_name.token();
                quote! { Vec<#sub_type> }
            }
            IceType::DictType(key_type, value_type) => {
                let key = key_type.token();
                let value = value_type.token();
                quote! { HashMap<#key, #value> }
            }
            IceType::Optional(type_name, _) => {
                let sub_type = type_name.token();
                quote! { Option<#sub_type> }
            }
            IceType::Proxy(inner) => match &**inner {
                IceType::CustomType(name) => {
                    let id = format_ident!("{}Prx", pascalcase::to_pascal_case(name));
                    quote! { #id }
                }
                other => {
                    let inner_t = other.token();
                    quote! { #inner_t }
                }
            },
            IceType::CustomType(type_name) => {
                if type_name.contains("::") {
                    scoped_tokens(type_name)
                } else {
                    let id = format_ident!("{}", pascalcase::to_pascal_case(type_name));
                    quote! { #id }
                }
            }
        }
    }

    pub fn as_ref(&self) -> bool {
        match self {
            IceType::StringType
            | IceType::SequenceType(_)
            | IceType::DictType(_, _)
            | IceType::CustomType(_)
            | IceType::Proxy(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Объявление и ссылка обязаны давать одно и то же имя.
    ///
    /// Раньше структуры/классы объявлялись через `to_class_case`, enum'ы и
    /// интерфейсы — через `to_pascal_case`, а все ссылки резолвились через
    /// `to_class_case`. `to_class_case` вдобавок отбрасывает множественное число
    /// последнего слова, поэтому Slice-тип `Users` объявлялся как `Users`
    /// (enum-путь) и упоминался как `User` — то есть ссылка на несуществующий тип.
    #[test]
    fn declaration_and_reference_mangling_agree() {
        for name in [
            "Users", "ACLList", "DBState", "Tree", "UserMap", "CertificateDer", "NameList",
        ] {
            let reference = IceType::CustomType(String::from(name)).token().to_string();
            let declaration = pascalcase::to_pascal_case(name);
            assert_eq!(
                declaration, reference,
                "объявление и ссылка разошлись для Slice-типа {}",
                name
            );
        }
    }

    /// Именно эта сингуляризация и ломала имена.
    #[test]
    fn plural_type_names_are_not_singularised() {
        let reference = IceType::CustomType(String::from("Users")).token().to_string();
        assert_ne!("User", reference, "имя типа не должно терять множественное число");
    }

    #[test]
    fn proxy_types_get_prx_suffix() {
        let t = IceType::Proxy(Box::new(IceType::CustomType(String::from("Server"))));
        assert_eq!("ServerPrx", t.token().to_string());
        assert_eq!("ServerPrx", t.rust_type());
    }

    #[test]
    fn nested_generics_still_unsupported_but_single_level_works() {
        // Один уровень разбирается.
        match IceType::from("sequence<string>").unwrap() {
            IceType::SequenceType(inner) => {
                assert!(matches!(*inner, IceType::StringType));
            }
            other => panic!("ожидали sequence, получили {:?}", other),
        }
        match IceType::from("dictionary<int, string>").unwrap() {
            IceType::DictType(k, v) => {
                assert!(matches!(*k, IceType::IntType));
                assert!(matches!(*v, IceType::StringType));
            }
            other => panic!("ожидали dictionary, получили {:?}", other),
        }
    }
}
