# Architecture

## Crate Dependency Graph

```
  ARXML files
       |
       v
+--------------+     generates     +----------+
| cargo-arxml  | ----------------> | Rust src |
|  (codegen)   |                   |  traits  |
+--------------+                   |  proxies |
  uses: autosar-data               | skeletons|
                                   +----+-----+
                                        | depends on
                                        v
                                  +----------+
                                  | ara-com  |
                                  | (traits) |
                                  +----+-----+
                                       | implemented by
                                       v
                                +--------------+
                                |ara-com-someip|
                                | (transport)  |
                                +--------------+
                                 uses: someip_parse
```

Key separation:

- **cargo-arxml** has no runtime dependency on the other crates. It generates code that `use`s `ara-com` types.
- **ara-com** defines the `Transport` trait. Backends implement it.
- **ara-com-someip** provides the SOME/IP implementation over UDP and TCP.

## cargo-arxml Pipeline

```
ARXML files
    │
    ▼
┌─────────┐    autosar-data
│  Parser  │◄── autosar-data-abstraction
└────┬─────┘
     │  Intermediate Representation (IR)
     ▼
┌───────────┐
│ Validator │  checks: duplicate IDs, missing refs, method ID conflicts
└────┬──────┘
     │  Validated IR
     ▼
┌─────────┐    quote / proc-macro2
│ Codegen │──► types.rs, traits.rs, proxy/*.rs, skeleton/*.rs
└─────────┘
```

The codegen uses `quote` and `proc-macro2` for hygienic `TokenStream` generation — not string templates. Output is formatted with `prettyplease`.

## ara-com Trait Design

The core abstraction is the `Transport` trait:

```rust,ignore
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send_request(&self, header: MessageHeader, payload: Bytes)
        -> Result<Bytes, AraComError>;
    async fn send_fire_and_forget(&self, header: MessageHeader, payload: Bytes)
        -> Result<(), AraComError>;
    async fn send_notification(&self, header: MessageHeader, payload: Bytes)
        -> Result<(), AraComError>;
    // ...
}
```

Generated proxies call `Transport` methods. Generated skeletons register handlers on a `Transport`. The transport backend (e.g., `SomeIpTransport`) handles the wire protocol.

## ara-com-someip Transport

The SOME/IP transport provides:

- **Dual UDP/TCP sockets** — payloads below `udp_threshold` go over UDP, larger ones over TCP with length-prefixed framing
- **Session tracking** — request/response correlation via session IDs
- **SOME/IP-SD** — multicast service discovery with offer/find/subscribe lifecycle and TTL tracking
- **Event-group routing** — notifications fan out only to subscribers of the matching event group
- **Instance binding** — one instance per service per transport, enforced at all entry points

## Serialization

`ara-com` defines `AraSerialize` and `AraDeserialize` traits. The SOME/IP wire format uses big-endian encoding:

- Primitives: native size, big-endian
- `String`: 32-bit length prefix (BOM) + UTF-8 bytes + null terminator
- `Vec<T>`: 32-bit length prefix + concatenated serialized elements

Generated type structs implement these traits automatically.
