use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::error::CargoArxmlError;
use crate::parser::ir::{ArxmlProject, Method, ServiceInterface};

use super::snake_case;
use super::types::resolve_type_name;

/// Generate a `traits.rs` file declaring a Rust trait for each service
/// interface.
pub fn generate_traits(project: &ArxmlProject) -> Result<String, CargoArxmlError> {
    let mut items: Vec<TokenStream> = Vec::new();

    // Check if any service references custom (non-primitive) types.
    let needs_types_import = project.services.iter().any(|svc| {
        svc.methods.iter().any(|m| {
            m.input_params
                .iter()
                .chain(m.output_params.iter())
                .any(|p| {
                    let resolved = resolve_type_name(&p.type_ref, project);
                    resolved.chars().next().is_some_and(|c| c.is_uppercase())
                })
        })
    });

    if needs_types_import {
        items.push(quote! {
            use super::types::*;
            use ara_com::error::AraComError;
        });
    } else {
        items.push(quote! {
            use ara_com::error::AraComError;
        });
    }

    for svc in &project.services {
        items.push(generate_service_trait(svc, project));
    }

    let combined: TokenStream = items.into_iter().collect();

    let file: syn::File = syn::parse2(combined).map_err(|e| CargoArxmlError::CodeGen {
        message: format!("failed to parse generated traits: {e}"),
    })?;

    Ok(prettyplease::unparse(&file))
}

// ---------------------------------------------------------------------------
// Per-service trait generation
// ---------------------------------------------------------------------------

fn generate_service_trait(svc: &ServiceInterface, project: &ArxmlProject) -> TokenStream {
    let trait_name = Ident::new(&svc.short_name, Span::call_site());
    let doc = svc
        .description
        .as_deref()
        .map(|d| quote! { #[doc = #d] })
        .unwrap_or_default();

    let methods: Vec<TokenStream> = svc
        .methods
        .iter()
        .map(|m| generate_method_sig(m, project))
        .collect();

    quote! {
        #doc
        #[async_trait::async_trait]
        pub trait #trait_name: Send + Sync {
            #(#methods)*
        }
    }
}

fn generate_method_sig(method: &Method, project: &ArxmlProject) -> TokenStream {
    let method_name = Ident::new(&snake_case(&method.name), Span::call_site());
    let doc = method
        .description
        .as_deref()
        .map(|d| quote! { #[doc = #d] })
        .unwrap_or_default();

    // Input parameters
    let input_params: Vec<TokenStream> = method
        .input_params
        .iter()
        .map(|p| {
            let pname = Ident::new(&snake_case(&p.name), Span::call_site());
            let ptype_str = resolve_type_name(&p.type_ref, project);
            let ptype: TokenStream = ptype_str.parse().unwrap_or_else(|_| quote! { () });
            quote! { #pname: #ptype, }
        })
        .collect();

    if method.fire_and_forget {
        quote! {
            #doc
            async fn #method_name(&self, #(#input_params)*) -> Result<(), AraComError>;
        }
    } else {
        // Build output return type
        let return_type = build_return_type(&method.output_params, project);
        quote! {
            #doc
            async fn #method_name(&self, #(#input_params)*) -> Result<#return_type, AraComError>;
        }
    }
}

/// Build the `Ok(...)` return type token stream from a list of output params.
fn build_return_type(
    output_params: &[crate::parser::ir::Parameter],
    project: &ArxmlProject,
) -> TokenStream {
    match output_params {
        [] => quote! { () },
        [single] => {
            let type_str = resolve_type_name(&single.type_ref, project);
            type_str.parse().unwrap_or_else(|_| quote! { () })
        }
        multiple => {
            let types: Vec<TokenStream> = multiple
                .iter()
                .map(|p| {
                    let type_str = resolve_type_name(&p.type_ref, project);
                    type_str.parse().unwrap_or_else(|_| quote! { () })
                })
                .collect();
            quote! { (#(#types),*) }
        }
    }
}
