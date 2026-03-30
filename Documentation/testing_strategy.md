# Testing Strategy: ara-rs Workspace

## 1. Scope & Summary

This strategy covers the three-crate `ara-rs` workspace for Adaptive AUTOSAR development in Rust:

- **cargo-arxml**: CLI tool that parses `.arxml` files into an IR, validates the IR, and generates Rust source code (types, traits, proxy, skeleton) that depends on `ara-com`.
- **ara-com**: Transport-agnostic core library defining the `Transport` async trait, `AraSerialize`/`AraDeserialize` wire-format traits, `ProxyBase<T>`/`SkeletonBase<T>` generic wrappers, and service/method/event/field configuration types.
- **ara-com-someip**: SOME/IP transport backend implementing the `Transport` trait over UDP/TCP with SOME/IP-SD service discovery.

**Current state**: 24 passing unit tests in `ara-com`, 0 tests in `cargo-arxml`, 0 tests in `ara-com-someip`. All codegen functions and all `SomeIpTransport` trait methods are `todo!()` stubs.

**Phasing**: Phase 1 (Weeks 1-4) focuses on code generation. Phase 2 (Weeks 5-8) focuses on SOME/IP transport and service discovery. This strategy covers both phases with clear priority assignments.

---

## 2. Risk-Based Coverage Map

| # | Component/Area | Risk Rating | Risk Category | Primary Test Level | Priority |
|---|---|---|---|---|---|
| R1 | Codegen: generated types compile and match IR | HIGH | Correctness | Integration | P0 |
| R2 | Codegen: generated proxy/skeleton compile against `ara-com` | HIGH | Correctness | Integration | P0 |
| R3 | Codegen: type name mapping (snake_case, PascalCase) | HIGH | Correctness | Unit | P0 |
| R4 | Codegen: handling of all `DataTypeKind` variants | HIGH | Correctness | Unit | P0 |
| R5 | Parser: IR extraction from real ARXML files | HIGH | Correctness | Integration | P0 |
| R6 | Parser: primitive type mapping (map_primitive_type) | MEDIUM | Correctness | Unit | P0 |
| R7 | Validator: duplicate service ID detection | MEDIUM | Correctness | Unit | P0 |
| R8 | Validator: missing type reference detection (not yet implemented) | MEDIUM | Correctness | Unit | P1 |
| R9 | Serialization: round-trip correctness for all primitive types | LOW | Data Integrity | Unit | P0 (done) |
| R10 | Serialization: Vec<T> with large/adversarial counts | MEDIUM | Security | Unit | P1 |
| R11 | SOME/IP: send_request / response correlation | HIGH | Correctness | Integration | P0 (Phase 2) |
| R12 | SOME/IP: UDP vs TCP threshold routing | HIGH | Correctness | Unit | P0 (Phase 2) |
| R13 | SOME/IP: session ID allocation and wraparound | MEDIUM | Correctness | Unit | P0 (Phase 2) |
| R14 | SOME/IP: request timeout and retry | MEDIUM | Reliability | Integration | P1 (Phase 2) |
| R15 | SOME/IP-SD: offer/find/subscribe state machines | HIGH | Correctness | Integration | P0 (Phase 2) |
| R16 | SOME/IP-SD: TTL expiration and re-offer | MEDIUM | Reliability | Integration | P1 (Phase 2) |
| R17 | SOME/IP: concurrent request handling | MEDIUM | Concurrency | Integration | P1 (Phase 2) |
| R18 | E2E: ARXML to generated code to loopback communication | HIGH | Correctness | E2E | P1 |
| R19 | CLI: validate/generate/inspect subcommand behavior | MEDIUM | Correctness | Integration | P1 |
| R20 | SOME/IP: wire format compliance with someip_parse | HIGH | Data Integrity | Unit | P0 (Phase 2) |
| R21 | Codegen: fire-and-forget method generation | MEDIUM | Correctness | Unit | P0 |
| R22 | Codegen: field getter/setter/notifier generation variants | MEDIUM | Correctness | Unit | P0 |
| R23 | SOME/IP serialization: fixed-size and dynamic-size payloads | HIGH | Data Integrity | Unit | P0 (Phase 2) |
| R24 | String deserialization: invalid UTF-8, length overflow | MEDIUM | Security | Unit | P1 |

---

## 3. Detailed Test Plan

### 3.1 Unit Tests

#### 3.1.1 cargo-arxml: Validator (risks R7, R8)

**Location**: `cargo-arxml/src/validator/mod.rs` (inline `#[cfg(test)]` module)

**Test: `test_duplicate_service_id_detected`** (P0, R7)
- Build an `ArxmlProject` with two `ServiceInterface` entries sharing `service_id = Some(0x1234)`.
- Call `validate()`, assert exactly one `DuplicateServiceId` error is returned with the correct service name and ID.

**Test: `test_no_duplicate_when_ids_differ`** (P0, R7)
- Two services with `service_id = Some(0x1234)` and `Some(0x5678)`.
- Assert `validate()` returns no `DuplicateServiceId` errors.

**Test: `test_no_duplicate_when_ids_absent`** (P0, R7)
- Two services with `service_id = None`.
- Assert no `DuplicateServiceId` errors (services without IDs should be skipped).

**Test: `test_empty_interface_detected`** (P0, R7)
- A service with empty `methods`, `events`, and `fields`.
- Assert `EmptyServiceInterface` error.

**Test: `test_non_empty_interface_passes`** (P0, R7)
- A service with one method. Assert no `EmptyServiceInterface` error.

**Test: `test_missing_type_ref_detected`** (P1, R8)
- When `check_missing_type_refs` is implemented: a method parameter whose `type_ref` does not match any `DataType.path`.
- Assert `MissingTypeRef` error with correct `element_path` and `type_ref`.

#### 3.1.2 cargo-arxml: Codegen Helpers (risk R3)

**Location**: `cargo-arxml/src/codegen/mod.rs` (inline tests)

**Test: `test_snake_case_basic`** (P0, R3)
- `"BatteryService"` -> `"battery_service"`
- `"getSOC"` -> `"get_s_o_c"` (current behavior; document if intentional or if acronym handling is needed)
- `"already_snake"` -> `"already_snake"`
- `""` -> `""`

#### 3.1.3 cargo-arxml: Codegen Types (risks R1, R4)

**Location**: `cargo-arxml/src/codegen/types.rs` (inline tests, added as codegen is implemented)

These tests verify the generated TokenStream string output. They do NOT compile the output (that is an integration test concern). They check structural correctness.

**Test: `test_generate_primitive_type`** (P0, R4)
- An `ArxmlProject` with one `DataType { kind: Primitive(U32), name: "Voltage", .. }`.
- Assert the generated string contains `pub type Voltage = u32;` (or equivalent struct/newtype, depending on codegen design).

**Test: `test_generate_struct_type`** (P0, R4)
- A `DataType { kind: Structure { fields: [("soc", "/types/uint8"), ("voltage", "/types/float32")] }, .. }`.
- Assert output contains a `pub struct` with the correct field names and mapped Rust types.

**Test: `test_generate_enum_type`** (P0, R4)
- A `DataType { kind: Enumeration { variants: [("Off", 0), ("On", 1), ("Error", 2)] }, .. }`.
- Assert output contains a `pub enum` with the correct variant names and `#[repr]` discriminants.

**Test: `test_generate_array_type`** (P0, R4)
- `DataTypeKind::Array { element_type_ref, size: Some(10) }`.
- Assert output maps to `[ElementType; 10]` or equivalent.

**Test: `test_generate_vector_type`** (P0, R4)
- `DataTypeKind::Vector { element_type_ref }`.
- Assert output maps to `Vec<ElementType>`.

**Test: `test_generate_string_type`** (P0, R4)
- `DataTypeKind::String { max_length: Some(255), encoding: Utf8 }`.
- Assert output maps to `String` (possibly with a max-length comment or newtype).

#### 3.1.4 cargo-arxml: Codegen Proxy/Skeleton (risks R2, R21, R22)

**Location**: `cargo-arxml/src/codegen/proxy.rs` and `skeleton.rs` (inline tests)

**Test: `test_generate_proxy_with_request_response_method`** (P0, R2)
- A `ServiceInterface` with one method that has input and output parameters.
- Assert the generated proxy string contains an `async fn` with the right signature, calling `self.base.call_method`.

**Test: `test_generate_proxy_fire_and_forget_method`** (P0, R21)
- A method with `fire_and_forget: true` (no output params).
- Assert proxy calls `self.base.call_fire_and_forget` and returns `MethodResult<()>`.

**Test: `test_generate_skeleton_with_offer_stop_offer`** (P0, R2)
- Assert generated skeleton contains `offer()` and `stop_offer()` methods delegating to `SkeletonBase`.

**Test: `test_generate_field_getter_setter_notifier`** (P0, R22)
- A `ServiceInterface` with a field that has `has_getter: true, has_setter: true, has_notifier: true`.
- Assert proxy generates `get_<field>`, `set_<field>`, `subscribe_<field>` methods.
- A field with `has_getter: true, has_setter: false, has_notifier: false` should only generate `get_<field>`.

#### 3.1.5 cargo-arxml: Parser Helpers (risk R6)

**Location**: `cargo-arxml/src/parser/mod.rs` (inline tests)

**Test: `test_map_primitive_type_all_variants`** (P0, R6)
- Assert each known base-type path segment maps correctly:
  - `"/types/boolean"` -> `Bool`, `"/types/uint8"` -> `U8`, `"/types/sint32"` -> `I32`, `"/types/float64"` -> `F64`
- Assert unknown paths return `None`.

**Test: `test_map_primitive_type_case_insensitive`** (P0, R6)
- `"UINT32"`, `"Uint32"`, `"uint32"` all map to `U32`.

#### 3.1.6 ara-com: Serialization Edge Cases (risks R10, R24)

**Location**: `ara-com/src/serialization.rs` (extend existing test module)

**Test: `test_vec_huge_count_rejected`** (P1, R10)
- Craft a buffer with a 4-byte count of `0xFFFFFFFF` followed by 0 bytes of payload.
- Assert `Vec::<u32>::ara_deserialize` returns an error (not an OOM panic).
- This test validates that the deserializer does not blindly `Vec::with_capacity(count)` on untrusted input.

**Test: `test_string_invalid_utf8`** (P1, R24)
- A buffer with length prefix 3, followed by bytes `[0xFF, 0xFE, 0xFD]`.
- Assert `String::ara_deserialize` returns a `Deserialization` error mentioning UTF-8.

**Test: `test_string_length_overflow`** (P1, R24)
- A buffer with length prefix `0x7FFFFFFF` but only 4 bytes of payload.
- Assert truncation error, not panic.

**Test: `test_vec_nested_round_trip`** (P1, R10)
- `Vec<Vec<u8>>` round-trip to verify nested dynamic-length handling.

#### 3.1.7 ara-com-someip: Session ID Management (risk R13)

**Location**: `ara-com-someip/src/transport/mod.rs` (inline tests, added when implemented)

**Test: `test_session_id_increments`** (P0 Phase 2, R13)
- After creating a `SomeIpTransport`, each call to the internal `next_session_id()` (or equivalent) returns a monotonically increasing u16.

**Test: `test_session_id_wraps_at_u16_max`** (P0 Phase 2, R13)
- Set internal counter to `0xFFFE`, call twice, assert second call returns `0x0000` or `0x0001` (depending on spec; 0x0000 is reserved in some SOME/IP implementations, verify against spec).

#### 3.1.8 ara-com-someip: UDP vs TCP Routing (risk R12)

**Location**: `ara-com-someip/src/transport/mod.rs` (inline tests)

**Test: `test_payload_below_threshold_uses_udp`** (P0 Phase 2, R12)
- Create an `EndpointConfig` with `udp_threshold: 1400`.
- Assert a 100-byte payload routes to UDP.

**Test: `test_payload_above_threshold_uses_tcp`** (P0 Phase 2, R12)
- Same config, 2000-byte payload routes to TCP.

**Test: `test_no_tcp_configured_falls_back_to_udp`** (P0 Phase 2, R12)
- `EndpointConfig { tcp: None, udp: Some(..), .. }` with a large payload. Assert it still sends (via UDP) or returns a clear error.

#### 3.1.9 ara-com-someip: SOME/IP Header Encoding (risk R20)

**Location**: `ara-com-someip/src/transport/mod.rs` or a dedicated `header.rs` (inline tests)

**Test: `test_someip_header_encoding_matches_spec`** (P0 Phase 2, R20)
- Encode a `MessageHeader` with known values.
- Assert the first 16 bytes match the SOME/IP header layout: `[service_id:2][method_id:2][length:4][client_id:2][session_id:2][proto_ver:1][iface_ver:1][msg_type:1][return_code:1]`.
- Cross-verify by parsing the encoded bytes with `someip_parse::SomeIpHeader::from_slice()`.

**Test: `test_someip_header_decode_from_someip_parse`** (P0 Phase 2, R20)
- Build a header with `someip_parse`, encode it, decode with our code. Assert field-by-field equality.

#### 3.1.10 ara-com-someip: SOME/IP Serialization (risk R23)

**Location**: `ara-com-someip/src/serialization/fixed.rs` and `dynamic.rs` (inline tests)

**Test: `test_fixed_size_serialization_big_endian`** (P0 Phase 2, R23)
- Serialize a u32 with `ByteOrder::BigEndian`. Assert bytes match `to_be_bytes()`.

**Test: `test_dynamic_string_with_length_field`** (P0 Phase 2, R23)
- Serialize a string with `length_field_size: 4`. Assert the 4-byte length prefix is correct.

**Test: `test_dynamic_string_utf16`** (P1 Phase 2, R23)
- Serialize `"hello"` with `StringEncoding::Utf16Be`. Assert byte-level correctness.

---

### 3.2 Integration Tests

#### 3.2.1 cargo-arxml: Parser + Real ARXML Fixtures (risk R5)

**Location**: `cargo-arxml/tests/parser_integration.rs`
**Fixtures**: `cargo-arxml/tests/fixtures/` directory

**Fixture: `battery_service.arxml`** (P0, R5)
- A minimal but realistic ARXML file defining:
  - A `BatteryService` ServiceInterface with service_id `0x1234`, one method (`GetSOC` with a `uint8` return), one event (`SOCChanged` with `uint8` data type), and one field (`Voltage` with getter+notifier).
  - Three `ImplementationDataType` entries: a `uint8` VALUE, a `float32` VALUE, and a `BatteryStatus` STRUCTURE with two fields.

**Test: `test_parse_battery_service_fixture`** (P0, R5)
- Load `battery_service.arxml` via `ArxmlParser::load()`.
- Call `extract_ir()`.
- Assert: 1 service, correct `short_name`, `service_id`, method count, event count, field count.
- Assert: data types extracted with correct names, kinds, and field counts.

**Fixture: `multi_service.arxml`** (P0, R5)
- Two services in one file.
- Test: both are extracted, no cross-contamination.

**Fixture: `empty_service.arxml`** (P1, R5)
- A service with no methods/events/fields.
- Test: IR extraction succeeds; validator catches it.

**Fixture: `malformed.arxml`** (P1, R5)
- Syntactically valid XML but semantically broken ARXML (e.g., missing SHORT-NAME).
- Test: `ArxmlParser::load()` either returns an error or `extract_ir()` silently skips the broken element (document which behavior is expected).

#### 3.2.2 cargo-arxml: Codegen Compilation Test (risks R1, R2)

**Location**: `cargo-arxml/tests/codegen_compiles.rs`

This is the single most important integration test for Phase 1. It verifies that generated code actually compiles against `ara-com`.

**Test: `test_generated_code_compiles`** (P0, R1, R2)
1. Build an `ArxmlProject` in-memory (or load from `battery_service.arxml`).
2. Run `CodeGenerator::new(&project).generate()`.
3. Write the generated files into a temporary directory.
4. Create a temporary `Cargo.toml` that depends on `ara-com` (via path dependency).
5. Run `cargo check` on the temporary crate.
6. Assert exit code 0 (compilation succeeds).

This test is expensive (~seconds) but catches the highest-impact failure mode: generated code that does not compile. It should run in CI on every PR that touches codegen.

**Test: `test_generated_code_compiles_complex_model`** (P1, R1, R2)
- Same approach with a more complex fixture: enums, nested structs, arrays, vectors, multiple services.

#### 3.2.3 cargo-arxml: CLI Integration Tests (risk R19)

**Location**: `cargo-arxml/tests/cli_integration.rs`

**Test: `test_cli_validate_valid_file`** (P1, R19)
- Run `cargo run -- validate tests/fixtures/battery_service.arxml` via `std::process::Command`.
- Assert exit code 0, stdout contains "Validation passed".

**Test: `test_cli_validate_invalid_file`** (P1, R19)
- Fixture with duplicate service IDs.
- Assert exit code 1, stderr contains "duplicate service ID".

**Test: `test_cli_inspect_outputs_json`** (P1, R19)
- Run `inspect` subcommand.
- Assert stdout parses as valid JSON containing the expected service names.

**Test: `test_cli_generate_writes_files`** (P1, R19)
- Run `generate --output-dir <tmpdir>`.
- Assert `types.rs`, `traits.rs`, `proxy/`, `skeleton/` exist in the output directory.

#### 3.2.4 ara-com: ProxyBase with Mock Transport (risk R2)

**Location**: `ara-com/tests/proxy_integration.rs`

**Test: `test_proxy_base_call_method_round_trip`** (P0, R2)
- Implement a `MockTransport` that captures the `MessageHeader` and `Bytes` from `send_request`, then returns a pre-serialized response.
- Create `ProxyBase::with_defaults(Arc::new(mock), service_id, instance_id)`.
- Call `proxy.call_method::<u32, u32>(method_id, &42)`.
- Assert the mock received the correctly serialized `42u32` payload.
- Assert the response deserializes to the expected value.

**Test: `test_proxy_base_fire_and_forget`** (P0, R2)
- Same mock approach. Call `call_fire_and_forget`. Assert mock's `send_fire_and_forget` was called with `MessageType::RequestNoReturn`.

#### 3.2.5 ara-com-someip: Transport Loopback (risks R11, R14, R17)

**Location**: `ara-com-someip/tests/transport_loopback.rs`

These tests require actual UDP/TCP sockets on loopback. They are Phase 2 tests.

**Test: `test_request_response_udp_loopback`** (P0 Phase 2, R11)
- Start two `SomeIpTransport` instances on `127.0.0.1` with different ports.
- Server registers a request handler that echoes the payload.
- Client sends a request, awaits response.
- Assert response payload matches request payload.

**Test: `test_fire_and_forget_udp`** (P0 Phase 2, R11)
- Client sends fire-and-forget.
- Server captures the received message via handler.
- Assert payload arrives correctly.

**Test: `test_request_timeout`** (P1 Phase 2, R14)
- Client sends a request to a port where no server is listening (or server deliberately never responds).
- Assert `AraComError::Timeout` within a reasonable time.

**Test: `test_concurrent_requests_correlate_correctly`** (P1 Phase 2, R17)
- Client sends 10 concurrent requests with different payloads.
- Server echoes each with a deliberate random delay (0-50ms).
- Assert each response matches its corresponding request (session ID correlation).

**Test: `test_tcp_fallback_for_large_payload`** (P1 Phase 2, R12)
- Configure `udp_threshold: 100`.
- Send a 1000-byte payload.
- Assert the server receives it over TCP (verify via server-side endpoint tracking or simply that the data arrives intact).

#### 3.2.6 ara-com-someip: Service Discovery (risks R15, R16)

**Location**: `ara-com-someip/tests/sd_integration.rs`

**Test: `test_offer_then_find_service`** (P0 Phase 2, R15)
- Server offers `ServiceId(0x1234)`.
- Client calls `find_service(0x1234, ..)`.
- Assert client receives a `ServiceInstanceId` within the SD timeout.

**Test: `test_find_service_before_offer_waits`** (P0 Phase 2, R15)
- Client calls `find_service` first, then after 500ms the server offers.
- Assert client unblocks and receives the instance.

**Test: `test_stop_offer_triggers_unavailability`** (P1 Phase 2, R15)
- Server offers, client finds, server calls `stop_offer`.
- Assert client's service state transitions to `Unavailable` (via availability handler or polling).

**Test: `test_subscribe_event_group`** (P0 Phase 2, R15)
- Server offers, client finds and subscribes to an event group.
- Server sends a notification.
- Assert client receives the event.

**Test: `test_ttl_expiration_removes_service`** (P1 Phase 2, R16)
- Server offers with `ttl: 1` (1 second).
- Client finds the service.
- Wait 2 seconds without re-offer.
- Assert client's `found_services` no longer contains the entry.

---

### 3.3 End-to-End Tests

**Location**: `tests/e2e/` at workspace root (or a dedicated `examples/` integration test)

#### 3.3.1 ARXML-to-Communication Pipeline (risk R18)

**Test: `test_full_pipeline_arxml_to_loopback`** (P1 Phase 2, R18)

This is the "golden path" test that validates the entire stack end-to-end:

1. **Parse**: Load `battery_service.arxml` with `ArxmlParser`.
2. **Validate**: Run `validator::validate()`, assert clean.
3. **Generate**: Run `CodeGenerator::generate()`, write files to a temp crate.
4. **Compile**: `cargo check` the generated crate (depends on `ara-com` + `ara-com-someip`).
5. **Run**: Build and execute a test binary within the generated crate that:
   - Creates a `SomeIpTransport` server, offers `BatteryService`.
   - Creates a `SomeIpTransport` client, finds and calls `GetSOC`.
   - Server handler returns `85u8`.
   - Assert client receives `85u8`.

This test is expensive (~10s) and should run in CI nightly or on release branches, not on every commit.

#### 3.3.2 Multi-Service E2E (P2, R18)

- Two services on the same transport.
- Client discovers both, calls methods on each.
- Validates no cross-service interference.

---

### 3.4 Performance Tests

**Location**: `benches/` at workspace root, using `criterion`

#### 3.4.1 Serialization Throughput (P2)

**Benchmark: `bench_serialize_u32_throughput`**
- Serialize 1M `u32` values in a tight loop.
- Baseline: establish ops/sec for regression tracking.

**Benchmark: `bench_serialize_struct_throughput`**
- A representative 5-field struct. Measures realistic workload.

#### 3.4.2 Codegen Latency (P2)

**Benchmark: `bench_codegen_battery_service`**
- Time `CodeGenerator::generate()` on the battery service fixture.
- Goal: sub-100ms for a single-service model (acceptable CLI latency).

#### 3.4.3 SOME/IP Request Latency (P2, Phase 2)

**Benchmark: `bench_request_response_latency_loopback`**
- Measure p50/p99 latency of a request-response round trip over UDP loopback.
- 10,000 iterations, warmup of 100.
- Goal: sub-1ms p99 on loopback (validates no unnecessary allocations in the hot path).

**Benchmark: `bench_concurrent_requests_throughput`**
- 100 concurrent clients, 1000 requests each.
- Measure aggregate requests/sec. Establishes concurrency scaling baseline.

---

### 3.5 Security Tests

#### 3.5.1 Deserialization Robustness (P1, risks R10, R24)

**Location**: Inline in unit tests (see 3.1.6)

The primary security surface is deserialization of untrusted wire data. Key concerns:

1. **Memory exhaustion via inflated length prefixes** (R10): A crafted `Vec` or `String` length prefix claiming billions of elements must not trigger OOM. The deserializer should validate that `count * element_size <= remaining_buffer_length` before allocating.

2. **Invalid UTF-8 in String payloads** (R24): Already handled correctly; the current `from_utf8` call returns an error. Test coverage exists but should be expanded with fuzz-like inputs.

3. **Integer overflow in offset arithmetic**: When deserializing `Vec<T>`, the `offset += item.serialized_size()` loop could overflow if `serialized_size()` returns a wrong value for a maliciously crafted buffer. Add a bounds check test.

#### 3.5.2 Fuzz Testing (P2)

**Location**: `fuzz/` directory using `cargo-fuzz`

**Target: `fuzz_ara_deserialize_u32`**
- Feed random bytes to `u32::ara_deserialize`. Must never panic.

**Target: `fuzz_ara_deserialize_string`**
- Feed random bytes to `String::ara_deserialize`. Must never panic (errors are fine).

**Target: `fuzz_ara_deserialize_vec_u8`**
- Feed random bytes to `Vec::<u8>::ara_deserialize`. Must never panic or OOM on inputs < 1MB.

**Target: `fuzz_someip_header_decode`** (Phase 2)
- Feed random bytes to the SOME/IP header decoder. Must never panic.

---

## 4. Entry Criteria

### Phase 1 (Weeks 1-4): Code Generation

| Criterion | Verification |
|---|---|
| ARXML parser loads and extracts IR from at least one real fixture | `test_parse_battery_service_fixture` passes |
| Validator tests exist and pass for all implemented checks | `cargo test -p cargo-arxml` green |
| `snake_case` helper is tested | Unit tests pass |
| At least `battery_service.arxml` fixture exists in `tests/fixtures/` | File present in repo |
| `ara-com` existing 24 tests still pass | `cargo test -p ara-com` green |
| CI pipeline runs `cargo test --workspace` | Pipeline config exists |

### Phase 2 (Weeks 5-8): SOME/IP Transport

| Criterion | Verification |
|---|---|
| All Phase 1 exit criteria met | CI green on main branch |
| `SomeIpConfig` and `EndpointConfig` structures finalized | Code review approved |
| `MockTransport` exists in `ara-com` test utils | File present |
| UDP/TCP socket binding works on CI environment | Smoke test passes |

---

## 5. Exit Criteria

### Phase 1 Gate: "Generated code compiles"

| Criterion | Target |
|---|---|
| `test_generated_code_compiles` passes | Required for merge |
| All codegen unit tests pass for each `DataTypeKind` variant | Required for merge |
| Validator unit tests pass | Required for merge |
| Parser integration test with fixture passes | Required for merge |
| No `todo!()` remaining in codegen `types.rs`, `traits.rs`, `proxy.rs`, `skeleton.rs` | Required for merge |
| CLI integration tests pass (validate, generate, inspect) | Required for release |
| `cargo clippy --workspace` clean | Required for merge |

### Phase 2 Gate: "Two processes communicate over loopback"

| Criterion | Target |
|---|---|
| `test_request_response_udp_loopback` passes | Required for merge |
| `test_offer_then_find_service` passes | Required for merge |
| `test_subscribe_event_group` passes | Required for merge |
| `test_concurrent_requests_correlate_correctly` passes | Required for release |
| `test_request_timeout` passes | Required for release |
| `test_full_pipeline_arxml_to_loopback` passes | Required for release |
| No `todo!()` remaining in `SomeIpTransport` methods | Required for merge |
| SOME/IP header encoding cross-validated with `someip_parse` | Required for merge |

---

## 6. Open Risks & Recommendations

### Risk: `Vec::with_capacity` on untrusted counts (MEDIUM, immediate)

The current `Vec::<T>::ara_deserialize` in `ara-com/src/serialization.rs` line 214 calls `Vec::with_capacity(count)` where `count` comes directly from the wire. A 4-byte count of `0xFFFFFFFF` means `with_capacity(4_294_967_295)` which will attempt to allocate ~16GB for `Vec<u32>`.

**Recommendation**: Add a bounds check before allocation:
```rust
let max_elements = (buf.len() - 4) / std::mem::size_of::<T>().max(1);
if count > max_elements {
    return Err(AraComError::Deserialization { message: "..." });
}
```
This should be done before Phase 2, as the same deserialization path will be used for SOME/IP payloads from the network.

### Risk: ARXML fixture coverage gaps

Real-world ARXML files are complex. The `autosar-data` crate handles XML parsing, but the IR extraction in `parser/mod.rs` makes many assumptions about element structure (e.g., `unwrap_or_default()` on missing type refs at line 244). These silent defaults could mask real model errors.

**Recommendation**: Obtain or create at least 3 representative ARXML fixtures from real Adaptive AUTOSAR toolchains (e.g., Vector DaVinci, EB tresos). If not available, create fixtures that exercise every `DataTypeKind` variant and every AUTOSAR element type the parser handles.

### Risk: `fire_and_forget` inference from output params

In `parser/mod.rs` line 199, `fire_and_forget` is set to `output_params.is_empty()`. This is an inference, not an explicit ARXML attribute. A method with no output parameters might still expect a transport-level acknowledgment in some AUTOSAR profiles.

**Recommendation**: Document this design decision. Add a test that verifies the inference. If an explicit `FIRE-AND-FORGET` ARXML element exists in the target AUTOSAR schema version, prefer it.

### Risk: No `MissingTypeRef` validation implemented

The validator defines `ValidationError::MissingTypeRef` but never checks for it. Method parameters with `type_ref: ""` (from `unwrap_or_default()`) will silently produce broken codegen.

**Recommendation**: Implement `check_missing_type_refs` in the validator before codegen goes live. This is a P0 blocker for Phase 1 correctness.

### Risk: SOME/IP-SD multicast on CI

Service Discovery tests use multicast (`239.224.224.224`). Many CI environments (Docker containers, GitHub Actions) do not support multicast by default.

**Recommendation**: Make SD tests configurable to use unicast-only mode for CI. Add a `#[ignore]` attribute to multicast-dependent tests with a note explaining the CI limitation, or use a feature flag like `#[cfg(feature = "multicast-tests")]`.

### Risk: No cross-crate API stability contract

`cargo-arxml` generates code that `use`s `ara-com` types by name. If `ara-com` renames `ProxyBase` or changes the `Transport` trait signature, all previously generated code breaks silently (at the user's compile time, not at the generator's).

**Recommendation**: The `test_generated_code_compiles` integration test is the primary defense here. It must always run against the current `ara-com` source. Additionally, consider generating a `Cargo.toml` in the output that pins the `ara-com` version to a semver-compatible range.

### Recommendation: Test infrastructure setup (immediate)

Before writing any tests, set up the following:

1. **ARXML fixtures directory**: `cargo-arxml/tests/fixtures/` with at least `battery_service.arxml`.
2. **Test helpers module**: `cargo-arxml/tests/helpers/mod.rs` with functions to build `ArxmlProject` / `ServiceInterface` instances for unit tests, avoiding boilerplate.
3. **Mock transport**: `ara-com/tests/mock_transport.rs` implementing `Transport` with configurable responses. This is reused by proxy/skeleton integration tests and eventually by codegen compilation tests.
4. **CI configuration**: Ensure `cargo test --workspace` runs on every PR. Add a separate nightly job for E2E and performance tests.
