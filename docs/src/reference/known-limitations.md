# Known Limitations

This page documents current limitations and gaps. Items here are candidates for future sprints.

## Code Generation (cargo-arxml)

- **No field codegen.** The `ara-com` crate defines `FieldConfig` and getter/setter/notifier traits, but `cargo-arxml` does not yet generate field implementations. ARXML field definitions are parsed but not wired into proxy/skeleton output.
- **No `--cxx` flag.** CXX bridge code must be hand-written. Automatic `#[cxx::bridge]` module generation from ARXML is planned (Sprint 14).
- **No error recovery.** Malformed or partial ARXML files may cause panics rather than graceful error messages with source locations.
- **Limited type support.** Serialization covers primitives, `String`, `Vec<T>`, and flat structs. Enumerations, unions, optional fields, and deeply nested type hierarchies are not yet supported.
- **No ARXML write-back.** The tool reads ARXML but cannot modify or generate ARXML files.

## Transport (ara-com-someip)

- **No simultaneous UDP + TCP.** The `udp_threshold` setting routes traffic to one transport or the other per message. Offering the same service on both UDP and TCP simultaneously is not supported.
- **No TLS.** All SOME/IP communication is unencrypted. TLS transport is not implemented.
- **No E2E protection.** AUTOSAR End-to-End (E2E) protection profiles are not implemented.
- **Single-process only.** Each `SomeIpTransport` instance owns its sockets. There is no shared routing daemon — each process is a standalone SOME/IP endpoint.
- **No SOME/IP-TP.** SOME/IP Transport Protocol (segmentation of large UDP messages) is not implemented. Large payloads are routed to TCP instead.
- **IPv4 only.** IPv6 is not supported.

## Service Discovery

- **No subscription acknowledgment.** The SD state machine sends `SubscribeEventgroup` but does not process `SubscribeEventgroupAck/Nack` responses.
- **No reboot detection.** The SD reboot flag is set but not used to invalidate stale discovered services.

## Serialization

- **No zero-copy paths.** All deserialization allocates. A `no_std` / zero-copy serialization core is planned but not implemented.
- **No `no_std` support.** The serialization traits require `std` (via `Vec<u8>` and `String`).

## Platform

- **Linux only.** The transport relies on Unix socket APIs. macOS may work for codegen but is untested. Windows is not supported.
- **No QEMU CI.** Cross-compilation is CI-checked (`cargo check`) but binaries are not executed on emulated ARM targets.

## CXX Bridge

- **One direction only.** The example bridges C++ -> Rust. Rust -> C++ callbacks across the bridge are not demonstrated.
- **Synchronous only.** Bridge functions block on the tokio runtime. Async C++ integration is not provided.
- **Manual only.** Bridge modules must be hand-written; `cargo-arxml` does not generate them.
