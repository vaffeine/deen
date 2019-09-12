use std::iter;

use quote::quote;
use syn::{
    parse::{self, Parse, ParseStream},
    Expr, Block, Stmt, Ident, Token,
};

use super::{decode_item, encode_item, Field, Item};

pub struct ExprIf {
    pub name: Option<Ident>,
    expr: syn::ExprIf,
}

impl Parse for ExprIf {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let name = if input.peek2(Token![~]) {
            let name = input.parse()?;
            input.parse::<Token![~]>()?;
            Some(name)
        } else {
            None
        };
        let expr = input.parse()?;
        Ok(ExprIf { name, expr })
    }
}

pub fn encode_if(i: &ExprIf) -> proc_macro2::TokenStream {
    let cond = &i.expr.cond;
    let then_branch = encode_block(&i.expr.then_branch, &i.name);
    let else_branch = if let Some((_, else_branch)) = &i.expr.else_branch {
        encode_else(else_branch, &i.name)
    } else {
        quote! {}
    };
    quote! {
        if #cond {
            #then_branch
        } #else_branch
    }
}

fn encode_block(block: &Block, name: &Option<Ident>) -> proc_macro2::TokenStream {
    if let Some((last, stmts)) = block.stmts.split_last() {
        let exprs = &mut stmts.iter().filter_map(|s| match s {
            Stmt::Expr(e) => Some(e),
            Stmt::Semi(e, _) => Some(e),
            _ => panic!("not supported"),
        });
        let last = match last {
            Stmt::Expr(e) => e,
            Stmt::Semi(e, _) => e,
            _ => panic!("not supported"),
        };
        let last = if let Some(name) = name {
            encode_item(&Item::Field(Field { name: name.clone(), init: last.clone() }))
        } else {
            encode_item(&Item::Value(last.clone()))
        };
        let encode = exprs
            .take(block.stmts.len() - 1)
            .map(|s| encode_item(&Item::Value(s.clone())))
            .chain(iter::once(last));
        quote! {
            #(#encode)*
        }
    } else {
        quote! {}
    }
}

fn encode_else(e: &Expr, name: &Option<Ident>) -> proc_macro2::TokenStream {
    let branch = match e {
        Expr::If(e) => {
            encode_if(&ExprIf { name: name.clone(), expr: e.clone() })
        },
        Expr::Block(b) => {
            let block = encode_block(&b.block, name);
            quote! { { #block } }
        },
        _ => unreachable!(),
    };
    quote! {
        else #branch
    }
}

pub fn decode_if(i: &ExprIf) -> proc_macro2::TokenStream {
    let if_block = decode_if_impl(i);
    if let Some(name) = &i.name {
        quote!{
            let #name = #if_block;
        }
    } else {
        if_block
    }
}

fn decode_if_impl(i: &ExprIf) -> proc_macro2::TokenStream {
    let cond = &i.expr.cond;
    let then_branch = decode_block(&i.expr.then_branch, &i.name);
    let else_branch = if let Some((_, else_branch)) = &i.expr.else_branch {
        decode_else(else_branch, &i.name)
    } else {
        quote! {}
    };
    quote! {
        if #cond {
            #then_branch
        } #else_branch
    }
}

fn decode_block(block: &Block, name: &Option<Ident>) -> proc_macro2::TokenStream {
    if let Some((last, stmts)) = block.stmts.split_last() {
        let exprs = &mut stmts.iter().filter_map(|s| match s {
            Stmt::Expr(e) => Some(e),
            Stmt::Semi(e, _) => Some(e),
            _ => panic!("not supported"),
        });
        let last = match last {
            Stmt::Expr(e) => e,
            Stmt::Semi(e, _) => e,
            _ => panic!("not supported"),
        };
        let last = if let Some(name) = name {
            decode_item(&Item::Field(Field { name: name.clone(), init: last.clone() }))
        } else {
            decode_item(&Item::Value(last.clone()))
        };
        let encode = exprs
            .take(block.stmts.len() - 1)
            .map(|s| decode_item(&Item::Value(s.clone())))
            .chain(iter::once(last));
        quote! {
            #(#encode)*
            #name
        }
    } else {
        quote! {}
    }
}

fn decode_else(e: &Expr, name: &Option<Ident>) -> proc_macro2::TokenStream {
    let branch = match e {
        Expr::If(e) => {
            decode_if_impl(&ExprIf { name: name.clone(), expr: e.clone() })
        },
        Expr::Block(b) => {
            let block = decode_block(&b.block, name);
            quote! { { #block } }
        },
        _ => unreachable!(),
    };
    quote! {
        else #branch
    }
}
