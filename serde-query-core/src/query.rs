use proc_macro2::TokenStream;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum QueryFragment {
    Accept,
    /// '.' <name> [.<rest>]
    Field {
        name: String,
        rest: Box<QueryFragment>,
    },
    /// '.' '[' <n> ']' [.<rest>]
    IndexArray {
        index: usize,
        rest: Box<QueryFragment>,
    },
    /// '.[]' [.<rest>]
    CollectArray {
        rest: Box<QueryFragment>,
    },
}

impl QueryFragment {
    pub(crate) fn accept() -> Self {
        Self::Accept
    }

    pub(crate) fn field(name: String, rest: Self) -> Self {
        Self::Field {
            name,
            rest: rest.into(),
        }
    }

    pub(crate) fn index_array(index: usize, rest: Self) -> Self {
        Self::IndexArray {
            index,
            rest: rest.into(),
        }
    }

    pub(crate) fn collect_array(rest: Self) -> Self {
        Self::CollectArray { rest: rest.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct QueryId(syn::Ident);

impl QueryId {
    pub(crate) fn new(identifier: syn::Ident) -> Self {
        Self(identifier)
    }

    pub(crate) fn ident(&self) -> &syn::Ident {
        &self.0
    }
}

#[derive(Debug)]
pub(crate) struct Query {
    pub(crate) id: QueryId,
    pub(crate) fragment: QueryFragment,
    pub(crate) ty: TokenStream,
}

impl Query {
    pub(crate) fn new(id: QueryId, fragment: QueryFragment, ty: TokenStream) -> Self {
        Self { id, fragment, ty }
    }
}
