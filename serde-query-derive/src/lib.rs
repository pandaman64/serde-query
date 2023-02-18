extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::{emit_error, proc_macro_error};
use quote::quote;
use serde_query_core::DeriveTarget;
use syn::{parse_macro_input, DeriveInput};

/// Generate a minimal, non-functioning Deserialize(Query) implementation on errors
fn set_dummy(input: &DeriveInput, target: DeriveTarget) {
    let name = &input.ident;
    let generics = &input.generics;

    match target {
        DeriveTarget::Deserialize => {
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
        DeriveTarget::DeserializeQuery => {
            proc_macro_error::set_dummy(quote! {
                const _: () = {
                    struct __QueryWrapper;

                    impl<'de> serde_query::DeserializeQuery<'de> for #name {
                        type Query = __QueryWrapper;
                    }
                };
            });
        }
    }
}

fn generate_derive(input: TokenStream, target: DeriveTarget) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    set_dummy(&input, target);

    if !input.generics.params.is_empty() {
        emit_error!(input.generics, "generic arguments are not supported");
        return TokenStream::new();
    }

    match serde_query_core::generate_derive(input, target) {
        Ok(stream) => stream.into(),
        Err(diagnostics) => {
            for diagnostic in diagnostics {
                diagnostic.emit();
            }
            TokenStream::new()
        }
    }
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
