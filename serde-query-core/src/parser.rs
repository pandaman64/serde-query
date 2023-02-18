use proc_macro_error::{diagnostic, Diagnostic, Level};
use quote::ToTokens;
use syn::{DeriveInput, LitStr};

use crate::{parse_query, Query, QueryId};

pub struct ParseResult {
    pub queries: Vec<Query>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse_input(input: &mut DeriveInput) -> ParseResult {
    let mut diagnostics = vec![];
    let queries = match &mut input.data {
        syn::Data::Struct(data) => data
            .fields
            .iter_mut()
            .flat_map(|field| {
                let mut attr_pos = None;
                for (pos, attr) in field.attrs.iter().enumerate() {
                    if attr.path.is_ident("query") {
                        if attr_pos.is_some() {
                            diagnostics.push(diagnostic!(
                                attr,
                                Level::Error,
                                "duplicated #[query(...)]"
                            ));
                        }
                        attr_pos = Some(pos);
                    }
                }

                match attr_pos {
                    None => {
                        diagnostics.push(diagnostic!(field, Level::Error, "no #[query(...)]"));
                        None
                    }
                    Some(pos) => {
                        let attr = field.attrs.remove(pos);
                        let argument = match attr.parse_args::<LitStr>() {
                            Err(_) => {
                                diagnostics.push(diagnostic!(
                                    field,
                                    Level::Error,
                                    "#[query(...)] takes a string literal"
                                ));
                                return None;
                            }
                            Ok(lit) => lit.value(),
                        };
                        let ident = match &field.ident {
                            None => {
                                diagnostics.push(diagnostic!(
                                    field.ident,
                                    Level::Error,
                                    "#[query(...)] field must be named",
                                ));
                                return None;
                            }
                            Some(ident) => ident.clone(),
                        };

                        let (fragment, errors) = parse_query::parse(&argument);
                        for error in errors {
                            diagnostics.push(diagnostic!(attr, Level::Error, error.message));
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
            diagnostics.push(diagnostic!(
                input,
                Level::Error,
                "serde-query supports only structs"
            ));
            vec![]
        }
    };

    ParseResult {
        queries,
        diagnostics,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use k9::snapshot;
    use syn::DeriveInput;

    fn to_snapshot_string<T: std::fmt::Debug>(xs: &[T]) -> String {
        let string_array: Vec<String> = xs.iter().map(|x| format!("{:?}", x)).collect();
        string_array.join("\n")
    }

    #[test]
    fn snapshot_test_parser() {
        let mut input: DeriveInput = syn::parse_str(
            r#"
struct Foo {
    #[query("")]
    with_query: i64,
    #[query(".x")]
    #[query(".y")]
    with_multiple_queries: i32,
    no_query: String,
}
"#,
        )
        .unwrap();

        let result = parse_input(&mut input);

        snapshot!(
            prettyplease::unparse(&syn::parse2(input.to_token_stream()).unwrap()),
            r#"
struct Foo {
    with_query: i64,
    #[query(".x")]
    with_multiple_queries: i32,
    no_query: String,
}

"#
        );
        snapshot!(
            to_snapshot_string(&result.queries),
            r#"
Query { id: QueryId(Ident { sym: with_query, span: bytes(36..46) }), fragment: Accept, ty: TokenStream [Ident { sym: i64, span: bytes(48..51) }] }
Query { id: QueryId(Ident { sym: with_multiple_queries, span: bytes(95..116) }), fragment: Field { name: "y", rest: Accept }, ty: TokenStream [Ident { sym: i32, span: bytes(118..121) }] }
"#
        );
        snapshot!(
            to_snapshot_string(&result.diagnostics),
            r#"
Diagnostic { level: Error, span_range: SpanRange { first: bytes(76..77), last: bytes(77..90) }, msg: "duplicated #[query(...)]", suggestions: [], children: [] }
Diagnostic { level: Error, span_range: SpanRange { first: bytes(127..135), last: bytes(137..143) }, msg: "no #[query(...)]", suggestions: [], children: [] }
"#
        );
    }
}
