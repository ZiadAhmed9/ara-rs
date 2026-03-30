use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::error::CargoArxmlError;
use crate::parser::ir::{ArxmlProject, DataType, DataTypeKind};

use super::snake_case;
use super::types::resolve_type_name;

/// Generate a `tests.rs` file with basic round-trip smoke tests for each
/// generated data type.
pub fn generate_tests(project: &ArxmlProject) -> Result<String, CargoArxmlError> {
    let mut test_fns: Vec<TokenStream> = Vec::new();

    for dt in &project.data_types {
        if let Some(ts) = generate_type_test(dt, project) {
            test_fns.push(ts);
        }
    }

    if test_fns.is_empty() {
        return Ok(String::from(
            "// Auto-generated tests — no testable types found.\n",
        ));
    }

    let cfg_test: TokenStream = "#[cfg(test)]".parse().unwrap();
    let tokens = quote! {
        #cfg_test
        mod generated_tests {
            use super::types::*;
            use ara_com::transport::{AraSerialize, AraDeserialize};

            #(#test_fns)*
        }
    };

    let file: syn::File = syn::parse2(tokens).map_err(|e| CargoArxmlError::CodeGen {
        message: format!("failed to parse generated tests: {e}"),
    })?;

    Ok(prettyplease::unparse(&file))
}

/// Generate a round-trip test for a single data type, if applicable.
fn generate_type_test(dt: &DataType, project: &ArxmlProject) -> Option<TokenStream> {
    let type_name = Ident::new(&dt.name, Span::call_site());
    let test_name = Ident::new(
        &format!("test_round_trip_{}", snake_case(&dt.name)),
        Span::call_site(),
    );

    match &dt.kind {
        DataTypeKind::Structure { fields } => {
            let field_inits: Vec<TokenStream> = fields
                .iter()
                .map(|f| {
                    let fname = Ident::new(&snake_case(&f.name), Span::call_site());
                    let default_val = default_value_for_type_ref(&f.type_ref, project);
                    quote! { #fname: #default_val, }
                })
                .collect();

            Some(quote! {
                #[test]
                fn #test_name() {
                    let original = #type_name {
                        #(#field_inits)*
                    };
                    let mut buf = Vec::new();
                    original.ara_serialize(&mut buf).expect("serialize failed");
                    let decoded = #type_name::ara_deserialize(&buf).expect("deserialize failed");
                    assert_eq!(original, decoded);
                }
            })
        }

        DataTypeKind::Enumeration { variants } => {
            if let Some(first) = variants.first() {
                let variant_name = Ident::new(&first.name, Span::call_site());
                Some(quote! {
                    #[test]
                    fn #test_name() {
                        let original = #type_name::#variant_name;
                        let mut buf = Vec::new();
                        original.ara_serialize(&mut buf).expect("serialize failed");
                        let decoded = #type_name::ara_deserialize(&buf).expect("deserialize failed");
                        assert_eq!(original, decoded);
                    }
                })
            } else {
                None
            }
        }

        // Primitive, Array, Vector, String, TypeReference are type aliases —
        // their serialization is already tested by ara-com's own test suite.
        _ => None,
    }
}

/// Produce a default/zero value token stream for a given type reference.
fn default_value_for_type_ref(type_ref: &str, project: &ArxmlProject) -> TokenStream {
    let resolved = resolve_type_name(type_ref, project);
    match resolved.as_str() {
        "bool" => quote! { false },
        "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => quote! { 0 },
        "f32" | "f64" => quote! { 0.0 },
        "String" => quote! { String::new() },
        "()" => quote! { () },
        _ => {
            let ident: TokenStream = resolved.parse().unwrap_or_else(|_| quote! { () });
            quote! { #ident::default() }
        }
    }
}
