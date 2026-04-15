# vsomeip Interoperability

ara-rs includes a Docker-based interop demo that proves live compatibility with [vsomeip](https://github.com/COVESA/vsomeip), the reference C++ SOME/IP implementation used across the automotive industry.

## What It Demonstrates

A vsomeip C++ client discovers and calls `GetVoltage` on an ara-rs Rust server. Both sides use SOME/IP-SD multicast for service discovery and exchange request/response messages with identical wire encoding.

## Running the Demo

```bash
# Prerequisites: Docker Engine 20.10+ and Docker Compose v2
./interop/run.sh
```

The script builds both containers, runs the demo, and exits with code 0 on success.

## How It Works

1. The **ara-rs server** (Rust) starts a BatteryService skeleton on `172.20.0.10:30509` and offers it via SOME/IP-SD multicast
2. The **vsomeip client** (C++) discovers the service via SD, sends a `GetVoltage` request with `battery_id=1`
3. The server computes `voltage = 12.6 + battery_id * 0.1 = 12.7` and returns it as a big-endian IEEE 754 `f64`
4. The client decodes the response and validates the value

Both containers run on a shared Docker bridge network (`172.20.0.0/24`) with multicast enabled for SD.

## Wire Compatibility

The demo validates the same wire format tested in `ara-com-someip/tests/wire_compat.rs`:

- 16-byte SOME/IP header (big-endian)
- Primitives: native size, big-endian
- `f64`: 8 bytes, IEEE 754 big-endian

No custom serialization adapters are needed — the formats match out of the box.

## Files

All interop files live in the `interop/` directory at the repository root. See [`interop/README.md`](https://github.com/ZiadAhmed9/ara-rs/blob/master/interop/README.md) for the full file listing, expected output, and troubleshooting guide.
