#[derive(serde_query::Deserialize, serde_query::DeserializeQuery)]
struct A {
    #[query(r#"."#)]
    missing_field: String,
    #[query(r#".[kubernetes_clusters]"#)]
    field_in_bracket: String,
    #[query(r#".😎"#)]
    unsupported_char: String,
}

fn assert_deserialize<'de, D: serde::Deserialize<'de>>() {}
fn assert_deserialize_query<'de, D: serde_query::DeserializeQuery<'de>>() {}

fn main() {
    // ensure that fallback implemenations work
    assert_deserialize::<A>();
    assert_deserialize_query::<A>();
}
