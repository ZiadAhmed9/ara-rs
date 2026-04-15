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

## Service Discovery

Before calling methods, discover the remote service. Discovery runs on the transport, not the proxy:

```rust,ignore
use std::sync::Arc;
use ara_com::types::{MajorVersion, MinorVersion};

// Subscribe to event notifications BEFORE discovery so the channel
// is ready when events start arriving.
let mut event_rx = transport.subscribe_notifications(
    ServiceId(0x4010), InstanceId(0x0001), MethodId(0x8001), 64,
)?;

let transport = Arc::new(transport);

// Find the service via SD multicast
let found = transport
    .find_service(ServiceId(0x4010), InstanceId(0x0001), MajorVersion(1), MinorVersion(0))
    .await?;
```

## Request/Response

The generated proxy exposes scalar-argument methods that handle serialization internally:

```rust,ignore
let proxy = BatteryServiceProxy::new(transport.clone(), InstanceId(0x0001));

// Call a method — returns the typed scalar response
let voltage = proxy.get_voltage(1u8).await?;
println!("Voltage: {voltage:.1}V");
```

## Fire-and-Forget

Methods marked as fire-and-forget in the ARXML use `send_fire_and_forget` under the hood:

```rust,ignore
proxy.set_charge_limit(32.0).await?;
// No response — returns Ok(()) on successful send
```

## Event Notifications

### Subscribing (Proxy Side)

The generated proxy has a `subscribe_<event>()` method that subscribes to the event group. Event payloads arrive through the transport's notification channel:

```rust,ignore
// Subscribe to the event group via the generated proxy method
proxy.subscribe_voltage_changed().await?;

// Receive raw event payloads from the transport channel
while let Ok(payload) = event_rx.recv().await {
    let voltage = f64::ara_deserialize(&payload)?;
    println!("Voltage changed: {voltage:.1}V");
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
