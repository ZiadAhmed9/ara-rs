# CXX Bridge

ara-rs includes a CXX bridge example that proves C++ code can call into Rust SOME/IP proxy methods without any vsomeip dependency on the C++ side.

## What It Demonstrates

A C++ function (`run_cxx_client`) connects to a BatteryService server through the Rust `ara-com-someip` transport, calls `GetVoltage` for multiple battery IDs, and validates the responses — all through the `#[cxx::bridge]` boundary. The Rust binary entry point calls into C++, which calls back into Rust via the bridge, proving the full C++ -> Rust path end to end.

## Architecture

```
┌──────────────┐                    ┌─────────────────────┐
│  main.rs     │ ── calls ──►      │  main.cpp           │
│  (Rust entry │    run_cxx_client  │  (C++ logic)        │
│   point)     │                    │                     │
└──────────────┘                    │  connect_battery_   │
                                    │    service()  ──────┼──► Rust bridge
                                    │  get_voltage() ─────┼──► Rust bridge
                                    └─────────────────────┘
                                               │
                                          SOME/IP UDP
                                               │
                                               ▼
                                        ┌──────────────┐
                                        │ ara-rs server │
                                        └──────────────┘
```

## Bridge API

The bridge exposes Rust functions to C++ and a C++ function to Rust:

```cpp
namespace ara_rs {
    // Rust -> C++ (extern "C++")
    // Entry point: connects and calls GetVoltage, returns 0 on success
    int32_t run_cxx_client(rust::Str host, uint16_t port);

    // C++ -> Rust (extern "Rust")
    rust::Box<BatteryClient> connect_battery_service(
        rust::Str host, uint16_t port);
    double get_voltage(
        const BatteryClient& client, uint8_t battery_id);
}
```

Both Rust functions throw `rust::Error` on failure (transport errors, parse errors, timeouts).

## Running

Start the battery service server, then run the bridge example:

```bash
# Terminal 1: Start the server
RUST_LOG=info cargo run -p battery-service-example --bin server

# Terminal 2: Run the CXX bridge example
cargo run -p cxx-bridge-example
```

Expected output:

```
[cxx-client] C++ calling into Rust bridge...
[cxx-client] Connecting to BatteryService at 127.0.0.1:30509
[cxx-client] GetVoltage(battery_id=0) -> 12.6V
[cxx-client] GetVoltage(battery_id=1) -> 12.7V
[cxx-client] GetVoltage(battery_id=2) -> 12.8V
[cxx-client] GetVoltage(battery_id=3) -> 12.9V
[cxx-client] CXX BRIDGE SUCCESS
```

## How It Works

1. `main.rs` (Rust) initializes tracing and calls `ffi::run_cxx_client()` — crossing into C++
2. `main.cpp` (C++) calls `connect_battery_service()` — crossing back into Rust, which creates a tokio runtime, configures `SomeIpTransport` with static discovery, and returns an opaque `BatteryClient`
3. C++ calls `get_voltage()` for each battery ID — each call crosses into Rust, which runs `runtime.block_on(proxy.get_voltage(id))` over SOME/IP
4. Errors propagate as C++ exceptions (`rust::Error`)

## Limitations

This is an intentionally narrow bridge proving the approach:

- **One direction only**: C++ calls Rust. Rust-to-C++ callbacks are not bridged.
- **One service**: Only BatteryService `GetVoltage` is exposed. Extending to other methods follows the same pattern.
- **Synchronous only**: Each `get_voltage` call blocks until the SOME/IP response arrives. For async C++ workflows, a future-based wrapper would be needed.
- **Static discovery**: The bridge uses pre-configured endpoints, not SOME/IP-SD.

## Files

All bridge files live in `examples/cxx-bridge/`:

| File | Description |
|------|-------------|
| `Cargo.toml` | Crate with `cxx` and `cxx-build` dependencies |
| `build.rs` | Compiles C++ source and header via `cxx_build` |
| `src/lib.rs` | `#[cxx::bridge]` module with Rust and C++ extern blocks |
| `src/cxx_client.h` | C++ header declaring `run_cxx_client` |
| `src/main.cpp` | C++ implementation — calls Rust bridge functions |
| `src/main.rs` | Rust entry point — calls into C++ `run_cxx_client` |
