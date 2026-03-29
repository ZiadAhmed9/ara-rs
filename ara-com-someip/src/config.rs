use std::net::{Ipv4Addr, SocketAddrV4};

use ara_com::types::{EventGroupId, EventId, InstanceId, ServiceId};

/// Top-level SOME/IP transport configuration
#[derive(Debug, Clone)]
pub struct SomeIpConfig {
    /// Unicast address for this application
    pub unicast: Ipv4Addr,
    /// Service discovery configuration
    pub sd_config: SdConfig,
    /// Per-service endpoint configurations
    pub services: Vec<ServiceConfig>,
}

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

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub service_id: ServiceId,
    pub instance_id: InstanceId,
    pub endpoint: EndpointConfig,
    pub event_groups: Vec<EventGroupConfig>,
}

#[derive(Debug, Clone)]
pub struct EndpointConfig {
    /// UDP endpoint address
    pub udp: Option<SocketAddrV4>,
    /// TCP endpoint address
    pub tcp: Option<SocketAddrV4>,
    /// Use UDP for messages smaller than this, TCP for larger
    pub udp_threshold: usize,
}

#[derive(Debug, Clone)]
pub struct EventGroupConfig {
    pub event_group_id: EventGroupId,
    pub multicast: Option<SocketAddrV4>,
    pub events: Vec<EventId>,
}
