//! A query language for Serde data model.
//!
//! This crate provides [`serde_query::Deserialize`] derive macro that generates
//! [`serde::Deserialize`] implementation with queries.
//!
//! # Example
//!
//! ```rust
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error + 'static>> {
//! #[derive(serde_query::Deserialize)]
//! struct Data {
//!     #[query(".commit.authors.[0]")]
//!     first_author: String,
//!     #[query(".hash")]
//!     hash_value: u64,
//! }
//!
//! let document = serde_json::to_string(&serde_json::json!({
//!     "commit": {
//!         "authors": ["Kou", "Kasumi", "Masaru"],
//!         "date": "2020-09-10",
//!     },
//!     "hash": 0xabcd,
//! }))?;
//!
//! // The query is compatible with arbitrary data formats with serde support.
//! let data: Data = serde_json::from_str(&document)?;
//!
//! assert_eq!(data.first_author, "Kou");
//! assert_eq!(data.hash_value, 0xabcd);
//! # Ok(())
//! # }
//! ```
//!
//! # Derive macros
//!
//! This crate provides the following two derive macros for declaring a query:
//! * [`serde_query::Deserialize`] generates an implementation of [`serde::Deserialize`] for the struct.
//!   We recommend using a full-path form (`serde_query::Deserialize`) when deriving to disambiguate
//!   between serde and this crate.
//! * [`serde_query::DeserializeQuery`] generates [`serde::Deserialize`] for [`Query<T>`] wrapper.
//!   This derive macro is useful if you want two `Deserialize` implementation.
//!   For example, you may want `DeserializeQuery` for querying an API and `Deserialize` for loading from file.
//!
//! Each field must have a `#[query(...)]` attribute for specifying
//! which part of the document should be retrieved, starting from the root.
//!
//! # `#[query(...)]` syntax
//! `serde-query` currently supports the following syntax for stepping one level inside the document.
//! You can combine them to go further.
//!
//! * `.field` for accessing a field with a name `field` of an object.
//!   The field name must be an alphabet followed by zero or more alphanumeric characters.
//! * `.["field"]` if the field name contains special characters.
//!   We recommend using a raw string literal for the query parameter (`#[query(r#"..."#)]`).
//! * `.[index]` for accessing an array element at position `index`.
//!
//! Note that mixing field access and index access at the same position of a document
//! is a compile error.
//!
//! [`serde::Deserialize`]: https://docs.serde.rs/serde/trait.Deserialize.html
//! [`serde_query::Deserialize`]: trait.Deserialize.html
//! [`serde_query::DeserializeQuery`]: trait.DeserializeQuery.html
//! [`Query<T>`]: trait.DeserializeQuery.html#associatedtype.Query
//! [its derive macro]: derive.DeserializeQuery.html

/// Derive macro that generates [`serde::Deserialize`] directly.
///
/// Please refer to the [module-level document] for the usage.
///
/// [`serde::Deserialize`]: https://docs.serde.rs/serde/trait.Deserialize.html
/// [module-level document]: index.html
pub use serde_query_derive::Deserialize;

/// Derive macro for [`DeserializeQuery`] trait.
///
/// # Example
///
/// ```rust
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error + 'static>> {
/// use serde_query::{DeserializeQuery, Query};
///
/// #[derive(DeserializeQuery)]
/// struct Data {
///     #[query(".commit.authors.[0]")]
///     first_author: String,
///     #[query(".hash")]
///     hash_value: u64,
/// }
///
/// let document = serde_json::to_string(&serde_json::json!({
///     "commit": {
///         "authors": ["Kou", "Kasumi", "Masaru"],
///         "date": "2020-09-10",
///     },
///     "hash": 0xabcd,
/// }))?;
///
/// // You can use `Query<T>` as a `Deserialize` type for any `Deserializer`
/// // and convert the result to the desired type using `From`/`Into`.
/// let data: Data = serde_json::from_str::<Query<Data>>(&document)?.into();
///
/// assert_eq!(data.first_author, "Kou");
/// assert_eq!(data.hash_value, 0xabcd);
/// # Ok(())
/// # }
/// ```
///
/// [`DeserializeQuery`]: trait.DeserializeQuery.html
/// [module-level document]: index.html
pub use serde_query_derive::DeserializeQuery;

use core::ops::{Deref, DerefMut};
use serde::de::Deserialize;

/// Convenient type alias for the query type.
///
/// Please refer to [`DeserializeQuery`] trait for details.
///
/// [`DeserializeQuery`]: trait.DeserializeQuery.html
/// [module-level document]: index.html
pub type Query<'de, T> = <T as DeserializeQuery<'de>>::Query;

/// A **data structure** that can be deserialized with a query.
///
/// The [`Query`] type is a `#[repr(transparent)]` wrapper automatically generated by
/// [the proc macro], and can be converted to the implementor
/// (the type with `#[derive(DeserializeQuery)`]) after deserializing from the document
///  using `Deserialize` implementation of the query type.
///
/// [`Query`]: trait.DeserializeQuery.html#associatedtype.Query
/// [the proc macro]: derive.DeserializeQuery.html
pub trait DeserializeQuery<'de>
where
    Self: From<<Self as DeserializeQuery<'de>>::Query>,
    Self::Query: Deserialize<'de> + Deref<Target = Self> + DerefMut,
{
    /// The query type.
    type Query;
}

// This module can only be used inside the generated code.
#[doc(hidden)]
pub mod __priv {
    pub use serde;

    pub trait Container {
        type Element;

        fn empty() -> Self;

        fn reserve(&mut self, additional: usize);

        fn extend_one(&mut self, element: Self::Element);
    }

    extern crate alloc;

    impl<T> Container for alloc::vec::Vec<T> {
        type Element = T;

        fn empty() -> Self {
            Self::new()
        }

        fn reserve(&mut self, additional: usize) {
            self.reserve(additional);
        }

        fn extend_one(&mut self, element: Self::Element) {
            self.push(element);
        }
    }

    impl<T> Container for alloc::collections::VecDeque<T> {
        type Element = T;

        fn empty() -> Self {
            Self::new()
        }

        fn reserve(&mut self, additional: usize) {
            self.reserve(additional);
        }

        fn extend_one(&mut self, element: Self::Element) {
            self.push_back(element);
        }
    }

    impl<T: core::cmp::Ord> Container for alloc::collections::BTreeSet<T> {
        type Element = T;

        fn empty() -> Self {
            Self::new()
        }

        fn reserve(&mut self, _additional: usize) {
            // do nothing
        }

        fn extend_one(&mut self, element: Self::Element) {
            self.insert(element);
        }
    }

    impl<T: core::cmp::Eq + core::hash::Hash> Container for std::collections::HashSet<T> {
        type Element = T;

        fn empty() -> Self {
            Self::new()
        }

        fn reserve(&mut self, additional: usize) {
            self.reserve(additional);
        }

        fn extend_one(&mut self, element: Self::Element) {
            self.insert(element);
        }
    }
}
