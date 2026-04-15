# Code Generation

`cargo-arxml` reads ARXML service interface definitions and generates idiomatic Rust code that compiles against `ara-com`.

## What Gets Generated

For each service interface in your ARXML:

| File | Contents |
|------|----------|
| `types.rs` | Data type structs with `AraSerialize` / `AraDeserialize` implementations |
| `traits.rs` | Service trait with async method signatures |
| `proxy/<service>.rs` | Proxy stub — client-side, calls through `ProxyBase` |
| `skeleton/<service>.rs` | Skeleton stub — server-side, wires into `SkeletonBase` |
| `mod.rs` | Module re-exports |

## SOME/IP Deployment

If your ARXML includes `SOMEIP-SERVICE-INTERFACE-DEPLOYMENT` elements, `cargo-arxml` extracts:

- Service IDs
- Method IDs
- Event IDs and event group mappings

When deployment IDs are missing, the generator auto-assigns collision-safe IDs and emits warnings. Use `--strict` to treat missing IDs as errors.

## Generated Type Example

Given an ARXML `IMPLEMENTATION-DATA-TYPE` named `BatteryVoltage` with a `voltage` field of type `float64`:

```rust,ignore
#[derive(Debug, Clone, PartialEq)]
pub struct GetVoltageResponse {
    pub voltage: f64,
}

impl AraSerialize for GetVoltageResponse {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.voltage.ara_serialize(buf)?;
        Ok(())
    }
}

impl AraDeserialize for GetVoltageResponse {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        let mut offset = 0;
        let voltage = f64::ara_deserialize(&buf[offset..])?;
        offset += std::mem::size_of::<f64>();
        Ok(Self { voltage })
    }
}
```

## Generated Proxy Example

```rust,ignore
pub struct BatteryServiceProxy<T: Transport> {
    pub base: ProxyBase<T>,
}

impl<T: Transport> BatteryServiceProxy<T> {
    pub fn new(transport: Arc<T>, instance_id: InstanceId) -> Self { /* ... */ }

    pub async fn get_voltage(&self, battery_id: u8) -> Result<f64, AraComError> {
        // Constructs GetVoltageRequest, calls base.call_method(), returns voltage
    }

    pub async fn set_charge_limit(&self, limit: f64) -> Result<(), AraComError> {
        // Constructs SetChargeLimitRequest, calls base.call_fire_and_forget()
    }

    pub async fn subscribe_voltage_changed(&self) -> Result<(), AraComError> {
        // Subscribes to the event group via the transport
    }
}
```

## Configuration with arxml.toml

You can configure the codegen pipeline via `arxml.toml`:

```toml
[input]
paths = ["arxml/"]
exclude = ["arxml/deprecated/"]

[output]
dir = "src/generated"
crate_name = "my_service"

[naming]
method_style = "snake_case"
type_style = "snake_case"
```

See the [Configuration](../reference/configuration.md) page for all available options.
