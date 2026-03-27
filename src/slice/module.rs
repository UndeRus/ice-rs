use crate::{errors::ParsingError, slice::enumeration::Enum};
use crate::slice::structure::Struct;
use crate::slice::interface::Interface;
use crate::slice::exception::Exception;
use crate::slice::class::Class;
use std::{path::Path, process::Stdio};
use std::fs::File;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::cell::RefCell;
use inflector::cases::{pascalcase, snakecase};
use quote::{__private::TokenStream, format_ident, quote};
use std::io::Write;
use super::types::IceType;
use std::process::Command;


struct UseStatements {
    uses: BTreeMap<String, TokenStream>,
}

impl UseStatements {
    fn new() -> UseStatements {
        UseStatements {
            uses: BTreeMap::new()
        }
    }

    fn use_crate(&mut self, token: TokenStream) {
        self.uses.insert(token.to_string(), token);
    }

    fn generate(&self) -> Result<TokenStream, Box<dyn std::error::Error>>{
        let tokens = self.uses.iter().map(|(_, token)| {
            quote! {
                #token;
            }
        }).collect::<Vec<_>>();
        Ok(quote! {
            #(#tokens)*
        })
    }
}

pub struct Module {
    pub name: String,
    pub full_name: String,
    pub sub_modules: Vec<Module>,
    enumerations: Vec<Enum>,
    exceptions: Vec<Exception>,
    structs: Vec<Struct>,
    interfaces: Vec<Interface>,
    typedefs: Vec<(String, IceType)>,
    classes: Vec<Class>,
    pub type_map: Rc<RefCell<BTreeMap<String, String>>>
}

impl Module {
    pub fn new(type_map: Rc<RefCell<BTreeMap<String, String>>>) -> Module {
        Module {
            name: String::new(),
            full_name: String::new(),
            sub_modules: vec![],
            enumerations: vec![],
            structs: vec![],
            interfaces: vec![],
            exceptions: vec![],
            typedefs: vec![],
            classes: vec![],
            type_map: type_map
        }
    }

    fn ice_type_has_dict(t: &IceType) -> bool {
        match t {
            IceType::DictType(_, _) => true,
            IceType::SequenceType(inner) | IceType::Optional(inner, _) | IceType::Proxy(inner) => {
                Self::ice_type_has_dict(inner)
            }
            _ => false,
        }
    }

    fn collect_custom_type_names(t: &IceType, out: &mut Vec<String>) {
        match t {
            IceType::CustomType(n) => out.push(n.clone()),
            IceType::Proxy(inner) => Self::collect_custom_type_names(inner, out),
            IceType::SequenceType(inner) | IceType::Optional(inner, _) => {
                Self::collect_custom_type_names(inner, out)
            }
            IceType::DictType(k, v) => {
                Self::collect_custom_type_names(k, out);
                Self::collect_custom_type_names(v, out);
            }
            _ => {}
        }
    }

    fn resolve_type_module(&self, name: &str) -> Option<String> {
        let m = self.type_map.borrow();
        m.get(name)
            .cloned()
            .or_else(|| name.rsplit("::").next().and_then(|k| m.get(k).cloned()))
    }

    fn should_emit_typedef_after_classes(&self, id: &str, vartype: &IceType) -> bool {
        if let IceType::SequenceType(inner) = vartype {
            if let IceType::CustomType(elem) = &**inner {
                return self.classes.iter().any(|c| {
                    c.ice_id == *elem
                        && c
                            .members
                            .iter()
                            .any(|m| matches!(&m.r#type, IceType::CustomType(t) if t == id))
                });
            }
        }
        false
    }

    pub fn has_dict(&self) -> bool {
        if self
            .typedefs
            .iter()
            .any(|(_, t)| Self::ice_type_has_dict(t))
        {
            return true;
        }
        if self
            .structs
            .iter()
            .any(|s| s.members.iter().any(|m| Self::ice_type_has_dict(&m.r#type)))
        {
            return true;
        }
        if self
            .classes
            .iter()
            .any(|c| c.members.iter().any(|m| Self::ice_type_has_dict(&m.r#type)))
        {
            return true;
        }
        if self
            .exceptions
            .iter()
            .any(|e| e.members.iter().any(|m| Self::ice_type_has_dict(&m.r#type)))
        {
            return true;
        }
        self.interfaces.iter().any(|i| {
            i.functions.iter().any(|f| {
                Self::ice_type_has_dict(&f.return_type.r#type)
                    || f.arguments
                        .iter()
                        .any(|a| Self::ice_type_has_dict(&a.r#type))
            })
        })
    }

    pub fn snake_name(&self) -> String {
        snakecase::to_snake_case(&self.name)
    }

    pub fn add_enum(&mut self, enumeration: Enum) {
        self.enumerations.push(enumeration);
    }

    pub fn add_struct(&mut self, structure: Struct) {
        self.structs.push(structure);
    }

    pub fn add_interface(&mut self, interface: Interface) {
        self.interfaces.push(interface);
    }

    pub fn add_exception(&mut self, exception: Exception) {
        self.exceptions.push(exception);
    }

    pub fn add_typedef(&mut self, id: &str, vartype: IceType) {
        self.typedefs.push((String::from(id), vartype.clone()));
    }

    pub fn add_class(&mut self, class: Class) {
        self.classes.push(class);
    }

    fn uses(&self, super_mod: &str) -> UseStatements {
        let mut use_statements = UseStatements::new();

        use_statements.use_crate(quote! { use async_trait::async_trait });
        if self.has_dict() {
            use_statements.use_crate(quote! { use std::collections::HashMap });
        }

        if self.enumerations.len() > 0 || self.structs.len() > 0 || self.interfaces.len() > 0 {
            use_statements.use_crate(quote! { use ice_rs::errors::* });
        }

        if self.enumerations.len() > 0 {
            use_statements.use_crate(quote! { use num_enum::TryFromPrimitive });
            use_statements.use_crate(quote! { use std::convert::TryFrom });
            use_statements.use_crate(quote! { use ice_rs::encoding::* });
        }

        if self.structs.len() > 0 {
            use_statements.use_crate(quote! { use ice_rs::IceDerive });
            use_statements.use_crate(quote! { use ice_rs::encoding::* });

            for item in &self.structs {
                let mut custom_names = Vec::new();
                for member in &item.members {
                    Self::collect_custom_type_names(&member.r#type, &mut custom_names);
                }
                for name in custom_names {
                    if let Some(use_statement) = self.resolve_type_module(&name) {
                        if !use_statement.eq(&self.snake_name()) {
                            let super_token = format_ident!("{}", super_mod);
                            let use_token = format_ident!("{}", use_statement);
                            use_statements.use_crate(quote! { use crate::#super_token::#use_token::* });
                        }
                    }
                }
            }
        }

        if self.classes.len() > 0 {
            use_statements.use_crate(quote! { use ice_rs::encoding::* });

            for item in &self.classes {
                let mut custom_names = Vec::new();
                for member in &item.members {
                    Self::collect_custom_type_names(&member.r#type, &mut custom_names);
                }
                for name in custom_names {
                    if let Some(use_statement) = self.resolve_type_module(&name) {
                        if !use_statement.eq(&self.snake_name()) {
                            let super_token = format_ident!("{}", super_mod);
                            let use_token = format_ident!("{}", use_statement);
                            use_statements.use_crate(quote! { use crate::#super_token::#use_token::* });
                        }
                    }
                }
            }
        }

        if self.interfaces.len() > 0 {
            use_statements.use_crate(quote! { use ice_rs::encoding::* });
            use_statements.use_crate(quote! { use ice_rs::proxy::Proxy });
            use_statements.use_crate(quote! { use ice_rs::iceobject::* });
            use_statements.use_crate(quote! { use ice_rs::protocol::* });            

            for item in &self.interfaces {
                for func in &item.functions {
                    use_statements.use_crate(quote! { use std::collections::HashMap });

                    let mut custom_names = Vec::new();
                    for arg in &func.arguments {
                        Self::collect_custom_type_names(&arg.r#type, &mut custom_names);
                    }
                    Self::collect_custom_type_names(&func.return_type.r#type, &mut custom_names);
                    if let Some(throws) = func.throws.types.first() {
                        Self::collect_custom_type_names(throws, &mut custom_names);
                    }
                    for name in custom_names {
                        if let Some(use_statement) = self.resolve_type_module(&name) {
                            if !use_statement.eq(&self.snake_name()) {
                                let super_token = format_ident!("{}", super_mod);
                                let use_token = format_ident!("{}", use_statement);
                                use_statements.use_crate(quote! { use crate::#super_token::#use_token::* });
                            }
                        }
                    }
                }
            }
        }

        use_statements
    }

    pub fn generate(&self, dest: &Path, mod_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut tokens = vec![];
        tokens.push(quote! {
            // This file has been generated.
            #[allow(dead_code)]
            #[allow(unused_imports)]
        });

        // build up use statements
        let mut use_path = mod_path;
        if use_path.len() == 0 {
            use_path = dest.iter().last().unwrap().to_str().unwrap();
        }
        tokens.push(self.uses(&use_path).generate()?);

        for sub_module in &self.sub_modules {
            let mod_name = sub_module.snake_name();
            let ident = format_ident!("{}", mod_name);
            tokens.push(quote! {
                pub mod #ident;
            });
            sub_module.generate(&dest.join(Path::new(&mod_name)), &use_path)?;
        }

        for enumeration in &self.enumerations {
            tokens.push(enumeration.generate()?);
        }

        for (id, vartype) in &self.typedefs {
            if self.should_emit_typedef_after_classes(id, vartype) {
                continue;
            }
            let id_str = format_ident!("{}", pascalcase::to_pascal_case(&id));
            let var_token = vartype.token();
            tokens.push(quote! {
                pub type #id_str = #var_token;
            });
        }

        for structure in &self.structs {
            tokens.push(structure.generate()?);
        }

        for class in &self.classes {
            tokens.push(class.generate(&self.full_name)?);
        }

        for (id, vartype) in &self.typedefs {
            if !self.should_emit_typedef_after_classes(id, vartype) {
                continue;
            }
            let id_str = format_ident!("{}", pascalcase::to_pascal_case(&id));
            let var_token = if let IceType::SequenceType(inner) = vartype {
                if let IceType::CustomType(elem) = &**inner {
                    let elem_ident = format_ident!("{}", elem);
                    quote! { Vec<Box<#elem_ident>> }
                } else {
                    vartype.token()
                }
            } else {
                vartype.token()
            };
            tokens.push(quote! {
                pub type #id_str = #var_token;
            });
        }

        for exception in &self.exceptions {
            tokens.push(exception.generate()?);
        }

        for interface in &self.interfaces {
            tokens.push(interface.generate(&self.full_name)?);
        }

        let mod_token = quote! { #(#tokens)* };

        std::fs::create_dir_all(dest)?;
        let mod_file = &dest.join(Path::new("mod.rs")); 
        let mut child = Command::new("rustfmt")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("--edition")
            .arg("2018")
            .spawn()?;
        {
            let stdin = child.stdin.as_mut().ok_or(ParsingError::new("Could not get stdin of rustfmt process"))?;
            stdin.write_all(mod_token.to_string().as_bytes())?;
        }    
        let output = child.wait_with_output()?;
        let mut file = File::create(mod_file)?;
        match file.write_all(&output.stdout) {
            Ok(_) => Ok(()),
            Err(_) =>  Err(Box::new(ParsingError::new("Could not write file")))
        }
    }
}