# Serde Query: An efficient query language for Serde

`sere-query` provides a query language for [Serde](https://serde.rs/) [data model](https://serde.rs/data-model.html).

`serde-query` is:

* Efficient. You can extract only the target parts from a potentially large document with a jq-like syntax. It works like a streaming parser and touches only a minimal amount of elements.
* Flexible. `serde-query` can work with any serde-compatible formats.
* Zero-cost. The traversal structure is encoded as types in compile time.

## Example
```rust
use serde_query::{DeserializeQuery, Query};

#[derive(DeserializeQuery)]
struct Data {
    #[query(".commit.authors.[0]")]
    first_author: String,
    #[query(".hash")]
    hash_value: u64,
}

let document = serde_json::to_string(&serde_json::json!({
    "commit": {
        "authors": ["Kou", "Kasumi", "Masaru"],
        "date": "2020-09-10",
    },
    "hash": 0xabcd,
}))?;

// You can use `Query<T>` as a `Deserialize` type for any `Deserializer`
// and convert the result to the desired type using `From`/`Into`.
let data: Data = serde_json::from_str::<Query<Data>>(&document)?.into();

assert_eq!(data.first_author, "Kou");
assert_eq!(data.hash_value, 0xabcd);
```

## Note

This library generates Rust types for each query segment (e.g., `.commit`, `.commit.message`, etc.), which may lead to binary bloat and longer compile time.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.