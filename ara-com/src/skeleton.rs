use async_trait::async_trait;
use std::sync::Arc;

use crate::error::AraComError;
use crate::service::ServiceDefinition;
use crate::transport::Transport;
use crate::types::*;

/// Base skeleton that generated skeletons wrap.
///
/// Holds the transport and service identity. Generated skeleton structs embed
/// a `SkeletonBase<T>` and call `offer` / `stop_offer` on it to register the
/// service instance with the transport layer.
pub struct SkeletonBase<T: Transport> {
    transport: Arc<T>,
    service_id: ServiceId,
    instance_id: InstanceId,
}

impl<T: Transport> SkeletonBase<T> {
    /// Create a new `SkeletonBase`.
    pub fn new(transport: Arc<T>, service_id: ServiceId, instance_id: InstanceId) -> Self {
        Self {
            transport,
            service_id,
            instance_id,
        }
    }

    pub fn transport(&self) -> &Arc<T> {
        &self.transport
    }

    pub fn service_id(&self) -> ServiceId {
        self.service_id
    }

    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }

    /// Advertise this service instance so that remote proxies can find it.
    pub async fn offer(
        &self,
        major_version: MajorVersion,
        minor_version: MinorVersion,
    ) -> Result<(), AraComError> {
        self.transport
            .offer_service(
                self.service_id,
                self.instance_id,
                major_version,
                minor_version,
            )
            .await
    }

    /// Withdraw the service advertisement.
    pub async fn stop_offer(&self) -> Result<(), AraComError> {
        self.transport
            .stop_offer_service(self.service_id, self.instance_id)
            .await
    }
}

/// Trait that generated skeleton structs implement.
///
/// Generated skeletons must also expose `offer` / `stop_offer` and carry a
/// reference to their service definition for type-checked version constants.
#[async_trait]
pub trait Skeleton: Send + Sync {
    type ServiceDef: ServiceDefinition;
    type TransportImpl: Transport;

    fn new(transport: Arc<Self::TransportImpl>, instance_id: InstanceId) -> Self;

    fn service_id() -> ServiceId {
        Self::ServiceDef::SERVICE_ID
    }

    async fn offer(&self) -> Result<(), AraComError>;
    async fn stop_offer(&self) -> Result<(), AraComError>;
}
