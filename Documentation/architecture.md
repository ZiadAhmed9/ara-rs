# ara-rs System Architecture

This document describes the building blocks, data flows, and design decisions of the ara-rs workspace.

---

## 1. Workspace Overview

```
ara-rs/
├── cargo-arxml/           CLI + library: ARXML → IR → Rust code
├── ara-com/               Core traits & async abstractions (transport-agnostic)
├── ara-com-someip/        SOME/IP transport backend
└── examples/              (planned) Full service examples
```

**Dependency graph** (compile-time):

```
cargo-arxml ──uses──▶ autosar-data        (ARXML parsing)
                      autosar-data-abstraction
                      quote / proc-macro2  (code generation)

ara-com ──────────▶ async-trait, bytes, futures-core, thiserror
                    tokio (optional, feature-gated)

ara-com-someip ──▶ ara-com               (implements Transport trait)
                   someip_parse           (SOME/IP header parsing)
                   tokio                  (async runtime, sockets)
```

**Key constraint:** cargo-arxml has NO runtime dependency on ara-com or ara-com-someip. It generates code that `use`s those crates. This keeps the tool usable even if the runtime crates evolve independently.

---

## 2. Data Flow: ARXML to Running Service

```
                        COMPILE TIME                              RUNTIME
  ┌──────────┐    ┌────────────┐    ┌─────────────┐      ┌──────────────────┐
  │  .arxml   │    │   Parser   │    │  Code Gen   │      │  User Application │
  │  files    │───▶│ (autosar-  │───▶│ (quote /    │─────▶│  uses generated   │
  │           │    │  data)     │    │  proc-macro2)│      │  traits, proxies, │
  └──────────┘    └─────┬──────┘    └──────┬──────┘      │  skeletons        │
                        │                   │             └────────┬─────────┘
                        ▼                   ▼                      │
                  ┌──────────┐      ┌──────────────┐              │ depends on
                  │    IR    │      │ Generated .rs │              ▼
                  │ (serde-  │      │   - types.rs  │        ┌──────────┐
                  │  capable)│      │   - traits.rs │        │  ara-com │
                  └──────────┘      │   - proxy/    │        │ (traits) │
                                    │   - skeleton/ │        └────┬─────┘
                                    │   - tests.rs  │             │ implemented by
                                    └──────────────┘             ▼
                                                          ┌──────────────┐
                                                          │ara-com-someip│
                                                          │ (transport)  │
                                                          └──────────────┘
```

### 2.1 Stage: Parsing (cargo-arxml)

**Input:** One or more `.arxml` files (AUTOSAR XML).

**Process:**
1. `ArxmlParser::load(path)` loads files using `autosar-data` crate
2. DFS walk over the AUTOSAR element tree
3. Extract `SERVICE-INTERFACE` elements → `ServiceInterface` IR
4. Extract `IMPLEMENTATION-DATA-TYPE` elements → `DataType` IR
5. Resolve parameter types, event data types, field types

**Output:** `ArxmlProject` IR — a serde-serializable, transport-independent representation.

### 2.2 Stage: Validation (cargo-arxml)

Runs between parsing and code generation. Current checks:
- Duplicate service IDs
- Empty service interfaces (no methods, events, or fields)

Planned checks:
- Unresolved type references
- Method ID conflicts within a service
- Version consistency across service dependencies

### 2.3 Stage: Code Generation (cargo-arxml)

**Input:** `ArxmlProject` IR

**Output:** HashMap<filename, source_code> written to `--output-dir`

Generated files per project:

| File | Contents |
|------|----------|
| `types.rs` | Rust structs/enums for each `DataType` in the IR, with `AraSerialize`/`AraDeserialize` impls |
| `traits.rs` | One trait per `ServiceInterface` with async methods matching the ARXML operations |
| `proxy/{service}.rs` | Typed proxy wrapper around `ara_com::proxy::ProxyBase<T>` |
| `skeleton/{service}.rs` | Typed skeleton wrapper around `ara_com::skeleton::SkeletonBase<T>` |
| `tests.rs` | Compile-check tests and basic round-trip serialization tests |

Code generation uses `quote!` macros producing `proc_macro2::TokenStream`, formatted by `prettyplease` before writing. This is hygienic — no string concatenation, no template injection risks.

---

## 3. Building Blocks

### 3.1 Intermediate Representation (IR)

The IR is the contract between parsing and code generation. It is intentionally decoupled from both AUTOSAR's XML schema and Rust's type system.

```
ArxmlProject
├── services: Vec<ServiceInterface>
│   ├── name, short_name, path
│   ├── service_id: Option<u16>
│   ├── major_version, minor_version
│   ├── methods: Vec<Method>
│   │   ├── method_id, fire_and_forget
│   │   ├── input_params: Vec<Parameter>  (name, type_ref, direction)
│   │   └── output_params: Vec<Parameter>
│   ├── events: Vec<Event>
│   │   ├── event_id, event_group_id
│   │   └── data_type_ref
│   └── fields: Vec<Field>
│       ├── has_getter, has_setter, has_notifier
│       └── getter_method_id, setter_method_id, notifier_event_id
├── data_types: Vec<DataType>
│   ├── name, path
│   └── kind: Primitive | Enumeration | Structure | Array | Vector | String | TypeReference
└── source_files: Vec<String>
```

The IR is `Serialize + Deserialize` — the `inspect` CLI subcommand dumps it as JSON for debugging and tooling integration.

### 3.2 Transport Trait (ara-com)

The central abstraction. All communication flows through this trait:

```rust
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    // Client-side (proxy)
    async fn send_request(header, payload) -> Result<(MessageHeader, Bytes)>;
    async fn send_fire_and_forget(header, payload) -> Result<()>;
    async fn find_service(service_id, instance_id, ...) -> Result<ServiceInstanceId>;
    async fn subscribe_event_group(service_id, instance_id, event_group_id) -> Result<()>;
    async fn unsubscribe_event_group(service_id, instance_id, event_group_id) -> Result<()>;

    // Server-side (skeleton)
    async fn send_notification(header, payload) -> Result<()>;
    async fn offer_service(service_id, instance_id, ...) -> Result<()>;
    async fn stop_offer_service(service_id, instance_id) -> Result<()>;
    async fn register_request_handler(service_id, instance_id, handler) -> Result<()>;
}
```

**Design rationale:**
- Generic over payload (`Bytes`) — serialization happens in the proxy/skeleton layer, not the transport
- Async-native — no blocking calls, compatible with tokio and other async runtimes
- `Send + Sync + 'static` — safe to share across tasks and hold in long-lived structs
- Request handlers use boxed futures — allows the skeleton to process requests concurrently

### 3.3 Serialization Layer (ara-com)

Two traits for wire-format encoding:

```rust
pub trait AraSerialize: Send + Sync {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError>;
    fn serialized_size(&self) -> usize;
}

pub trait AraDeserialize: Sized + Send + Sync {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError>;
}
```

Built-in implementations (big-endian, AUTOSAR wire format):
- Primitives: `bool`, `u8`-`u64`, `i8`-`i64`, `f32`, `f64`
- `String`: 4-byte length prefix + UTF-8 payload
- `Vec<T>`: 4-byte count prefix + serialized elements

Code generation produces `AraSerialize`/`AraDeserialize` impls for generated structs and enums, composing the built-in primitive impls.

### 3.4 Proxy & Skeleton Base (ara-com)

```
┌──────────────────────────────────────┐
│          Generated Proxy             │  ← from cargo-arxml
│  (typed methods matching ARXML)      │
├──────────────────────────────────────┤
│          ProxyBase<T>                │  ← from ara-com
│  (call_method, call_fire_and_forget) │
├──────────────────────────────────────┤
│          Transport (trait)           │  ← from ara-com
│  (send_request, find_service, ...)   │
├──────────────────────────────────────┤
│       SomeIpTransport (impl)         │  ← from ara-com-someip
│  (UDP/TCP sockets, SD, correlation)  │
└──────────────────────────────────────┘
```

`ProxyBase<T: Transport>` provides:
- `call_method<Req, Resp>()` — serialize → send_request → deserialize
- `call_fire_and_forget<Req>()` — serialize → send_fire_and_forget
- Configurable timeout and retry via `MethodConfig`

`SkeletonBase<T: Transport>` provides:
- `offer()` / `stop_offer()` — delegates to transport
- Access to the transport for registering request handlers

### 3.5 Service Communication Patterns (ara-com)

Three communication patterns from Adaptive AUTOSAR, each with dedicated types:

| Pattern | Proxy Side | Skeleton Side |
|---------|-----------|--------------|
| **Method** (request/response) | `ProxyBase::call_method()` | `Transport::register_request_handler()` |
| **Event** (pub/sub) | `Transport::subscribe_event_group()` → `EventStream<T>` | `Transport::send_notification()` |
| **Field** (get/set/notify) | `FieldGetter<T>`, `FieldSetter<T>`, `FieldNotifier<T>` traits | Skeleton implements getter/setter logic |

### 3.6 SOME/IP Transport (ara-com-someip)

Implements `ara_com::Transport` over SOME/IP protocol on Linux sockets.

```
┌─────────────────────────────────────────────────────┐
│                 SomeIpTransport                       │
├─────────────────────────────────────────────────────┤
│                                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  Endpoint     │  │  Session     │  │  Handler   │ │
│  │  Manager      │  │  Tracker     │  │  Registry  │ │
│  │  (UDP/TCP)    │  │  (req/resp)  │  │  (skeleton)│ │
│  └──────┬───────┘  └──────┬───────┘  └─────┬──────┘ │
│         │                  │                 │        │
│  ┌──────▼──────────────────▼─────────────────▼──────┐│
│  │              Socket I/O Layer                     ││
│  │         (tokio::net UdpSocket / TcpStream)        ││
│  └──────────────────────────────────────────────────┘│
│                                                       │
│  ┌──────────────────────────────────────────────────┐│
│  │           Service Discovery (SOME/IP-SD)          ││
│  │  ┌────────┐  ┌──────────┐  ┌──────────────────┐ ││
│  │  │ Offer  │  │  Find    │  │  Subscribe /      │ ││
│  │  │ State  │  │  State   │  │  Eventgroup Mgmt  │ ││
│  │  └────────┘  └──────────┘  └──────────────────┘ ││
│  │           Multicast (239.224.224.224:30490)       ││
│  └──────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────┘
```

**Key components:**

| Component | Responsibility |
|-----------|---------------|
| **Endpoint Manager** | Manages UDP/TCP sockets per service. Routes messages by service/method ID. Selects UDP vs TCP based on `udp_threshold`. |
| **Session Tracker** | Assigns session IDs to outgoing requests. Correlates responses. Handles timeouts. |
| **Handler Registry** | Maps (service_id, instance_id) → request handler function. Dispatches incoming requests to skeleton handlers. |
| **Service Discovery** | SOME/IP-SD state machine over multicast. Offer/Find/Subscribe/Unsubscribe with TTL management. |
| **Serialization Context** | SOME/IP-specific encoding: byte order, string encoding, length field sizes. Extends ara-com's base serialization. |

**Configuration** (`SomeIpConfig`):
- Unicast address for this application
- SD multicast group and port (default 239.224.224.224:30490)
- Per-service endpoint config (UDP/TCP addresses, event groups)
- SD timing (initial delay, repetition, TTL)

---

## 4. Generated Code Structure

For a service interface `BatteryService` with method `GetVoltage` and event `VoltageChanged`:

```
generated/
├── types.rs           ← VoltageRequest, VoltageResponse structs
├── traits.rs          ← trait BatteryService { async fn get_voltage(...) }
├── proxy/
│   └── battery_service.rs  ← BatteryServiceProxy<T: Transport>
├── skeleton/
│   └── battery_service.rs  ← BatteryServiceSkeleton<T: Transport>
└── tests.rs           ← compile checks, serialization round-trips
```

**Generated proxy** (simplified):
```rust
pub struct BatteryServiceProxy<T: Transport> {
    base: ProxyBase<T>,
}

impl<T: Transport> BatteryServiceProxy<T> {
    pub async fn get_voltage(&self, req: VoltageRequest) -> Result<VoltageResponse> {
        self.base.call_method(METHOD_ID_GET_VOLTAGE, req).await
    }

    pub async fn subscribe_voltage_changed(&self) -> Result<EventStream<VoltageChanged>> {
        // subscribe to event group, return typed stream
    }
}
```

**Generated skeleton** (simplified):
```rust
pub struct BatteryServiceSkeleton<T: Transport> {
    base: SkeletonBase<T>,
}

impl<T: Transport> BatteryServiceSkeleton<T> {
    pub async fn offer(&self) -> Result<()> {
        self.base.offer().await
    }
    // Handler registration for get_voltage dispatched via Transport
}
```

---

## 5. Error Handling Architecture

```
┌──────────────────────────────────┐
│         AraComError              │  ← unified error type (ara-com)
│  Transport | Serialization |     │
│  Deserialization | Timeout |     │
│  ServiceNotAvailable | ...       │
└──────────┬───────────────────────┘
           │ From<SomeIpError>
┌──────────▼───────────────────────┐
│         SomeIpError              │  ← backend-specific (ara-com-someip)
│  Header | Endpoint | Discovery | │
│  Timeout(session_id) | Io        │
└──────────────────────────────────┘

┌──────────────────────────────────┐
│         ArxmlError               │  ← tooling errors (cargo-arxml)
│  ArxmlLoad | Validation |        │
│  CodeGen | Io | Config           │
└──────────────────────────────────┘
```

All errors use `thiserror` for derive-based error definitions. Backend errors convert into `AraComError` via `From` impls — user code only sees the unified error type.

---

## 6. Design Principles

1. **Transport-agnostic core.** `ara-com` defines the contract. Backends are pluggable. SOME/IP is first but not only.

2. **Codegen is a build step, not a runtime dependency.** `cargo-arxml` produces plain `.rs` files. No proc-macros at runtime. No hidden codegen. Users can read and modify the generated code.

3. **Async-first, runtime-flexible.** All traits are async. Tokio is the default runtime, feature-gated so other runtimes can be supported.

4. **Serialization at the right layer.** Primitives serialize in `ara-com` (big-endian AUTOSAR format). SOME/IP-specific encoding (alignment, TLV) lives in `ara-com-someip`. Generated code composes both.

5. **IR as the pivot.** The intermediate representation decouples parsing from generation. New generators (C bindings, documentation, validation reports) can consume the same IR.

6. **No unsafe.** The codebase targets Ferrocene compatibility. No unsafe blocks unless strictly necessary and audited.
