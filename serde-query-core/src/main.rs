use std::collections::BTreeMap;

use proc_macro2::TokenStream;

#[derive(Debug)]
pub enum QueryFragment {
    Accept,
    /// .<name> [.<rest>]
    Field {
        name: String,
        rest: Box<QueryFragment>,
    },
    // TODO: name
    /// .[] [.<rest>]
    CollectArray {
        rest: Box<QueryFragment>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct QueryId(String);

#[derive(Debug)]
pub struct Query {
    id: QueryId,
    fragment: QueryFragment,
    // ident: String,
    ty: TokenStream,
}

#[derive(Debug, Default)]
struct Env {
    node_count: usize,
}

impl Env {
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
    Struct { fields: BTreeMap<String, Node> },
    Array { child: Box<Node> },
}

impl NodeKind {
    /// Collect queries in all children.
    fn child_queries(&self) -> BTreeMap<QueryId, TokenStream> {
        match self {
            NodeKind::None => BTreeMap::new(),
            NodeKind::Accept => BTreeMap::new(),
            NodeKind::Struct { fields } => fields
                .values()
                .flat_map(|child| child.queries.iter())
                .map(|(id, ty)| (id.clone(), ty.clone()))
                .collect(),
            NodeKind::Array { child } => child.queries.clone(),
        }
    }

    fn merge(&mut self, other: Self) {
        let this = std::mem::replace(self, Self::None);
        *self = match (this, other) {
            (NodeKind::None, other) => other,
            (NodeKind::Accept, NodeKind::Accept) => NodeKind::Accept,
            (NodeKind::Struct { mut fields }, NodeKind::Struct { fields: other }) => {
                for (field, node) in other.into_iter() {
                    use std::collections::btree_map::Entry;
                    match fields.entry(field) {
                        Entry::Vacant(v) => {
                            v.insert(node);
                        }
                        Entry::Occupied(mut o) => {
                            o.get_mut().merge(node);
                        }
                    }
                }
                NodeKind::Struct { fields }
            }
            (NodeKind::Array { mut child }, NodeKind::Array { child: other }) => {
                child.merge(*other);
                NodeKind::Array { child }
            }
            // TODO: better handling
            _ => panic!("user error"),
        }
    }
}

#[derive(Debug)]
struct Node {
    name: String,
    // map of (id, ty)
    queries: BTreeMap<QueryId, TokenStream>,
    kind: NodeKind,
}

impl Node {
    fn from_query(env: &mut Env, id: QueryId, fragment: QueryFragment, ty: TokenStream) -> Self {
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
                let kind = NodeKind::Struct {
                    fields: BTreeMap::from_iter([(
                        field_name,
                        Self::from_query(env, id, *rest, ty),
                    )]),
                };
                Self {
                    name,
                    queries: kind.child_queries(),
                    kind,
                }
            }
            QueryFragment::CollectArray { rest } => {
                let kind = NodeKind::Array {
                    child: Box::new(Self::from_query(env, id, *rest, ty)),
                };
                Self {
                    name,
                    queries: kind.child_queries(),
                    kind,
                }
            }
        }
    }

    fn merge(&mut self, other: Self) {
        self.kind.merge(other.kind);
        self.queries = self.kind.child_queries();
    }

    fn deserialize_seed_ty(&self) -> syn::Ident {
        quote::format_ident!("DeserializeSeed{}", self.name)
    }

    fn visitor_ty(&self) -> syn::Ident {
        quote::format_ident!("Visitor{}", self.name)
    }

    fn generate_internal(&self) -> TokenStream {
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
            NodeKind::Struct { fields } => {
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

                let child_code = fields.values().map(|node| node.generate_internal());

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
            NodeKind::Array { child } => {
                let deserialize_seed_ty = self.deserialize_seed_ty();
                let visitor_ty = self.visitor_ty();

                let query_names: Vec<_> = self
                    .queries
                    .keys()
                    .map(|id| quote::format_ident!("{}", id.0))
                    .collect();
                let query_types: Vec<_> = self.queries.values().collect();

                let child_code = child.generate_internal();
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

fn compile<I: Iterator<Item = Query>>(env: &mut Env, queries: I) -> Node {
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

fn generate_preamble() -> TokenStream {
    quote::quote! {
        extern crate alloc;

        trait UnVec {
            type Element;
        }

        impl<T> UnVec for alloc::vec::Vec<T> {
            type Element = T;
        }
    }
}

// TODO: remove syn features
fn main() {
    let queries = vec![
        Query {
            id: QueryId("x".into()),
            fragment: QueryFragment::Field {
                name: "locs".into(),
                rest: QueryFragment::Field {
                    name: "x".into(),
                    rest: QueryFragment::Accept.into(),
                }
                .into(),
            },
            ty: quote::quote!(String),
        },
        Query {
            id: QueryId("y".into()),
            fragment: QueryFragment::Field {
                name: "locs".into(),
                rest: QueryFragment::Field {
                    name: "y".into(),
                    rest: QueryFragment::Accept.into(),
                }
                .into(),
            },
            ty: quote::quote!(String),
        },
    ];
    let node = compile(&mut Env::default(), queries.into_iter());

    // let x_node = Node {
    //     name: "XNode".into(),
    //     queries: BTreeMap::from_iter([(QueryId("q0".into()), quote::quote!(f32))]),
    //     kind: NodeKind::Accept,
    // };
    // let y_node = Node {
    //     name: "YNode".into(),
    //     queries: BTreeMap::from_iter([(QueryId("q1".into()), quote::quote!(f32))]),
    //     kind: NodeKind::Accept,
    // };
    // let xy_node = Node {
    //     name: "XYNode".into(),
    //     queries: BTreeMap::from_iter([
    //         (QueryId("q0".into()), quote::quote!(f32)),
    //         (QueryId("q1".into()), quote::quote!(f32)),
    //     ]),
    //     kind: NodeKind::Struct {
    //         fields: BTreeMap::from_iter([("x".into(), x_node), ("y".into(), y_node)]),
    //     },
    // };
    // let array_node = Node {
    //     name: "XYArrayNode".into(),
    //     queries: BTreeMap::from_iter([
    //         (QueryId("q0".into()), quote::quote!(Vec<f32>)),
    //         (QueryId("q1".into()), quote::quote!(Vec<f32>)),
    //     ]),
    //     kind: NodeKind::Array {
    //         child: Box::new(xy_node),
    //     },
    // };
    // let node = Node {
    //     name: "Root".into(),
    //     queries: BTreeMap::from_iter([
    //         (QueryId("q0".into()), quote::quote!(Vec<f32>)),
    //         (QueryId("q1".into()), quote::quote!(Vec<f32>)),
    //     ]),
    //     kind: NodeKind::Struct {
    //         fields: {
    //             let mut fields = BTreeMap::new();
    //             fields.insert("locs".into(), array_node);
    //             fields
    //         },
    //     },
    // };
    // println!("{node:#?}");

    let code = {
        let mut code = TokenStream::new();
        code.extend(generate_preamble());
        code.extend(node.generate_internal());
        code
    };

    println!("{code}");
}
