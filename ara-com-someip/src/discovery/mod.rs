//! SOME/IP Service Discovery (SOME/IP-SD) stub.
//!
//! The full SD state machine (initial delay, repetition phase, main phase)
//! is implemented in Week 7. This module defines the key data structures and
//! the `ServiceDiscovery` handle that the transport layer will drive.

use std::collections::HashMap;

use ara_com::types::{InstanceId, MajorVersion, MinorVersion, ServiceId};

use crate::config::SdConfig;

/// SOME/IP Service Discovery handler.
///
/// Tracks which services are locally offered and which remote services have
/// been discovered. Will be driven by a background tokio task in Week 7.
pub struct ServiceDiscovery {
    config: SdConfig,
    /// Locally offered services: (service_id, instance_id) → metadata
    offered_services: HashMap<(ServiceId, InstanceId), OfferedService>,
    /// Remotely discovered services: (service_id, instance_id) → metadata
    found_services: HashMap<(ServiceId, InstanceId), FoundService>,
}

impl ServiceDiscovery {
    /// Create a new `ServiceDiscovery` with the given SD configuration.
    pub fn new(config: SdConfig) -> Self {
        Self {
            config,
            offered_services: HashMap::new(),
            found_services: HashMap::new(),
        }
    }
}

/// Metadata for a locally offered service instance.
#[derive(Debug, Clone)]
pub struct OfferedService {
    pub service_id: ServiceId,
    pub instance_id: InstanceId,
    pub major_version: MajorVersion,
    pub minor_version: MinorVersion,
    // TODO (Week 7): udp_endpoint: Option<SocketAddrV4>
    // TODO (Week 7): tcp_endpoint: Option<SocketAddrV4>
}

/// Metadata for a remotely discovered service instance.
#[derive(Debug, Clone)]
pub struct FoundService {
    pub service_id: ServiceId,
    pub instance_id: InstanceId,
    pub major_version: MajorVersion,
    pub minor_version: MinorVersion,
    // TODO (Week 7): udp_endpoint: Option<SocketAddrV4>
    // TODO (Week 7): tcp_endpoint: Option<SocketAddrV4>
    // TODO (Week 7): ttl_expires_at: Instant
}
