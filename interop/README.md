# ara-rs / vsomeip Interop Demo

Demonstrates live interoperability between an ara-rs Rust service and a C++ vsomeip client using Docker Compose.

## What It Proves

A vsomeip C++ client calls `GetVoltage` on an ara-rs battery service server over SOME/IP. The request and response cross the language boundary with matching wire format — same service ID, method ID, and payload encoding.

## Architecture

```
┌─────────────────────┐         SOME/IP-SD          ┌─────────────────────┐
│  ara-rs server      │◄────── multicast ──────────►│  vsomeip client     │
│  (Rust)             │         discovery            │  (C++)              │
│                     │                              │                     │
│  BatteryService     │◄──── GetVoltage req ────────│  request(0x4010,    │
│  0x4010:0x0001      │                              │    0x0001, [0x01])  │
│  port 30509 UDP     │───── GetVoltage resp ──────►│                     │
│                     │      f64 = 12.7V             │  validates voltage  │
└─────────────────────┘                              └─────────────────────┘
   172.20.0.10                                          172.20.0.20
```

## Prerequisites

- Docker Engine 20.10+
- Docker Compose v2

## Running

```bash
./interop/run.sh
```

Or manually:

```bash
docker compose -f interop/docker-compose.yml up --build --abort-on-container-exit
```

## Expected Output

```
[ara-rs-server]    Starting BatteryService interop server (advertise=172.20.0.10, port=30509)...
[ara-rs-server]    BatteryService offered via SD — waiting for interop requests...
[vsomeip-client]   Service 0x4010 is AVAILABLE
[vsomeip-client]   Sending GetVoltage(battery_id=1)...
[ara-rs-server]    GetVoltage(battery_id=1) -> 12.7V
[vsomeip-client]   GetVoltage response: 12.7V
[vsomeip-client]   INTEROP SUCCESS — voltage matches expected value
```

The vsomeip client exits with code 0 on success, non-zero on failure.

## Wire Format

Both sides use identical SOME/IP encoding:

- **Header**: 16 bytes, big-endian (service ID, method ID, length, client/session IDs, protocol/interface version, message type, return code)
- **Request payload**: 1 byte (`battery_id` as `u8`)
- **Response payload**: 8 bytes (`voltage` as IEEE 754 `f64`, big-endian)

Validated byte-for-byte against the wire compatibility tests in `ara-com-someip/tests/wire_compat.rs`.

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| vsomeip client times out waiting for service | SD multicast not reaching across containers | Ensure both containers are on `someip-net` with fixed IPs |
| "Failed to init vsomeip application" | Missing or invalid JSON config | Check `VSOMEIP_CONFIGURATION` env var points to `/etc/vsomeip/vsomeip_client.json` |
| Wrong voltage value | Payload endianness mismatch | Verify both sides use big-endian IEEE 754 for `f64` |
| Container build fails on vsomeip | Network issue fetching vsomeip source | Check Docker has internet access; vsomeip is cloned from GitHub |

## Files

| File | Description |
|------|-------------|
| `docker-compose.yml` | Orchestrates both containers on a shared bridge network |
| `Dockerfile.ara-rs` | Builds the Rust interop server from the battery-service example |
| `Dockerfile.vsomeip` | Builds vsomeip from source and compiles the C++ client |
| `vsomeip-client/vsomeip_client.cpp` | C++ client source — calls GetVoltage, validates response |
| `vsomeip-client/vsomeip_client.json` | vsomeip application config with SD settings |
| `vsomeip-client/CMakeLists.txt` | CMake build for the C++ client |
| `ara-rs-server/main.rs` | Rust interop server — battery service with configurable unicast |
| `run.sh` | Convenience script — builds, runs, checks exit code |
