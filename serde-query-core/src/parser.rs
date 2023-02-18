use proc_macro_error::emit_error;
use quote::ToTokens;
use syn::{DeriveInput, LitStr};

use crate::{parse_query, Query, QueryId};

pub struct ParseResult {
    pub queries: Vec<Query>,
    pub has_error: bool,
}

pub fn parse_input(input: &mut DeriveInput) -> ParseResult {
    let mut has_error = false;
    let queries = match &mut input.data {
        syn::Data::Struct(data) => data
            .fields
            .iter_mut()
            .flat_map(|field| {
                let mut attr_pos = None;
                for (pos, attr) in field.attrs.iter().enumerate() {
                    if attr.path.is_ident("query") {
                        if attr_pos.is_some() {
                            emit_error!(attr, "duplicated #[query(...)]");
                            has_error = true;
                        }
                        attr_pos = Some(pos);
                    }
                }

                match attr_pos {
                    None => {
                        emit_error!(field, "no #[query(...)]");
                        has_error = true;
                        None
                    }
                    Some(pos) => {
                        let attr = field.attrs.remove(pos);
                        let argument = match attr.parse_args::<LitStr>() {
                            Err(_) => {
                                emit_error!(field, "#[query(...)] takes a string literal");
                                has_error = true;
                                return None;
                            }
                            Ok(lit) => lit.value(),
                        };
                        let ident = match &field.ident {
                            None => {
                                emit_error!(field.ident, "#[query(...)] field must be named");
                                has_error = true;
                                return None;
                            }
                            Some(ident) => ident.clone(),
                        };

                        let (fragment, errors) = parse_query::parse(&argument);
                        for error in errors {
                            emit_error!(attr, error.message);
                            has_error = true;
                        }
                        Some(Query::new(
                            QueryId::new(ident),
                            fragment,
                            field.ty.to_token_stream(),
                        ))
                    }
                }
            })
            .collect(),
        _ => {
            emit_error!(input, "serde-query supports only structs");
            has_error = true;
            vec![]
        }
    };

    ParseResult { queries, has_error }
}

#[cfg(test)]
mod test {
    use syn::DeriveInput;

    #[test]
    fn test_emit_outside_proc_macro() {
        let mut input: DeriveInput = syn::parse_quote! {
            struct Foo {
                #[query("")]
                with_query: i64,
                #[query(".x")]
                #[query(".y")]
                with_multiple_queries: i32,
                no_query: String,
            }
        };

        assert!(super::parse_input(&mut input).has_error);
    }
}
