# Runtime Communication

Once you have generated code from your ARXML, you wire it to a transport backend to communicate over SOME/IP.

## Creating a Transport

```rust,ignore
use std::net::{Ipv4Addr, SocketAddrV4};
use ara_com_someip::config::{
    DiscoveryMode, EndpointConfig, ServiceConfig, SomeIpConfig, SdConfig,
};
use ara_com_someip::transport::SomeIpTransport;

let config = SomeIpConfig {
    unicast: Ipv4Addr::LOCALHOST,
    client_id: 0x0001,
    discovery_mode: DiscoveryMode::SomeIpSd,
    sd_config: SdConfig::default(),
    services: vec![ServiceConfig {
        service_id: ServiceId(0x4010),
        instance_id: InstanceId(0x0001),
        endpoint: EndpointConfig {
            udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 30509)),
            tcp: None,
            udp_threshold: 1400,
        },
        event_groups: vec![],
    }],
    remote_services: vec![],
    udp_threshold: 1400,
};

let mut transport = SomeIpTransport::new(config);
transport.bind().await?;
```

## Request/Response

The generated proxy handles serialization and transport calls:

```rust,ignore
use std::sync::Arc;

let transport = Arc::new(transport);
let proxy = BatteryServiceProxy::new(transport.clone(), InstanceId(0x0001));

// Find the service (via SD or static config)
proxy.find().await?;

// Call a method — returns a typed response
let request = GetVoltageRequest { battery_id: 1 };
let response = proxy.get_voltage(&request).await?;
println!("Voltage: {:.1}V", response.voltage);
```

## Fire-and-Forget

Methods marked as fire-and-forget in the ARXML use `send_fire_and_forget` under the hood:

```rust,ignore
let request = SetChargeLimitRequest { limit: 10.0 };
proxy.set_charge_limit(&request).await?;
// No response — returns Ok(()) on successful send
```

## Event Notifications

### Subscribing (Proxy Side)

```rust,ignore
// Subscribe to an event group
proxy.subscribe_voltage_changed().await?;

// Receive typed events as a stream
while let Some(event) = proxy.next_voltage_changed().await {
    println!("Voltage changed: {:.1}V", event.voltage);
}
```

### Publishing (Skeleton Side)

```rust,ignore
let skeleton = BatteryServiceSkeleton::new(transport.clone(), InstanceId(0x0001));
skeleton.offer().await?;

// Publish an event to all subscribers
skeleton.notify_voltage_changed(&12.6).await?;
```

## UDP/TCP Routing

The transport automatically routes based on payload size:

- Payloads smaller than `udp_threshold` (default: 1400 bytes) go over UDP
- Larger payloads go over TCP with length-prefixed framing

Configure per-service or globally via `SomeIpConfig::udp_threshold`.
