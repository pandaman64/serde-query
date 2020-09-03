# An efficient query language for Serde

`sere-query` provides a query language for [Serde](https://serde.rs/) [data model](https://serde.rs/data-model.html).

`serde-query` is:

* Efficient. You can extract only the target parts from a potentially large document with a jq-like syntax. It works like a streaming parser and touches only a minimal amount of elements.
* Flexible. `serde-query` can work with any serde-compatible formats.
* Zero-cost. The traversal structure is encoded as types in compile time.

## Example
```rust
use serde_query::{DeserializeQuery, Query};

#[derive(DeserializeQuery)]
struct Summary {
    #[query(".commit.message")]
    commit_message: String,
    #[query(".commit.url")]
    commit_url: String,
    #[query(".author.login")]
    author_login: String,
}

#[test]
fn test() {
    const INPUT: &str = include_str!("input.json");
    let summary: Summary = serde_json::from_str::<Query<Summary>>(INPUT)
        .unwrap()
        .into();
    assert_eq!(
        summary.commit_message,
        "Add some missing code quoting to the manual"
    );
    assert_eq!(summary.commit_url, "https://api.github.com/repos/stedolan/jq/git/commits/a17dd3248a666d01be75f6b16be37e80e20b0954");
    assert_eq!(summary.author_login, "max-sixty");
}
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