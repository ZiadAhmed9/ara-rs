# Quick Start

This page walks through the core workflow: ARXML in, Rust code out, services running.

## 1. Validate Your ARXML

Check that your ARXML files are well-formed and contain the expected service interfaces:

```bash
cargo run -p cargo-arxml -- validate path/to/service.arxml
```

The validator catches duplicate service/method IDs, missing type references, empty services, and invalid method IDs.

## 2. Inspect the Extracted IR

See what `cargo-arxml` extracts from your ARXML before generating code:

```bash
cargo run -p cargo-arxml -- inspect path/to/service.arxml
```

This dumps the intermediate representation as JSON — useful for debugging unexpected codegen output.

## 3. Generate Rust Code

Generate typed Rust source files from your ARXML definitions:

```bash
cargo run -p cargo-arxml -- generate path/to/arxml/ --output-dir src/generated/
```

This produces:

- **types.rs** — Data type structs with `AraSerialize` / `AraDeserialize` implementations
- **traits.rs** — Service traits with async method signatures
- **proxy/** — Proxy stubs that call through `ProxyBase`
- **skeleton/** — Skeleton stubs that wire into `SkeletonBase`

### Strict Mode

By default, `cargo-arxml` warns when SOME/IP deployment IDs are missing and auto-assigns them. Use `--strict` to treat missing IDs as errors:

```bash
cargo run -p cargo-arxml -- generate path/to/arxml/ --output-dir src/generated/ --strict
```

## 4. Run an Example

The fastest way to see everything working end-to-end is the battery service example:

```bash
# Terminal 1: Start the server
RUST_LOG=info cargo run -p battery-service-example --bin server

# Terminal 2: Run the client
RUST_LOG=info cargo run -p battery-service-example --bin client
```

The client discovers the service via SOME/IP-SD multicast, calls `GetVoltage` (request/response), sends `SetChargeLimit` (fire-and-forget), and subscribes to `VoltageChanged` events — all over real SOME/IP UDP frames on loopback.

See the [Battery Service](../examples/battery-service.md) and [Diagnostics Service](../examples/diagnostics-service.md) pages for detailed walkthroughs.
