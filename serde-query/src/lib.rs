pub use serde_query_derive::DeserializeQuery;

use serde::de::Deserialize;

pub type Query<'de, T> = <T as DeserializeQuery<'de>>::Query;

pub trait DeserializeQuery<'de>
where
    Self: From<<Self as DeserializeQuery<'de>>::Query>,
    Self::Query: Deserialize<'de>,
{
    type Query;
}
