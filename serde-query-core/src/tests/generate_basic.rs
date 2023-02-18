use crate::{
    node::Node,
    query::{Query, QueryFragment, QueryId},
};

#[test]
fn test_basic() {
    let queries = vec![
        Query::new(
            QueryId::new(quote::format_ident!("x")),
            QueryFragment::field(
                "locs".into(),
                QueryFragment::collect_array(QueryFragment::field(
                    "x".into(),
                    QueryFragment::accept(),
                )),
            ),
            quote::quote!(Vec<f32>),
        ),
        Query::new(
            QueryId::new(quote::format_ident!("y")),
            QueryFragment::field(
                "locs".into(),
                QueryFragment::collect_array(QueryFragment::field(
                    "y".into(),
                    QueryFragment::accept(),
                )),
            ),
            quote::quote!(Vec<f32>),
        ),
    ];
    let node = Node::from_queries(queries.into_iter());

    let code = node.generate();
    let pretty = prettyplease::unparse(&syn::parse2(code).unwrap());
    k9::snapshot!(
        pretty,
        r#"
struct DeserializeSeedNode0<'query> {
    x: &'query mut core::option::Option<Vec<f32>>,
    y: &'query mut core::option::Option<Vec<f32>>,
}
impl<'query, 'de> serde::de::DeserializeSeed<'de> for DeserializeSeedNode0<'query> {
    type Value = ();
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let visitor = VisitorNode0 {
            x: self.x,
            y: self.y,
        };
        deserializer.deserialize_map(visitor)
    }
}
struct VisitorNode0<'query> {
    x: &'query mut core::option::Option<Vec<f32>>,
    y: &'query mut core::option::Option<Vec<f32>>,
}
impl<'query, 'de> serde::de::Visitor<'de> for VisitorNode0<'query> {
    type Value = ();
    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Formatter::write_str(formatter, "one of the following fields: 'locs'")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<FieldNode0>()? {
            match key {
                FieldNode0::Field0 => {
                    map.next_value_seed(DeserializeSeedNode2 {
                        x: self.x,
                        y: self.y,
                    })?;
                }
                FieldNode0::Ignore => {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}
enum FieldNode0 {
    Field0,
    Ignore,
}
impl<'de> serde::de::Deserialize<'de> for FieldNode0 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(FieldVisitorNode0)
    }
}
struct FieldVisitorNode0;
impl<'de> serde::de::Visitor<'de> for FieldVisitorNode0 {
    type Value = FieldNode0;
    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Formatter::write_str(f, "one of the following fields: 'locs'")
    }
    fn visit_str<E>(self, value: &str) -> core::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match value {
            "locs" => core::result::Result::Ok(FieldNode0::Field0),
            _ => core::result::Result::Ok(FieldNode0::Ignore),
        }
    }
    fn visit_bytes<E>(self, value: &[u8]) -> core::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match value {
            b"locs" => core::result::Result::Ok(FieldNode0::Field0),
            _ => core::result::Result::Ok(FieldNode0::Ignore),
        }
    }
}
struct DeserializeSeedNode2<'query> {
    x: &'query mut core::option::Option<Vec<f32>>,
    y: &'query mut core::option::Option<Vec<f32>>,
}
impl<'query, 'de> serde::de::DeserializeSeed<'de> for DeserializeSeedNode2<'query> {
    type Value = ();
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut x = <Vec<f32> as serde_query::__priv::Container>::empty();
        let mut y = <Vec<f32> as serde_query::__priv::Container>::empty();
        let visitor = VisitorNode2 {
            x: &mut x,
            y: &mut y,
        };
        deserializer.deserialize_seq(visitor)?;
        *self.x = Some(x);
        *self.y = Some(y);
        Ok(())
    }
}
struct VisitorNode2<'query> {
    x: &'query mut Vec<f32>,
    y: &'query mut Vec<f32>,
}
impl<'query, 'de> serde::de::Visitor<'de> for VisitorNode2<'query> {
    type Value = ();
    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Formatter::write_str(formatter, "a sequence")
    }
    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        if let Some(additional) = seq.size_hint() {
            <Vec<
                f32,
            > as serde_query::__priv::Container>::reserve(&mut self.x, additional);
            <Vec<
                f32,
            > as serde_query::__priv::Container>::reserve(&mut self.y, additional);
        }
        loop {
            let mut x = None;
            let mut y = None;
            match seq
                .next_element_seed(DeserializeSeedNode3 {
                    x: &mut x,
                    y: &mut y,
                })?
            {
                Some(()) => {
                    <Vec<
                        f32,
                    > as serde_query::__priv::Container>::extend_one(
                        &mut self.x,
                        match x {
                            core::option::Option::Some(v) => v,
                            core::option::Option::None => {
                                return core::result::Result::Err(
                                    <<A as serde::de::SeqAccess<
                                        'de,
                                    >>::Error as serde::de::Error>::custom(
                                        "Query for 'x' failed to run",
                                    ),
                                );
                            }
                        },
                    );
                    <Vec<
                        f32,
                    > as serde_query::__priv::Container>::extend_one(
                        &mut self.y,
                        match y {
                            core::option::Option::Some(v) => v,
                            core::option::Option::None => {
                                return core::result::Result::Err(
                                    <<A as serde::de::SeqAccess<
                                        'de,
                                    >>::Error as serde::de::Error>::custom(
                                        "Query for 'y' failed to run",
                                    ),
                                );
                            }
                        },
                    );
                }
                None => {
                    break;
                }
            };
        }
        Ok(())
    }
}
struct DeserializeSeedNode3<'query> {
    x: &'query mut core::option::Option<
        <Vec<f32> as serde_query::__priv::Container>::Element,
    >,
    y: &'query mut core::option::Option<
        <Vec<f32> as serde_query::__priv::Container>::Element,
    >,
}
impl<'query, 'de> serde::de::DeserializeSeed<'de> for DeserializeSeedNode3<'query> {
    type Value = ();
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let visitor = VisitorNode3 {
            x: self.x,
            y: self.y,
        };
        deserializer.deserialize_map(visitor)
    }
}
struct VisitorNode3<'query> {
    x: &'query mut core::option::Option<
        <Vec<f32> as serde_query::__priv::Container>::Element,
    >,
    y: &'query mut core::option::Option<
        <Vec<f32> as serde_query::__priv::Container>::Element,
    >,
}
impl<'query, 'de> serde::de::Visitor<'de> for VisitorNode3<'query> {
    type Value = ();
    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Formatter::write_str(
            formatter,
            "one of the following fields: 'x', or 'y'",
        )
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<FieldNode3>()? {
            match key {
                FieldNode3::Field0 => {
                    map.next_value_seed(DeserializeSeedNode4 { x: self.x })?;
                }
                FieldNode3::Field1 => {
                    map.next_value_seed(DeserializeSeedNode8 { y: self.y })?;
                }
                FieldNode3::Ignore => {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}
enum FieldNode3 {
    Field0,
    Field1,
    Ignore,
}
impl<'de> serde::de::Deserialize<'de> for FieldNode3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(FieldVisitorNode3)
    }
}
struct FieldVisitorNode3;
impl<'de> serde::de::Visitor<'de> for FieldVisitorNode3 {
    type Value = FieldNode3;
    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Formatter::write_str(f, "one of the following fields: 'x', or 'y'")
    }
    fn visit_str<E>(self, value: &str) -> core::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match value {
            "x" => core::result::Result::Ok(FieldNode3::Field0),
            "y" => core::result::Result::Ok(FieldNode3::Field1),
            _ => core::result::Result::Ok(FieldNode3::Ignore),
        }
    }
    fn visit_bytes<E>(self, value: &[u8]) -> core::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match value {
            b"x" => core::result::Result::Ok(FieldNode3::Field0),
            b"y" => core::result::Result::Ok(FieldNode3::Field1),
            _ => core::result::Result::Ok(FieldNode3::Ignore),
        }
    }
}
struct DeserializeSeedNode4<'query> {
    x: &'query mut core::option::Option<
        <Vec<f32> as serde_query::__priv::Container>::Element,
    >,
}
impl<'query, 'de> serde::de::DeserializeSeed<'de> for DeserializeSeedNode4<'query> {
    type Value = ();
    fn deserialize<D>(
        self,
        deserializer: D,
    ) -> core::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *self
            .x = core::option::Option::Some(
            <<Vec<
                f32,
            > as serde_query::__priv::Container>::Element as serde::Deserialize<
                'de,
            >>::deserialize(deserializer)?,
        );
        Ok(())
    }
}
struct DeserializeSeedNode8<'query> {
    y: &'query mut core::option::Option<
        <Vec<f32> as serde_query::__priv::Container>::Element,
    >,
}
impl<'query, 'de> serde::de::DeserializeSeed<'de> for DeserializeSeedNode8<'query> {
    type Value = ();
    fn deserialize<D>(
        self,
        deserializer: D,
    ) -> core::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *self
            .y = core::option::Option::Some(
            <<Vec<
                f32,
            > as serde_query::__priv::Container>::Element as serde::Deserialize<
                'de,
            >>::deserialize(deserializer)?,
        );
        Ok(())
    }
}

"#
    );
}
