SUMMARY = "Core traits and async abstractions for Adaptive AUTOSAR communication in Rust"
HOMEPAGE = "https://github.com/ZiadAhmed9/ara-rs"
LICENSE = "MIT | Apache-2.0"
LIC_FILES_CHKSUM = " \
    file://LICENSE-MIT;md5=b278a92d2c1509760384428817710378 \
    file://LICENSE-APACHE;md5=34400b68072d710fecd0a2940a0d1658 \
"

SRC_URI += " \
    crate://crates.io/ara-com/${PV} \
    crate://crates.io/async-trait/0.1.89 \
    crate://crates.io/bytes/1.11.1 \
    crate://crates.io/errno/0.3.14 \
    crate://crates.io/futures-core/0.3.32 \
    crate://crates.io/libc/0.2.183 \
    crate://crates.io/mio/1.2.0 \
    crate://crates.io/pin-project-lite/0.2.17 \
    crate://crates.io/proc-macro2/1.0.106 \
    crate://crates.io/quote/1.0.45 \
    crate://crates.io/signal-hook-registry/1.4.8 \
    crate://crates.io/socket2/0.6.3 \
    crate://crates.io/syn/2.0.117 \
    crate://crates.io/thiserror/2.0.18 \
    crate://crates.io/thiserror-impl/2.0.18 \
    crate://crates.io/tokio/1.50.0 \
    crate://crates.io/tokio-macros/2.6.1 \
    crate://crates.io/unicode-ident/1.0.24 \
    crate://crates.io/wasi/0.11.1+wasi-snapshot-preview1 \
    crate://crates.io/windows-link/0.2.1 \
    crate://crates.io/windows-sys/0.61.2 \
"

S = "${WORKDIR}/ara-com-${PV}"

inherit cargo

CARGO_BUILD_FLAGS += "--lib"

# ara-com is a Rust library crate consumed as a build dependency by
# downstream service binaries. It is not a standalone deployable artifact.
# Use this recipe as a DEPENDS entry in application recipes that need it.
DEPENDS = ""
