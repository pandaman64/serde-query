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
        r#"Err(Error("Query for '_commiter_ids' failed to run", line: 81, column: 2))"#
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
        // TODO: currently, we emit only the first error.
        r#"Err(Error("Query for '_commiter_ids' failed to run", line: 81, column: 2))"#
    );
}
