# serde-query

Welcome to serde-query, a Rust library that lets you write jq-like queries for your data.

## Why serde-query?

1. **Efficiency**: With serde-query, you can efficiently extract exactly what you need without wasting the memory.
2. **Helpful error messages**: When queries fail, you'll get clear, concise error messages that tell you where and why the failure happens.
3. **Flexibility**: serde-query supports any serde-compatible data formats.

## Getting started

To get started with serde-query, add it to your Rust project using Cargo:

```bash
cargo add serde-query
```

Or, add it to your `Cargo.toml`:

```toml
[dependencies]
serde-query = "0.2.0"
```

## Example

### Array queries
```rust
use serde_query::{DeserializeQuery, Query};

#[derive(DeserializeQuery)]
struct Data {
    #[query(".commits.[].author")]
    authors: Vec<String>,
    #[query(".count")]
    count: usize,
}

let document = serde_json::json!({
    "commits": [
        { "author":    "Kou", "hash": 0x0202 },
        { "author": "Kasumi", "hash": 0x1013 },
        { "author": "Masaru", "hash": 0x0809 },
    ],
    "count": 3,
}).to_string();

// You can use `Query<T>` as a `Deserialize` type for any `Deserializer`
// and convert the result to the desired type using `From`/`Into`.
let data: Data = serde_json::from_str::<Query<Data>>(&document)?.into();

assert_eq!(data.authors, vec!["Kou", "Kasumi", "Masaru"]);
assert_eq!(data.count, 3);
```

### Errors
```rust
use serde_query::Deserialize;

#[derive(Debug, Deserialize)]
struct Data {
    // missing field
    #[query(".author.name")]
    author_name: String,
    // typo
    #[query(".commit.commiter.name")]
    committer_name: String,
    // type error
    #[query(".author.id")]
    id: String,
}

let error = serde_json::from_str::<Data>(INPUT).unwrap_err();
assert_eq!(
    error.to_string(),
    r#"
Queries failed for fields: 'author_name', 'committer_name', 'id'
  1. Query for field 'author_name' failed at '.author': missing field 'name'
  2. Query for field 'committer_name' failed at '.commit': missing field 'commiter'
  3. Query for field 'id' failed at '.author.id': invalid type: integer `5635139`, expected a string at line 34 column 17
"#
    .trim_start()
);
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
