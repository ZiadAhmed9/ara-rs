//! Configuration types for the SOME/IP transport and service discovery.

use std::net::{Ipv4Addr, SocketAddrV4};

use ara_com::types::{EventGroupId, EventId, InstanceId, ServiceId};

/// How the transport discovers remote service endpoints.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum DiscoveryMode {
    /// Endpoints are pre-configured — no SD multicast.
    #[default]
    Static,
    /// Use SOME/IP-SD for dynamic service discovery.
    SomeIpSd,
}

/// Top-level SOME/IP transport configuration
#[derive(Debug, Clone)]
pub struct SomeIpConfig {
    /// Unicast address for this application
    pub unicast: Ipv4Addr,
    /// Client ID for this application (unique per ECU)
    pub client_id: u16,
    /// Discovery mode: static endpoints or SOME/IP-SD
    pub discovery_mode: DiscoveryMode,
    /// Service discovery configuration (used when discovery_mode == SomeIpSd)
    pub sd_config: SdConfig,
    /// Local service endpoint configurations
    pub services: Vec<ServiceConfig>,
    /// Remote service endpoints for static discovery mode
    pub remote_services: Vec<RemoteServiceConfig>,
}

/// Pre-configured remote service endpoint (for static discovery mode).
#[derive(Debug, Clone)]
pub struct RemoteServiceConfig {
    pub service_id: ServiceId,
    pub instance_id: InstanceId,
    pub endpoint: EndpointConfig,
}

/// SOME/IP-SD multicast configuration and timing parameters.
#[derive(Debug, Clone)]
pub struct SdConfig {
    /// SD multicast group (default 239.224.224.224)
    pub multicast_group: Ipv4Addr,
    /// SD port (default 30490)
    pub port: u16,
    /// Initial delay before first offer (ms)
    pub initial_delay_min: u32,
    pub initial_delay_max: u32,
    /// Repetition base delay (ms)
    pub repetition_base_delay: u32,
    /// Max repetitions
    pub repetition_max: u32,
    /// TTL for offered services (seconds)
    pub ttl: u32,
}

impl Default for SdConfig {
    fn default() -> Self {
        Self {
            multicast_group: Ipv4Addr::new(239, 224, 224, 224),
            port: 30490,
            initial_delay_min: 0,
            initial_delay_max: 3000,
            repetition_base_delay: 10,
            repetition_max: 3,
            ttl: 3,
        }
    }
}

/// Configuration for a locally hosted service instance.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// The SOME/IP service ID.
    pub service_id: ServiceId,
    /// The instance ID.
    pub instance_id: InstanceId,
    /// Network endpoint (UDP/TCP addresses).
    pub endpoint: EndpointConfig,
    /// Event groups hosted by this service.
    pub event_groups: Vec<EventGroupConfig>,
}

/// Network endpoint configuration for a service.
#[derive(Debug, Clone)]
pub struct EndpointConfig {
    /// UDP endpoint address.
    pub udp: Option<SocketAddrV4>,
    /// TCP endpoint address (not yet implemented).
    pub tcp: Option<SocketAddrV4>,
    /// Payload size threshold: messages smaller than this use UDP,
    /// larger ones would use TCP (when available).
    pub udp_threshold: usize,
}

/// Configuration for an event group within a service.
#[derive(Debug, Clone)]
pub struct EventGroupConfig {
    /// The event group identifier.
    pub event_group_id: EventGroupId,
    /// Optional multicast address for event delivery.
    pub multicast: Option<SocketAddrV4>,
    /// Event IDs belonging to this group.
    pub events: Vec<EventId>,
}
