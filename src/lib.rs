#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Visibility, Ident, Field, Token, token};

enum GenField {
    Add(Field),
    Remove(Ident),
}

impl Parse for GenField {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(
            match input.parse::<Token![-]>() {
                Ok(_) => GenField::Remove(input.parse()?),
                Err(_) => GenField::Add(input.call(Field::parse_named)?),
            }
        )
    }
}

struct GenParent {
    paren_token: token::Paren,
    parent: Ident,
}

impl Parse for GenParent {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(GenParent {
            paren_token: parenthesized!(content in input),
            parent: content.parse()?,
        })
    }
}

struct GenStruct {
    visibility: Visibility,
    struct_token: Token![struct],
    name: Ident,
    parent: Option<GenParent>,
    brace_token: token::Brace,
    fields: Punctuated<GenField, Token![,]>,
}

impl Parse for GenStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(GenStruct {
            visibility: input.parse()?,
            struct_token: input.parse()?,
            name: input.parse()?,
            parent: input.parse().ok(),
            brace_token: braced!(content in input),
            fields: content.parse_terminated(GenField::parse)?,
        })
    }
}

struct GenStructs(Punctuated<GenStruct, Token![,]>);

impl Parse for GenStructs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(GenStructs(Punctuated::parse_terminated(input)?))
    }
}

#[proc_macro]
pub fn gen_structs(input: TokenStream) -> TokenStream {
    input
}
