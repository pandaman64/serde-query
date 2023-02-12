use std::collections::{BTreeSet, HashSet};

use serde_query::Deserialize;

#[derive(Debug, Deserialize)]
struct Commits {
    #[query(".[].sha")]
    _shas: Vec<String>,
    #[query(".[].committer.id")]
    _commiter_ids: HashSet<i64>,
    #[query(".[].committer.login")]
    _commiters: HashSet<String>,
    #[query(".[].commit.author.date")]
    _dates: BTreeSet<String>,
}

#[test]
fn test_list_commits() {
    const INPUT: &str = include_str!("./commits.json");
    let commits: Commits = serde_json::from_str(INPUT).unwrap();
    println!("{:?}", commits);
}
