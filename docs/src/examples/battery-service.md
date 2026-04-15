# Battery Service Example

A complete end-to-end example showing two Rust processes communicating over SOME/IP with dynamic service discovery and events.

## What It Demonstrates

- SOME/IP-SD multicast discovery (offer + find)
- Request/response (`GetVoltage`)
- Fire-and-forget (`SetChargeLimit`)
- Event subscription and notification (`VoltageChanged`)
- Generated types for serialization

## Running

```bash
# Terminal 1: Start the skeleton (server)
RUST_LOG=info cargo run -p battery-service-example --bin server

# Terminal 2: Run the proxy (client)
RUST_LOG=info cargo run -p battery-service-example --bin client
```

## Server Walkthrough

The server:

1. Creates a `SomeIpTransport` with SD enabled and a service endpoint on port 30509
2. Registers a request handler that dispatches by method ID:
   - Method 1 (`GetVoltage`): deserializes `GetVoltageRequest`, computes a voltage, returns `GetVoltageResponse`
   - Method 2 (`SetChargeLimit`): deserializes `SetChargeLimitRequest`, logs the limit, returns empty
3. Offers the service via SD using `BatteryServiceSkeleton::offer()`
4. Spawns a background task that publishes `VoltageChanged` events every 2 seconds

## Client Walkthrough

The client:

1. Creates a `SomeIpTransport` with SD enabled
2. Creates a `BatteryServiceProxy` and calls `find()` to discover the server via SD multicast
3. Calls `get_voltage()` — sends a SOME/IP request, receives a typed response
4. Calls `set_charge_limit()` — fire-and-forget, no response expected
5. Subscribes to the `VoltageChanged` event group and prints incoming events

## ARXML Source

The service definition lives in `cargo-arxml/tests/fixtures/battery_service.arxml`. It defines:

- Service interface `BatteryService` (service ID `0x4010`)
- Methods: `GetVoltage` (method ID 1), `SetChargeLimit` (method ID 2)
- Event: `VoltageChanged` (event ID `0x8001`, event group 1)
- Types: `GetVoltageRequest`, `GetVoltageResponse`, `SetChargeLimitRequest`

The generated code in `examples/battery-service/src/generated/` was produced by `cargo arxml generate`.
