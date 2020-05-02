use quote::{ToTokens, quote};
use syn::{braced, parse_macro_input, parse_quote};
use syn::{Attribute, Visibility, Ident, Field, Fields, FieldsNamed, Token, ItemStruct, Generics, token};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;

enum MetaField {
    Add(Field),
    Remove(Ident),
}

impl Parse for MetaField {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(
            match input.parse::<Token![-]>() {
                Ok(_) => MetaField::Remove(input.parse()?),
                Err(_) => MetaField::Add(input.call(Field::parse_named)?),
            }
        )
    }
}

struct MetaFields {
    add_fields: Punctuated<Field, Token![,]>,
    rem_fields: Vec<Ident>,
}

impl Parse for MetaFields {
    fn parse(input: ParseStream) -> Result<Self> {
        let fields: Punctuated<_, Token![,]> = Punctuated::parse_terminated(input)?;

        Ok(
            MetaFields {
                add_fields: fields.iter().filter_map(|f| match f {
                    MetaField::Add(a) => Some(a.clone()),
                    _ => None,
                }).collect(),
                rem_fields: fields.iter().filter_map(|f| match f {
                    MetaField::Remove(r) => Some(r.clone()),
                    _ => None,
                }).collect(),
            }
        )
    }
}

struct MetaStruct {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    struct_token: Token![struct],
    name: Ident,
    generics: Generics,
    question_token: Option<Token![?]>,
    brace_token: token::Brace,
    fields: MetaFields,
}

impl Parse for MetaStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(MetaStruct {
            attrs: input.call(Attribute::parse_outer)?,
            visibility: input.parse()?,
            struct_token: input.parse()?,
            name: input.parse()?,
            generics: input.parse()?,
            question_token: input.parse().ok(),
            brace_token: braced!(content in input),
            fields: content.parse()?,
        })
    }
}

struct MetaStructs(Vec<MetaStruct>);

impl Parse for MetaStructs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut structs = vec![];

        while !input.is_empty() {
            structs.push(input.parse()?);
        }

        Ok(MetaStructs(structs))
    }
}

impl ToTokens for MetaStructs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for (i, meta_struct) in self.0.iter().enumerate() {
            let mut named = Punctuated::new();

            'outer: for field in &meta_struct.fields.add_fields {
                for rem_field in &meta_struct.fields.rem_fields {
                    if field.ident.as_ref().unwrap().eq(rem_field) {
                        continue 'outer;
                    }
                }

                named.push(field.clone());
            }

            if i != 0 {
                let parent_fields = &self.0.first().unwrap().fields;

                'parent_outer: for field in &parent_fields.add_fields {
                    for rem_field in &meta_struct.fields.rem_fields {
                        if field.ident.as_ref().unwrap().eq(rem_field) {
                            continue 'parent_outer;
                        }
                    }
    
                    named.push(field.clone());
                }

                if meta_struct.question_token.is_some() {
                    named = named.into_iter().map(|f| {
                        let orig = f.ty;
                        Field {
                            ty: parse_quote! {
                                Option<#orig>
                            },
                            ..f 
                        }
                    }).collect();
                }
            }

            ItemStruct {
                attrs: meta_struct.attrs.clone(),
                vis: meta_struct.visibility.clone(),
                struct_token: meta_struct.struct_token,
                ident: meta_struct.name.clone(),
                generics: meta_struct.generics.clone(),
                fields: Fields::Named(FieldsNamed {
                    brace_token: meta_struct.brace_token,
                    named,
                }),
                semi_token: None,
            }.to_tokens(tokens);
        }
    }
}

pub fn gen_structs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let meta_structs = parse_macro_input!(input as MetaStructs);

    let token_stream = quote! {
        #meta_structs
    };

    println!("{}", token_stream);

    token_stream.into()
}
