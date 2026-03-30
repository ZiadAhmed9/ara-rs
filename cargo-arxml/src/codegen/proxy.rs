use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;

use crate::error::CargoArxmlError;
use crate::parser::ir::{Method, ServiceInterface};

use super::snake_case;
use super::types::resolve_type_name;

/// Generate a proxy stub for `svc`.
///
/// The proxy is the client-side handle used to call methods and subscribe to
/// events on a remote service.
pub fn generate_proxy(svc: &ServiceInterface) -> Result<String, CargoArxmlError> {
    let empty_project = crate::parser::ir::ArxmlProject::default();

    let struct_name_str = format!("{}Proxy", svc.short_name);
    let struct_name = Ident::new(&struct_name_str, Span::call_site());

    let doc = svc
        .description
        .as_deref()
        .map(|d| quote! { #[doc = #d] })
        .unwrap_or_default();

    let service_id_val = svc.service_id.unwrap_or(0);
    let service_id_lit = Literal::u16_unsuffixed(service_id_val);

    // Split each method into (struct_defs, method_body)
    let mut all_struct_defs: Vec<TokenStream> = Vec::new();
    let mut all_method_bodies: Vec<TokenStream> = Vec::new();

    for m in &svc.methods {
        let (structs, body) = generate_proxy_method(m, &empty_project);
        all_struct_defs.push(structs);
        all_method_bodies.push(body);
    }

    // Event subscribe methods (no struct defs needed)
    let event_impls: Vec<TokenStream> = svc
        .events
        .iter()
        .map(generate_event_subscribe)
        .collect();

    let tokens = quote! {
        use std::sync::Arc;
        use ara_com::proxy::ProxyBase;
        use ara_com::transport::{AraSerialize, AraDeserialize, Transport};
        use ara_com::types::{InstanceId, MethodId, ServiceId, EventGroupId};
        use ara_com::error::AraComError;

        #(#all_struct_defs)*

        #doc
        pub struct #struct_name<T: Transport> {
            pub base: ProxyBase<T>,
        }

        impl<T: Transport> #struct_name<T> {
            /// Create a new proxy connected to the given transport and instance.
            pub fn new(transport: Arc<T>, instance_id: InstanceId) -> Self {
                Self {
                    base: ProxyBase::with_defaults(
                        transport,
                        ServiceId(#service_id_lit),
                        instance_id,
                    ),
                }
            }

            #(#all_method_bodies)*
            #(#event_impls)*
        }
    };

    let file: syn::File = syn::parse2(tokens).map_err(|e| CargoArxmlError::CodeGen {
        message: format!("failed to parse generated proxy for '{}': {e}", svc.short_name),
    })?;

    Ok(prettyplease::unparse(&file))
}

// ---------------------------------------------------------------------------
// Per-method proxy stub — returns (struct_definitions, method_body)
// ---------------------------------------------------------------------------

fn generate_proxy_method(
    method: &Method,
    project: &crate::parser::ir::ArxmlProject,
) -> (TokenStream, TokenStream) {
    let method_fn_name = Ident::new(&snake_case(&method.name), Span::call_site());
    let req_struct_name = Ident::new(
        &format!("{}Request", method.name),
        Span::call_site(),
    );
    let method_id_val = method.method_id.unwrap_or(0);
    let method_id_lit = Literal::u16_unsuffixed(method_id_val);

    let doc = method
        .description
        .as_deref()
        .map(|d| quote! { #[doc = #d] })
        .unwrap_or_default();

    // Input param list for the function signature
    let input_params: Vec<TokenStream> = method
        .input_params
        .iter()
        .map(|p| {
            let pname = Ident::new(&snake_case(&p.name), Span::call_site());
            let ptype_str = resolve_type_name(&p.type_ref, project);
            let ptype: TokenStream = ptype_str.parse().unwrap_or_else(|_| quote! { () });
            quote! { #pname: #ptype }
        })
        .collect();

    // Request struct fields
    let req_fields: Vec<TokenStream> = method
        .input_params
        .iter()
        .map(|p| {
            let pname = Ident::new(&snake_case(&p.name), Span::call_site());
            let ptype_str = resolve_type_name(&p.type_ref, project);
            let ptype: TokenStream = ptype_str.parse().unwrap_or_else(|_| quote! { () });
            quote! { pub #pname: #ptype, }
        })
        .collect();

    let serialize_fields: Vec<TokenStream> = method
        .input_params
        .iter()
        .map(|p| {
            let pname = Ident::new(&snake_case(&p.name), Span::call_site());
            quote! { self.#pname.ara_serialize(buf)?; }
        })
        .collect();

    let size_fields: Vec<TokenStream> = if method.input_params.is_empty() {
        vec![quote! { 0 }]
    } else {
        method
            .input_params
            .iter()
            .map(|p| {
                let pname = Ident::new(&snake_case(&p.name), Span::call_site());
                quote! { self.#pname.serialized_size() }
            })
            .collect()
    };

    let deser_fields: Vec<TokenStream> = method
        .input_params
        .iter()
        .map(|p| {
            let pname = Ident::new(&snake_case(&p.name), Span::call_site());
            let ptype_str = resolve_type_name(&p.type_ref, project);
            let ptype: TokenStream = ptype_str.parse().unwrap_or_else(|_| quote! { () });
            quote! {
                let #pname = <#ptype as AraDeserialize>::ara_deserialize(&buf[offset..])?;
                offset += #pname.serialized_size();
            }
        })
        .collect();

    let req_field_names: Vec<Ident> = method
        .input_params
        .iter()
        .map(|p| Ident::new(&snake_case(&p.name), Span::call_site()))
        .collect();

    let req_constructor_fields: Vec<TokenStream> = method
        .input_params
        .iter()
        .map(|p| {
            let pname = Ident::new(&snake_case(&p.name), Span::call_site());
            quote! { #pname, }
        })
        .collect();

    // Request struct + serialization impls (goes outside impl block)
    let req_struct_defs = quote! {
        pub struct #req_struct_name {
            #(#req_fields)*
        }

        impl AraSerialize for #req_struct_name {
            fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
                #(#serialize_fields)*
                Ok(())
            }

            fn serialized_size(&self) -> usize {
                #(#size_fields)+*
            }
        }

        impl AraDeserialize for #req_struct_name {
            fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
                let mut offset = 0usize;
                #(#deser_fields)*
                Ok(Self { #(#req_field_names,)* })
            }
        }
    };

    if method.fire_and_forget {
        let method_body = quote! {
            #doc
            pub async fn #method_fn_name(&self, #(#input_params,)*) -> Result<(), AraComError> {
                let req = #req_struct_name { #(#req_constructor_fields)* };
                self.base.call_fire_and_forget(MethodId(#method_id_lit), &req).await
            }
        };
        (req_struct_defs, method_body)
    } else {
        let return_type = build_return_type_tokens(&method.output_params, project);
        let resp_struct_name = Ident::new(
            &format!("{}Response", method.name),
            Span::call_site(),
        );

        let resp_fields: Vec<TokenStream> = method
            .output_params
            .iter()
            .map(|p| {
                let pname = Ident::new(&snake_case(&p.name), Span::call_site());
                let ptype_str = resolve_type_name(&p.type_ref, project);
                let ptype: TokenStream = ptype_str.parse().unwrap_or_else(|_| quote! { () });
                quote! { pub #pname: #ptype, }
            })
            .collect();

        let resp_serialize_fields: Vec<TokenStream> = method
            .output_params
            .iter()
            .map(|p| {
                let pname = Ident::new(&snake_case(&p.name), Span::call_site());
                quote! { self.#pname.ara_serialize(buf)?; }
            })
            .collect();

        let resp_size_fields: Vec<TokenStream> = if method.output_params.is_empty() {
            vec![quote! { 0 }]
        } else {
            method
                .output_params
                .iter()
                .map(|p| {
                    let pname = Ident::new(&snake_case(&p.name), Span::call_site());
                    quote! { self.#pname.serialized_size() }
                })
                .collect()
        };

        let resp_deser_fields: Vec<TokenStream> = method
            .output_params
            .iter()
            .map(|p| {
                let pname = Ident::new(&snake_case(&p.name), Span::call_site());
                let ptype_str = resolve_type_name(&p.type_ref, project);
                let ptype: TokenStream = ptype_str.parse().unwrap_or_else(|_| quote! { () });
                quote! {
                    let #pname = <#ptype as AraDeserialize>::ara_deserialize(&buf[offset..])?;
                    offset += #pname.serialized_size();
                }
            })
            .collect();

        let resp_field_names: Vec<Ident> = method
            .output_params
            .iter()
            .map(|p| Ident::new(&snake_case(&p.name), Span::call_site()))
            .collect();

        let extract_result = build_extract_result_tokens(&method.output_params);

        // Combine request + response struct defs
        let all_struct_defs = quote! {
            #req_struct_defs

            pub struct #resp_struct_name {
                #(#resp_fields)*
            }

            impl AraSerialize for #resp_struct_name {
                fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
                    #(#resp_serialize_fields)*
                    Ok(())
                }

                fn serialized_size(&self) -> usize {
                    #(#resp_size_fields)+*
                }
            }

            impl AraDeserialize for #resp_struct_name {
                fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
                    let mut offset = 0usize;
                    #(#resp_deser_fields)*
                    Ok(Self { #(#resp_field_names,)* })
                }
            }
        };

        let method_body = quote! {
            #doc
            pub async fn #method_fn_name(&self, #(#input_params,)*) -> Result<#return_type, AraComError> {
                let req = #req_struct_name { #(#req_constructor_fields)* };
                let resp: #resp_struct_name = self.base.call_method(MethodId(#method_id_lit), &req).await?;
                Ok(#extract_result)
            }
        };

        (all_struct_defs, method_body)
    }
}

fn build_return_type_tokens(
    output_params: &[crate::parser::ir::Parameter],
    project: &crate::parser::ir::ArxmlProject,
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

fn build_extract_result_tokens(output_params: &[crate::parser::ir::Parameter]) -> TokenStream {
    match output_params {
        [] => quote! { () },
        [single] => {
            let pname = Ident::new(&snake_case(&single.name), Span::call_site());
            quote! { resp.#pname }
        }
        multiple => {
            let names: Vec<TokenStream> = multiple
                .iter()
                .map(|p| {
                    let pname = Ident::new(&snake_case(&p.name), Span::call_site());
                    quote! { resp.#pname }
                })
                .collect();
            quote! { (#(#names),*) }
        }
    }
}

fn generate_event_subscribe(event: &crate::parser::ir::Event) -> TokenStream {
    let subscribe_fn = Ident::new(
        &format!("subscribe_{}", snake_case(&event.name)),
        Span::call_site(),
    );
    let event_group_id_val = event.event_group_id.unwrap_or(0);
    let event_group_id_lit = Literal::u16_unsuffixed(event_group_id_val);

    let doc = event
        .description
        .as_deref()
        .map(|d| quote! { #[doc = #d] })
        .unwrap_or_default();

    quote! {
        #doc
        pub async fn #subscribe_fn(&self) -> Result<(), AraComError> {
            self.base.transport().subscribe_event_group(
                self.base.service_id(),
                self.base.instance_id(),
                EventGroupId(#event_group_id_lit),
            ).await
        }
    }
}
