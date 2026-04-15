# ara-rs

[![crates.io](https://img.shields.io/crates/v/ara-com.svg)](https://crates.io/crates/ara-com)
[![crates.io](https://img.shields.io/crates/v/ara-com-someip.svg)](https://crates.io/crates/ara-com-someip)
[![crates.io](https://img.shields.io/crates/v/cargo-arxml.svg)](https://crates.io/crates/cargo-arxml)
[![CI](https://github.com/ZiadAhmed9/ara-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/ZiadAhmed9/ara-rs/actions/workflows/ci.yml)
[![docs](https://img.shields.io/badge/docs-mdBook-blue)](https://ziadahmed9.github.io/ara-rs/)
[![license](https://img.shields.io/crates/l/ara-com.svg)](LICENSE-MIT)
[![downloads](https://img.shields.io/crates/d/ara-com.svg)](https://crates.io/crates/ara-com)

**Rust-first ARXML codegen, typed async SOME/IP communication, and C++ interop for Adaptive AUTOSAR on embedded Linux.**

---

## What is this?

A focused, cargo-native Rust toolkit that solves the daily pain points of Adaptive AUTOSAR development:

| Crate | What it does |
|-------|-------------|
| **cargo-arxml** | Parses ARXML service definitions, validates them, and generates idiomatic Rust traits, proxies, and skeletons |
| **ara-com** | Transport-agnostic async traits for service-oriented communication (methods, events, fields) |
| **ara-com-someip** | Concrete SOME/IP backend with UDP/TCP transport, payload-based routing, request/response correlation, service discovery, and event notifications |

## Why?

Teams migrating from C++ to Rust for Adaptive AUTOSAR hit the same wall: there's no open-source path from ARXML files to running Rust services. Existing tools either stop at ARXML parsing (`autosar-data`) or only handle wire-format headers (`someip_parse`).

ara-rs bridges that gap — standing on top of those crates, not reimplementing them.

## Quick Start

```bash
# Build
cargo build --workspace

# Run tests (125 tests across the workspace)
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

## Examples

### Battery Service Demo

A complete end-to-end example showing two Rust processes communicating over SOME/IP with dynamic service discovery and events:

```bash
# Terminal 1: Start the skeleton (server) — offers via SD, publishes events
RUST_LOG=info cargo run -p battery-service-example --bin server

# Terminal 2: Run the proxy (client) — discovers via SD, calls methods, subscribes to events
RUST_LOG=info cargo run -p battery-service-example --bin client
```

The client discovers the `BatteryService` via SOME/IP-SD multicast, calls `GetVoltage` (request/response), `SetChargeLimit` (fire-and-forget), and subscribes to `VoltageChanged` events — all over real SOME/IP UDP frames on loopback.

### Diagnostics Service Demo

A more complex example exercising capabilities beyond the battery service: 4 methods (including multi-parameter and zero-parameter), nested struct responses (`DtcSnapshot`, `EcuInfo`, `DataRecord`), fire-and-forget (`ClearDtc`), and 2 event groups.

```bash
# Terminal 1: Start the diagnostics skeleton
RUST_LOG=info cargo run -p diagnostics-service-example --bin diag_server

# Terminal 2: Run the diagnostics client
RUST_LOG=info cargo run -p diagnostics-service-example --bin diag_client
```

The client calls `ReadDtc` (nested struct response), `ClearDtc` (fire-and-forget), `ReadEcuIdentification` (no input params), `ReadDataByIdentifier` (multiple inputs), and subscribes to `DtcStatusChanged` events.

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
- **ara-com-someip** provides the SOME/IP implementation over UDP and TCP

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
| TCP transport with length-prefixed framing | Done |
| Payload-size-based UDP/TCP routing (udp_threshold) | Done |
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
| Diagnostics-service example (nested structs, multi-param, 2 event groups) | Done |
| TCP transport | Done |
| C++ interop (CXX bridge) | Done |
| Yocto meta-layer | Done |

## Test Suite

125 tests across the workspace:

- **26** ara-com unit tests (serialization, types, service state machine)
- **22** ara-com-someip unit tests (SOME/IP header, SD message format, session IDs, transport state)
- **12** UDP loopback integration tests (request/response, fire-and-forget, notifications, event channels, concurrent requests, backpressure, instance binding)
- **6** TCP integration tests (request/response, fire-and-forget, concurrent requests, large payload, error handling, UDP/TCP routing)
- **3** SD integration tests (offer/find round-trip, stop-offer, subscribe/event delivery)
- **15** wire compatibility tests (byte-level vsomeip format validation)
- **13** cargo-arxml codegen integration tests (parser, codegen, SOME/IP deployment)
- **7** diagnostics-service codegen tests (nested structs, custom type imports, multi-param methods)
- **20** cargo-arxml validator tests (missing type refs, invalid/duplicate method IDs, auto-ID collision avoidance, case-insensitive primitives)

All tests pass with zero clippy warnings.

## Roadmap

ara-rs v0.1.0 is published. The next sprints focus on closing feature gaps and growing adoption:

| Sprint | Topic | Status |
|--------|-------|--------|
| 11 | Field support (getter/setter/notifier codegen) | Next up |
| 12 | Error recovery for malformed ARXML | Planned |
| 13 | `cargo arxml generate --cxx` (auto CXX bridge) | Planned |
| 14 | Multi-binding (UDP + TCP simultaneous) | Planned |
| 15 | Community launch (blog, Awesome lists, outreach) | Planned |

See [Documentation/phases.md](Documentation/phases.md) for the full roadmap and sprint definitions.

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions, code style, and the PR process. Issues labeled [`good first issue`](https://github.com/ZiadAhmed9/ara-rs/labels/good%20first%20issue) are scoped for newcomers.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
