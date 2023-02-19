use syn::DeriveInput;

use crate::{tests::snapshot_derive, DeriveTarget};

#[test]
fn test_basic() {
    let input: DeriveInput = syn::parse_quote! {
        struct Locations {
            #[query(".locs.[].x")]
            x: Vec<f32>,
            #[query(".locs.[].y")]
            y: Vec<f32>,
        }
    };

    k9::snapshot!(
        snapshot_derive(input, DeriveTarget::Deserialize),
        r#"
const _: () = {
    struct DeserializeSeedNode0<'query> {
        x: &'query mut core::option::Option<
            core::result::Result<Vec<f32>, serde_query::__priv::Error>,
        >,
        y: &'query mut core::option::Option<
            core::result::Result<Vec<f32>, serde_query::__priv::Error>,
        >,
    }
    impl<'query, 'de> serde_query::__priv::serde::de::DeserializeSeed<'de>
    for DeserializeSeedNode0<'query> {
        type Value = ();
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde_query::__priv::serde::Deserializer<'de>,
        {
            let visitor = VisitorNode0 {
                x: self.x,
                y: self.y,
            };
            deserializer.deserialize_map(visitor)?;
            if self.x.is_none() {
                *self
                    .x = core::option::Option::Some(
                    core::result::Result::Err(
                        serde_query::__priv::Error::borrowed(
                            "x",
                            ".",
                            "missing field 'locs'",
                        ),
                    ),
                );
            }
            if self.y.is_none() {
                *self
                    .y = core::option::Option::Some(
                    core::result::Result::Err(
                        serde_query::__priv::Error::borrowed(
                            "y",
                            ".",
                            "missing field 'locs'",
                        ),
                    ),
                );
            }
            core::result::Result::Ok(())
        }
    }
    struct VisitorNode0<'query> {
        x: &'query mut core::option::Option<
            core::result::Result<Vec<f32>, serde_query::__priv::Error>,
        >,
        y: &'query mut core::option::Option<
            core::result::Result<Vec<f32>, serde_query::__priv::Error>,
        >,
    }
    impl<'query, 'de> serde_query::__priv::serde::de::Visitor<'de>
    for VisitorNode0<'query> {
        type Value = ();
        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            core::fmt::Formatter::write_str(
                formatter,
                "one of the following fields: 'locs'",
            )
        }
        fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde_query::__priv::serde::de::MapAccess<'de>,
        {
            while let core::option::Option::Some(key) = map.next_key::<FieldNode0>()? {
                match key {
                    FieldNode0::Field0 => {
                        let mut x = core::option::Option::None;
                        let mut y = core::option::Option::None;
                        let x = match &mut self.x {
                            core::option::Option::Some(core::result::Result::Ok(_)) => {
                                *self
                                    .x = core::option::Option::Some(
                                    core::result::Result::Err(
                                        serde_query::__priv::Error::borrowed(
                                            "x",
                                            ".",
                                            "duplicated field 'locs'",
                                        ),
                                    ),
                                );
                                &mut x
                            }
                            core::option::Option::Some(core::result::Result::Err(_)) => {
                                &mut x
                            }
                            core::option::Option::None => &mut self.x,
                        };
                        let y = match &mut self.y {
                            core::option::Option::Some(core::result::Result::Ok(_)) => {
                                *self
                                    .y = core::option::Option::Some(
                                    core::result::Result::Err(
                                        serde_query::__priv::Error::borrowed(
                                            "y",
                                            ".",
                                            "duplicated field 'locs'",
                                        ),
                                    ),
                                );
                                &mut y
                            }
                            core::option::Option::Some(core::result::Result::Err(_)) => {
                                &mut y
                            }
                            core::option::Option::None => &mut self.y,
                        };
                        map.next_value_seed(DeserializeSeedNode2 { x, y })?;
                    }
                    FieldNode0::Ignore => {
                        map.next_value::<serde_query::__priv::serde::de::IgnoredAny>()?;
                    }
                }
            }
            core::result::Result::Ok(())
        }
    }
    enum FieldNode0 {
        Field0,
        Ignore,
    }
    impl<'de> serde_query::__priv::serde::de::Deserialize<'de> for FieldNode0 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde_query::__priv::serde::Deserializer<'de>,
        {
            deserializer.deserialize_identifier(FieldVisitorNode0)
        }
    }
    struct FieldVisitorNode0;
    impl<'de> serde_query::__priv::serde::de::Visitor<'de> for FieldVisitorNode0 {
        type Value = FieldNode0;
        fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            core::fmt::Formatter::write_str(f, "one of the following fields: 'locs'")
        }
        fn visit_str<E>(self, value: &str) -> core::result::Result<Self::Value, E>
        where
            E: serde_query::__priv::serde::de::Error,
        {
            match value {
                "locs" => core::result::Result::Ok(FieldNode0::Field0),
                _ => core::result::Result::Ok(FieldNode0::Ignore),
            }
        }
        fn visit_bytes<E>(self, value: &[u8]) -> core::result::Result<Self::Value, E>
        where
            E: serde_query::__priv::serde::de::Error,
        {
            match value {
                b"locs" => core::result::Result::Ok(FieldNode0::Field0),
                _ => core::result::Result::Ok(FieldNode0::Ignore),
            }
        }
    }
    struct DeserializeSeedNode2<'query> {
        x: &'query mut core::option::Option<
            core::result::Result<Vec<f32>, serde_query::__priv::Error>,
        >,
        y: &'query mut core::option::Option<
            core::result::Result<Vec<f32>, serde_query::__priv::Error>,
        >,
    }
    impl<'query, 'de> serde_query::__priv::serde::de::DeserializeSeed<'de>
    for DeserializeSeedNode2<'query> {
        type Value = ();
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde_query::__priv::serde::Deserializer<'de>,
        {
            let mut x = core::result::Result::Ok(
                <Vec<f32> as serde_query::__priv::Container>::empty(),
            );
            let mut y = core::result::Result::Ok(
                <Vec<f32> as serde_query::__priv::Container>::empty(),
            );
            let visitor = VisitorNode2 {
                x: &mut x,
                y: &mut y,
            };
            deserializer.deserialize_seq(visitor)?;
            *self.x = core::option::Option::Some(x);
            *self.y = core::option::Option::Some(y);
            core::result::Result::Ok(())
        }
    }
    struct VisitorNode2<'query> {
        x: &'query mut core::result::Result<Vec<f32>, serde_query::__priv::Error>,
        y: &'query mut core::result::Result<Vec<f32>, serde_query::__priv::Error>,
    }
    impl<'query, 'de> serde_query::__priv::serde::de::Visitor<'de>
    for VisitorNode2<'query> {
        type Value = ();
        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            core::fmt::Formatter::write_str(formatter, "a sequence")
        }
        fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde_query::__priv::serde::de::SeqAccess<'de>,
        {
            if let core::option::Option::Some(additional) = seq.size_hint() {
                <Vec<
                    f32,
                > as serde_query::__priv::Container>::reserve(
                    self.x.as_mut().unwrap(),
                    additional,
                );
                <Vec<
                    f32,
                > as serde_query::__priv::Container>::reserve(
                    self.y.as_mut().unwrap(),
                    additional,
                );
            }
            loop {
                let mut x = core::option::Option::None;
                let mut y = core::option::Option::None;
                match seq
                    .next_element_seed(DeserializeSeedNode3 {
                        x: &mut x,
                        y: &mut y,
                    })?
                {
                    core::option::Option::None => break,
                    core::option::Option::Some(()) => {
                        match &mut self.x {
                            core::result::Result::Ok(ref mut container) => {
                                match x {
                                    core::option::Option::Some(core::result::Result::Ok(v)) => {
                                        <Vec<
                                            f32,
                                        > as serde_query::__priv::Container>::extend_one(
                                            container,
                                            v,
                                        )
                                    }
                                    core::option::Option::Some(
                                        core::result::Result::Err(e),
                                    ) => {
                                        *self.x = core::result::Result::Err(e);
                                    }
                                    core::option::Option::None => unreachable!(),
                                }
                            }
                            core::result::Result::Err(_) => {}
                        }
                        match &mut self.y {
                            core::result::Result::Ok(ref mut container) => {
                                match y {
                                    core::option::Option::Some(core::result::Result::Ok(v)) => {
                                        <Vec<
                                            f32,
                                        > as serde_query::__priv::Container>::extend_one(
                                            container,
                                            v,
                                        )
                                    }
                                    core::option::Option::Some(
                                        core::result::Result::Err(e),
                                    ) => {
                                        *self.y = core::result::Result::Err(e);
                                    }
                                    core::option::Option::None => unreachable!(),
                                }
                            }
                            core::result::Result::Err(_) => {}
                        }
                    }
                };
            }
            core::result::Result::Ok(())
        }
    }
    struct DeserializeSeedNode3<'query> {
        x: &'query mut core::option::Option<
            core::result::Result<
                <Vec<f32> as serde_query::__priv::Container>::Element,
                serde_query::__priv::Error,
            >,
        >,
        y: &'query mut core::option::Option<
            core::result::Result<
                <Vec<f32> as serde_query::__priv::Container>::Element,
                serde_query::__priv::Error,
            >,
        >,
    }
    impl<'query, 'de> serde_query::__priv::serde::de::DeserializeSeed<'de>
    for DeserializeSeedNode3<'query> {
        type Value = ();
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde_query::__priv::serde::Deserializer<'de>,
        {
            let visitor = VisitorNode3 {
                x: self.x,
                y: self.y,
            };
            deserializer.deserialize_map(visitor)?;
            if self.x.is_none() {
                *self
                    .x = core::option::Option::Some(
                    core::result::Result::Err(
                        serde_query::__priv::Error::borrowed(
                            "x",
                            ".locs.[]",
                            "missing field 'x'",
                        ),
                    ),
                );
            }
            if self.y.is_none() {
                *self
                    .y = core::option::Option::Some(
                    core::result::Result::Err(
                        serde_query::__priv::Error::borrowed(
                            "y",
                            ".locs.[]",
                            "missing field 'y'",
                        ),
                    ),
                );
            }
            core::result::Result::Ok(())
        }
    }
    struct VisitorNode3<'query> {
        x: &'query mut core::option::Option<
            core::result::Result<
                <Vec<f32> as serde_query::__priv::Container>::Element,
                serde_query::__priv::Error,
            >,
        >,
        y: &'query mut core::option::Option<
            core::result::Result<
                <Vec<f32> as serde_query::__priv::Container>::Element,
                serde_query::__priv::Error,
            >,
        >,
    }
    impl<'query, 'de> serde_query::__priv::serde::de::Visitor<'de>
    for VisitorNode3<'query> {
        type Value = ();
        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            core::fmt::Formatter::write_str(
                formatter,
                "one of the following fields: 'x', or 'y'",
            )
        }
        fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde_query::__priv::serde::de::MapAccess<'de>,
        {
            while let core::option::Option::Some(key) = map.next_key::<FieldNode3>()? {
                match key {
                    FieldNode3::Field0 => {
                        let mut x = core::option::Option::None;
                        let x = match &mut self.x {
                            core::option::Option::Some(core::result::Result::Ok(_)) => {
                                *self
                                    .x = core::option::Option::Some(
                                    core::result::Result::Err(
                                        serde_query::__priv::Error::borrowed(
                                            "x",
                                            ".locs.[]",
                                            "duplicated field 'x'",
                                        ),
                                    ),
                                );
                                &mut x
                            }
                            core::option::Option::Some(core::result::Result::Err(_)) => {
                                &mut x
                            }
                            core::option::Option::None => &mut self.x,
                        };
                        map.next_value_seed(DeserializeSeedNode4 { x })?;
                    }
                    FieldNode3::Field1 => {
                        let mut y = core::option::Option::None;
                        let y = match &mut self.y {
                            core::option::Option::Some(core::result::Result::Ok(_)) => {
                                *self
                                    .y = core::option::Option::Some(
                                    core::result::Result::Err(
                                        serde_query::__priv::Error::borrowed(
                                            "y",
                                            ".locs.[]",
                                            "duplicated field 'y'",
                                        ),
                                    ),
                                );
                                &mut y
                            }
                            core::option::Option::Some(core::result::Result::Err(_)) => {
                                &mut y
                            }
                            core::option::Option::None => &mut self.y,
                        };
                        map.next_value_seed(DeserializeSeedNode8 { y })?;
                    }
                    FieldNode3::Ignore => {
                        map.next_value::<serde_query::__priv::serde::de::IgnoredAny>()?;
                    }
                }
            }
            core::result::Result::Ok(())
        }
    }
    enum FieldNode3 {
        Field0,
        Field1,
        Ignore,
    }
    impl<'de> serde_query::__priv::serde::de::Deserialize<'de> for FieldNode3 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde_query::__priv::serde::Deserializer<'de>,
        {
            deserializer.deserialize_identifier(FieldVisitorNode3)
        }
    }
    struct FieldVisitorNode3;
    impl<'de> serde_query::__priv::serde::de::Visitor<'de> for FieldVisitorNode3 {
        type Value = FieldNode3;
        fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            core::fmt::Formatter::write_str(
                f,
                "one of the following fields: 'x', or 'y'",
            )
        }
        fn visit_str<E>(self, value: &str) -> core::result::Result<Self::Value, E>
        where
            E: serde_query::__priv::serde::de::Error,
        {
            match value {
                "x" => core::result::Result::Ok(FieldNode3::Field0),
                "y" => core::result::Result::Ok(FieldNode3::Field1),
                _ => core::result::Result::Ok(FieldNode3::Ignore),
            }
        }
        fn visit_bytes<E>(self, value: &[u8]) -> core::result::Result<Self::Value, E>
        where
            E: serde_query::__priv::serde::de::Error,
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
            core::result::Result<
                <Vec<f32> as serde_query::__priv::Container>::Element,
                serde_query::__priv::Error,
            >,
        >,
    }
    impl<'query, 'de> serde_query::__priv::serde::de::DeserializeSeed<'de>
    for DeserializeSeedNode4<'query> {
        type Value = ();
        fn deserialize<D>(
            self,
            deserializer: D,
        ) -> core::result::Result<Self::Value, D::Error>
        where
            D: serde_query::__priv::serde::Deserializer<'de>,
        {
            let result = match <<Vec<
                f32,
            > as serde_query::__priv::Container>::Element as serde_query::__priv::serde::Deserialize<
                'de,
            >>::deserialize(deserializer) {
                core::result::Result::Ok(v) => core::result::Result::Ok(v),
                core::result::Result::Err(e) => {
                    core::result::Result::Err(
                        serde_query::__priv::Error::owned(
                            "x",
                            ".locs.[].x",
                            e.to_string(),
                        ),
                    )
                }
            };
            *self.x = core::option::Option::Some(result);
            core::result::Result::Ok(())
        }
    }
    struct DeserializeSeedNode8<'query> {
        y: &'query mut core::option::Option<
            core::result::Result<
                <Vec<f32> as serde_query::__priv::Container>::Element,
                serde_query::__priv::Error,
            >,
        >,
    }
    impl<'query, 'de> serde_query::__priv::serde::de::DeserializeSeed<'de>
    for DeserializeSeedNode8<'query> {
        type Value = ();
        fn deserialize<D>(
            self,
            deserializer: D,
        ) -> core::result::Result<Self::Value, D::Error>
        where
            D: serde_query::__priv::serde::Deserializer<'de>,
        {
            let result = match <<Vec<
                f32,
            > as serde_query::__priv::Container>::Element as serde_query::__priv::serde::Deserialize<
                'de,
            >>::deserialize(deserializer) {
                core::result::Result::Ok(v) => core::result::Result::Ok(v),
                core::result::Result::Err(e) => {
                    core::result::Result::Err(
                        serde_query::__priv::Error::owned(
                            "y",
                            ".locs.[].y",
                            e.to_string(),
                        ),
                    )
                }
            };
            *self.y = core::option::Option::Some(result);
            core::result::Result::Ok(())
        }
    }
    impl<'de> serde_query::__priv::serde::de::Deserialize<'de> for Locations {
        fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
        where
            D: serde_query::__priv::serde::de::Deserializer<'de>,
        {
            let mut x = core::option::Option::None;
            let mut y = core::option::Option::None;
            let root = DeserializeSeedNode0 {
                x: &mut x,
                y: &mut y,
            };
            <DeserializeSeedNode0 as serde_query::__priv::serde::de::DeserializeSeed<
                'de,
            >>::deserialize(root, deserializer)?;
            let x = x.unwrap();
            let y = y.unwrap();
            let has_error = false || x.is_err() || y.is_err();
            if !has_error {
                let value = Locations {
                    x: x.unwrap(),
                    y: y.unwrap(),
                };
                core::result::Result::Ok(value)
            } else {
                let errors = [x.err(), y.err()];
                core::result::Result::Err(
                    <D::Error as serde_query::__priv::serde::de::Error>::custom(
                        serde_query::__priv::Errors::new(&errors),
                    ),
                )
            }
        }
    }
};

"#
    );
}
