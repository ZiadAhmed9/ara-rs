use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;

use crate::error::CargoArxmlError;
use crate::parser::ir::ServiceInterface;

use super::snake_case;

pub fn generate_skeleton(svc: &ServiceInterface) -> Result<String, CargoArxmlError> {
    let struct_name_str = format!("{}Skeleton", svc.short_name);
    let struct_name = Ident::new(&struct_name_str, Span::call_site());

    let doc = svc
        .description
        .as_deref()
        .map(|d| quote! { #[doc = #d] })
        .unwrap_or_default();

    let service_id_val = svc.service_id.unwrap_or(0);
    let service_id_lit = Literal::u16_unsuffixed(service_id_val);

    let major_lit = Literal::u8_unsuffixed(svc.major_version);
    let minor_lit = Literal::u32_unsuffixed(svc.minor_version);

    let event_impls: Vec<TokenStream> = svc.events.iter().map(generate_event_notify).collect();

    let tokens = quote! {
        use std::sync::Arc;
        use ara_com::skeleton::SkeletonBase;
        use ara_com::transport::Transport;
        use ara_com::types::{InstanceId, ServiceId, MajorVersion, MinorVersion};
        use ara_com::error::AraComError;

        #doc
        pub struct #struct_name<T: Transport> {
            pub base: SkeletonBase<T>,
        }

        impl<T: Transport> #struct_name<T> {
            pub fn new(transport: Arc<T>, instance_id: InstanceId) -> Self {
                Self {
                    base: SkeletonBase::new(transport, ServiceId(#service_id_lit), instance_id),
                }
            }

            pub async fn offer(&self) -> Result<(), AraComError> {
                self.base.offer(MajorVersion(#major_lit), MinorVersion(#minor_lit)).await
            }

            pub async fn stop_offer(&self) -> Result<(), AraComError> {
                self.base.stop_offer().await
            }

            pub fn transport(&self) -> &Arc<T> {
                self.base.transport()
            }

            #(#event_impls)*
        }
    };

    let file: syn::File = syn::parse2(tokens).map_err(|e| CargoArxmlError::CodeGen {
        message: format!(
            "failed to parse generated skeleton for '{}': {e}",
            svc.short_name
        ),
    })?;

    Ok(prettyplease::unparse(&file))
}

fn generate_event_notify(event: &crate::parser::ir::Event) -> TokenStream {
    let notify_fn = Ident::new(
        &format!("notify_{}", snake_case(&event.name)),
        Span::call_site(),
    );
    let doc = event
        .description
        .as_deref()
        .map(|d| quote! { #[doc = #d] })
        .unwrap_or_default();

    quote! {
        #doc
        pub async fn #notify_fn(&self) -> Result<(), AraComError> {
            let _ = &self.base;
            Ok(())
        }
    }
}
