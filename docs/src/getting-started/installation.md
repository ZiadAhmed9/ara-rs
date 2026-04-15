# Installation

## Prerequisites

- Rust 1.75+ (2021 edition)
- Linux (for SOME/IP socket communication)

## From Source (Development)

Clone the repository and build the workspace:

```bash
git clone https://github.com/ZiadAhmed9/ara-rs.git
cd ara-rs
cargo build --workspace
```

Run the test suite to verify everything works:

```bash
cargo test --workspace
```

## Installing cargo-arxml

The code generator installs as a cargo subcommand:

```bash
cargo install cargo-arxml
```

After installation, use it as `cargo arxml <command>`.

## Adding ara-com to Your Project

For library usage, add the crates to your `Cargo.toml`:

```toml
[dependencies]
ara-com = "0.1"
ara-com-someip = "0.1"   # if using SOME/IP transport
```

`ara-com` is transport-agnostic. You only need `ara-com-someip` if you want the SOME/IP backend.

## Feature Flags

### ara-com

| Flag | Default | Description |
|------|---------|-------------|
| `tokio-runtime` | yes | Enables `tokio` channel types for event streams |

### ara-com-someip

No optional features — `tokio` is always required for async socket I/O.
