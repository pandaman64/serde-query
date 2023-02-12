use serde_query_core::{compile, Env, Query, QueryFragment, QueryId};

#[test]
fn test_basic() {
    let queries = vec![
        Query::new(
            QueryId::new(quote::format_ident!("x")),
            QueryFragment::field(
                "locs".into(),
                QueryFragment::collect_array(QueryFragment::field(
                    "x".into(),
                    QueryFragment::accept(),
                )),
            ),
            quote::quote!(Vec<f32>),
        ),
        Query::new(
            QueryId::new(quote::format_ident!("y")),
            QueryFragment::field(
                "locs".into(),
                QueryFragment::collect_array(QueryFragment::field(
                    "y".into(),
                    QueryFragment::accept(),
                )),
            ),
            quote::quote!(Vec<f32>),
        ),
    ];
    let node = compile(&mut Env::new(), queries.into_iter());

    let code = node.generate();
    println!("{code}");
}
