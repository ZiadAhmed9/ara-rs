# ara-rs — Rust ARA Developer Kit for Adaptive AUTOSAR

## Project Overview

A cargo-native Rust developer toolkit for Adaptive AUTOSAR on embedded Linux ECUs. Three independently usable crates that provide a license-free path from ARXML to running services.

Builds on top of `autosar-data` (ARXML parsing) and `someip_parse` (SOME/IP headers). Adds the missing layers: code generation, typed async communication, and a full SOME/IP transport.

## Workspace Structure

```
ara-rs/
├── cargo-arxml/       ← CLI + library: ARXML parser, validator, Rust code generator
├── ara-com/           ← Core traits + async abstractions (transport-agnostic)
├── ara-com-someip/    ← SOME/IP transport backend
└── examples/          ← (planned) battery-service, diagnostics-service, cxx-interop
```

## Architecture Decisions

- Code generation uses `quote`/`proc-macro2` (hygienic TokenStream, not string templates)
- `ara-com` is transport-agnostic — the `Transport` trait is the extension point for backends
- `cargo-arxml` has NO runtime dependency on `ara-com` — it generates code that `use`s `ara-com` types
- SOME/IP payload serialization built from scratch — `someip_parse` only handles headers
- Async-first — all communication traits are async, `tokio` is the default runtime (feature-gated)

## Key External Dependencies

| Crate | Version | Role |
|-------|---------|------|
| `autosar-data` | 0.21.x | ARXML file parsing + element model |
| `autosar-data-abstraction` | 0.10.x | Higher-level ARXML access |
| `someip_parse` | 0.6.x | SOME/IP wire-format header parsing |
| `tokio` | 1.x | Async runtime |
| `quote` + `proc-macro2` | 1.x | Code generation |
| `clap` | 4.x | CLI argument parsing |

## Coding Conventions

- **License:** MIT OR Apache-2.0 on all crates
- **Error handling:** `thiserror` for library errors
- **Async:** `async-trait` for trait definitions
- **Naming:** Rust API guidelines — snake_case methods, PascalCase types
- **Safety:** Minimize `unsafe`. Ferrocene-friendly.
