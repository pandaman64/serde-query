use std::collections::HashSet;

use serde_query::Deserialize;

const COMMITS_JSON: &str = include_str!("./commits.json");

#[test]
fn test_typo() {
    #[derive(Debug, Deserialize)]
    struct Commits {
        #[query(".[].commiter.id")]
        _commiter_ids: HashSet<i64>,
    }

    let snapshot = format!("{:?}", serde_json::from_str::<Commits>(COMMITS_JSON));
    k9::snapshot!(
        snapshot,
        r#"Err(Error("Query for field '_commiter_ids' failed at '.[]': missing field 'commiter'", line: 0, column: 0))"#
    );
}

#[test]
fn test_multiple_errors() {
    #[derive(Debug, Deserialize)]
    struct Commits {
        // typo in "committer"
        #[query(".[].commiter.id")]
        _commiter_ids: HashSet<i64>,
        // no field named "username"
        #[query(".[].committer.username")]
        _committers: Vec<String>,
    }

    let snapshot = format!("{:?}", serde_json::from_str::<Commits>(COMMITS_JSON));
    k9::snapshot!(
        snapshot,
        r#"
Err(Error("Queries failed for fields: '_commiter_ids', '_committers'\
  1. Query for field '_commiter_ids' failed at '.[]': missing field 'commiter'\
  2. Query for field '_committers' failed at '.[].committer': missing field 'username'\
", line: 0, column: 0))
"#
    );
}

#[test]
fn test_type_error() {
    #[derive(Debug, Deserialize)]
    struct Data {
        // type mismatch.
        #[query(".foo.bar")]
        _expect_integer: i64,
    }

    let input = serde_json::json!({
        "foo": {
            "bar": "str",
        },
    });
    let snapshot = format!("{:?}", serde_json::from_str::<Data>(&input.to_string()));
    k9::snapshot!(
        snapshot,
        r#"Err(Error("Query for field '_expect_integer' failed at '.foo.bar': invalid type: string \\"str\\", expected i64", line: 1, column: 19))"#
    );
}
