//! SOME/IP proxy stub.
//!
//! `SomeIpProxy` is the runtime object that generated proxy code uses to make
//! method calls, subscribe to events, and get/set fields over SOME/IP.
//! The full implementation is in Week 5–7; this stub defines the type and its
//! constructor so that generated code can reference it today.

use std::sync::Arc;

use ara_com::types::{InstanceId, ServiceId};

use crate::transport::SomeIpTransport;

/// Concrete SOME/IP proxy that generated proxies use.
///
/// Holds a shared reference to the transport so multiple proxy instances for
/// different service instances can share a single socket pair.
#[allow(dead_code)]
pub struct SomeIpProxy {
    transport: Arc<SomeIpTransport>,
    service_id: ServiceId,
    instance_id: InstanceId,
}

impl SomeIpProxy {
    /// Create a new proxy for the given service/instance pair.
    pub fn new(
        transport: Arc<SomeIpTransport>,
        service_id: ServiceId,
        instance_id: InstanceId,
    ) -> Self {
        Self {
            transport,
            service_id,
            instance_id,
        }
    }

    /// The SOME/IP service ID this proxy targets.
    pub fn service_id(&self) -> ServiceId {
        self.service_id
    }

    /// The instance ID this proxy targets.
    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }
}
