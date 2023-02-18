extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{emit_error, proc_macro_error};
use quote::quote;
use serde_query_core::{compile, parser::parse_input, DeriveTarget, Env};
use syn::{parse_macro_input, DeriveInput, Ident};

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
    let mut input = parse_macro_input!(input as DeriveInput);

    set_dummy(&input, target);

    if !input.generics.params.is_empty() {
        emit_error!(input.generics, "generic arguments are not supported");
        return TokenStream::new();
    }

    let parse_input_result = parse_input(&mut input);
    if !parse_input_result.diagnostics.is_empty() {
        for diagnostic in parse_input_result.diagnostics.into_iter() {
            diagnostic.emit();
        }
        return TokenStream::new();
    }

    let name = &input.ident;
    let node = compile(&mut Env::new(), parse_input_result.queries.into_iter());
    let mut stream = node.generate();

    // generate the root code
    match target {
        // generate DeserializeQuery and conversion traits
        DeriveTarget::DeserializeQuery => {
            let wrapper_ty = Ident::new("__QueryWrapper", Span::call_site());

            // Inherit visibility of the wrapped struct to avoid error E0446
            // See: https://github.com/pandaman64/serde-query/issues/7
            let vis = input.vis;

            let deserialize_impl =
                node.generate_deserialize(name, &wrapper_ty, |value| quote!(#wrapper_ty(#value)));

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
            let deserialize_impl = node.generate_deserialize(name, name, |value| value);
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
