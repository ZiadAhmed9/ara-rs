//! SOME/IP skeleton stub.
//!
//! `SomeIpSkeleton` is the runtime object that generated skeleton code uses to
//! offer a service, receive method calls, and publish events over SOME/IP.
//! The full implementation is in Week 6–7; this stub defines the type and its
//! constructor so that generated code can reference it today.

use std::sync::Arc;

use ara_com::types::{InstanceId, MajorVersion, MinorVersion, ServiceId};

use crate::transport::SomeIpTransport;

/// Concrete SOME/IP skeleton that generated skeletons use.
///
/// Holds a shared reference to the transport and the identity of the service
/// instance it hosts. The application's handler functions are registered via
/// `Transport::register_request_handler` once the receive loop is running.
#[allow(dead_code)]
pub struct SomeIpSkeleton {
    transport: Arc<SomeIpTransport>,
    service_id: ServiceId,
    instance_id: InstanceId,
    major_version: MajorVersion,
    minor_version: MinorVersion,
}

impl SomeIpSkeleton {
    /// Create a new skeleton for the given service/instance.
    pub fn new(
        transport: Arc<SomeIpTransport>,
        service_id: ServiceId,
        instance_id: InstanceId,
        major_version: MajorVersion,
        minor_version: MinorVersion,
    ) -> Self {
        Self {
            transport,
            service_id,
            instance_id,
            major_version,
            minor_version,
        }
    }

    /// The SOME/IP service ID this skeleton hosts.
    pub fn service_id(&self) -> ServiceId {
        self.service_id
    }

    /// The instance ID this skeleton hosts.
    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }
}
