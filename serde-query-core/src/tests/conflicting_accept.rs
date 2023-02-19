use syn::DeriveInput;

use crate::{tests::snapshot_derive, DeriveTarget};

#[test]
fn test_conflicting_accept() {
    let input: DeriveInput = syn::parse_quote! {
        struct Data {
            #[query(".foo.bar")]
            bar1: String,
            #[query(".foo.bar")]
            bar2: String,
        }
    };

    k9::snapshot!(
        snapshot_derive(input, DeriveTarget::Deserialize),
        r#"Diagnostic { level: Error, span_range: SpanRange { first: bytes(0..0), last: bytes(0..0) }, msg: "Cannot use the same query for two or more fields: 'bar1', 'bar2'", suggestions: [], children: [] }"#
    );
}
