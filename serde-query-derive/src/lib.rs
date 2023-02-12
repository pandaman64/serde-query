extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{emit_error, proc_macro_error};
use quote::{quote, ToTokens};
use serde_query_core::{compile, Env, Query, QueryId};
use syn::{parse_macro_input, DeriveInput, Ident, LitStr};

mod parse_query;

#[derive(Debug, PartialEq, Eq)]
enum DeriveTarget {
    Deserialize,
    DeserializeQuery,
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

    if !input.generics.params.is_empty() {
        emit_error!(input.generics, "generic arguments are not supported");
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

    let node = compile(&mut Env::new(), fields.into_iter());
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
