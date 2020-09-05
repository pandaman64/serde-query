#[test]
fn test_field_name() {
    use serde_query::{DeserializeQuery, Query};

    #[derive(DeserializeQuery)]
    struct Data {
        #[query(r#".["field name with spaces"]"#)]
        with_space: i64,
    }

    let document = serde_json::to_string(&serde_json::json!({
        "field name with spaces": 42,
    }))
    .unwrap();

    let data: Data = serde_json::from_str::<Query<Data>>(&document)
        .unwrap()
        .into();

    assert_eq!(data.with_space, 42);
}
