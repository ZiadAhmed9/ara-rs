# Diagnostics Service Example

A more complex example exercising capabilities beyond the battery service.

## What It Demonstrates

- Multiple methods (4 total) including multi-parameter and zero-parameter variants
- Nested struct responses (`DtcSnapshot`, `EcuInfo`, `DataRecord`)
- Fire-and-forget (`ClearDtc`)
- Multiple event groups (2)
- Richer type generation from ARXML

## Running

```bash
# Terminal 1: Start the diagnostics skeleton
RUST_LOG=info cargo run -p diagnostics-service-example --bin diag_server

# Terminal 2: Run the diagnostics client
RUST_LOG=info cargo run -p diagnostics-service-example --bin diag_client
```

## Methods

| Method | Type | Input | Output |
|--------|------|-------|--------|
| `ReadDtc` | Request/Response | `dtc_id: u32` | `DtcSnapshot` (nested struct) |
| `ClearDtc` | Fire-and-forget | `dtc_id: u32` | — |
| `ReadEcuIdentification` | Request/Response | (none) | `EcuInfo` |
| `ReadDataByIdentifier` | Request/Response | `identifier: u16`, `sub_function: u8` | `DataRecord` |

## Nested Types

The diagnostics service uses nested structs to exercise the codegen's handling of complex types:

- **`DtcSnapshot`** — contains a DTC ID, status byte, and snapshot data
- **`EcuInfo`** — ECU identification with hardware/software version strings
- **`DataRecord`** — generic data container with identifier and byte payload

All types have generated `AraSerialize` / `AraDeserialize` implementations for SOME/IP wire format.

## ARXML Source

The service definition lives in `cargo-arxml/tests/fixtures/diagnostics_service.arxml`. The generated code is in `examples/diagnostics-service/src/generated/`.
