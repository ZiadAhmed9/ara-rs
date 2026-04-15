# Introduction

**ara-rs** is a cargo-native Rust toolkit for Adaptive AUTOSAR on embedded Linux. It bridges the gap between ARXML service definitions and running Rust services communicating over SOME/IP.

## The Problem

Teams migrating from C++ to Rust for Adaptive AUTOSAR hit the same wall: there is no open-source path from ARXML files to running Rust services. Existing tools either stop at ARXML parsing ([autosar-data](https://crates.io/crates/autosar-data)) or only handle wire-format headers ([someip_parse](https://crates.io/crates/someip_parse)).

ara-rs bridges that gap — standing on top of those crates, not reimplementing them.

## Three Crates

| Crate | What it does |
|-------|-------------|
| **cargo-arxml** | Parses ARXML service definitions, validates them, and generates idiomatic Rust traits, proxies, and skeletons |
| **ara-com** | Transport-agnostic async traits for service-oriented communication (methods, events, fields) |
| **ara-com-someip** | SOME/IP backend with UDP/TCP transport, service discovery, and event notifications |

## What You Can Do Today

- Parse and validate ARXML service interface definitions
- Generate typed Rust code (structs, traits, proxies, skeletons) from ARXML
- Run services that communicate over SOME/IP with request/response, fire-and-forget, and event notifications
- Discover services dynamically using SOME/IP-SD multicast
- Route payloads between UDP and TCP based on size

## Next Steps

- [Install ara-rs](./getting-started/installation.md) to get started
- [Run the quick start](./getting-started/quick-start.md) to see it in action
- [Read the architecture overview](./guide/architecture.md) to understand how the pieces fit together
