use quote::{ToTokens, quote};
use syn::{parenthesized, braced, parse_macro_input};
use syn::{Visibility, Ident, Field, Fields, FieldsNamed, Token, ItemStruct, Generics, token};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;

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
    generics: Generics,
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
            generics: input.parse()?,
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

impl ToTokens for GenStructs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for gen_struct in &self.0 {
            let named = Punctuated::new();

            ItemStruct {
                attrs: vec![],
                vis: gen_struct.visibility.clone(),
                struct_token: gen_struct.struct_token,
                ident: gen_struct.name.clone(),
                generics: gen_struct.generics.clone(),
                fields: Fields::Named(FieldsNamed {
                    brace_token: gen_struct.brace_token,
                    named
                }),
                semi_token: None,
            }.to_tokens(tokens);
        }
    }
}

#[proc_macro]
pub fn gen_structs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let gen_structs = parse_macro_input!(input as GenStructs);

    let token_stream = quote! {
        #gen_structs
    };

    println!("{}", token_stream.to_string());

    token_stream.into()
}
