# cargo-arxml

ARXML parser, validator, and Rust code generator for Adaptive AUTOSAR.

## Overview

`cargo-arxml` reads ARXML service interface definitions and generates idiomatic Rust code:

- **Type structs** with `AraSerialize` / `AraDeserialize` implementations
- **Service traits** with async method signatures
- **Proxy stubs** that call through `ProxyBase`
- **Skeleton stubs** that wire into `SkeletonBase`

Generated code depends on [`ara-com`](https://crates.io/crates/ara-com) and is ready to use with any `Transport` backend (e.g., [`ara-com-someip`](https://crates.io/crates/ara-com-someip)).

## Installation

```bash
cargo install cargo-arxml
```

## Usage

```bash
# Validate ARXML files
cargo arxml validate path/to/service.arxml

# Generate Rust code from ARXML
cargo arxml generate path/to/arxml/ --output-dir src/generated/

# Strict mode: error if any SOME/IP deployment IDs are missing
cargo arxml generate path/to/arxml/ --output-dir src/generated/ --strict

# Inspect extracted intermediate representation as JSON
cargo arxml inspect path/to/service.arxml
```

## Features

- Extracts SOME/IP deployment info (service/method/event IDs) from ARXML
- Validates duplicate service/method IDs, empty services, missing type refs, invalid method IDs
- Collision-safe auto-ID assignment when deployment IDs are absent
- Configuration via `arxml.toml`

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

Part of the [ara-rs](https://github.com/ZiadAhmed9/ara-rs) workspace.
