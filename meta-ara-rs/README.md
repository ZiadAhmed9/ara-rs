# meta-ara-rs

Yocto/OpenEmbedded layer for the [ara-rs](https://github.com/ZiadAhmed9/ara-rs) Adaptive AUTOSAR Rust toolkit.

## Layer Dependencies

- `meta` (OpenEmbedded core)
- `meta-rust-bin` — provides the Rust toolchain and `cargo` class

## Recipes

| Recipe | Type | Role |
|--------|------|------|
| `ara-com` | Library crate | Build dependency — consumed by application recipes via `DEPENDS` |
| `ara-com-someip` | Library crate | Build dependency — consumed by application recipes via `DEPENDS` |
| `cargo-arxml` | Binary (native) | Host tool — code generator, runs at build time (`cargo-arxml-native`) |

Each recipe includes the full transitive `crate://` dependency list so BitBake can fetch all sources in an offline-capable build.

## Usage

1. Add this layer to your `bblayers.conf`:

```
BBLAYERS += "/path/to/meta-ara-rs"
```

2. Reference from your application recipe:

```bitbake
# In your service binary recipe (e.g., my-autosar-app_1.0.bb):
DEPENDS += "ara-com ara-com-someip"

# For code generation at build time:
DEPENDS += "cargo-arxml-native"
```

`ara-com` and `ara-com-someip` are Rust library crates — they are not standalone deployable artifacts and should not be added to `IMAGE_INSTALL`. Your application binary (which links against them) is what gets installed to the target image.

## Compatibility

Compatible with Yocto Scarthgap (5.0) and Styhead (5.1).

## License

MIT OR Apache-2.0, matching the upstream ara-rs workspace.
