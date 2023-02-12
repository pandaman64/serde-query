extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::{emit_error, proc_macro_error};
use quote::{quote, ToTokens};
use serde_query_core::{compile, Env, Query, QueryId};
use syn::{parse_macro_input, DeriveInput, Ident, LitStr};

mod parse_query;

// struct Field {
//     query: Vec<Query>,
//     ident: Ident,
//     ty: Type,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum Traversal {
//     Unknown,
//     Seq,
//     Map,
// }

// // forms a lattice
// impl PartialOrd for Traversal {
//     fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
//         use std::cmp::Ordering;
//         use Traversal::*;

//         match (self, other) {
//             (Unknown, Unknown) => Some(Ordering::Equal),
//             (Seq, Seq) => Some(Ordering::Equal),
//             (Map, Map) => Some(Ordering::Equal),

//             (Unknown, _) => Some(Ordering::Less),
//             (_, Unknown) => Some(Ordering::Greater),

//             (Seq, Map) => None,
//             (Map, Seq) => None,
//         }
//     }
// }

// enum Node {
//     Action {
//         ident: Ident,
//         ty: Type,
//     },
//     Internal {
//         children: BTreeMap<Query, Node>,
//         traversal: Traversal,
//     },
// }

// fn construct_suffix(buf: &mut String, levels: &[usize]) {
//     use std::fmt::Write;
//     for level in levels.iter() {
//         write!(buf, "__{}", level).unwrap();
//     }
// }

// impl Node {
//     fn merge(&mut self, other: Self) {
//         use Node::*;

//         // waiting for https://github.com/rust-lang/rust/pull/76119 to remove duplicated code
//         match self {
//             Action { .. } => panic!("query conflict"),
//             Internal {
//                 children: this,
//                 traversal: this_traversal,
//             } => match other {
//                 Action { .. } => panic!("query conflict"),
//                 Internal {
//                     children: other,
//                     traversal: other_traversal,
//                 } => {
//                     if *this_traversal <= other_traversal {
//                         *this_traversal = other_traversal;
//                     } else {
//                         panic!(
//                             "seq-map conflict: {:?} vs {:?}",
//                             this_traversal, other_traversal
//                         );
//                     }

//                     for (query, child) in other.into_iter() {
//                         use std::collections::btree_map::Entry;
//                         match this.entry(query) {
//                             Entry::Vacant(e) => {
//                                 e.insert(child);
//                             }
//                             Entry::Occupied(mut e) => {
//                                 e.get_mut().merge(child);
//                             }
//                         }
//                     }
//                 }
//             },
//         }
//     }

//     fn generate(
//         &self,
//         field_to_positions: &mut BTreeMap<Ident, Vec<usize>>,
//         positions: &mut Vec<usize>,
//     ) -> (Ident, TokenStream2) {
//         use Node::*;
//         let visitor_name = {
//             let mut name = String::from("SQ_Vis");
//             construct_suffix(&mut name, positions);
//             Ident::new(&name, Span::call_site())
//         };
//         let deserialize_name = {
//             let mut name = String::from("SQ_Des");
//             construct_suffix(&mut name, positions);
//             Ident::new(&name, Span::call_site())
//         };
//         let field_enum_name = {
//             let mut name = String::from("SQ_Field");
//             construct_suffix(&mut name, positions);
//             Ident::new(&name, Span::call_site())
//         };
//         let field_visitor_name = {
//             let mut name = String::from("SQ_FieldVis");
//             construct_suffix(&mut name, positions);
//             Ident::new(&name, Span::call_site())
//         };

//         let mut children_stream = TokenStream2::new();
//         let mut current_stream = TokenStream2::new();

//         match self {
//             Action { ident, ty } => {
//                 assert!(
//                     field_to_positions
//                         .insert(ident.clone(), positions.clone())
//                         .is_none(),
//                     "duplicated field"
//                 );

//                 current_stream.extend(quote! {
//                     #[allow(non_camel_case_types)]
//                     struct #deserialize_name {
//                         child__0: #ty,
//                     }

//                     impl<'de> serde::de::Deserialize<'de> for #deserialize_name {
//                         fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
//                         where
//                             D: serde::de::Deserializer<'de>,
//                         {
//                             core::result::Result::Ok(#deserialize_name {
//                                 child__0: <#ty as serde::de::Deserialize<'de>>::deserialize(deserializer)?,
//                             })
//                         }
//                     }
//                 });
//             }
//             Internal {
//                 children,
//                 traversal,
//             } => {
//                 let mut bare_names = vec![];
//                 let mut print_names = vec![];
//                 let mut child_ids = vec![];
//                 let mut child_tys = vec![];
//                 let mut arms: Vec<_> = children
//                     .iter()
//                     .enumerate()
//                     .map(|(idx, (query, child))| {
//                         let child_id = Ident::new(&format!("child__{}", idx), Span::call_site());
//                         child_ids.push(child_id.clone());

//                         positions.push(idx);
//                         let (child_ty, stream) = child.generate(field_to_positions, positions);
//                         child_tys.push(child_ty.clone());
//                         children_stream.extend(stream);
//                         positions.pop();

//                         match query {
//                             Query::Field(name) => {
//                                 bare_names.push(name);
//                                 print_names.push(format!("'{}'", name));
//                                 quote! {
//                                     #field_enum_name :: #child_id => {
//                                         if #child_id.is_some() {
//                                             return core::result::Result::Err(
//                                                 <<A as serde::de::MapAccess<'de>>::Error as serde::de::Error>::duplicate_field(#name)
//                                             );
//                                         }
//                                         let child: #child_ty = map.next_value()?;
//                                         #child_id = core::option::Option::Some(child);
//                                     },
//                                 }
//                             }
//                             Query::Index(index) => {
//                                 print_names.push(format!("[{}]", index));
//                                 quote! {
//                                     #index => {
//                                         let child: #child_ty = match seq.next_element()? {
//                                             core::option::Option::Some(val) => val,
//                                             core::option::Option::None => return core::result::Result::Err(
//                                                 <<A as serde::de::SeqAccess<'de>>::Error as serde::de::Error>::invalid_length(#index, &self)
//                                             ),
//                                         };
//                                         #child_id = core::option::Option::Some(child);
//                                     },
//                                 }
//                             }
//                         }
//                     })
//                     .collect();
//                 let rest = match traversal {
//                     Traversal::Unknown => unreachable!(),
//                     Traversal::Map => quote! {
//                         _ => {
//                             map.next_value::<serde::de::IgnoredAny>()?;
//                         }
//                     },
//                     Traversal::Seq => quote! {
//                         _ => {
//                             match seq.next_element::<serde::de::IgnoredAny>()? {
//                                 core::option::Option::Some(_) => {},
//                                 core::option::Option::None => break,
//                             };
//                         }
//                     },
//                 };
//                 arms.push(rest);
//                 let byte_identifiers = bare_names
//                     .iter()
//                     .map(|ident| proc_macro2::Literal::byte_string(ident.as_bytes()))
//                     .collect::<Vec<_>>();

//                 let expecting = {
//                     let mut s = String::from("a field ");
//                     for (idx, name) in print_names.iter().enumerate() {
//                         if idx == 0 {
//                             s.push_str(name);
//                         } else if idx + 1 == print_names.len() {
//                             s.push_str(&format!(", or {}", name));
//                         } else {
//                             s.push_str(&format!(", {}", name));
//                         }
//                     }
//                     s
//                 };

//                 let deserialize_impl = match traversal {
//                     Traversal::Unknown => unreachable!(),
//                     Traversal::Map => quote! {
//                         impl<'de> serde::de::Deserialize<'de> for #deserialize_name {
//                             fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
//                             where
//                                 D: serde::de::Deserializer<'de>
//                             {
//                                 deserializer.deserialize_map(#visitor_name)
//                             }
//                         }
//                     },
//                     Traversal::Seq => quote! {
//                         impl<'de> serde::de::Deserialize<'de> for #deserialize_name {
//                             fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
//                             where
//                                 D: serde::de::Deserializer<'de>
//                             {
//                                 deserializer.deserialize_seq(#visitor_name)
//                             }
//                         }
//                     },
//                 };

//                 let visit_fn = match traversal {
//                     Traversal::Unknown => unreachable!(),
//                     Traversal::Map => quote! {
//                         fn visit_map<A>(self, mut map: A) -> core::result::Result<Self::Value, A::Error>
//                         where
//                             A: serde::de::MapAccess<'de>,
//                         {
//                             #(let mut #child_ids = None;)*

//                             while let core::option::Option::Some(key) = map.next_key::<#field_enum_name>()? {
//                                 match key {
//                                     #(#arms)*
//                                 }
//                             }

//                             #(let #child_ids = match #child_ids {
//                                 core::option::Option::Some(v) => v,
//                                 core::option::Option::None => return core::result::Result::Err(
//                                     <<A as serde::de::MapAccess<'de>>::Error as serde::de::Error>::missing_field(#print_names)
//                                 ),
//                             };)*

//                             core::result::Result::Ok(#deserialize_name {
//                                 #(#child_ids,)*
//                             })
//                         }
//                     },
//                     Traversal::Seq => quote! {
//                         fn visit_seq<A>(self, mut seq: A) -> core::result::Result<Self::Value, A::Error>
//                         where
//                             A: serde::de::SeqAccess<'de>,
//                         {
//                             #(let mut #child_ids = core::option::Option::None;)*
//                             let mut count: usize = 0;

//                             loop {
//                                 match count {
//                                     #(#arms)*
//                                 }

//                                 count += 1;
//                             }

//                             #(let #child_ids = match #child_ids {
//                                 core::option::Option::Some(v) => v,
//                                 core::option::Option::None => return core::result::Result::Err(
//                                     <<A as serde::de::SeqAccess<'de>>::Error as serde::de::Error>::invalid_length(count, &self)
//                                 ),
//                             };)*

//                             core::result::Result::Ok(#deserialize_name {
//                                 #(#child_ids,)*
//                             })
//                         }
//                     },
//                 };

//                 current_stream.extend(quote! {
//                     struct #field_visitor_name;

//                     impl<'de> serde::de::Visitor<'de> for #field_visitor_name {
//                         type Value = #field_enum_name;

//                         fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
//                             core::fmt::Formatter::write_str(f, #expecting)
//                         }

//                         fn visit_str<E>(self, value: &str) -> core::result::Result<Self::Value, E>
//                         where
//                             E: serde::de::Error,
//                         {
//                             match value {
//                                 #(#bare_names => core::result::Result::Ok(#field_enum_name :: #child_ids),)*
//                                 _ => core::result::Result::Ok(#field_enum_name :: ignore),
//                             }
//                         }

//                         fn visit_bytes<E>(self, value: &[u8]) -> core::result::Result<Self::Value, E>
//                         where
//                             E: serde::de::Error,
//                         {
//                             match value {
//                                 #(#byte_identifiers => core::result::Result::Ok(#field_enum_name :: #child_ids),)*
//                                 _ => core::result::Result::Ok(#field_enum_name :: ignore),
//                             }
//                         }
//                     }

//                     enum #field_enum_name {
//                         #(#child_ids,)*
//                         ignore,
//                     }

//                     impl<'de> serde::de::Deserialize<'de> for #field_enum_name {
//                         fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
//                         where
//                             D: serde::de::Deserializer<'de>
//                         {
//                             deserializer.deserialize_identifier(#field_visitor_name)
//                         }
//                     }

//                     struct #deserialize_name {
//                         #(#child_ids: #child_tys,)*
//                     }

//                     #deserialize_impl

//                     struct #visitor_name;

//                     impl<'de> serde::de::Visitor<'de> for #visitor_name {
//                         type Value = #deserialize_name;

//                         fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
//                             core::fmt::Formatter::write_str(f, #expecting)
//                         }

//                         #visit_fn
//                     }
//                 })
//             }
//         }

//         children_stream.extend(current_stream);

//         (deserialize_name, children_stream)
//     }
// }

// fn generate(
//     fields: &[Field],
//     field_to_positions: &mut BTreeMap<Ident, Vec<usize>>,
// ) -> (Ident, TokenStream2) {
//     let mut root = Node::Internal {
//         children: BTreeMap::default(),
//         traversal: Traversal::Unknown,
//     };

//     for field in fields.iter() {
//         // construct a Node
//         fn construct_node(field: &Field, query: &[Query]) -> Node {
//             match query {
//                 [] => Node::Action {
//                     ident: field.ident.clone(),
//                     ty: field.ty.clone(),
//                 },
//                 [head, tail @ ..] => {
//                     let traversal = match head {
//                         Query::Field(_) => Traversal::Map,
//                         Query::Index(_) => Traversal::Seq,
//                     };
//                     let mut children = BTreeMap::default();
//                     children.insert(head.clone(), construct_node(field, tail));
//                     Node::Internal {
//                         children,
//                         traversal,
//                     }
//                 }
//             }
//         }

//         root.merge(construct_node(field, &field.query));
//     }

//     let (root_ty, stream) = root.generate(field_to_positions, &mut vec![]);
//     (root_ty, stream)
// }

#[derive(Debug, PartialEq, Eq)]
enum DeriveTarget {
    Deserialize,
    DeserializeQuery,
}

fn generate_root_deserialize(
    struct_ty: &Ident,
    implementor_ty: &Ident,
    query_names: &[&Ident],
    deserialize_seed_ty: &Ident,
    target: DeriveTarget,
) -> TokenStream2 {
    let error_messages: Vec<_> = query_names
        .iter()
        .map(|name| format!("Query for '{}' failed to run", name))
        .collect();
    let construction = match target {
        DeriveTarget::Deserialize => quote!(value),
        DeriveTarget::DeserializeQuery => quote!(#implementor_ty(value)),
    };
    quote! {
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

fn generate_derive(input: TokenStream, target: DeriveTarget) -> TokenStream {
    let mut interrupt = false;
    let mut input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    if target == DeriveTarget::Deserialize {
        let generics = &input.generics;
        // generate Deserialize implementation on error
        proc_macro_error::set_dummy(quote! {
            const _: () = {
                impl<'de> serde::de::Deserialize<'de> for #name #generics {
                    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
                    where
                        D: serde::de::Deserializer<'de>
                    {
                        unimplemented!()
                    }
                }
            };
        });
    }

    // TODO: we need to propagate generic parameters
    if !input.generics.params.is_empty() {
        emit_error!(input.generics, "generic arguments are not supported yet");
        interrupt = true;
    }

    let fields: Vec<_> = match &mut input.data {
        syn::Data::Struct(data) => data
            .fields
            .iter_mut()
            .flat_map(|field| {
                let mut attr_pos = None;
                for (pos, attr) in field.attrs.iter().enumerate() {
                    if attr.path.is_ident("query") {
                        if attr_pos.is_some() {
                            emit_error!(attr, "duplicated #[query(...)]");
                            interrupt = true;
                        }
                        attr_pos = Some(pos);
                    }
                }

                match attr_pos {
                    None => {
                        emit_error!(field, "no #[query(...)]");
                        interrupt = true;
                        None
                    }
                    Some(pos) => {
                        let attr = field.attrs.remove(pos);
                        let argument = match attr.parse_args::<LitStr>() {
                            Err(_) => {
                                emit_error!(field, "#[query(...)] takes a string literal");
                                interrupt = true;
                                return None;
                            }
                            Ok(lit) => lit.value(),
                        };
                        let ident = match &field.ident {
                            None => {
                                emit_error!(field.ident, "#[query(...)] field must be named");
                                interrupt = true;
                                return None;
                            }
                            Some(ident) => ident.clone(),
                        };

                        let (fragment, errors) = parse_query::parse(&argument);
                        for error in errors {
                            emit_error!(attr, error.message);
                            interrupt = true;
                        }
                        Some(Query::new(
                            QueryId::new(ident),
                            fragment,
                            field.ty.to_token_stream(),
                        ))
                    }
                }
            })
            .collect(),
        _ => {
            emit_error!(input, "serde-query supports only structs");
            interrupt = true;
            vec![]
        }
    };

    if interrupt {
        return TokenStream::new();
    }

    let (root_ty, node) = compile(&mut Env::new(), fields.into_iter());
    let query_names = node.query_names();

    let mut stream = quote! {
        // TODO: generate preamble
    };
    stream.extend(node.generate());

    // generate the root code
    match target {
        // generate DeserializeQuery and conversion traits
        DeriveTarget::DeserializeQuery => {
            let wrapper_ty = Ident::new("__QueryWrapper", Span::call_site());

            // Inherit visibility of the wrapped struct to avoid error E0446
            // See: https://github.com/pandaman64/serde-query/issues/7
            let vis = input.vis;

            let deserialize_impl =
                generate_root_deserialize(name, &wrapper_ty, &query_names, &root_ty, target);

            stream.extend(quote! {
                #[repr(transparent)]
                #vis struct #wrapper_ty (#name);

                #deserialize_impl

                impl core::convert::From<#wrapper_ty> for #name {
                    fn from(val: #wrapper_ty) -> Self {
                        val.0
                    }
                }

                impl core::ops::Deref for #wrapper_ty {
                    type Target = #name;

                    fn deref(&self) -> &Self::Target {
                        &self.0
                    }
                }

                impl core::ops::DerefMut for #wrapper_ty {
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        &mut self.0
                    }
                }
            });

            stream.extend(quote! {
                impl<'de> serde_query::DeserializeQuery<'de> for #name {
                    type Query = #wrapper_ty;
                }
            });
        }
        DeriveTarget::Deserialize => {
            let deserialize_impl =
                generate_root_deserialize(name, name, &query_names, &root_ty, target);
            stream.extend(deserialize_impl);
        }
    }

    // Cargo-culting serde. Possibly for scoping?
    TokenStream::from(quote! {
        const _: () = {
            #stream
        };
    })
}

#[proc_macro_error]
#[proc_macro_derive(DeserializeQuery, attributes(query))]
pub fn derive_deserialize_query(input: TokenStream) -> TokenStream {
    generate_derive(input, DeriveTarget::DeserializeQuery)
}

#[proc_macro_error]
#[proc_macro_derive(Deserialize, attributes(query))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    generate_derive(input, DeriveTarget::Deserialize)
}
