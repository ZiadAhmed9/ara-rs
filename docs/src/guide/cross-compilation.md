# Cross-Compilation and Yocto

ara-rs targets embedded Linux ECUs. This page covers how to cross-compile for ARM targets and integrate with Yocto-based build systems.

## Supported Targets

| Target Triple | Architecture | Typical Hardware |
|---------------|-------------|-----------------|
| `aarch64-unknown-linux-gnu` | ARMv8 64-bit | Raspberry Pi 4, NXP S32G, Renesas R-Car |
| `armv7-unknown-linux-gnueabihf` | ARMv7 32-bit | Raspberry Pi 3, older automotive ECUs |

## Cross-Compilation with Cargo

### Prerequisites

Install the Rust target and the corresponding GCC cross-linker:

```bash
# Add Rust targets
rustup target add aarch64-unknown-linux-gnu
rustup target add armv7-unknown-linux-gnueabihf

# Install cross-linkers (Debian/Ubuntu)
sudo apt-get install gcc-aarch64-linux-gnu gcc-arm-linux-gnueabihf
```

### Configuration

The workspace includes `.cargo/config.toml` with linker settings for both targets:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

### Building

```bash
# Check compilation (no linker needed)
cargo check --workspace --target aarch64-unknown-linux-gnu

# Full build (requires cross-linker installed)
cargo build --workspace --target aarch64-unknown-linux-gnu

# Build a specific example
cargo build -p battery-service-example --target aarch64-unknown-linux-gnu
```

The resulting binaries are in `target/<target-triple>/debug/` (or `release/` with `--release`).

### CI

The CI pipeline runs `cargo check --workspace` for both `aarch64-unknown-linux-gnu` and `armv7-unknown-linux-gnueabihf` on every push and PR. This catches compilation issues for embedded targets without needing physical hardware.

## Yocto Integration

The `meta-ara-rs/` directory at the repository root contains a Yocto/OpenEmbedded layer.

### Layer Dependencies

- `meta` (OpenEmbedded core)
- `meta-rust-bin` — provides the Rust toolchain and `cargo` class

### Adding the Layer

```bash
bitbake-layers add-layer /path/to/ara-rs/meta-ara-rs
```

Or add to `bblayers.conf`:

```
BBLAYERS += "/path/to/meta-ara-rs"
```

### Available Recipes

| Recipe | Type | Role |
|--------|------|------|
| `ara-com` | Library crate | Build dependency for application recipes (`DEPENDS`) |
| `ara-com-someip` | Library crate | Build dependency for application recipes (`DEPENDS`) |
| `cargo-arxml` | Binary (native) | Host tool — code generator, runs at build time |

Each recipe includes the full transitive `crate://` dependency list so BitBake can fetch all sources in an offline-capable build.

### Using in Your Application Recipe

`ara-com` and `ara-com-someip` are Rust library crates, not standalone deployable artifacts. Reference them as build dependencies from your service binary recipe:

```bitbake
# my-autosar-app_1.0.bb
DEPENDS += "ara-com ara-com-someip"

# For code generation at build time:
DEPENDS += "cargo-arxml-native"
```

Your application binary (which statically links against these crates) is what gets installed to the target image via `IMAGE_INSTALL`.

### Compatibility

The layer is compatible with Yocto Scarthgap (5.0) and Styhead (5.1).

### Note on cargo-arxml

`cargo-arxml` is a code generator that runs at build time, not on the target ECU. The recipe includes `BBCLASSEXTEND = "native nativesdk"` so it can be built as a host tool or included in an extensible SDK.
