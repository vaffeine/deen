#![recursion_limit = "512"]

extern crate proc_macro;

mod items;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    braced, parenthesized,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token, Attribute, Ident, Token, Visibility,
};

use items::{Item, encode_item, decode_item};

struct Deen {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    parser_name: Ident,
    struct_name: Ident,
    params: Option<Punctuated<Param, Token![,]>>,
    items: Punctuated<Item, Token![,]>,
}

impl Parse for Deen {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        input.parse::<Token![struct]>()?;
        let parser_name = input.parse()?;
        let params = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            Some(content.parse_terminated(Param::parse)?)
        } else {
            None
        };
        input.parse::<Token![for]>()?;
        let struct_name = input.parse()?;
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

fn parser_declaration(named: &Deen) -> proc_macro2::TokenStream {
    let attrs = &named.attrs;
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
                #(#attrs)*
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
    let write_to = named.items.iter().map(encode_item);
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
    let read_from = named.items.iter().map(decode_item);
    let names = named
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Field(f) => Some(&f.name),
            Item::If(f) => f.name.as_ref(),
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
    let parser_decl = parser_declaration(&named);
    let encoder = encode_impl(&named);
    let decoder = decode_impl(&named);
    let expanded = quote! {
        use std::io;

        use deen::{Deen, Value};

        #parser_decl

        impl Deen for #parser_name {
            type Item = #struct_name;

            #encoder
            #decoder
        }
    };

    TokenStream::from(expanded)
}
