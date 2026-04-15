# Support Matrix

## Rust

| Version | Status |
|---------|--------|
| Current stable | Supported, CI-tested |
| Nightly | Not tested, may work |
| MSRV | Intended 1.75 (2021 edition), not yet CI-enforced |

## Operating Systems

| OS | Status | Notes |
|----|--------|-------|
| Linux x86_64 | Fully supported | CI-tested, primary development platform |
| Linux aarch64 | Cross-compilation checked | CI runs `cargo check`, no runtime tests |
| Linux armv7hf | Cross-compilation checked | CI runs `cargo check`, no runtime tests |
| macOS | Not tested | Likely works for `cargo-arxml` codegen; transport needs UDP multicast support |
| Windows | Not supported | SOME/IP socket code assumes Unix APIs (`socket2` with `SO_REUSEADDR`) |

## Target Triples (Cross-Compilation)

| Target | CI Status | Notes |
|--------|-----------|-------|
| `x86_64-unknown-linux-gnu` | Build + test | Full test suite |
| `aarch64-unknown-linux-gnu` | Build check | `cargo check --workspace` |
| `armv7-unknown-linux-gnueabihf` | Build check | `cargo check --workspace` |

## External Dependencies

| Dependency | Version | Role |
|------------|---------|------|
| `autosar-data` | 0.21.x | ARXML file parsing |
| `autosar-data-abstraction` | 0.10.x | Higher-level ARXML access |
| `someip_parse` | 0.6.x | SOME/IP header parsing |
| `tokio` | 1.x | Async runtime |
| `cxx` | 1.x | C++ bridge (example only) |

## Yocto

| Release | Status |
|---------|--------|
| Scarthgap (5.0) | Layer compatible |
| Styhead (5.1) | Layer compatible |

Requires `meta-rust-bin` for the Rust toolchain and `cargo` class.

## vsomeip Interop

| vsomeip Version | Status |
|-----------------|--------|
| 3.4.10 | Docker demo validated |
| Other versions | Should work (same wire format), not tested |

Wire compatibility is validated byte-for-byte in `ara-com-someip/tests/wire_compat.rs`.
