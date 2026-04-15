SUMMARY = "ARXML parser, validator, and Rust code generator for Adaptive AUTOSAR"
HOMEPAGE = "https://github.com/ZiadAhmed9/ara-rs"
LICENSE = "MIT | Apache-2.0"
LIC_FILES_CHKSUM = " \
    file://LICENSE-MIT;md5=b278a92d2c1509760384428817710378 \
    file://LICENSE-APACHE;md5=34400b68072d710fecd0a2940a0d1658 \
"

SRC_URI += " \
    crate://crates.io/cargo-arxml/${PV} \
    crate://crates.io/anstream/1.0.0 \
    crate://crates.io/anstyle/1.0.14 \
    crate://crates.io/anstyle-parse/1.0.0 \
    crate://crates.io/anstyle-query/1.1.5 \
    crate://crates.io/anstyle-wincon/3.0.11 \
    crate://crates.io/anyhow/1.0.102 \
    crate://crates.io/autocfg/1.5.0 \
    crate://crates.io/autosar-data/0.21.1 \
    crate://crates.io/autosar-data-abstraction/0.10.1 \
    crate://crates.io/autosar-data-specification/0.21.0 \
    crate://crates.io/bitflags/2.11.0 \
    crate://crates.io/byteorder/1.5.0 \
    crate://crates.io/cfg-if/1.0.4 \
    crate://crates.io/clap/4.6.0 \
    crate://crates.io/clap_builder/4.6.0 \
    crate://crates.io/clap_derive/4.6.0 \
    crate://crates.io/clap_lex/1.1.0 \
    crate://crates.io/colorchoice/1.0.5 \
    crate://crates.io/equivalent/1.0.2 \
    crate://crates.io/errno/0.3.14 \
    crate://crates.io/fastrand/2.3.0 \
    crate://crates.io/foldhash/0.1.5 \
    crate://crates.io/fxhash/0.2.1 \
    crate://crates.io/getrandom/0.4.2 \
    crate://crates.io/hashbrown/0.15.5 \
    crate://crates.io/hashbrown/0.16.1 \
    crate://crates.io/heck/0.5.0 \
    crate://crates.io/id-arena/2.3.0 \
    crate://crates.io/indexmap/2.13.0 \
    crate://crates.io/is_terminal_polyfill/1.70.2 \
    crate://crates.io/itoa/1.0.18 \
    crate://crates.io/leb128fmt/0.1.0 \
    crate://crates.io/libc/0.2.183 \
    crate://crates.io/linux-raw-sys/0.12.1 \
    crate://crates.io/lock_api/0.4.14 \
    crate://crates.io/log/0.4.29 \
    crate://crates.io/memchr/2.8.0 \
    crate://crates.io/num-traits/0.2.19 \
    crate://crates.io/once_cell/1.21.4 \
    crate://crates.io/once_cell_polyfill/1.70.2 \
    crate://crates.io/parking_lot/0.12.5 \
    crate://crates.io/parking_lot_core/0.9.12 \
    crate://crates.io/prettyplease/0.2.37 \
    crate://crates.io/proc-macro2/1.0.106 \
    crate://crates.io/quote/1.0.45 \
    crate://crates.io/r-efi/6.0.0 \
    crate://crates.io/redox_syscall/0.5.18 \
    crate://crates.io/rustix/1.1.4 \
    crate://crates.io/scopeguard/1.2.0 \
    crate://crates.io/semver/1.0.27 \
    crate://crates.io/serde/1.0.228 \
    crate://crates.io/serde_core/1.0.228 \
    crate://crates.io/serde_derive/1.0.228 \
    crate://crates.io/serde_json/1.0.149 \
    crate://crates.io/serde_spanned/0.6.9 \
    crate://crates.io/smallvec/1.15.1 \
    crate://crates.io/strsim/0.11.1 \
    crate://crates.io/syn/2.0.117 \
    crate://crates.io/tempfile/3.27.0 \
    crate://crates.io/thiserror/2.0.18 \
    crate://crates.io/thiserror-impl/2.0.18 \
    crate://crates.io/toml/0.8.23 \
    crate://crates.io/toml_datetime/0.6.11 \
    crate://crates.io/toml_edit/0.22.27 \
    crate://crates.io/toml_write/0.1.2 \
    crate://crates.io/unicode-ident/1.0.24 \
    crate://crates.io/unicode-xid/0.2.6 \
    crate://crates.io/utf8parse/0.2.2 \
    crate://crates.io/wasip2/1.0.2+wasi-0.2.9 \
    crate://crates.io/wasip3/0.4.0+wasi-0.3.0-rc-2026-01-06 \
    crate://crates.io/wasm-encoder/0.244.0 \
    crate://crates.io/wasm-metadata/0.244.0 \
    crate://crates.io/wasmparser/0.244.0 \
    crate://crates.io/windows-link/0.2.1 \
    crate://crates.io/windows-sys/0.61.2 \
    crate://crates.io/winnow/0.7.15 \
    crate://crates.io/wit-bindgen/0.51.0 \
    crate://crates.io/wit-bindgen-core/0.51.0 \
    crate://crates.io/wit-bindgen-rust/0.51.0 \
    crate://crates.io/wit-bindgen-rust-macro/0.51.0 \
    crate://crates.io/wit-component/0.244.0 \
    crate://crates.io/wit-parser/0.244.0 \
    crate://crates.io/zmij/1.0.21 \
"

S = "${WORKDIR}/cargo-arxml-${PV}"

inherit cargo

# cargo-arxml is a host tool (code generator), typically runs on the build
# machine rather than the target ECU.
BBCLASSEXTEND = "native nativesdk"

DEPENDS = ""
