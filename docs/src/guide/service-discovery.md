# Service Discovery

ara-com-someip supports two discovery modes: static configuration and SOME/IP-SD dynamic discovery.

## Static Discovery

In static mode, endpoints are pre-configured — no multicast traffic is needed:

```rust,ignore
use ara_com_someip::config::{DiscoveryMode, RemoteServiceConfig, EndpointConfig};

let config = SomeIpConfig {
    discovery_mode: DiscoveryMode::Static,
    remote_services: vec![RemoteServiceConfig {
        service_id: ServiceId(0x4010),
        instance_id: InstanceId(0x0001),
        endpoint: EndpointConfig {
            udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 30509)),
            tcp: None,
            udp_threshold: 1400,
        },
    }],
    // ...
};
```

Use static discovery for simple setups, testing, or environments where multicast is not available.

## SOME/IP-SD Dynamic Discovery

Set `discovery_mode: DiscoveryMode::SomeIpSd` to enable the full SD state machine:

```rust,ignore
let config = SomeIpConfig {
    discovery_mode: DiscoveryMode::SomeIpSd,
    sd_config: SdConfig::default(), // 239.224.224.224:30490
    // ...
};
```

### SD Lifecycle

1. **Server** calls `skeleton.offer()` — sends OfferService entries via SD multicast
2. **Client** calls `proxy.find()` — sends FindService entries and waits for matching offers
3. **Client** subscribes to event groups — sends SubscribeEventgroup entries
4. **Server** publishes events — delivered only to subscribed clients

### TTL Tracking

Discovered services have a time-to-live (TTL). When a TTL expires without renewal, the service is considered unavailable. The SD state machine handles TTL refresh automatically during the offer cycle.

### SD Configuration

```rust,ignore
use ara_com_someip::config::SdConfig;

let sd_config = SdConfig {
    multicast_addr: SocketAddrV4::new(
        Ipv4Addr::new(239, 224, 224, 224),
        30490,
    ),
    offer_interval: std::time::Duration::from_secs(1),
    ttl: 3, // seconds
    ..SdConfig::default()
};
```

### Event Groups

Events are grouped into event groups for subscription granularity. Configure them per-service:

```rust,ignore
use ara_com_someip::config::EventGroupConfig;

let service_config = ServiceConfig {
    // ...
    event_groups: vec![EventGroupConfig {
        event_group_id: EventGroupId(1),
        multicast: None,
        events: vec![EventId(0x8001)],
    }],
};
```

When a client subscribes to event group 1, it receives only events with IDs listed in that group's configuration. Notifications fan out only to matching subscribers.
