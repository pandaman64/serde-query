mod node;
mod parse_input;
mod parse_query;
mod query;

#[cfg(test)]
mod tests;

use node::Node;
use parse_input::parse_input;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::Diagnostic;
use syn::DeriveInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeriveTarget {
    Deserialize,
    DeserializeQuery,
}

pub fn generate_derive(
    mut input: DeriveInput,
    target: DeriveTarget,
) -> Result<TokenStream, Vec<Diagnostic>> {
    let parse_input_result = parse_input(&mut input);
    if !parse_input_result.diagnostics.is_empty() {
        return Err(parse_input_result.diagnostics);
    }

    let name = &input.ident;
    let node = Node::from_queries(parse_input_result.queries.into_iter())?;
    let mut stream = node.generate().map_err(|diagnostic| vec![diagnostic])?;

    // generate the root code
    match target {
        // generate DeserializeQuery and conversion traits
        DeriveTarget::DeserializeQuery => {
            let wrapper_ty = syn::Ident::new("__QueryWrapper", Span::call_site());

            // Inherit visibility of the wrapped struct to avoid error E0446
            // See: https://github.com/pandaman64/serde-query/issues/7
            let vis = input.vis;

            let deserialize_impl = node.generate_deserialize(
                name,
                &wrapper_ty,
                |value| quote::quote!(#wrapper_ty(#value)),
            );

            stream.extend(quote::quote! {
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

            stream.extend(quote::quote! {
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
    Ok(quote::quote! {
        const _: () = {
            #stream
        };
    })
}
