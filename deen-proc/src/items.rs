mod condition;

use quote::quote;
use syn::{
    parse::{self, Parse, ParseStream},
    Expr, Ident, Token,
};

use condition::{ExprIf, encode_if, decode_if};

pub enum Item {
    Value(Expr),
    Field(Field),
    If(ExprIf),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        if input.peek(Token![if]) {
            Ok(Item::If(input.parse()?))
        } else if input.peek2(Token![~]) || (input.peek2(Token![:]) && !input.peek2(Token![::])) {
            if input.peek3(Token![if]) {
                Ok(Item::If(input.parse()?))
            } else {
                Ok(Item::Field(input.parse()?))
            }
        } else {
            Ok(Item::Value(input.parse()?))
        }
    }
}

pub struct Field {
    pub name: Ident,
    init: Expr,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![~]>()?;
        let init = input.parse()?;

        Ok(Field { name, init })
    }
}

pub fn encode_item(item: &Item) -> proc_macro2::TokenStream {
    match item {
        Item::Field(f) => {
            let init = &f.init;
            let name = &f.name;
            let tf = try_from(name);
            quote! {
                let #name = value.#name;
                #init.encode(&#tf, &mut buf)?;
            }
        }
        Item::Value(c) => {
            quote! {
                #c.encode_value(&mut buf)?;
            }
        }
        Item::If(i) => encode_if(i),
    }
}

pub fn decode_item(item: &Item) -> proc_macro2::TokenStream {
    match item {
        Item::Field(f) => {
            let init = &f.init;
            let name = &f.name;
            let tf = try_from(name);
            quote! {
                let #name = #init.decode(&mut buf)?;
                let #name = #tf;
            }
        }
        Item::Value(c) => {
            quote! {
                #c.compare(&mut buf)?;
            }
        }
        Item::If(i) => decode_if(i),
    }
}

fn try_from(item: &Ident) -> proc_macro2::TokenStream {
    let name = format!("{}", item);
    quote! {
        core::convert::TryFrom::try_from(#item).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("failed to convert {}: {}", #name, e),
            )
        })?
    }
}
