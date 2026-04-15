SUMMARY = "SOME/IP transport backend for ara-com Adaptive AUTOSAR communication"
HOMEPAGE = "https://github.com/ZiadAhmed9/ara-rs"
LICENSE = "MIT | Apache-2.0"
LIC_FILES_CHKSUM = " \
    file://LICENSE-MIT;md5=b278a92d2c1509760384428817710378 \
    file://LICENSE-APACHE;md5=34400b68072d710fecd0a2940a0d1658 \
"

SRC_URI += " \
    crate://crates.io/ara-com-someip/${PV} \
    crate://crates.io/ara-com/${PV} \
    crate://crates.io/async-trait/0.1.89 \
    crate://crates.io/bitflags/2.11.0 \
    crate://crates.io/bytes/1.11.1 \
    crate://crates.io/cfg-if/1.0.4 \
    crate://crates.io/crossbeam-utils/0.8.21 \
    crate://crates.io/dashmap/6.1.0 \
    crate://crates.io/errno/0.3.14 \
    crate://crates.io/futures-core/0.3.32 \
    crate://crates.io/futures-sink/0.3.32 \
    crate://crates.io/hashbrown/0.14.5 \
    crate://crates.io/lazy_static/1.5.0 \
    crate://crates.io/libc/0.2.183 \
    crate://crates.io/lock_api/0.4.14 \
    crate://crates.io/log/0.4.29 \
    crate://crates.io/mio/1.2.0 \
    crate://crates.io/nu-ansi-term/0.50.3 \
    crate://crates.io/once_cell/1.21.4 \
    crate://crates.io/parking_lot_core/0.9.12 \
    crate://crates.io/pin-project-lite/0.2.17 \
    crate://crates.io/proc-macro2/1.0.106 \
    crate://crates.io/quote/1.0.45 \
    crate://crates.io/redox_syscall/0.5.18 \
    crate://crates.io/scopeguard/1.2.0 \
    crate://crates.io/sharded-slab/0.1.7 \
    crate://crates.io/signal-hook-registry/1.4.8 \
    crate://crates.io/smallvec/1.15.1 \
    crate://crates.io/socket2/0.5.10 \
    crate://crates.io/socket2/0.6.3 \
    crate://crates.io/someip_parse/0.6.2 \
    crate://crates.io/syn/2.0.117 \
    crate://crates.io/thiserror/2.0.18 \
    crate://crates.io/thiserror-impl/2.0.18 \
    crate://crates.io/thread_local/1.1.9 \
    crate://crates.io/tokio/1.50.0 \
    crate://crates.io/tokio-macros/2.6.1 \
    crate://crates.io/tokio-util/0.7.18 \
    crate://crates.io/tracing/0.1.44 \
    crate://crates.io/tracing-attributes/0.1.31 \
    crate://crates.io/tracing-core/0.1.36 \
    crate://crates.io/tracing-log/0.2.0 \
    crate://crates.io/tracing-subscriber/0.3.23 \
    crate://crates.io/unicode-ident/1.0.24 \
    crate://crates.io/valuable/0.1.1 \
    crate://crates.io/wasi/0.11.1+wasi-snapshot-preview1 \
    crate://crates.io/windows-link/0.2.1 \
    crate://crates.io/windows-sys/0.52.0 \
    crate://crates.io/windows-sys/0.61.2 \
    crate://crates.io/windows-targets/0.52.6 \
    crate://crates.io/windows_aarch64_gnullvm/0.52.6 \
    crate://crates.io/windows_aarch64_msvc/0.52.6 \
    crate://crates.io/windows_i686_gnu/0.52.6 \
    crate://crates.io/windows_i686_gnullvm/0.52.6 \
    crate://crates.io/windows_i686_msvc/0.52.6 \
    crate://crates.io/windows_x86_64_gnu/0.52.6 \
    crate://crates.io/windows_x86_64_gnullvm/0.52.6 \
    crate://crates.io/windows_x86_64_msvc/0.52.6 \
"

S = "${WORKDIR}/ara-com-someip-${PV}"

inherit cargo

CARGO_BUILD_FLAGS += "--lib"

# ara-com-someip is a Rust library crate consumed as a build dependency by
# downstream service binaries. It is not a standalone deployable artifact.
# Use this recipe as a DEPENDS entry in application recipes that need it.
DEPENDS = "ara-com"
