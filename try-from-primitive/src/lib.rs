#![recursion_limit = "512"]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident, Meta, NestedMeta};

#[proc_macro_derive(TryFromPrimitive)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ty = input
        .attrs
        .iter()
        .filter_map(|a| match a.parse_meta() {
            Ok(m) => Some(m),
            Err(_) => None,
        })
        .find(|m| m.name() == "repr")
        .map(|m| match m {
            Meta::List(l) => {
                let ty = match l.nested.first() {
                    Some(t) => match t.value() {
                        NestedMeta::Meta(m) => match m {
                            Meta::Word(i) => i.to_string(),
                            _ => panic!("wrong content of \"repr\" attribute"),
                        },
                        _ => panic!("wrong content of \"repr\" attribute"),
                    },
                    _ => panic!("wrong content of \"repr\" attribute"),
                };
                let (s, bits) = ty.split_at(1);
                (
                    s.to_string(),
                    bits.parse().expect("wrong content of \"repr\" attribute"),
                )
            }
            _ => panic!("wrong content of \"repr\" attribute"),
        });
    let ty_bits = ty
        .as_ref()
        .map(|(v, i)| {
            let dec = if v == "i" { 1 } else { 0 };
            i - dec
        })
        .unwrap_or(32);

    let ident = input.ident;
    let data = match input.data {
        Data::Enum(data) => data,
        Data::Struct(data) => {
            return syn::Error::new(data.struct_token.span, "Can only derive primitive enums")
                .to_compile_error()
                .into();
        }
        Data::Union(data) => {
            return syn::Error::new(data.union_token.span, "Can only derive primitive enums")
                .to_compile_error()
                .into();
        }
    };

    let has_discriminant = if data.variants.iter().all(|v| v.discriminant.is_some()) {
        true
    } else if data.variants.iter().all(|v| v.discriminant.is_none()) {
        false
    } else {
        unimplemented!();
    };

    let variants_from = data
        .variants
        .iter()
        .enumerate()
        .map(|(i, variant)| {
            let name = &variant.ident;
            let value = if has_discriminant {
                let v = &variant.discriminant.as_ref().unwrap().1;
                quote! { #v }
            } else {
                quote! { #i }
            };
            quote! {
                #value => Ok(#ident::#name),
            }
        })
        .collect::<Vec<_>>();
    let variants_to = data
        .variants
        .iter()
        .enumerate()
        .map(|(i, variant)| {
            let name = &variant.ident;
            let value = if has_discriminant {
                let v = &variant.discriminant.as_ref().unwrap().1;
                quote! { #v }
            } else {
                quote! { #i }
            };
            quote! {
                #ident::#name => Ok(#value),
            }
        })
        .collect::<Vec<_>>();

    let impls = [8, 16, 32, 64, 128]
        .iter()
        .map(|n| {
            ['u', 'i']
                .iter()
                .filter(|&&s| {
                    let dec = if s == 'i' { 1 } else { 0 };
                    n - dec >= ty_bits
                })
                .map(|s| {
                    let variants_from = variants_from.iter();
                    let variants_to = variants_to.iter();
                    let ty = Ident::new(&format!("{}{}", s, n), Span::call_site());
                    quote! {
                        impl core::convert::TryFrom<#ty> for #ident {
                            type Error = String;

                            fn try_from(value: #ty) -> Result<Self, Self::Error> {
                                match value {
                                    #(#variants_from)*
                                    _ => Err(format!("{} is not a valid enum value", value))
                                }
                            }
                        }
                        impl core::convert::TryFrom<#ident> for #ty {
                            type Error = String;

                            fn try_from(value: #ident) -> Result<Self, Self::Error> {
                                match value {
                                    #(#variants_to)*
                                }
                            }
                        }
                    }
                })
                .collect::<Vec<_>>()
        })
        .flatten();

    let expanded = quote! {
        #(#impls)*
    };

    TokenStream::from(expanded)
}
