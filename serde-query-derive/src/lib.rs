extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::collections::BTreeMap;
use std::error::Error;
use syn::{parse_macro_input, DeriveInput, Ident, LitStr, Type};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
enum Query {
    Field(String),
}

fn ws(input: &str) -> &str {
    match input.find(|c: char| !c.is_whitespace()) {
        Some(n) => &input[n..],
        None => input,
    }
}

fn eat(pat: &str) -> impl for<'i> Fn(&'i str) -> Result<&'i str, Box<dyn Error + 'static>> + '_ {
    move |input| {
        if !input.starts_with(pat) {
            return Err(format!("expecting {}", pat).into());
        }
        Ok(&input[pat.len()..])
    }
}

fn ident(input: &str) -> Result<(&str, &str), Box<dyn Error + 'static>> {
    // alpha alphanumeric*
    for (idx, c) in input.char_indices() {
        if idx == 0 {
            if !c.is_alphabetic() {
                if idx == 0 {
                    return Err(format!("expecting alphabets, got {}", c).into());
                }
            }
        } else {
            if !c.is_alphanumeric() {
                let (ident, input) = input.split_at(idx);
                return Ok((input, ident));
            }
        }
    }

    Ok((&input[input.len()..], input))
}

fn parse_query(mut input: &str) -> Result<(&str, Vec<Query>), Box<dyn Error + 'static>> {
    let mut ret = vec![];
    loop {
        input = ws(input);
        if input.is_empty() {
            return Ok((input, ret));
        }
        input = ws(eat(".")(input)?);
        let (input2, ident) = ident(input)?;
        input = input2;
        ret.push(Query::Field(ident.into()))
    }
}

struct Field {
    query: Vec<Query>,
    ident: Ident,
    ty: Type,
}

enum Node {
    Action { ident: Ident, ty: Type },
    Internal { children: BTreeMap<Query, Node> },
}

fn construct_suffix(buf: &mut String, levels: &[usize]) {
    use std::fmt::Write;
    for level in levels.iter() {
        write!(buf, "__{}", level).unwrap();
    }
}

impl Node {
    fn merge(&mut self, other: Self) {
        use Node::*;

        // waiting for https://github.com/rust-lang/rust/pull/76119 to remove duplicated code
        match self {
            Action { .. } => panic!("query conflict"),
            Internal { children: this } => match other {
                Action { .. } => panic!("query conflict"),
                Internal { children: other } => {
                    for (query, child) in other.into_iter() {
                        use std::collections::btree_map::Entry;
                        match this.entry(query) {
                            Entry::Vacant(e) => {
                                e.insert(child);
                            }
                            Entry::Occupied(mut e) => {
                                e.get_mut().merge(child);
                            }
                        }
                    }
                }
            },
        }
    }

    fn generate(
        &self,
        field_to_positions: &mut BTreeMap<Ident, Vec<usize>>,
        positions: &mut Vec<usize>,
    ) -> (Ident, TokenStream2) {
        use Node::*;
        let visitor_name = {
            let mut name = String::from("SQ_Vis");
            construct_suffix(&mut name, &positions);
            Ident::new(&name, Span::call_site())
        };
        let deserialize_name = {
            let mut name = String::from("SQ_Des");
            construct_suffix(&mut name, &positions);
            Ident::new(&name, Span::call_site())
        };

        let mut children_stream = TokenStream2::new();
        let mut current_stream = TokenStream2::new();

        match self {
            Action { ident, ty } => {
                assert!(
                    field_to_positions
                        .insert(ident.clone(), positions.clone())
                        .is_none(),
                    "duplicated field"
                );

                current_stream.extend(quote! {
                    #[allow(non_camel_case_types)]
                    struct #deserialize_name {
                        child__0: #ty,
                    }

                    impl<'de> serde::Deserialize<'de> for #deserialize_name {
                        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where
                            D: serde::de::Deserializer<'de>,
                        {
                            core::result::Result::Ok(#deserialize_name {
                                child__0: <#ty as serde::Deserialize<'de>>::deserialize(deserializer)?,
                            })
                        }
                    }
                });
            }
            Internal { children } => {
                let mut child_ids = vec![];
                let mut child_tys = vec![];
                let mut arms: Vec<_> = children
                    .iter()
                    .enumerate()
                    .map(|(idx, (query, child))| {
                        let child_id = Ident::new(&format!("child__{}", idx), Span::call_site());
                        child_ids.push(child_id.clone());

                        positions.push(idx);
                        let (child_ty, stream) = child.generate(field_to_positions, positions);
                        child_tys.push(child_ty.clone());
                        children_stream.extend(stream);
                        positions.pop();

                        match query {
                            Query::Field(name) => {
                                quote! {
                                    #name => {
                                        if #child_id.is_some() {
                                            return core::result::Result::Err(
                                                <<A as serde::de::MapAccess<'de>>::Error as serde::de::Error>::duplicate_field(#name)
                                            );
                                        }
                                        let child: #child_ty = map.next_value()?;
                                        #child_id = Some(child);
                                    },
                                }
                            }
                        }
                    })
                    .collect();
                arms.push(quote! {
                    _ => {
                        map.next_value::<serde::de::IgnoredAny>()?;
                    }
                });

                current_stream.extend(quote! {
                    #[allow(non_camel_case_types)]
                    struct #deserialize_name {
                        #(#child_ids: #child_tys,)*
                    }

                    impl<'de> serde::de::Deserialize<'de> for #deserialize_name {
                        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where
                            D: serde::de::Deserializer<'de>
                        {
                            deserializer.deserialize_map(#visitor_name)
                        }
                    }

                    #[allow(non_camel_case_types)]
                    struct #visitor_name;

                    impl<'de> serde::de::Visitor<'de> for #visitor_name {
                        type Value = #deserialize_name;

                        fn expecting(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
                            Ok(())
                        }

                        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::MapAccess<'de>,
                        {
                            #(let mut #child_ids = None;)*

                            while let Some(key) = map.next_key()? {
                                match key {
                                    #(#arms)*
                                }
                            }

                            #(let #child_ids = match #child_ids {
                                Some(v) => v,
                                None => return core::result::Result::Err(
                                    <<A as serde::de::MapAccess<'de>>::Error as serde::de::Error>::missing_field(#names)
                                ),
                            };)*

                            core::result::Result::Ok(#deserialize_name {
                                #(#child_ids,)*
                            })
                        }
                    }
                })
            }
        }

        children_stream.extend(current_stream);

        (deserialize_name, children_stream)
    }
}

fn generate(
    fields: &[Field],
    field_to_positions: &mut BTreeMap<Ident, Vec<usize>>,
) -> (Ident, TokenStream2) {
    let mut root = Node::Internal {
        children: BTreeMap::default(),
    };

    for field in fields.iter() {
        // construct a Node
        fn construct_node(field: &Field, query: &[Query]) -> Node {
            match query {
                [] => Node::Action {
                    ident: field.ident.clone(),
                    ty: field.ty.clone(),
                },
                [head, tail @ ..] => {
                    let mut children = BTreeMap::default();
                    children.insert(head.clone(), construct_node(field, tail));
                    Node::Internal { children }
                }
            }
        }

        root.merge(construct_node(&field, &field.query));
    }

    let (root_ty, stream) = root.generate(field_to_positions, &mut vec![]);
    (root_ty, stream)
}

#[proc_macro_derive(DeserializeQuery, attributes(query))]
pub fn derive_deserialize_query(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = input.generics;

    // TODO: we need to propagate generic parameters
    if !generics.params.is_empty() {
        unimplemented!("DeserializeQuery with a generic argument is not supported yet");
    }

    let fields: Vec<_> = match &mut input.data {
        syn::Data::Struct(data) => data
            .fields
            .iter_mut()
            .map(|field| {
                let mut attr_pos = None;
                for (pos, attr) in field.attrs.iter().enumerate() {
                    if attr.path.is_ident("query") {
                        if attr_pos.is_some() {
                            panic!("duplicated #[query(...)]");
                        }
                        attr_pos = Some(pos);
                    }
                }

                let attr = field.attrs.remove(attr_pos.expect("no #[query(...)]"));
                let argument = attr
                    .parse_args::<LitStr>()
                    .expect("#[query(...)] takes a string literal")
                    .value();
                let ident = field
                    .ident
                    .clone()
                    .expect("#[query(...)] field must be named");
                let ty = field.ty.clone();

                Field {
                    query: parse_query(&argument).unwrap().1,
                    ident,
                    ty,
                }
            })
            .collect(),
        _ => panic!("serde-query supports only structs"),
    };

    // generate Deserialize
    let mut field_to_positions = BTreeMap::default();
    let (root_ty, mut stream) = generate(&fields, &mut field_to_positions);

    // generate From and DeserializeQuery
    let constructors: Vec<_> = fields
        .iter()
        .map(|field| {
            let ident = &field.ident;
            let fields: Vec<_> = field_to_positions
                .get(ident)
                .unwrap()
                .iter()
                .map(|pos| Ident::new(&format!("child__{}", pos), Span::call_site()))
                .collect();
            quote! {
                #ident: val.#(#fields).*.child__0
            }
        })
        .collect();
    stream.extend(quote! {
        impl #generics core::convert::From<#root_ty> for #name #generics {
            fn from(val: #root_ty) -> Self {
                Self {
                    #(#constructors,)*
                }
            }
        }
    });

    // add lifetime argument for `impl DeserializeQuery`
    let mut dq_generics = generics.clone();
    dq_generics.params.push(syn::parse_quote! { 'de_SQ });
    stream.extend(quote! {
        impl #dq_generics serde_query::DeserializeQuery<'de_SQ> for #name #generics {
            type Query = #root_ty;
        }
    });

    // Cargo-culting serde. Possibly for scoping?
    TokenStream::from(quote! {
        const _: () = {
            #stream

            ()
        };
    })
}
