# ara-rs Project Phases

Goal: Step-by-step path from working tooling to recognized, adopted project in the SDV/Adaptive AUTOSAR Rust community.

---

## Phase 1 — Foundation & First Demo (Weeks 1-4) **[COMPLETE]**

**Objective:** Produce a working end-to-end flow that can be shown in a 2-minute terminal recording: ARXML in, Rust code out, service compiles.

### Deliverables

- cargo-arxml code generation: types, service traits, proxy stubs, skeleton stubs
- Generated code compiles against ara-com without manual edits
- CLI polish: clear error messages, `--help` documentation, colored output
- Sample ARXML fixtures (battery-management service) checked into repo
- Unit + integration tests for parser and codegen (generated code compiles and passes basic assertions)

### Implementation Milestones

| Week | Milestone | Exit Criteria |
|------|-----------|---------------|
| 1 | Parser + IR + Validator complete | `cargo arxml inspect` dumps valid JSON from sample ARXML |
| 2 | types.rs + traits.rs codegen | Generated structs implement `AraSerialize`/`AraDeserialize`; generated traits have correct async method signatures |
| 3 | proxy.rs + skeleton.rs codegen | Generated proxy calls `ProxyBase::call_method`; generated skeleton wires `SkeletonBase::offer` |
| 4 | Integration tests + CLI polish | `cargo arxml generate` on battery-service ARXML → output compiles with `cargo check` |

### Entry Criteria
- Workspace builds with `cargo build --workspace`
- Parser correctly extracts ServiceInterface and DataType from ARXML
- ara-com core traits and serialization are stable (24 tests passing)

### Exit Criteria
- All codegen modules produce valid Rust from battery-management ARXML fixture
- Generated code compiles against ara-com without manual edits
- >80% unit test coverage on parser and codegen
- CI passes: check, test, clippy, fmt

### Community Signal

- Push to public GitHub with README, architecture diagram, and MIT/Apache-2.0 license
- Record a short terminal demo (asciinema or GIF): `cargo arxml generate` on a real ARXML project
- First LinkedIn post: "I built a cargo-native ARXML-to-Rust code generator for Adaptive AUTOSAR"

---

## Phase 2 — Typed Communication & SOME/IP Transport (Weeks 5-8) **[COMPLETE]**

**Objective:** Generated proxies and skeletons can actually talk over SOME/IP on Linux. This is the moment the tool becomes useful, not just interesting.

### Deliverables

- ara-com-someip: UDP socket transport, request/response correlation, session management
- SOME/IP serialization: fixed and dynamic-length payloads (big-endian wire format)
- Service Discovery (SOME/IP-SD): multicast state machine with offer/find/subscribe lifecycle, TTL tracking with expiry, `SO_REUSEADDR` via `socket2`
- Event streams: `broadcast::Sender/Receiver` notification channels with backpressure (slow consumers get `Lagged`)
- Event-group-aware routing: `send_notification` resolves events to event groups via `EventGroupConfig` and only fans out to matching subscribers
- Instance binding invariant: one instance per service per transport, enforced at all entry points (`offer_service`, `register_request_handler`, `subscribe_notifications`, `subscribe_event_group`)
- Full battery-management example: SD discovery, request/response, fire-and-forget, VoltageChanged event subscription — all from generated code
- 91 tests across the workspace (loopback integration, SD integration, wire compat, instance binding rejection)

### Implementation Milestones

| Week | Milestone | Exit Criteria |
|------|-----------|---------------|
| 5 | Socket I/O + fire-and-forget | UDP send/receive works; `send_fire_and_forget` sends valid SOME/IP frame on loopback |
| 6 | Request/response correlation | `send_request` returns correct response; session tracking works; request handler dispatches to skeleton |
| 7 | Service Discovery (SD) | OfferService/FindService/SubscribeEventgroup work over multicast; SD state machine handles TTL |
| 8 | Event streams + battery example | Full battery-management example: proxy discovers service, calls method, subscribes to events |

### Entry Criteria
- Phase 1 complete: code generation produces compilable output
- ara-com traits are stable and tested

### Exit Criteria
- Two processes communicate over loopback using generated code
- Battery-management example runs end-to-end without manual intervention
- Integration tests pass: request/response, fire-and-forget, event subscription
- SOME/IP frames are wire-compatible (validated with Wireshark or byte-level assertions)

### Community Signal

- Second LinkedIn post with a video: "Generated Rust services talking SOME/IP — zero manual code"
- Post to r/rust and r/embedded with the example walkthrough
- Open GitHub Discussions for early feedback
- Tag autosar-data and someip_parse maintainers — acknowledge the ecosystem

---

## Phase 3 — C++ Interop & Real-World Validation (Weeks 9-12)

**Objective:** Prove the tool works alongside existing C++ Adaptive stacks. This is the trust-builder for production teams.

### Deliverables

- C++ interop example: Rust service called from a C++ vsomeip client, and vice-versa
- CXX bridge generation: cargo-arxml emits optional `cxx::bridge` modules for cross-language method calls
- Wire compatibility tests against vsomeip (same SOME/IP messages, byte-for-byte)
- Yocto meta-layer recipe: build ara-rs crates in a Yocto SDK
- Basic benchmark page: request/response latency and pub/sub throughput vs. vsomeip on ARM64 (e.g., Raspberry Pi 4 or QEMU)

### Implementation Milestones

| Week | Milestone | Exit Criteria |
|------|-----------|---------------|
| 9 | vsomeip wire compatibility | Captured SOME/IP frames match vsomeip byte-for-byte for same service definition |
| 10 | CXX bridge generation | `cargo arxml generate --cxx` produces cxx::bridge modules; Rust service callable from C++ |
| 11 | Yocto recipe + cross-compilation | `bitbake ara-rs` succeeds; binaries run on aarch64 QEMU |
| 12 | Benchmark page | Published latency/throughput numbers vs. vsomeip on ARM64 |

### Entry Criteria
- Phase 2 complete: SOME/IP transport works end-to-end
- Battery-management example runs on Linux

### Exit Criteria
- C++ client calls Rust service and vice-versa over SOME/IP
- Wire compatibility proven with byte-level test suite
- Yocto recipe builds all three crates
- Benchmark results published and reproducible

### Community Signal

- Blog post: "Calling Rust Adaptive services from C++ (and back) — wire-compatible with vsomeip"
- Submit talk proposal to Rust Embedded WG meetup or ELCE/Automotive Linux Summit
- Engage with Ferrocene and S-CORE communities — position ara-rs as the lightweight cargo-native complement
- Aim for first external contributor or issue from a real team

---

## Phase 4 — Production Hardening & Ecosystem Presence (Weeks 13-20)

**Objective:** Make the tool reliable enough that teams depend on it. Shift from "cool demo" to "we use this in our CI pipeline."

### Deliverables

- Publish all three crates to crates.io with stable 0.1.0 versions
- cargo-arxml: config file support (arxml.toml), naming style options, output filtering
- Validator expansion: full reference checking, method ID conflicts, version consistency
- Error recovery: graceful handling of malformed/partial ARXML
- Documentation: rustdoc on all public APIs, mdBook user guide with tutorials
- Diagnostics service example (second real-world use case)
- CI: cross-compilation targets (aarch64-unknown-linux-gnu, armv7), QEMU integration tests

### Implementation Milestones

| Week | Milestone | Exit Criteria |
|------|-----------|---------------|
| 13-14 | crates.io publish preparation | All public APIs have rustdoc; CHANGELOG.md written; version 0.1.0 tagged |
| 15-16 | Config file + validator expansion | `arxml.toml` config works; validator catches unresolved type refs, method ID conflicts |
| 17-18 | Error recovery + diagnostics example | Malformed ARXML produces helpful errors (not panics); diagnostics-service example runs |
| 19-20 | mdBook + cross-compilation CI | User guide published on GitHub Pages; CI builds for aarch64 + armv7 |

### Entry Criteria
- Phase 3 complete: C++ interop proven, benchmarks published

### Exit Criteria
- All three crates published to crates.io with 0.1.0
- mdBook user guide live on GitHub Pages
- CI passes on x86_64, aarch64, armv7
- Two complete example services (battery + diagnostics)

### Community Signal

- Announce crates.io publish with changelog and migration guide
- Create a project website or mdBook hosted on GitHub Pages
- Apply for inclusion in Awesome Embedded Rust and Awesome AUTOSAR lists
- Present at a Rust meetup (virtual or local)

---

## Phase 5 — Advanced Features & Adoption Growth (Weeks 21-30)

**Objective:** Expand capability based on real user feedback. Become the default answer to "how do I use ARXML with Rust?"

### Deliverables

- Field support: getter/setter/notifier code generation and transport wiring
- Multi-binding support: same service offered over UDP + TCP simultaneously
- Security: TLS transport option, basic authentication hooks
- Performance: zero-copy deserialization paths, optional `no_std` serialization core
- cargo-arxml watch mode: re-generate on ARXML file changes (dev inner loop)
- Plugin system or trait-based extension point for custom transport backends (DDS, MQTT)

### Community Signal

- Track GitHub stars, crates.io downloads, and issue activity as adoption metrics
- Write case study with an early adopter team (even if internal/anonymized)
- Contribute upstream improvements to autosar-data or someip_parse if discovered during development
- Engage AUTOSAR Rust SIG if one forms around R25-11

---

## Phase 6 — Ecosystem Leadership (Weeks 30+)

**Objective:** Establish ara-rs as critical infrastructure in the Rust automotive ecosystem.

### Potential Directions (driven by community demand)

- Lifecycle & Execution Management traits (ara-exec)
- Persistency traits (ara-per)
- Safety evidence generation (Ferrocene + ISO 26262 tooling support)
- AI-assisted service generation from natural language or architecture diagrams
- Integration with Eclipse S-CORE as an alternative frontend
- Commercial support or consulting offering for OEM/Tier-1 adoption

### Community Signal

- Conference talks at Embedded World, FOSDEM Automotive, or RustConf
- RFC process for major design decisions — invite community co-ownership
- Aim for corporate sponsors or AUTOSAR working group recognition

---

## Guiding Principles Across All Phases

1. **Ship small, ship often.** Every phase ends with something demonstrable. No silent 8-week gaps.
2. **Show, don't tell.** Terminal recordings, benchmarks, and working examples beat slide decks.
3. **Complement, never compete.** Position alongside S-CORE and AUTOSAR's official Rust proposal. Acknowledge the ecosystem publicly.
4. **Solve real pain first.** ARXML-to-Rust codegen is the hook. SOME/IP transport is the retention. C++ interop is the trust-builder.
5. **Earn trust through transparency.** Public roadmap, open issues, honest benchmarks. Automotive teams are skeptical of hype — show the warts.

---

## Implementation Sprint Sequence

The roadmap above describes product phases. For day-to-day execution and low-risk merging, use the sprint files below in order. Each sprint is intentionally scoped so it can land as one milestone or a short stack of focused PRs.

1. [Sprint 01 - Correctness Traps](./sprints/sprint-01-correctness-traps.md)
2. [Sprint 02 - Public API Docs](./sprints/sprint-02-public-api-docs.md)
3. [Sprint 03 - TCP Transport](./sprints/sprint-03-tcp-transport.md)
4. [Sprint 04 - Diagnostics Example](./sprints/sprint-04-diagnostics-example.md)
5. [Sprint 05 - Crates.io Release Prep](./sprints/sprint-05-crates-io-release-prep.md)
6. [Sprint 06 - mdBook Docs Site](./sprints/sprint-06-mdbook-docs-site.md)
7. [Sprint 07 - Cross Compilation And Yocto](./sprints/sprint-07-cross-compilation-and-yocto.md)
8. [Sprint 08 - vsomeip Interop Demo](./sprints/sprint-08-vsomeip-interop-demo.md)
9. [Sprint 09 - CXX Bridge](./sprints/sprint-09-cxx-bridge.md)
10. [Sprint 10 - Benchmarks](./sprints/sprint-10-benchmarks.md)

Recommended merge rule: do not start a later sprint until the current sprint has green tests, updated docs, and a clear demo or acceptance check.
