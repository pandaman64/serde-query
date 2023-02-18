#[derive(serde_query::Deserialize)]
struct A {
    #[query(r#".foo.x"#)]
    field_access: String,
    #[query(r#".foo.[0]"#)]
    index_access: String,
}

#[derive(serde_query::Deserialize)]
struct B {
    #[query(r#".foo"#)]
    expect_value: String,
    #[query(r#".foo.bar"#)]
    expect_struct: String,
}

fn assert_deserialize<'de, D: serde::Deserialize<'de>>() {}

fn main() {
    // ensure that fallback implemenations work
    assert_deserialize::<A>();
    assert_deserialize::<B>();
}
