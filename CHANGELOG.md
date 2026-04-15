# Changelog

All notable changes to ara-rs will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0] — 2026-04-15

Initial public release of the ara-rs workspace: three crates providing a cargo-native path from ARXML to running Adaptive AUTOSAR services in Rust.

### cargo-arxml

- ARXML parsing and intermediate representation extraction via `autosar-data`
- SOME/IP deployment parsing (service, method, event, and event-group IDs from ARXML)
- Validation: duplicate service/method IDs, empty services, missing type refs, invalid method IDs, case-insensitive primitive matching
- Collision-safe auto-ID assignment when deployment IDs are absent
- Rust code generation: type structs with `AraSerialize`/`AraDeserialize`, service traits, proxy stubs, skeleton stubs
- CLI commands: `validate`, `generate` (with `--strict` mode), `inspect` (JSON IR dump)
- Configuration via `arxml.toml`

### ara-com

- Transport-agnostic async traits for service-oriented communication
- `Transport` trait as the backend extension point
- `AraSerialize` / `AraDeserialize` for wire-format serialization (primitives, `String`, `Vec`)
- Method abstractions: request/response (`MethodConfig`, `MethodResult`) and fire-and-forget
- Event abstractions: `EventConfig`, `EventStream`, `SubscriptionState`
- Field abstractions: getter/setter/notifier via `FieldConfig`
- Proxy and skeleton base types (`ProxyBase`, `SkeletonBase`)
- Service lifecycle: `ServiceDefinition`, `ServiceHandle`, `ServiceState`
- Newtype wrappers for AUTOSAR identifiers (`ServiceId`, `MethodId`, `EventId`, etc.)

### ara-com-someip

- SOME/IP transport implementing `ara-com`'s `Transport` trait
- UDP and TCP socket transport with payload-size-based routing (`udp_threshold`)
- TCP length-prefixed framing for reliable large-payload delivery
- Request/response correlation with session tracking
- Fire-and-forget (`RequestNoReturn`) support
- Event notifications with `broadcast::Sender/Receiver` and backpressure handling
- SOME/IP-SD multicast service discovery: offer/find/subscribe lifecycle, TTL tracking with expiry
- Event-group-aware notification routing via `EventGroupConfig`
- One-instance-per-service-per-transport invariant enforced at all entry points
- Wire compatibility with vsomeip (byte-level validated)

### Examples

- **battery-service**: SD discovery, request/response, fire-and-forget, `VoltageChanged` event subscription
- **diagnostics-service**: 4 methods (multi-param, zero-param), nested struct responses (`DtcSnapshot`, `EcuInfo`, `DataRecord`), fire-and-forget, 2 event groups

[0.1.0]: https://github.com/ZiadAhmed9/ara-rs/releases/tag/v0.1.0
