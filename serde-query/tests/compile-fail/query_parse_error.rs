#[derive(serde_query::Deserialize)]
struct A {
    #[query(r#"."#)]
    missing_field: String,
    #[query(r#".[kubernetes_clusters]"#)]
    field_in_bracket: String,
    #[query(r#".😎"#)]
    unsupported_char: String,
}

fn assert_deserialize<'de, D: serde::Deserialize<'de>>() {}

fn main() {
    // ensure that fallback implemenation works
    assert_deserialize::<A>();
}