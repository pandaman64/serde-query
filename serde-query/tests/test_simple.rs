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
