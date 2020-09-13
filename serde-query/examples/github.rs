use serde_query::{DeserializeQuery, Query};

#[derive(DeserializeQuery)]
struct Message {
    #[query(".commit.message")]
    message: String,
}

fn main() {
    let reader = ureq::get("https://api.github.com/repos/pandaman64/serde-query/commits")
        .call()
        .into_reader();

    let messages: Vec<Query<Message>> = serde_json::from_reader(reader).unwrap();

    for message in messages.into_iter() {
        println!("{}", message.message);
    }
}
