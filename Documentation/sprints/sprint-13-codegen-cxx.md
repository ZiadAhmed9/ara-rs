# Sprint 13 — Codegen CXX Bridge

## Goal

Add a `--cxx` flag to `cargo arxml generate` that emits `#[cxx::bridge]` modules alongside the normal Rust output.

## Why Now

The manual CXX bridge example (Sprint 09) proved the pattern works. Automating it removes the last manual step for teams that need C++ interop.

## Scope

- New `--cxx` flag on `cargo arxml generate`
- Generate a `cxx_bridge.rs` module per service with:
  - Opaque client handle type
  - `connect_<service>(host, port)` function
  - Blocking wrappers for each proxy method
- Generate a corresponding C++ header
- Integration test: generate + compile + link

## Out of Scope

- Rust-to-C++ callbacks
- Async C++ bridge
- Skeleton-side (C++ server) bridge

## Exit Criteria

- `cargo arxml generate --cxx` produces compilable CXX bridge for the battery service
- Generated bridge matches the hand-written Sprint 09 example in functionality
- C++ header is usable from a standalone C++ project
