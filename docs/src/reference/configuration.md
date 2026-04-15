# Configuration

## cargo-arxml: arxml.toml

The code generator can be configured via an `arxml.toml` file in your project root.

### Input

```toml
[input]
paths = ["arxml/services/", "arxml/types/"]
exclude = ["arxml/deprecated/"]
```

| Field | Type | Description |
|-------|------|-------------|
| `paths` | `[String]` | Glob patterns or explicit paths to ARXML files/directories |
| `exclude` | `[String]` (optional) | Patterns to exclude from parsing |

### Output

```toml
[output]
dir = "src/generated"
crate_name = "my_service"
```

| Field | Type | Description |
|-------|------|-------------|
| `dir` | `String` | Directory to write generated Rust source files |
| `crate_name` | `String` (optional) | Name of the generated crate (defaults to `generated`) |

### Naming

```toml
[naming]
method_style = "snake_case"
type_style = "snake_case"
```

| Field | Type | Options | Default |
|-------|------|---------|---------|
| `method_style` | `String` (optional) | `snake_case`, `camelCase` | `snake_case` |
| `type_style` | `String` (optional) | `snake_case`, `camelCase` | `snake_case` |

## ara-com-someip: SomeIpConfig

The SOME/IP transport is configured programmatically via `SomeIpConfig`:

```rust,ignore
SomeIpConfig {
    unicast: Ipv4Addr,          // This application's address
    client_id: u16,             // Unique per ECU
    discovery_mode: DiscoveryMode, // Static or SomeIpSd
    sd_config: SdConfig,        // SD multicast settings
    services: Vec<ServiceConfig>,       // Local services to offer
    remote_services: Vec<RemoteServiceConfig>, // Static remote endpoints
    udp_threshold: usize,       // Payload size for UDP/TCP routing (default: 1400)
}
```

### Discovery Mode

| Mode | Description |
|------|-------------|
| `DiscoveryMode::Static` | Endpoints pre-configured, no multicast |
| `DiscoveryMode::SomeIpSd` | SOME/IP-SD multicast discovery |

### SD Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `multicast_group` | `Ipv4Addr` | `239.224.224.224` | SD multicast group address |
| `port` | `u16` | `30490` | SD multicast port |
| `initial_delay_min` | `u32` | `0` | Minimum delay before first offer (ms) |
| `initial_delay_max` | `u32` | `3000` | Maximum initial delay random bound (ms) |
| `repetition_base_delay` | `u32` | `10` | Base delay between offer repetitions (ms) |
| `repetition_max` | `u32` | `3` | Maximum number of offer repetitions |
| `ttl` | `u32` | `3` | Time-to-live for service announcements (seconds) |

### Endpoint Configuration

Each service or remote service has an `EndpointConfig`:

| Field | Description |
|-------|-------------|
| `udp` | UDP socket address (optional) |
| `tcp` | TCP socket address (optional) |
| `udp_threshold` | Payload size threshold for UDP/TCP routing |
