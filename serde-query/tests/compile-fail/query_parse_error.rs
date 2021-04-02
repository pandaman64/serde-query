#[derive(serde_query::Deserialize)]
struct A {
    #[query(r#"."#)]
    missing_field: String,
    #[query(r#".[kubernetes_clusters]"#)]
    field_in_bracket: String,
    #[query(r#".😎"#)]
    unsupported_char: String,
}

fn main() {}