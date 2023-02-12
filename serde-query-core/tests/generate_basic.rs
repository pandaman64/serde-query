use proc_macro2::TokenStream;
use serde_query_core::{compile, Env, Query, QueryFragment, QueryId};

fn generate_preamble() -> TokenStream {
    quote::quote! {
        extern crate alloc;

        trait UnVec {
            type Element;
        }

        impl<T> UnVec for alloc::vec::Vec<T> {
            type Element = T;
        }
    }
}

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

    let code = {
        let mut code = TokenStream::new();
        code.extend(generate_preamble());
        code.extend(node.generate());
        code
    };

    println!("{code}");
}
