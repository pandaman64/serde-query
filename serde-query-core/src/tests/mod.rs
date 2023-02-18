use syn::DeriveInput;

use crate::{generate_derive, DeriveTarget};

mod generate_basic;

fn snapshot_derive(input: DeriveInput, target: DeriveTarget) -> String {
    match generate_derive(input, target) {
        Ok(stream) => prettyplease::unparse(&syn::parse2(stream).unwrap()),
        Err(diagnostics) => {
            let diagnostics: Vec<String> = diagnostics
                .into_iter()
                .map(|diagnostic| format!("{:?}", diagnostic))
                .collect();
            diagnostics.join("\n")
        }
    }
}
