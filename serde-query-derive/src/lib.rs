extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use proc_macro_error::proc_macro_error;
use std::collections::BTreeMap;
use std::error::Error;
use syn::{parse_macro_input, DeriveInput, Ident, LitStr, Type};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
enum Query {
    Field(String),
    Index(usize),
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
            match input.chars().next() {
                Some(c) => return Err(format!("expecting {}, got {}", pat, c).into()),
                None => return Err(format!("expecting {}, got EOF", pat).into()),
            }
        }
        Ok(&input[pat.len()..])
    }
}

fn peek(input: &str) -> Result<char, Box<dyn Error + 'static>> {
    match input.chars().next() {
        Some(c) => Ok(c),
        None => Err("expecting a character, got EOF".into()),
    }
}

fn simple_ident(input: &str) -> Result<(&str, Query), Box<dyn Error + 'static>> {
    // simple_ident ::= alpha alphanumeric*
    for (idx, c) in input.char_indices() {
        if idx == 0 {
            if !c.is_alphabetic() && idx == 0 {
                return Err(format!("expecting alphabets, got {}", c).into());
            }
        } else if !c.is_alphanumeric() {
            let (ident, input) = input.split_at(idx);
            return Ok((input, Query::Field(ident.into())));
        }
    }

    Ok((&input[input.len()..], Query::Field(input.into())))
}

fn index(input: &str) -> Result<(&str, usize), Box<dyn Error + 'static>> {
    // numbers ::= [0-9]+
    for (idx, c) in input.char_indices() {
        if !c.is_ascii_digit() {
            if idx == 0 {
                return Err(format!("expecting ascii digits, got {}", c).into());
            } else {
                let (digits, input) = input.split_at(idx);
                return Ok((input, digits.parse()?));
            }
        }
    }

    Ok((&input[input.len()..], input.parse()?))
}

fn string_literal(input: &str) -> Result<(&str, String), Box<dyn Error + 'static>> {
    let mut ret = String::new();
    let mut escape = false;

    let input = ws(eat("\"")(input)?);

    for (idx, c) in input.char_indices() {
        if c == '\\' {
            escape = true;
        } else {
            if c == '\"' && !escape {
                let input = eat("\"")(&input[idx..])?;
                return Ok((input, ret));
            }
            escape = false;
            ret.push(c);
        }
    }

    Err("expecting \", got EOF".into())
}

fn bracket(input: &str) -> Result<(&str, Query), Box<dyn Error + 'static>> {
    // bracket ::= '[' number ']'
    // bracket ::= '[' '"' ('\"' | any character)+ '"' ']'
    let input = ws(eat("[")(input)?);
    let (input, inner) = match peek(input)? {
        '\"' => {
            let (input, lit) = string_literal(input)?;
            (input, Query::Field(lit))
        }
        c if c.is_ascii_digit() => {
            let (input, index) = index(input)?;
            (input, Query::Index(index))
        }
        c => return Err(format!("expecting \" or an ascii digit, got {}", c).into()),
    };
    let input = eat("]")(ws(input))?;

    Ok((input, inner))
}

fn parse_query(mut input: &str) -> Result<(&str, Vec<Query>), Box<dyn Error + 'static>> {
    let mut ret = vec![];
    loop {
        input = ws(input);
        if input.is_empty() {
            return Ok((input, ret));
        }
        input = ws(eat(".")(input)?);
        let (input2, query) = match peek(input)? {
            '[' => bracket(input)?,
            c if c.is_alphabetic() => simple_ident(input)?,
            c => return Err(format!("expecting an alphabet or '[', got {}", c).into()),
        };
        input = input2;
        ret.push(query);
    }
}

struct Field {
    query: Vec<Query>,
    ident: Ident,
    ty: Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Traversal {
    Unknown,
    Seq,
    Map,
}

// forms a lattice
impl PartialOrd for Traversal {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        use Traversal::*;

        match (self, other) {
            (Unknown, Unknown) => Some(Ordering::Equal),
            (Seq, Seq) => Some(Ordering::Equal),
            (Map, Map) => Some(Ordering::Equal),

            (Unknown, _) => Some(Ordering::Less),
            (_, Unknown) => Some(Ordering::Greater),

            (Seq, Map) => None,
            (Map, Seq) => None,
        }
    }
}

enum Node {
    Action {
        ident: Ident,
        ty: Type,
    },
    Internal {
        children: BTreeMap<Query, Node>,
        traversal: Traversal,
    },
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
            Internal {
                children: this,
                traversal: this_traversal,
            } => match other {
                Action { .. } => panic!("query conflict"),
                Internal {
                    children: other,
                    traversal: other_traversal,
                } => {
                    if *this_traversal <= other_traversal {
                        *this_traversal = other_traversal;
                    } else {
                        panic!(
                            "seq-map conflict: {:?} vs {:?}",
                            this_traversal, other_traversal
                        );
                    }

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
        let field_enum_name = {
            let mut name = String::from("SQ_Field");
            construct_suffix(&mut name, &positions);
            Ident::new(&name, Span::call_site())
        };
        let field_visitor_name = {
            let mut name = String::from("SQ_FieldVis");
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

                    impl<'de> serde::de::Deserialize<'de> for #deserialize_name {
                        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where
                            D: serde::de::Deserializer<'de>,
                        {
                            core::result::Result::Ok(#deserialize_name {
                                child__0: <#ty as serde::de::Deserialize<'de>>::deserialize(deserializer)?,
                            })
                        }
                    }
                });
            }
            Internal {
                children,
                traversal,
            } => {
                let mut bare_names = vec![];
                let mut print_names = vec![];
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
                                bare_names.push(name);
                                print_names.push(format!("'{}'", name));
                                quote! {
                                    #field_enum_name :: #child_id => {
                                        if #child_id.is_some() {
                                            return core::result::Result::Err(
                                                <<A as serde::de::MapAccess<'de>>::Error as serde::de::Error>::duplicate_field(#name)
                                            );
                                        }
                                        let child: #child_ty = map.next_value()?;
                                        #child_id = core::option::Option::Some(child);
                                    },
                                }
                            }
                            Query::Index(index) => {
                                print_names.push(format!("[{}]", index));
                                quote! {
                                    #index => {
                                        let child: #child_ty = match seq.next_element()? {
                                            core::option::Option::Some(val) => val,
                                            core::option::Option::None => return core::result::Result::Err(
                                                <<A as serde::de::SeqAccess<'de>>::Error as serde::de::Error>::invalid_length(#index, &self)
                                            ),
                                        };
                                        #child_id = core::option::Option::Some(child);
                                    },
                                }
                            }
                        }
                    })
                    .collect();
                let rest = match traversal {
                    Traversal::Unknown => unreachable!(),
                    Traversal::Map => quote! {
                        _ => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    },
                    Traversal::Seq => quote! {
                        _ => {
                            match seq.next_element::<serde::de::IgnoredAny>()? {
                                core::option::Option::Some(_) => {},
                                core::option::Option::None => break,
                            };
                        }
                    },
                };
                arms.push(rest);
                let byte_identifiers = bare_names
                    .iter()
                    .map(|ident| proc_macro2::Literal::byte_string(ident.as_bytes()))
                    .collect::<Vec<_>>();

                let expecting = {
                    let mut s = String::from("a field ");
                    for (idx, name) in print_names.iter().enumerate() {
                        if idx == 0 {
                            s.push_str(name);
                        } else if idx + 1 == print_names.len() {
                            s.push_str(&format!(", or {}", name));
                        } else {
                            s.push_str(&format!(", {}", name));
                        }
                    }
                    s
                };

                let deserialize_impl = match traversal {
                    Traversal::Unknown => unreachable!(),
                    Traversal::Map => quote! {
                        impl<'de> serde::de::Deserialize<'de> for #deserialize_name {
                            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                            where
                                D: serde::de::Deserializer<'de>
                            {
                                deserializer.deserialize_map(#visitor_name)
                            }
                        }
                    },
                    Traversal::Seq => quote! {
                        impl<'de> serde::de::Deserialize<'de> for #deserialize_name {
                            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                            where
                                D: serde::de::Deserializer<'de>
                            {
                                deserializer.deserialize_seq(#visitor_name)
                            }
                        }
                    },
                };

                let visit_fn = match traversal {
                    Traversal::Unknown => unreachable!(),
                    Traversal::Map => quote! {
                        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::MapAccess<'de>,
                        {
                            #(let mut #child_ids = None;)*

                            while let core::option::Option::Some(key) = map.next_key::<#field_enum_name>()? {
                                match key {
                                    #(#arms)*
                                }
                            }

                            #(let #child_ids = match #child_ids {
                                core::option::Option::Some(v) => v,
                                core::option::Option::None => return core::result::Result::Err(
                                    <<A as serde::de::MapAccess<'de>>::Error as serde::de::Error>::missing_field(#print_names)
                                ),
                            };)*

                            core::result::Result::Ok(#deserialize_name {
                                #(#child_ids,)*
                            })
                        }
                    },
                    Traversal::Seq => quote! {
                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::SeqAccess<'de>,
                        {
                            #(let mut #child_ids = core::option::Option::None;)*
                            let mut count: usize = 0;

                            loop {
                                match count {
                                    #(#arms)*
                                }

                                count += 1;
                            }

                            #(let #child_ids = match #child_ids {
                                core::option::Option::Some(v) => v,
                                core::option::Option::None => return core::result::Result::Err(
                                    <<A as serde::de::SeqAccess<'de>>::Error as serde::de::Error>::invalid_length(count, &self)
                                ),
                            };)*

                            core::result::Result::Ok(#deserialize_name {
                                #(#child_ids,)*
                            })
                        }
                    },
                };

                current_stream.extend(quote! {
                    struct #field_visitor_name;

                    impl<'de> serde::de::Visitor<'de> for #field_visitor_name {
                        type Value = #field_enum_name;

                        fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                            core::fmt::Formatter::write_str(f, #expecting)
                        }

                        fn visit_str<E>(self, value: &str) -> core::result::Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            match value {
                                #(#bare_names => core::result::Result::Ok(#field_enum_name :: #child_ids),)*
                                _ => core::result::Result::Ok(#field_enum_name :: ignore),
                            }
                        }

                        fn visit_bytes<E>(self, value: &[u8]) -> core::result::Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            match value {
                                #(#byte_identifiers => core::result::Result::Ok(#field_enum_name :: #child_ids),)*
                                _ => core::result::Result::Ok(#field_enum_name :: ignore),
                            }
                        }
                    }

                    enum #field_enum_name {
                        #(#child_ids,)*
                        ignore,
                    }

                    impl<'de> serde::de::Deserialize<'de> for #field_enum_name {
                        fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
                        where
                            D: serde::de::Deserializer<'de>
                        {
                            deserializer.deserialize_identifier(#field_visitor_name)
                        }
                    }

                    struct #deserialize_name {
                        #(#child_ids: #child_tys,)*
                    }

                    #deserialize_impl

                    struct #visitor_name;

                    impl<'de> serde::de::Visitor<'de> for #visitor_name {
                        type Value = #deserialize_name;

                        fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                            core::fmt::Formatter::write_str(f, #expecting)
                        }

                        #visit_fn
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
        traversal: Traversal::Unknown,
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
                    let traversal = match head {
                        Query::Field(_) => Traversal::Map,
                        Query::Index(_) => Traversal::Seq,
                    };
                    let mut children = BTreeMap::default();
                    children.insert(head.clone(), construct_node(field, tail));
                    Node::Internal {
                        children,
                        traversal,
                    }
                }
            }
        }

        root.merge(construct_node(&field, &field.query));
    }

    let (root_ty, stream) = root.generate(field_to_positions, &mut vec![]);
    (root_ty, stream)
}

enum DeriveTarget {
    Deserialize,
    DeserializeQuery,
}

fn generate_derive(input: TokenStream, target: DeriveTarget) -> TokenStream {
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

    // generate DeserializeQuery and conversion traits
    let wrapper_ty = Ident::new("__QueryWrapper", Span::call_site());

    // add lifetime argument for deserializers
    let mut dq_generics = generics.clone();
    dq_generics.params.push(syn::parse_quote! { 'de_SQ });

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

    match target {
        DeriveTarget::DeserializeQuery => {
            stream.extend(quote! {
                #[repr(transparent)]
                struct #wrapper_ty #generics (#name #generics);

                impl #generics #wrapper_ty #generics {
                    fn __serde_query_from_root(val: #root_ty) -> Self {
                        Self (
                            #name #generics {
                                #(#constructors,)*
                            }
                        )
                    }
                }

                impl #dq_generics serde::de::Deserialize<'de_SQ> for #wrapper_ty #generics {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: serde::de::Deserializer<'de_SQ>
                    {
                        let root = <#root_ty as serde::de::Deserialize<'de_SQ>>::deserialize(deserializer)?;
                        Ok(Self::__serde_query_from_root(root))
                    }
                }

                impl #generics core::convert::From<#wrapper_ty #generics> for #name #generics {
                    fn from(val: #wrapper_ty #generics) -> Self {
                        val.0
                    }
                }

                impl #generics core::ops::Deref for #wrapper_ty #generics {
                    type Target = #name #generics;

                    fn deref(&self) -> &Self::Target {
                        &self.0
                    }
                }

                impl #generics core::ops::DerefMut for #wrapper_ty #generics {
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        &mut self.0
                    }
                }
            });

            stream.extend(quote! {
                impl #dq_generics serde_query::DeserializeQuery<'de_SQ> for #name #generics {
                    type Query = #wrapper_ty #generics;
                }
            });
        }
        DeriveTarget::Deserialize => {
            stream.extend(quote!{
                impl #dq_generics serde::de::Deserialize<'de_SQ> for #name #generics {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: serde::de::Deserializer<'de_SQ>
                    {
                        let val = <#root_ty as serde::de::Deserialize<'de_SQ>>::deserialize(deserializer)?;
                        let this = #name #generics {
                            #(#constructors,)*
                        };
                        Ok(this)
                    }
                }
            });
        }
    }

    // Cargo-culting serde. Possibly for scoping?
    TokenStream::from(quote! {
        const _: () = {
            #stream

            ()
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
