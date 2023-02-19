#![allow(dead_code)]

use serde_query::Deserialize;

const INPUT: &str = include_str!("./input.json");

#[test]
fn test_error_example() {
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
}
