# ara-com

Core traits and async abstractions for Adaptive AUTOSAR communication in Rust.

## Overview

`ara-com` defines the vocabulary types and async abstractions that all transport backends implement. It has **zero** transport dependencies — it is the stable API surface that user-generated code (from [`cargo-arxml`](https://crates.io/crates/cargo-arxml)) depends on.

## Key Types

| Module | Contents |
|--------|----------|
| `types` | Newtype wrappers for SOME/IP and AUTOSAR identifiers (`ServiceId`, `MethodId`, `EventId`, etc.) |
| `transport` | `Transport` backend trait + `AraSerialize` / `AraDeserialize` traits |
| `service` | `ServiceDefinition` marker trait + `ServiceHandle` |
| `method` | `MethodConfig` + `MethodResult` for request/response and fire-and-forget |
| `event` | `SubscriptionState`, `EventConfig`, `EventStream` |
| `field` | `FieldConfig` + getter/setter/notifier traits |
| `proxy` | `ProxyBase` + `Proxy` trait |
| `skeleton` | `SkeletonBase` + `Skeleton` trait |

## Usage

Add `ara-com` to your `Cargo.toml`:

```toml
[dependencies]
ara-com = "0.1"
```

In practice, you won't write proxy/skeleton implementations by hand. Use `cargo-arxml` to generate typed Rust code from your ARXML service definitions, and wire it to a transport backend like [`ara-com-someip`](https://crates.io/crates/ara-com-someip).

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `tokio-runtime` | yes | Enables `tokio` channel types for event streams |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

Part of the [ara-rs](https://github.com/ZiadAhmed9/ara-rs) workspace.
