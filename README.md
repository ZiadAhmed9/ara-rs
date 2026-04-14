# ara-rs

**Rust-first ARXML codegen, typed async SOME/IP communication, and C++ interop for Adaptive AUTOSAR on embedded Linux.**

---

## What is this?

A focused, cargo-native Rust toolkit that solves the daily pain points of Adaptive AUTOSAR development:

| Crate | What it does |
|-------|-------------|
| **cargo-arxml** | Parses ARXML service definitions, validates them, and generates idiomatic Rust traits, proxies, and skeletons |
| **ara-com** | Transport-agnostic async traits for service-oriented communication (methods, events, fields) |
| **ara-com-someip** | Concrete SOME/IP backend with UDP transport, request/response correlation, service discovery, and event notifications |

## Why?

Teams migrating from C++ to Rust for Adaptive AUTOSAR hit the same wall: there's no open-source path from ARXML files to running Rust services. Existing tools either stop at ARXML parsing (`autosar-data`) or only handle wire-format headers (`someip_parse`).

ara-rs bridges that gap — standing on top of those crates, not reimplementing them.

## Quick Start

```bash
# Build
cargo build --workspace

# Run tests (111 tests across the workspace)
cargo test --workspace

# Validate ARXML files
cargo run -p cargo-arxml -- validate path/to/service.arxml

# Generate Rust code from ARXML (warns if SOME/IP IDs are missing)
cargo run -p cargo-arxml -- generate path/to/arxml/ --output-dir src/generated/

# Strict mode: error if any SOME/IP IDs are not in the ARXML deployment
cargo run -p cargo-arxml -- generate path/to/arxml/ --output-dir src/generated/ --strict

# Inspect extracted IR as JSON
cargo run -p cargo-arxml -- inspect path/to/service.arxml
```

## Battery Service Demo

A complete end-to-end example showing two Rust processes communicating over SOME/IP with dynamic service discovery and events:

```bash
# Terminal 1: Start the skeleton (server) — offers via SD, publishes events
RUST_LOG=info cargo run -p battery-service-example --bin server

# Terminal 2: Run the proxy (client) — discovers via SD, calls methods, subscribes to events
RUST_LOG=info cargo run -p battery-service-example --bin client
```

The client discovers the `BatteryService` via SOME/IP-SD multicast, calls `GetVoltage` (request/response), `SetChargeLimit` (fire-and-forget), and subscribes to `VoltageChanged` events — all over real SOME/IP UDP frames on loopback.

## Architecture

```
  ARXML files
       |
       v
+--------------+     generates     +----------+
| cargo-arxml  | -----------------> | Rust src |
|  (codegen)   |                   |  traits  |
+--------------+                   |  proxies |
  uses: autosar-data              | skeletons|
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

- **cargo-arxml** has no runtime dependency on the other crates — it generates code that `use`s `ara-com`
- **ara-com** defines the `Transport` trait — backends implement it
- **ara-com-someip** provides the SOME/IP implementation over UDP

## What Works

| Feature | Status |
|---------|--------|
| ARXML parsing + IR extraction | Done |
| SOME/IP deployment parsing (service/method/event IDs from ARXML) | Done |
| Validation (duplicate service/method IDs, empty services, missing type refs, invalid method IDs) | Done |
| Code generation (types, traits, proxy, skeleton) | Done |
| Serialization (primitives, String, Vec) — SOME/IP wire-compatible | Done |
| SOME/IP header encode/decode (cross-validated with `someip_parse`) | Done |
| UDP transport with background receive loop | Done |
| Request/response correlation with session tracking | Done |
| Fire-and-forget (RequestNoReturn) | Done |
| Event notifications (send + subscribe + typed event streams) | Done |
| Static service discovery (pre-configured endpoints) | Done |
| SOME/IP-SD message format (Find/Offer/Subscribe entries) | Done |
| SOME/IP-SD multicast state machine (offer/find/subscribe lifecycle) | Done |
| Event-group-aware notification routing | Done |
| TTL tracking with expiry on discovered services | Done |
| One-instance-per-service-per-transport invariant (wire-format safety) | Done |
| Wire compatibility tests (byte-level vsomeip format validation) | Done |
| Battery-service end-to-end example (SD discovery + events) | Done |
| TCP transport | Planned |
| C++ interop (CXX bridge generation) | Planned |
| Yocto meta-layer | Planned |

## Test Suite

111 tests across the workspace:

- **26** ara-com unit tests (serialization, types, service state machine)
- **22** ara-com-someip unit tests (SOME/IP header, SD message format, session IDs, transport state)
- **12** loopback integration tests (request/response, fire-and-forget, notifications, event channels, concurrent requests, backpressure, instance binding)
- **3** SD integration tests (offer/find round-trip, stop-offer, subscribe/event delivery)
- **15** wire compatibility tests (byte-level vsomeip format validation)
- **13** cargo-arxml codegen integration tests (parser, codegen, SOME/IP deployment)
- **20** cargo-arxml validator tests (missing type refs, invalid/duplicate method IDs, auto-ID collision avoidance, case-insensitive primitives)

All tests pass with zero clippy warnings.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
