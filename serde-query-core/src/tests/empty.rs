use syn::DeriveInput;

use crate::{tests::snapshot_derive, DeriveTarget};

#[test]
fn test_empty() {
    let input: DeriveInput = syn::parse_quote! {
        struct EmptyInput {
        }
    };

    k9::snapshot!(
        snapshot_derive(input, DeriveTarget::Deserialize),
        r#"
const _: () = {
    struct DeserializeSeedNode0 {}
    impl<'de> serde_query::__priv::serde::de::DeserializeSeed<'de>
    for DeserializeSeedNode0 {
        type Value = ();
        fn deserialize<D>(
            self,
            deserializer: D,
        ) -> core::result::Result<Self::Value, D::Error>
        where
            D: serde_query::__priv::serde::Deserializer<'de>,
        {
            core::result::Result::Ok(())
        }
    }
    impl<'de> serde_query::__priv::serde::de::Deserialize<'de> for EmptyInput {
        fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
        where
            D: serde_query::__priv::serde::de::Deserializer<'de>,
        {
            let root = DeserializeSeedNode0 {};
            <DeserializeSeedNode0 as serde_query::__priv::serde::de::DeserializeSeed<
                'de,
            >>::deserialize(root, deserializer)?;
            let has_error = false;
            if !has_error {
                let value = EmptyInput {};
                core::result::Result::Ok(value)
            } else {
                let errors = [];
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
