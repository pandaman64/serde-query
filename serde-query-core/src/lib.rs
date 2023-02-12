use std::collections::BTreeMap;

use proc_macro2::{Literal, TokenStream};

#[derive(Debug, PartialEq, Eq)]
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
pub struct QueryId(syn::Ident);

impl QueryId {
    pub fn new(identifier: syn::Ident) -> Self {
        Self(identifier)
    }

    fn ident(&self) -> &syn::Ident {
        &self.0
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
                let element_ty = quote::quote!(<#ty as serde_query::__priv::Container>::Element);
                let kind = NodeKind::CollectArray {
                    child: Box::new(Self::from_query(env, id.clone(), *rest, element_ty)),
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

    fn field_deserialize_enum_ty(&self) -> syn::Ident {
        quote::format_ident!("Field{}", self.name)
    }

    fn field_visitor_ty(&self) -> syn::Ident {
        quote::format_ident!("FieldVisitor{}", self.name)
    }

    fn query_names(&self) -> Vec<&syn::Ident> {
        self.queries.keys().map(QueryId::ident).collect()
    }

    fn query_types(&self) -> Vec<&TokenStream> {
        self.queries.values().collect()
    }

    pub fn generate(&self) -> TokenStream {
        match &self.kind {
            NodeKind::Accept => {
                // TODO: what if we have multiple queries?
                let (query_id, query_type) = self.queries.first_key_value().unwrap();
                let query_name = query_id.ident();

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
                let field_deserialize_enum_ty = self.field_deserialize_enum_ty();
                let field_visitor_ty = self.field_visitor_ty();

                let query_names = self.query_names();
                let query_types = self.query_types();

                let field_ids: Vec<_> = (0..fields.len())
                    .map(|idx| quote::format_ident!("Field{}", idx))
                    .collect();
                let field_names: Vec<_> = fields.keys().collect();
                let byte_field_names: Vec<_> = fields
                    .keys()
                    .map(|name| Literal::byte_string(name.as_bytes()))
                    .collect();

                let match_arms =
                    fields
                        .iter()
                        .zip(field_ids.iter())
                        .map(|((_, node), field_id)| {
                            let deserialize_seed_ty = node.deserialize_seed_ty();
                            let query_names = node.query_names();

                            quote::quote! {
                                #field_deserialize_enum_ty :: #field_id => {
                                    map.next_value_seed(#deserialize_seed_ty {
                                        #(
                                            #query_names: self.#query_names,
                                        )*
                                    })?;
                                }
                            }
                        });

                let expecting = {
                    let field_names: Vec<_> =
                        fields.keys().map(|name| format!("'{name}'")).collect();
                    format!("one of the following fields: {}", field_names.join(", or "))
                };

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

                    impl<'query, 'de> serde::de::Visitor<'de> for #visitor_ty<'query> {
                        type Value = ();

                        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                            core::fmt::Formatter::write_str(formatter, #expecting)
                        }

                        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::MapAccess<'de>,
                        {
                            while let Some(key) = map.next_key::<#field_deserialize_enum_ty>()? {
                                match key {
                                    #(#match_arms)*
                                    #field_deserialize_enum_ty :: Ignore => {
                                        map.next_value::<serde::de::IgnoredAny>()?;
                                    }
                                }
                            }
                            Ok(())
                        }
                    }

                    enum #field_deserialize_enum_ty {
                        #(
                            #field_ids,
                        )*
                        Ignore,
                    }

                    impl<'de> serde::de::Deserialize<'de> for #field_deserialize_enum_ty {
                        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where
                            D: serde::Deserializer<'de>,
                        {
                            deserializer.deserialize_identifier(#field_visitor_ty)
                        }
                    }

                    struct #field_visitor_ty;

                    impl<'de> serde::de::Visitor<'de> for #field_visitor_ty {
                        type Value = #field_deserialize_enum_ty;

                        fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                            core::fmt::Formatter::write_str(f, #expecting)
                        }

                        fn visit_str<E>(self, value: &str) -> core::result::Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            match value {
                                #(
                                    #field_names => core::result::Result::Ok(#field_deserialize_enum_ty :: #field_ids),
                                )*
                                _ => core::result::Result::Ok(#field_deserialize_enum_ty :: Ignore),
                            }
                        }

                        fn visit_bytes<E>(self, value: &[u8]) -> core::result::Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            match value {
                                #(
                                    #byte_field_names => core::result::Result::Ok(#field_deserialize_enum_ty :: #field_ids),
                                )*
                                _ => core::result::Result::Ok(#field_deserialize_enum_ty :: Ignore),
                            }
                        }
                    }

                    #(#child_code)*
                }
            }
            NodeKind::IndexArray { indices } => {
                let deserialize_seed_ty = self.deserialize_seed_ty();
                let visitor_ty = self.visitor_ty();

                let query_names = self.query_names();
                let query_types = self.query_types();

                let match_arms = indices.iter().map(|(index, node)| {
                    let deserialize_seed_ty = node.deserialize_seed_ty();
                    let query_names = node.query_names();

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

                let (max_index, _) = indices
                    .last_key_value()
                    .expect("IndexArray node must have at least one element");
                let expecting = format!("a sequence with at least {} elements", max_index + 1);

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
                            core::fmt::Formatter::write_str(formatter, #expecting)
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
                                current_index += 1;
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

                let query_names = self.query_names();
                let query_types = self.query_types();
                let error_messages: Vec<_> = query_names
                    .iter()
                    .map(|name| format!("Query for '{}' failed to run", name))
                    .collect();

                let child_code = child.generate();
                let child_deserialize_seed_ty = child.deserialize_seed_ty();
                // child_query_names should be equal to those of self

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
                            #(
                                let mut #query_names = <#query_types as serde_query::__priv::Container>::empty();
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
                            #query_names: &'query mut #query_types,
                        )*
                    }

                    impl<'query, 'de> serde::de::Visitor<'de> for #visitor_ty<'query> {
                        type Value = ();

                        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                            core::fmt::Formatter::write_str(formatter, "a sequence")
                        }

                        fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
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
                                            <#query_types as serde_query::__priv::Container>::extend_one(
                                                &mut self.#query_names,
                                                match #query_names {
                                                    core::option::Option::Some(v) => v,
                                                    core::option::Option::None => {
                                                        return core::result::Result::Err(
                                                            <<A as serde::de::SeqAccess<'de>>::Error as serde::de::Error>::custom(#error_messages)
                                                        )
                                                    }
                                                },
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

    pub fn generate_deserialize<F: FnOnce(TokenStream) -> TokenStream>(
        &self,
        struct_ty: &syn::Ident,
        implementor_ty: &syn::Ident,
        construction: F,
    ) -> TokenStream {
        let deserialize_seed_ty = self.deserialize_seed_ty();
        let query_names = self.query_names();
        let error_messages: Vec<_> = query_names
            .iter()
            .map(|name| format!("Query for '{}' failed to run", name))
            .collect();
        let construction = construction(quote::quote!(value));
        quote::quote! {
            impl<'de> serde::de::Deserialize<'de> for #implementor_ty {
                fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
                where
                    D: serde::de::Deserializer<'de>
                {
                    #(
                        let mut #query_names = None;
                    )*
                    let root = #deserialize_seed_ty {
                        #(
                            #query_names: &mut #query_names,
                        )*
                    };
                    <#deserialize_seed_ty as serde::de::DeserializeSeed<'de>>::deserialize(root, deserializer)?;
                    let value = #struct_ty {
                        #(
                            #query_names: match #query_names {
                                core::option::Option::Some(v) => v,
                                core::option::Option::None => {
                                    return core::result::Result::Err(<D::Error as serde::de::Error>::custom(#error_messages))
                                }
                            },
                        )*
                    };
                    Ok(#construction)
                }
            }
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
