use serde_query::Deserialize;

#[derive(Debug, Deserialize)]
struct Data {
    #[query(".foo.bar")]
    bar1: String,
    #[query(".foo.bar")]
    bar2: String,
}

fn main() {}
