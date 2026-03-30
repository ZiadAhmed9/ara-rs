# ara-rs

**Rust-first ARXML codegen, typed async SOME/IP communication, and C++ interop for Adaptive AUTOSAR on embedded Linux.**

---

## What is this?

A focused, cargo-native Rust toolkit that solves the daily pain points of Adaptive AUTOSAR development:

| Crate | What it does |
|-------|-------------|
| **cargo-arxml** | Parses ARXML service definitions, validates them, and generates idiomatic Rust traits, proxies, and skeletons |
| **ara-com** | Transport-agnostic async traits for service-oriented communication (methods, events, fields) |
| **ara-com-someip** | Concrete SOME/IP backend with serialization, service discovery, and Linux socket transport |

## Why?

Teams migrating from C++ to Rust for Adaptive AUTOSAR hit the same wall: there's no open-source path from ARXML files to running Rust services. Existing tools either stop at ARXML parsing (`autosar-data`) or only handle wire-format headers (`someip_parse`).

ara-rs bridges that gap — standing on top of those crates, not reimplementing them.

## Quick Start

```bash
# Build
cargo build --workspace

# Run tests
cargo test --workspace

# Validate ARXML files
cargo run -p cargo-arxml -- validate path/to/service.arxml

# Generate Rust code from ARXML
cargo run -p cargo-arxml -- generate path/to/arxml/ --output-dir src/generated/

# Inspect extracted IR as JSON
cargo run -p cargo-arxml -- inspect path/to/service.arxml
```

## Architecture

```
  ARXML files
       │
       ▼
┌─────────────┐     generates     ┌──────────┐
│ cargo-arxml │ ─────────────────▶│ Rust src │
│  (codegen)  │                   │  traits  │
└─────────────┘                   │  proxies │
  uses: autosar-data              │ skeletons│
                                  └────┬─────┘
                                       │ depends on
                                       ▼
                                 ┌──────────┐
                                 │ ara-com  │
                                 │ (traits) │
                                 └────┬─────┘
                                      │ implemented by
                                      ▼
                               ┌──────────────┐
                               │ara-com-someip│
                               │ (transport)  │
                               └──────────────┘
                                uses: someip_parse
```

- **cargo-arxml** has no runtime dependency on the other crates — it generates code that `use`s `ara-com`
- **ara-com** defines the `Transport` trait — backends implement it
- **ara-com-someip** provides the SOME/IP implementation over UDP/TCP

## Status

Active development. The core traits, ARXML parser, serialization framework, and code generation pipeline are functional. SOME/IP transport is in progress.

Code generation (`types.rs`, `traits.rs`, `proxy.rs`, `skeleton.rs`, `tests_gen.rs`) produces structs/enums with `AraSerialize`/`AraDeserialize` impls, async service traits, and typed proxy/skeleton wrappers from ARXML input. 31 tests pass (24 `ara-com` unit tests + 7 `cargo-arxml` integration tests against `tests/fixtures/battery_service.arxml`).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
