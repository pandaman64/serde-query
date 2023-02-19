//! A query language for Serde data model.
//!
//! This crate provides [`serde_query::Deserialize`] and [`serde_query::DeserializeQuery`] derive
//! macros that generate [`serde::Deserialize`] implementations with queries.
//!
//! # Example
//!
//! ```rust
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error + 'static>> {
//! use serde_query::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Data {
//!     #[query(".commits.[].author")]
//!     authors: Vec<String>,
//!     #[query(".count")]
//!     count: usize,
//! }
//!
//! let document = serde_json::json!({
//!     "commits": [
//!         {
//!             "author": "Kou",
//!             "hash": 0x0202,
//!         },
//!         {
//!             "author": "Kasumi",
//!             "hash": 0x1013,
//!         },
//!         {
//!             "author": "Masaru",
//!             "hash": 0x0809,
//!         },
//!     ],
//!     "count": 3,
//! }).to_string();
//!
//! let data: Data = serde_json::from_str(&document)?;
//!
//! assert_eq!(data.authors, vec!["Kou", "Kasumi", "Masaru"]);
//! assert_eq!(data.count, 3);
//! # Ok(())
//! # }
//! ```
//!
//! # Derive macros
//!
//! This crate provides the following derive macros for declaring queries:
//! * [`serde_query::Deserialize`] generates an implementation of [`serde::Deserialize`] for the struct.
//!   We recommend using a full-path form (`serde_query::Deserialize`) when deriving to disambiguate
//!   between serde and this crate.
//! * [`serde_query::DeserializeQuery`] generates [`serde::Deserialize`] for [`Query<T>`] wrapper.
//!   This derive macro is useful if you want two `Deserialize` implementation.
//!   For example, you may want `DeserializeQuery` for querying an API and `Deserialize` for loading from file.
//!
//! # Using the `#[query("...")]` annotation
//!
//! serde-query let you write a jq-like query inside the `#[query("...")]` annotation.
//! Note that every field must have a query annotation.
//!
//! The supported syntaxes are as follows:
//!
//! * **`.field` syntax:** You can use the `.field` syntax to extract a field from a struct.
//!   For example, `.name` extracts the `name` field. If the field name contains special characters, you can use the `.["field"]` syntax to quote the field name.
//!   For example, `.["first-name"]` extracts the `first-name` field.
//!   When quoting a field name, try using a raw string literal (i.e., `#[query(r#"..."#)]`).
//! * **`.[]` syntax:** You can use the `.[]` syntax to run the rest of the query for each element in an array and collect the results.
//!   For example, `.friends.[].name` extracts the `name` field from each element in the `friends` array.
//! * **`.[n]` syntax:** You can use the `.[n]` syntax to extract the nth element from an array.
//!   For example, `.friends.[0]` extracts the first element of the `friends` array.
//!
//! [`serde::Deserialize`]: https://docs.serde.rs/serde/trait.Deserialize.html
//! [`serde_query::Deserialize`]: derive.Deserialize.html
//! [`serde_query::DeserializeQuery`]: trait.DeserializeQuery.html
//! [`Query<T>`]: trait.DeserializeQuery.html#associatedtype.Query

/// Derive macro that generates [`serde::Deserialize`] directly.
///
/// Please refer to the [module-level documentation] for the usage.
///
/// [`serde::Deserialize`]: https://docs.serde.rs/serde/trait.Deserialize.html
/// [module-level documentation]: index.html
pub use serde_query_derive::Deserialize;

/// Derive macro for [`DeserializeQuery`] trait.
///
/// Please refer to the [module-level documentation] for the usage.
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
///     #[query(".commits.[].author")]
///     authors: Vec<String>,
///     #[query(".count")]
///     count: usize,
/// }
///
/// let document = serde_json::json!({
///     "commits": [
///         {
///             "author": "Kou",
///             "hash": 0x0202,
///         },
///         {
///             "author": "Kasumi",
///             "hash": 0x1013,
///         },
///         {
///             "author": "Masaru",
///             "hash": 0x0809,
///         },
///     ],
///     "count": 3,
/// }).to_string();
///
/// // You can use `Query<T>` as a `Deserialize` type for any `Deserializer`
/// // and convert the result to the desired type using `From`/`Into`.
/// let data: Data = serde_json::from_str::<Query<Data>>(&document)?.into();
///
/// assert_eq!(data.authors, vec!["Kou", "Kasumi", "Masaru"]);
/// assert_eq!(data.count, 3);
/// # Ok(())
/// # }
/// ```
///
/// [`DeserializeQuery`]: trait.DeserializeQuery.html
/// [module-level documentation]: index.html
pub use serde_query_derive::DeserializeQuery;

use core::ops::{Deref, DerefMut};
use serde::de::Deserialize;

/// Convenient type alias for the query type.
///
/// Please refer to the [`DeserializeQuery`] trait for details.
///
/// [`DeserializeQuery`]: trait.DeserializeQuery.html
pub type Query<'de, T> = <T as DeserializeQuery<'de>>::Query;

/// A **data structure** that can be deserialized with a query.
///
/// The [`Query`] type is a `#[repr(transparent)]` wrapper automatically generated by
/// [the proc macro]. You can deserialize `Query<YourType>` first and then call `.into()`
/// to get `YourType`.
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

    extern crate alloc;

    #[derive(Debug)]
    pub struct Error {
        field: &'static str,
        prefix: &'static str,
        message: alloc::borrow::Cow<'static, str>,
    }

    impl core::fmt::Display for Error {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "Query for field '{}' failed at '{}': {}",
                self.field, self.prefix, self.message
            )
        }
    }

    impl Error {
        pub fn owned(field: &'static str, prefix: &'static str, message: String) -> Self {
            Error {
                field,
                prefix,
                message: alloc::borrow::Cow::Owned(message),
            }
        }

        pub fn borrowed(field: &'static str, prefix: &'static str, message: &'static str) -> Self {
            Error {
                field,
                prefix,
                message: alloc::borrow::Cow::Borrowed(message),
            }
        }
    }

    #[derive(Debug)]
    pub struct Errors<'a> {
        errors: &'a [Option<Error>],
    }

    impl<'a> core::fmt::Display for Errors<'a> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self.error_count() {
                0 => Ok(()),
                1 => {
                    let error = self.errors().next().unwrap();
                    error.fmt(f)
                }
                _ => {
                    write!(f, "Queries failed for fields: ")?;
                    let mut following = false;
                    for error in self.errors() {
                        if following {
                            f.write_str(", ")?;
                        }
                        write!(f, "'{}'", error.field)?;
                        following = true;
                    }
                    f.write_str("\n")?;

                    let mut index = 1;
                    for error in self.errors() {
                        writeln!(f, "  {}. {}", index, error)?;
                        index += 1;
                    }

                    Ok(())
                }
            }
        }
    }

    impl<'a> Errors<'a> {
        pub fn new(errors: &'a [Option<Error>]) -> Self {
            Self { errors }
        }

        fn error_count(&self) -> usize {
            self.errors
                .iter()
                .map(|opt| if opt.is_some() { 1 } else { 0 })
                .sum()
        }

        fn errors(&self) -> impl Iterator<Item = &Error> {
            self.errors.iter().flatten()
        }
    }

    pub trait Container {
        type Element;

        fn empty() -> Self;

        fn reserve(&mut self, additional: usize);

        fn extend_one(&mut self, element: Self::Element);
    }

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
