use std::collections::BTreeMap;

use proc_macro2::{Literal, TokenStream};
use proc_macro_error::{diagnostic, Diagnostic, Level};

use crate::query::{Query, QueryFragment, QueryId};

#[derive(Debug, Default)]
struct Env {
    node_count: usize,
}

impl Env {
    fn new() -> Self {
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
    ) -> Result<BTreeMap<K, Node>, Diagnostic> {
        for (key, node) in other.into_iter() {
            use std::collections::btree_map::Entry;
            match tree.entry(key) {
                Entry::Vacant(v) => {
                    v.insert(node);
                }
                Entry::Occupied(mut o) => {
                    o.get_mut().merge(node)?;
                }
            }
        }
        Ok(tree)
    }

    fn merge(
        &mut self,
        other: Self,
        prefix: &str,
        self_query: Option<&QueryId>,
        other_query: &QueryId,
    ) -> Result<(), Diagnostic> {
        let this = std::mem::replace(self, Self::None);
        *self = match (this, other) {
            (NodeKind::None, other) => other,
            (NodeKind::Accept, NodeKind::Accept) => NodeKind::Accept,
            (NodeKind::Field { fields }, NodeKind::Field { fields: other }) => NodeKind::Field {
                fields: Self::merge_trees(fields, other)?,
            },
            (NodeKind::IndexArray { indices }, NodeKind::IndexArray { indices: other }) => {
                NodeKind::IndexArray {
                    indices: Self::merge_trees(indices, other)?,
                }
            }
            (NodeKind::CollectArray { mut child }, NodeKind::CollectArray { child: other }) => {
                child.merge(*other)?;
                NodeKind::CollectArray { child }
            }
            (this, other) => {
                let self_ident = self_query.expect("This node must have at least one query because the kind is not NodeKind::None").ident();
                let other_ident = other_query.ident();
                return Err(diagnostic!(
                    self_ident,
                    Level::Error,
                    "Conflicting query at '{}'. The query for field '{}' expects {} while the query for field '{}' expects {}.",
                    prefix,
                    self_ident,
                    this.descripion(),
                    other_ident,
                    other.descripion()
                ));
            }
        };
        Ok(())
    }

    fn descripion(&self) -> &str {
        match self {
            NodeKind::None => "none",
            NodeKind::Accept => "a value here",
            NodeKind::Field { .. } => "a struct",
            NodeKind::IndexArray { .. } | NodeKind::CollectArray { .. } => "a sequence",
        }
    }
}

#[derive(Debug)]
pub(crate) struct Node {
    name: String,
    // map of (id, ty)
    queries: BTreeMap<QueryId, TokenStream>,
    kind: NodeKind,

    // fields for diagnostics
    /// The prefix of the queries to reach this node.
    prefix: String,
}

impl Node {
    pub(crate) fn from_queries<I: Iterator<Item = Query>>(
        queries: I,
    ) -> Result<Node, Vec<Diagnostic>> {
        let mut diagnostics = vec![];
        let mut env = Env::new();
        let mut node = Node {
            name: env.new_node_name(),
            queries: BTreeMap::new(),
            kind: NodeKind::None,
            prefix: String::from("."),
        };
        for query in queries {
            if let Err(diagnostic) = node.merge(Node::from_query(
                &mut env,
                query.id,
                query.fragment,
                query.ty,
                String::new(),
            )) {
                diagnostics.push(diagnostic);
            }
        }

        if diagnostics.is_empty() {
            Ok(node)
        } else {
            Err(diagnostics)
        }
    }

    fn from_query(
        env: &mut Env,
        id: QueryId,
        fragment: QueryFragment,
        ty: TokenStream,
        prefix: String,
    ) -> Self {
        let name = env.new_node_name();
        match fragment {
            QueryFragment::Accept => Self {
                name,
                queries: BTreeMap::from_iter([(id, ty)]),
                kind: NodeKind::Accept,
                prefix,
            },
            QueryFragment::Field {
                name: field_name,
                quoted,
                rest,
            } => {
                let rest_prefix = if quoted {
                    format!("{}.[\"{}\"]", prefix, field_name)
                } else {
                    format!("{}.{}", prefix, field_name)
                };
                let child = Self::from_query(env, id.clone(), *rest, ty.clone(), rest_prefix);
                let kind = NodeKind::Field {
                    fields: BTreeMap::from_iter([(field_name, child)]),
                };
                Self {
                    name,
                    queries: BTreeMap::from_iter([(id, ty)]),
                    kind,
                    prefix,
                }
            }
            QueryFragment::IndexArray { index, rest } => {
                let child = Self::from_query(
                    env,
                    id.clone(),
                    *rest,
                    ty.clone(),
                    format!("{}.[{}]", prefix, index),
                );
                let kind = NodeKind::IndexArray {
                    indices: BTreeMap::from_iter([(index, child)]),
                };
                Self {
                    name,
                    queries: BTreeMap::from_iter([(id, ty)]),
                    kind,
                    prefix,
                }
            }
            QueryFragment::CollectArray { rest } => {
                let element_ty = quote::quote!(<#ty as serde_query::__priv::Container>::Element);
                let child = Box::new(Self::from_query(
                    env,
                    id.clone(),
                    *rest,
                    element_ty,
                    format!("{}.[]", prefix),
                ));
                let kind = NodeKind::CollectArray { child };
                Self {
                    name,
                    queries: BTreeMap::from_iter([(id, ty)]),
                    kind,
                    prefix,
                }
            }
        }
    }

    fn merge(&mut self, other: Self) -> Result<(), Diagnostic> {
        let self_query = self.queries.first_key_value().map(|(id, _)| id);
        let other_query = other.queries.first_key_value().unwrap().0;

        self.kind
            .merge(other.kind, &self.prefix, self_query, other_query)?;
        self.queries.extend(other.queries.into_iter());
        Ok(())
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

    fn missing_fields_error_triple<K: Clone + Ord>(
        children: &BTreeMap<K, Node>,
    ) -> (Vec<K>, Vec<syn::Ident>, Vec<String>) {
        let mut keys = vec![];
        let mut idents = vec![];
        let mut ident_strings = vec![];

        for (field, node) in children.iter() {
            for id in node.queries.keys() {
                keys.push(field.clone());
                idents.push(id.ident().clone());
                ident_strings.push(id.ident().to_string());
            }
        }

        (keys, idents, ident_strings)
    }

    pub(crate) fn generate(&self) -> TokenStream {
        match &self.kind {
            NodeKind::Accept => {
                // TODO: what if we have multiple queries?
                let (query_id, query_type) = self.queries.first_key_value().unwrap();
                let query_name = query_id.ident();

                let deserialize_seed_ty = self.deserialize_seed_ty();

                let field = query_name.to_string();
                let prefix = &self.prefix;

                quote::quote! {
                    struct #deserialize_seed_ty<'query> {
                        #query_name: &'query mut core::option::Option<
                            core::result::Result<
                                #query_type,
                                serde_query::__priv::Error,
                            >
                        >,
                    }

                    impl<'query, 'de> serde_query::__priv::serde::de::DeserializeSeed<'de> for #deserialize_seed_ty<'query> {
                        type Value = ();

                        fn deserialize<D>(self, deserializer: D) -> core::result::Result<Self::Value, D::Error>
                        where
                            D: serde_query::__priv::serde::Deserializer<'de>,
                        {
                            let result = match <#query_type as serde_query::__priv::serde::Deserialize<'de>>::deserialize(deserializer) {
                                core::result::Result::Ok(v) => core::result::Result::Ok(v),
                                core::result::Result::Err(e) => core::result::Result::Err(
                                    serde_query::__priv::Error::owned(#field, #prefix, e.to_string())
                                ),
                            };
                            *self.#query_name = core::option::Option::Some(result);
                            core::result::Result::Ok(())
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

                let (missing_field_names, missing_query_names, missing_query_name_strings) =
                    Self::missing_fields_error_triple(fields);
                let missing_field_error_messages = missing_field_names
                    .into_iter()
                    .map(|field_name| format!("missing field '{}'", field_name));
                let prefix = &self.prefix;

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
                            #query_names: &'query mut core::option::Option<
                                core::result::Result<
                                    #query_types,
                                    serde_query::__priv::Error,
                                >
                            >,
                        )*
                    }

                    impl<'query, 'de> serde_query::__priv::serde::de::DeserializeSeed<'de> for #deserialize_seed_ty<'query> {
                        type Value = ();

                        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where
                            D: serde_query::__priv::serde::Deserializer<'de>,
                        {
                            let visitor = #visitor_ty {
                                #(
                                    #query_names: self.#query_names,
                                )*
                            };
                            deserializer.deserialize_map(visitor)?;
                            #(
                                if self.#missing_query_names.is_none() {
                                    *self.#missing_query_names = core::option::Option::Some(
                                        core::result::Result::Err(
                                            serde_query::__priv::Error::borrowed(
                                                #missing_query_name_strings,
                                                #prefix,
                                                #missing_field_error_messages,
                                            )
                                        )
                                    );
                                }
                            )*

                            core::result::Result::Ok(())
                        }
                    }

                    struct #visitor_ty<'query> {
                        #(
                            #query_names: &'query mut core::option::Option<
                                core::result::Result<
                                    #query_types,
                                    serde_query::__priv::Error,
                                >
                            >,
                        )*
                    }

                    impl<'query, 'de> serde_query::__priv::serde::de::Visitor<'de> for #visitor_ty<'query> {
                        type Value = ();

                        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                            core::fmt::Formatter::write_str(formatter, #expecting)
                        }

                        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde_query::__priv::serde::de::MapAccess<'de>,
                        {
                            while let core::option::Option::Some(key) = map.next_key::<#field_deserialize_enum_ty>()? {
                                match key {
                                    #(#match_arms)*
                                    #field_deserialize_enum_ty :: Ignore => {
                                        map.next_value::<serde_query::__priv::serde::de::IgnoredAny>()?;
                                    }
                                }
                            }
                            core::result::Result::Ok(())
                        }
                    }

                    enum #field_deserialize_enum_ty {
                        #(
                            #field_ids,
                        )*
                        Ignore,
                    }

                    impl<'de> serde_query::__priv::serde::de::Deserialize<'de> for #field_deserialize_enum_ty {
                        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where
                            D: serde_query::__priv::serde::Deserializer<'de>,
                        {
                            deserializer.deserialize_identifier(#field_visitor_ty)
                        }
                    }

                    struct #field_visitor_ty;

                    impl<'de> serde_query::__priv::serde::de::Visitor<'de> for #field_visitor_ty {
                        type Value = #field_deserialize_enum_ty;

                        fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                            core::fmt::Formatter::write_str(f, #expecting)
                        }

                        fn visit_str<E>(self, value: &str) -> core::result::Result<Self::Value, E>
                        where
                            E: serde_query::__priv::serde::de::Error,
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
                            E: serde_query::__priv::serde::de::Error,
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

                let (missing_field_names, missing_query_names, missing_query_name_strings) =
                    Self::missing_fields_error_triple(indices);
                let missing_field_error_messages = missing_field_names
                    .into_iter()
                    .map(|index| format!("the sequence must have at least {} elements", index + 1));
                let prefix = &self.prefix;

                let (max_index, _) = indices
                    .last_key_value()
                    .expect("IndexArray node must have at least one element");
                let expecting = format!("a sequence with at least {} elements", max_index + 1);

                let child_code = indices.values().map(|node| node.generate());

                quote::quote! {
                    struct #deserialize_seed_ty<'query> {
                        #(
                            #query_names: &'query mut core::option::Option<
                                core::result::Result<
                                    #query_types,
                                    serde_query::__priv::Error,
                                >
                            >,
                        )*
                    }

                    impl<'query, 'de> serde_query::__priv::serde::de::DeserializeSeed<'de> for #deserialize_seed_ty<'query> {
                        type Value = ();

                        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where
                            D: serde_query::__priv::serde::Deserializer<'de>,
                        {
                            let visitor = #visitor_ty {
                                #(
                                    #query_names: self.#query_names,
                                )*
                            };
                            deserializer.deserialize_seq(visitor)?;
                            #(
                                if self.#missing_query_names.is_none() {
                                    *self.#missing_query_names = core::option::Option::Some(
                                        core::result::Result::Err(
                                            serde_query::__priv::Error::borrowed(
                                                #missing_query_name_strings,
                                                #prefix,
                                                #missing_field_error_messages,
                                            )
                                        )
                                    );
                                }
                            )*
                            core::result::Result::Ok(())
                        }
                    }

                    struct #visitor_ty<'query> {
                        #(
                            #query_names: &'query mut core::option::Option<
                                core::result::Result<
                                    #query_types,
                                    serde_query::__priv::Error,
                                >
                            >,
                        )*
                    }

                    impl<'query, 'de> serde_query::__priv::serde::de::Visitor<'de> for #visitor_ty<'query> {
                        type Value = ();

                        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                            core::fmt::Formatter::write_str(formatter, #expecting)
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde_query::__priv::serde::de::SeqAccess<'de>,
                        {
                            let mut current_index = 0usize;
                            loop {
                                match current_index {
                                    #(#match_arms)*
                                    _ => {
                                        match seq.next_element::<serde_query::__priv::serde::de::IgnoredAny>()? {
                                            core::option::Option::Some(_) => {},
                                            core::option::Option::None => break,
                                        }
                                    }
                                }
                                current_index += 1;
                            }
                            core::result::Result::Ok(())
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

                let child_code = child.generate();
                let child_deserialize_seed_ty = child.deserialize_seed_ty();
                // child_query_names should be equal to those of self

                quote::quote! {
                    struct #deserialize_seed_ty<'query> {
                        #(
                            #query_names: &'query mut core::option::Option<
                                core::result::Result<
                                    #query_types,
                                    serde_query::__priv::Error,
                                >
                            >,
                        )*
                    }

                    impl<'query, 'de> serde_query::__priv::serde::de::DeserializeSeed<'de> for #deserialize_seed_ty<'query> {
                        type Value = ();

                        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where
                            D: serde_query::__priv::serde::Deserializer<'de>,
                        {
                            #(
                                let mut #query_names = core::result::Result::Ok(
                                    <#query_types as serde_query::__priv::Container>::empty()
                                );
                            )*
                            let visitor = #visitor_ty {
                                #(
                                    #query_names: &mut #query_names,
                                )*
                            };
                            deserializer.deserialize_seq(visitor)?;
                            #(
                                *self.#query_names = core::option::Option::Some(#query_names);
                            )*
                            core::result::Result::Ok(())
                        }
                    }

                    struct #visitor_ty<'query> {
                        #(
                            #query_names: &'query mut core::result::Result<#query_types, serde_query::__priv::Error>,
                        )*
                    }

                    impl<'query, 'de> serde_query::__priv::serde::de::Visitor<'de> for #visitor_ty<'query> {
                        type Value = ();

                        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                            core::fmt::Formatter::write_str(formatter, "a sequence")
                        }

                        fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde_query::__priv::serde::de::SeqAccess<'de>,
                        {
                            if let core::option::Option::Some(additional) = seq.size_hint() {
                                #(
                                    <#query_types as serde_query::__priv::Container>::reserve(
                                        self.#query_names.as_mut().unwrap(),
                                        additional,
                                    );
                                )*
                            }
                            loop {
                                #(
                                    let mut #query_names = core::option::Option::None;
                                )*
                                match seq.next_element_seed(#child_deserialize_seed_ty {
                                    #(
                                        #query_names: &mut #query_names,
                                    )*
                                })? {
                                    core::option::Option::None => break,
                                    core::option::Option::Some(()) => {
                                        #(
                                            match &mut self.#query_names {
                                                core::result::Result::Ok(ref mut container) => match #query_names {
                                                    core::option::Option::Some(core::result::Result::Ok(v)) => {
                                                        <#query_types as serde_query::__priv::Container>::extend_one(
                                                            container,
                                                            v,
                                                        )
                                                    },
                                                    core::option::Option::Some(core::result::Result::Err(e)) => {
                                                        *self.#query_names = core::result::Result::Err(e);
                                                    },
                                                    core::option::Option::None => unreachable!(),
                                                },
                                                core::result::Result::Err(_) => {},
                                            }
                                        )*
                                    }
                                };
                            }
                            core::result::Result::Ok(())
                        }
                    }

                    #child_code
                }
            }
            NodeKind::None => {
                // No queries. Generate an empty DeserializeSeed for the root node.
                let deserialize_seed_ty = self.deserialize_seed_ty();

                quote::quote! {
                    struct #deserialize_seed_ty {}

                    impl<'de> serde_query::__priv::serde::de::DeserializeSeed<'de> for #deserialize_seed_ty {
                        type Value = ();

                        fn deserialize<D>(self, deserializer: D) -> core::result::Result<Self::Value, D::Error>
                        where
                            D: serde_query::__priv::serde::Deserializer<'de>,
                        {
                            core::result::Result::Ok(())
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn generate_deserialize<F: FnOnce(TokenStream) -> TokenStream>(
        &self,
        struct_ty: &syn::Ident,
        implementor_ty: &syn::Ident,
        construction: F,
    ) -> TokenStream {
        let deserialize_seed_ty = self.deserialize_seed_ty();
        let query_names = self.query_names();
        let construction = construction(quote::quote!(value));
        quote::quote! {
            impl<'de> serde_query::__priv::serde::de::Deserialize<'de> for #implementor_ty {
                fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
                where
                    D: serde_query::__priv::serde::de::Deserializer<'de>
                {
                    #(
                        let mut #query_names = core::option::Option::None;
                    )*
                    let root = #deserialize_seed_ty {
                        #(
                            #query_names: &mut #query_names,
                        )*
                    };
                    <#deserialize_seed_ty as serde_query::__priv::serde::de::DeserializeSeed<'de>>::deserialize(root, deserializer)?;

                    #(
                        let #query_names = #query_names.unwrap();
                    )*
                    let has_error = false #(
                        || #query_names.is_err()
                    )*;

                    if !has_error {
                        let value = #struct_ty {
                            #(
                                #query_names: #query_names.unwrap(),
                            )*
                        };
                        core::result::Result::Ok(#construction)
                    } else {
                        let errors = [
                            #(
                                #query_names.err(),
                            )*
                        ];
                        core::result::Result::Err(
                            <D::Error as serde_query::__priv::serde::de::Error>::custom(serde_query::__priv::Errors::new(&errors))
                        )
                    }
                }
            }
        }
    }
}
