use bytes::Bytes;
use std::sync::Arc;

use crate::method::{MethodConfig, MethodResult};
use crate::service::ServiceDefinition;
use crate::transport::{
    AraDeserialize, AraSerialize, MessageHeader, MessageType, ReturnCode, Transport,
};
use crate::types::*;

/// Base proxy that generated proxies wrap.
///
/// Holds transport, identity, and default method configuration. Generated
/// proxy structs embed a `ProxyBase<T>` and delegate all wire-level calls to
/// its helper methods.
pub struct ProxyBase<T: Transport> {
    transport: Arc<T>,
    service_id: ServiceId,
    instance_id: InstanceId,
    method_config: MethodConfig,
}

impl<T: Transport> ProxyBase<T> {
    /// Create a new `ProxyBase`.
    pub fn new(
        transport: Arc<T>,
        service_id: ServiceId,
        instance_id: InstanceId,
        method_config: MethodConfig,
    ) -> Self {
        Self {
            transport,
            service_id,
            instance_id,
            method_config,
        }
    }

    /// Create a new `ProxyBase` with default `MethodConfig`.
    pub fn with_defaults(
        transport: Arc<T>,
        service_id: ServiceId,
        instance_id: InstanceId,
    ) -> Self {
        Self::new(transport, service_id, instance_id, MethodConfig::default())
    }

    /// Accessor for the underlying transport (needed by generated code that
    /// calls event/field subscription methods directly).
    pub fn transport(&self) -> &Arc<T> {
        &self.transport
    }

    /// The SOME/IP service ID this proxy targets.
    pub fn service_id(&self) -> ServiceId {
        self.service_id
    }

    /// The instance ID this proxy targets.
    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }

    /// The default method call configuration (timeout, retries).
    pub fn method_config(&self) -> &MethodConfig {
        &self.method_config
    }

    /// Encode `request`, dispatch it via the transport, then decode the
    /// response payload into `Resp`.
    ///
    /// This is the primary helper used by generated proxy method stubs.
    pub async fn call_method<Req, Resp>(
        &self,
        method_id: MethodId,
        request: &Req,
    ) -> MethodResult<Resp>
    where
        Req: AraSerialize,
        Resp: AraDeserialize,
    {
        // Serialize the request
        let mut buf = Vec::with_capacity(request.serialized_size());
        request.ara_serialize(&mut buf)?;

        let header = MessageHeader {
            service_id: self.service_id,
            method_id,
            instance_id: self.instance_id,
            session_id: 0,
            message_type: MessageType::Request,
            return_code: ReturnCode::Ok,
        };

        let (_resp_header, resp_payload) = self
            .transport
            .send_request(header, Bytes::from(buf))
            .await?;

        let result = Resp::ara_deserialize(&resp_payload)?;
        Ok(result)
    }

    /// Send a fire-and-forget (no-return) method call.
    pub async fn call_fire_and_forget<Req>(
        &self,
        method_id: MethodId,
        request: &Req,
    ) -> MethodResult<()>
    where
        Req: AraSerialize,
    {
        let mut buf = Vec::with_capacity(request.serialized_size());
        request.ara_serialize(&mut buf)?;

        let header = MessageHeader {
            service_id: self.service_id,
            method_id,
            instance_id: self.instance_id,
            session_id: 0,
            message_type: MessageType::RequestNoReturn,
            return_code: ReturnCode::Ok,
        };

        self.transport
            .send_fire_and_forget(header, Bytes::from(buf))
            .await
    }
}

/// Trait that generated proxy structs implement.
///
/// Enforces that every proxy knows which service it represents and can be
/// constructed from a transport + instance ID.
pub trait Proxy: Send + Sync {
    type ServiceDef: crate::service::ServiceDefinition;
    type TransportImpl: Transport;

    fn new(transport: Arc<Self::TransportImpl>, instance_id: InstanceId) -> Self;

    fn service_id() -> ServiceId {
        Self::ServiceDef::SERVICE_ID
    }
}
