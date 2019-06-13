#![recursion_limit = "512"]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced, parenthesized,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token, Attribute, Expr, Ident, Token, Visibility,
};

struct Deen {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    struct_name: Ident,
    parser_name: Ident,
    params: Option<Punctuated<Param, Token![,]>>,
    items: Punctuated<Item, Token![,]>,
}

impl Parse for Deen {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        input.parse::<Token![struct]>()?;
        let struct_name = input.parse()?;
        input.parse::<Token![<-]>()?;
        let parser_name = input.parse()?;
        let params = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            Some(content.parse_terminated(Param::parse)?)
        } else {
            None
        };
        let items = {
            let content;
            braced!(content in input);
            content.parse_terminated(Item::parse)?
        };
        Ok(Deen {
            attrs,
            visibility,
            struct_name,
            parser_name,
            params,
            items,
        })
    }
}

struct Param {
    name: Ident,
    ty: Ident,
}

impl Parse for Param {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Param { name, ty })
    }
}

enum Item {
    Value(Value),
    Field(Field),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        if input.peek2(Token![<-]) {
            Ok(Item::Field(input.parse()?))
        } else {
            Ok(Item::Value(input.parse()?))
        }
    }
}

struct Value {
    init: Expr,
}

impl Parse for Value {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let init = input.parse()?;
        Ok(Value { init })
    }
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.init.to_tokens(tokens);
    }
}

struct Field {
    name: Ident,
    ty: Ident,
    init: Expr,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![<-]>()?;
        let ty = input.fork().parse()?;
        let init = input.parse()?;

        Ok(Field { name, ty, init })
    }
}

fn struct_declaration(named: &Deen) -> proc_macro2::TokenStream {
    let attrs = &named.attrs;
    let visibility = &named.visibility;
    let name = &named.struct_name;
    let member_items = named
        .items
        .iter()
        .filter_map(|f| match f {
            Item::Field(field) => Some(field),
            _ => None,
        })
        .map(|f| {
            let ty = &f.ty;
            let name = &f.name;
            quote! {
                #name: <#ty as Deen>::Item,
            }
        });
    quote! {
        #(#attrs)*
        #visibility struct #name {
            #(#member_items)*
        }
    }
}

fn parser_declaration(named: &Deen) -> proc_macro2::TokenStream {
    let visibility = &named.visibility;
    let name = &named.parser_name;
    match &named.params {
        Some(params) => {
            let params = params.iter().map(|p| {
                let name = &p.name;
                let ty = &p.ty;
                quote! {
                    #name: #ty,
                }
            });
            quote! {
                #visibility struct #name {
                    #(#params)*
                }
            }
        }
        None => {
            quote! {
                #visibility struct #name;
            }
        }
    }
}

fn encode_impl(named: &Deen) -> proc_macro2::TokenStream {
    let params = params_declaration(named);
    let write_to = named.items.iter().map(|item| match item {
        Item::Field(f) => {
            let init = &f.init;
            let name = &f.name;
            quote! {
                let #name = value.#name;
                #init.encode(&#name, &mut buf)?;
            }
        }
        Item::Value(c) => {
            quote! {
                #c.encode_value(&mut buf)?;
            }
        }
    });
    quote! {
        fn encode(&self, value: &Self::Item, mut buf: impl io::Write) -> io::Result<()> {
            #(#params)*
            #(#write_to)*
            Ok(())
        }
    }
}

fn decode_impl(named: &Deen) -> proc_macro2::TokenStream {
    let params = params_declaration(named);
    let read_from = named.items.iter().map(|item| match item {
        Item::Field(f) => {
            let init = &f.init;
            let name = &f.name;
            quote! {
                let #name = #init.decode(&mut buf)?;
            }
        }
        Item::Value(c) => {
            quote! {
                #c.compare(&mut buf)?;
            }
        }
    });
    let names = named
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Field(f) => Some(&f.name),
            _ => None,
        })
        .map(|n| quote! { #n, });

    quote! {
        fn decode(&self, mut buf: impl io::Read) -> io::Result<Self::Item> {
            #(#params)*
            #(#read_from)*
            Ok(Self::Item {
                #(#names)*
            })
        }
    }
}

fn params_declaration(named: &Deen) -> Vec<proc_macro2::TokenStream> {
    match &named.params {
        Some(params) => params
            .iter()
            .map(|p| {
                let name = &p.name;
                quote! {
                    let #name = self.#name;
                }
            })
            .collect(),
        None => vec![quote! {}],
    }
}

#[proc_macro]
pub fn deen(input: TokenStream) -> TokenStream {
    let named = parse_macro_input!(input as Deen);

    let struct_name = &named.struct_name;
    let parser_name = &named.parser_name;
    let struct_decl = struct_declaration(&named);
    let parser_decl = parser_declaration(&named);
    let encoder = encode_impl(&named);
    let decoder = decode_impl(&named);
    let expanded = quote! {
        use std::io;

        use deen::{Deen, Value};

        #struct_decl

        #parser_decl

        impl Deen for #parser_name {
            type Item = #struct_name;

            #encoder
            #decoder
        }
    };

    TokenStream::from(expanded)
}
