#[test]
fn test_index() {
    use serde_query::{DeserializeQuery, Query};

    #[derive(DeserializeQuery)]
    struct Data {
        #[query(".[1]")]
        second_elem: i64,
    }

    let document = serde_json::json!([
        "abc",
        42,
        {
            "key": "value",
        }
    ])
    .to_string();

    let data: Data = serde_json::from_str::<Query<Data>>(&document)
        .unwrap()
        .into();

    assert_eq!(data.second_elem, 42);
}
