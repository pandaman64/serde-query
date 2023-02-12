use std::collections::BTreeMap;

use proc_macro2::TokenStream;

#[derive(Debug)]
pub enum QueryFragment {
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
    pub fn accept() -> Self {
        Self::Accept
    }

    pub fn field(name: String, rest: Self) -> Self {
        Self::Field {
            name,
            rest: rest.into(),
        }
    }

    pub fn index_array(index: usize, rest: Self) -> Self {
        Self::IndexArray {
            index,
            rest: rest.into(),
        }
    }

    pub fn collect_array(rest: Self) -> Self {
        Self::CollectArray { rest: rest.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QueryId(String);

impl QueryId {
    pub fn new(identifier: String) -> Self {
        Self(identifier)
    }
}

#[derive(Debug)]
pub struct Query {
    id: QueryId,
    fragment: QueryFragment,
    ty: TokenStream,
}

impl Query {
    pub fn new(id: QueryId, fragment: QueryFragment, ty: TokenStream) -> Self {
        Self { id, fragment, ty }
    }
}

#[derive(Debug, Default)]
pub struct Env {
    node_count: usize,
}

impl Env {
    pub fn new() -> Self {
        Self::default()
    }

    fn new_node_name(&mut self) -> String {
        let node_id = self.node_count;
        self.node_count += 1;
        format!("Node{node_id}")
    }
}

#[derive(Debug)]
enum NodeKind {
    None,
    Accept,
    Field { fields: BTreeMap<String, Node> },
    IndexArray { indices: BTreeMap<usize, Node> },
    CollectArray { child: Box<Node> },
}

impl NodeKind {
    fn merge_trees<K: Ord>(
        mut tree: BTreeMap<K, Node>,
        other: BTreeMap<K, Node>,
    ) -> BTreeMap<K, Node> {
        for (key, node) in other.into_iter() {
            use std::collections::btree_map::Entry;
            match tree.entry(key) {
                Entry::Vacant(v) => {
                    v.insert(node);
                }
                Entry::Occupied(mut o) => {
                    o.get_mut().merge(node);
                }
            }
        }
        tree
    }

    fn merge(&mut self, other: Self) {
        let this = std::mem::replace(self, Self::None);
        *self = match (this, other) {
            (NodeKind::None, other) => other,
            (NodeKind::Accept, NodeKind::Accept) => NodeKind::Accept,
            (NodeKind::Field { fields }, NodeKind::Field { fields: other }) => NodeKind::Field {
                fields: Self::merge_trees(fields, other),
            },
            (NodeKind::IndexArray { indices }, NodeKind::IndexArray { indices: other }) => {
                NodeKind::IndexArray {
                    indices: Self::merge_trees(indices, other),
                }
            }
            (NodeKind::CollectArray { mut child }, NodeKind::CollectArray { child: other }) => {
                child.merge(*other);
                NodeKind::CollectArray { child }
            }
            // TODO: better handling
            _ => panic!("user error"),
        }
    }
}

#[derive(Debug)]
pub struct Node {
    name: String,
    // map of (id, ty)
    queries: BTreeMap<QueryId, TokenStream>,
    kind: NodeKind,
}

impl Node {
    pub fn from_query(
        env: &mut Env,
        id: QueryId,
        fragment: QueryFragment,
        ty: TokenStream,
    ) -> Self {
        let name = env.new_node_name();
        match fragment {
            QueryFragment::Accept => Self {
                name,
                queries: BTreeMap::from_iter([(id, ty)]),
                kind: NodeKind::Accept,
            },
            QueryFragment::Field {
                name: field_name,
                rest,
            } => {
                let kind = NodeKind::Field {
                    fields: BTreeMap::from_iter([(
                        field_name,
                        Self::from_query(env, id.clone(), *rest, ty.clone()),
                    )]),
                };
                Self {
                    name,
                    queries: BTreeMap::from_iter([(id, ty)]),
                    kind,
                }
            }
            QueryFragment::IndexArray { index, rest } => {
                let kind = NodeKind::IndexArray {
                    indices: BTreeMap::from_iter([(
                        index,
                        Self::from_query(env, id.clone(), *rest, ty.clone()),
                    )]),
                };
                Self {
                    name,
                    queries: BTreeMap::from_iter([(id, ty)]),
                    kind,
                }
            }
            QueryFragment::CollectArray { rest } => {
                let unvec_ty = quote::quote!(<#ty as UnVec>::Element);
                let kind = NodeKind::CollectArray {
                    child: Box::new(Self::from_query(env, id.clone(), *rest, unvec_ty)),
                };
                Self {
                    name,
                    queries: BTreeMap::from_iter([(id, ty)]),
                    kind,
                }
            }
        }
    }

    fn merge(&mut self, other: Self) {
        self.kind.merge(other.kind);
        self.queries.extend(other.queries.into_iter());
    }

    fn deserialize_seed_ty(&self) -> syn::Ident {
        quote::format_ident!("DeserializeSeed{}", self.name)
    }

    fn visitor_ty(&self) -> syn::Ident {
        quote::format_ident!("Visitor{}", self.name)
    }

    pub fn generate(&self) -> TokenStream {
        match &self.kind {
            NodeKind::Accept => {
                // TODO: what if we have multiple queries?
                let first_query = self.queries.first_key_value().unwrap();
                let query_name = quote::format_ident!("{}", first_query.0 .0);
                let query_type = first_query.1;

                let deserialize_seed_ty = self.deserialize_seed_ty();

                quote::quote! {
                    struct #deserialize_seed_ty<'query> {
                        #query_name: &'query mut core::option::Option<#query_type>,
                    }

                    impl<'query, 'de> serde::de::DeserializeSeed<'de> for #deserialize_seed_ty<'query> {
                        type Value = ();

                        fn deserialize<D>(self, deserializer: D) -> core::result::Result<Self::Value, D::Error>
                        where
                            D: serde::Deserializer<'de>,
                        {
                            *self.#query_name = core::option::Option::Some(<#query_type as serde::Deserialize<'de>>::deserialize(deserializer)?);
                            Ok(())
                        }
                    }
                }
            }
            NodeKind::Field { fields } => {
                let deserialize_seed_ty = self.deserialize_seed_ty();
                let visitor_ty = self.visitor_ty();

                let query_names: Vec<_> = self
                    .queries
                    .keys()
                    .map(|id| quote::format_ident!("{}", id.0))
                    .collect();
                let query_types: Vec<_> = self.queries.values().collect();

                let match_arms = fields.iter().map(|(field, node)| {
                    let deserialize_seed_ty = node.deserialize_seed_ty();
                    let query_names: Vec<_> = node
                        .queries
                        .keys()
                        .map(|id| quote::format_ident!("{}", id.0))
                        .collect();

                    quote::quote! {
                        #field => {
                            map.next_value_seed(#deserialize_seed_ty {
                                #(
                                    #query_names: self.#query_names,
                                )*
                            })?;
                        }
                    }
                });

                let child_code = fields.values().map(|node| node.generate());

                quote::quote! {
                    struct #deserialize_seed_ty<'query> {
                        #(
                            #query_names: &'query mut core::option::Option<#query_types>,
                        )*
                    }

                    impl<'query, 'de> serde::de::DeserializeSeed<'de> for #deserialize_seed_ty<'query> {
                        type Value = ();

                        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where
                            D: serde::Deserializer<'de>,
                        {
                            let visitor = #visitor_ty {
                                #(
                                    #query_names: self.#query_names,
                                )*
                            };
                            deserializer.deserialize_map(visitor)
                        }
                    }

                    struct #visitor_ty<'query> {
                        #(
                            #query_names: &'query mut core::option::Option<#query_types>,
                        )*
                    }

                    // TODO: efficient parsing for keys
                    impl<'query, 'de> serde::de::Visitor<'de> for #visitor_ty<'query> {
                        type Value = ();

                        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                            // TODO: implement
                            Ok(())
                        }

                        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::MapAccess<'de>,
                        {
                            while let Some(key) = map.next_key::<String>()? {
                                match key.as_str() {
                                    #(#match_arms)*
                                    _ => {
                                        map.next_value::<serde::de::IgnoredAny>()?;
                                    }
                                }
                            }
                            Ok(())
                        }
                    }

                    #(#child_code)*
                }
            }
            NodeKind::IndexArray { indices } => {
                let deserialize_seed_ty = self.deserialize_seed_ty();
                let visitor_ty = self.visitor_ty();

                let query_names: Vec<_> = self
                    .queries
                    .keys()
                    .map(|id| quote::format_ident!("{}", id.0))
                    .collect();
                let query_types: Vec<_> = self.queries.values().collect();

                let match_arms = indices.iter().map(|(index, node)| {
                    let deserialize_seed_ty = node.deserialize_seed_ty();
                    let query_names: Vec<_> = node
                        .queries
                        .keys()
                        .map(|id| quote::format_ident!("{}", id.0))
                        .collect();

                    quote::quote! {
                        #index => {
                            match seq.next_element_seed(#deserialize_seed_ty {
                                #(
                                    #query_names: self.#query_names,
                                )*
                            })? {
                                core::option::Option::Some(_) => {},
                                core::option::Option::None => break,
                            };
                        }
                    }
                });

                let child_code = indices.values().map(|node| node.generate());

                quote::quote! {
                    struct #deserialize_seed_ty<'query> {
                        #(
                            #query_names: &'query mut core::option::Option<#query_types>,
                        )*
                    }

                    impl<'query, 'de> serde::de::DeserializeSeed<'de> for #deserialize_seed_ty<'query> {
                        type Value = ();

                        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where
                            D: serde::Deserializer<'de>,
                        {
                            let visitor = #visitor_ty {
                                #(
                                    #query_names: self.#query_names,
                                )*
                            };
                            deserializer.deserialize_seq(visitor)
                        }
                    }

                    struct #visitor_ty<'query> {
                        #(
                            #query_names: &'query mut core::option::Option<#query_types>,
                        )*
                    }

                    impl<'query, 'de> serde::de::Visitor<'de> for #visitor_ty<'query> {
                        type Value = ();

                        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                            // TODO: implement
                            Ok(())
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::SeqAccess<'de>,
                        {
                            let mut current_index = 0usize;
                            loop {
                                match current_index {
                                    #(#match_arms)*
                                    _ => {
                                        match seq.next_element::<serde::de::IgnoredAny>()? {
                                            core::option::Option::Some(_) => {},
                                            core::option::Option::None => break,
                                        }
                                    }
                                }
                            }
                            Ok(())
                        }
                    }

                    #(#child_code)*
                }
            }
            NodeKind::CollectArray { child } => {
                let deserialize_seed_ty = self.deserialize_seed_ty();
                let visitor_ty = self.visitor_ty();

                let query_names: Vec<_> = self
                    .queries
                    .keys()
                    .map(|id| quote::format_ident!("{}", id.0))
                    .collect();
                let query_types: Vec<_> = self.queries.values().collect();

                let child_code = child.generate();
                let child_deserialize_seed_ty = child.deserialize_seed_ty();
                // child_query_names should be equal to those of self

                quote::quote! {
                    struct #deserialize_seed_ty<'query> {
                        #(
                            #query_names: &'query mut core::option::Option<alloc::vec::Vec<<#query_types as UnVec>::Element>>,
                        )*
                    }

                    impl<'query, 'de> serde::de::DeserializeSeed<'de> for #deserialize_seed_ty<'query> {
                        type Value = ();

                        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where
                            D: serde::Deserializer<'de>,
                        {
                            #(
                                let mut #query_names = alloc::vec::Vec::new();
                            )*
                            let visitor = #visitor_ty {
                                #(
                                    #query_names: &mut #query_names,
                                )*
                            };
                            deserializer.deserialize_seq(visitor)?;
                            #(
                                *self.#query_names = Some(#query_names);
                            )*
                            Ok(())
                        }
                    }

                    struct #visitor_ty<'query> {
                        #(
                            #query_names: &'query mut alloc::vec::Vec<<#query_types as UnVec>::Element>,
                        )*
                    }

                    impl<'query, 'de> serde::de::Visitor<'de> for #visitor_ty<'query> {
                        type Value = ();

                        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                            // TODO: implement
                            Ok(())
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::SeqAccess<'de>,
                        {
                            // TODO: extend
                            loop {
                                #(
                                    let mut #query_names = None;
                                )*
                                match seq.next_element_seed(#child_deserialize_seed_ty {
                                    #(
                                        #query_names: &mut #query_names,
                                    )*
                                })? {
                                    Some(()) => {
                                        #(
                                            self.#query_names.push(
                                                #query_names.unwrap()
                                            );
                                        )*
                                    }
                                    None => {
                                        break;
                                    }
                                };
                            }
                            Ok(())
                        }
                    }

                    #child_code
                }
            }
            NodeKind::None => unreachable!(),
        }
    }
}

pub fn compile<I: Iterator<Item = Query>>(env: &mut Env, queries: I) -> Node {
    let mut node = Node {
        name: env.new_node_name(),
        queries: BTreeMap::new(),
        kind: NodeKind::None,
    };
    for query in queries {
        node.merge(Node::from_query(env, query.id, query.fragment, query.ty));
    }
    node
}
