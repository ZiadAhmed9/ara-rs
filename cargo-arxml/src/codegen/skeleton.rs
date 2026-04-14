use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;

use crate::error::CargoArxmlError;
use crate::parser::ir::{ArxmlProject, ServiceInterface};

use super::snake_case;

pub fn generate_skeleton(
    svc: &ServiceInterface,
    _project: &ArxmlProject,
) -> Result<String, CargoArxmlError> {
    let struct_name_str = format!("{}Skeleton", svc.short_name);
    let struct_name = Ident::new(&struct_name_str, Span::call_site());

    let doc = svc
        .description
        .as_deref()
        .map(|d| quote! { #[doc = #d] })
        .unwrap_or_default();

    let service_id_val = svc.service_id.expect("ID must be set after assign_default_ids");
    let service_id_lit = Literal::u16_unsuffixed(service_id_val);

    let major_lit = Literal::u8_unsuffixed(svc.major_version);
    let minor_lit = Literal::u32_unsuffixed(svc.minor_version);

    let event_impls: Vec<TokenStream> = svc
        .events
        .iter()
        .map(|e| generate_event_notify(e, svc))
        .collect();

    let tokens = quote! {
        use std::sync::Arc;
        use ara_com::skeleton::SkeletonBase;
        use ara_com::transport::{Transport, AraSerialize, MessageHeader, MessageType, ReturnCode};
        use ara_com::types::{InstanceId, MethodId, ServiceId, MajorVersion, MinorVersion};
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

fn generate_event_notify(event: &crate::parser::ir::Event, svc: &ServiceInterface) -> TokenStream {
    let notify_fn = Ident::new(
        &format!("notify_{}", snake_case(&event.name)),
        Span::call_site(),
    );
    let doc = event
        .description
        .as_deref()
        .map(|d| quote! { #[doc = #d] })
        .unwrap_or_default();

    let event_id_val = event.event_id.expect("event ID must be set after assign_default_ids");
    let event_id_lit = Literal::u16_unsuffixed(event_id_val);
    let service_id_val = svc.service_id.expect("ID must be set after assign_default_ids");
    let service_id_lit = Literal::u16_unsuffixed(service_id_val);

    quote! {
        #doc
        pub async fn #notify_fn(&self, payload: &impl AraSerialize) -> Result<(), AraComError> {
            let mut buf = Vec::new();
            payload.ara_serialize(&mut buf)?;
            let header = MessageHeader {
                service_id: ServiceId(#service_id_lit),
                method_id: MethodId(#event_id_lit),
                instance_id: self.base.instance_id(),
                session_id: 0,
                message_type: MessageType::Notification,
                return_code: ReturnCode::Ok,
            };
            self.base.transport().send_notification(header, bytes::Bytes::from(buf)).await
        }
    }
}
