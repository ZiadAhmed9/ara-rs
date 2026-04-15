//! CXX bridge demo — Rust entry point that calls into C++.
//!
//! The C++ `run_cxx_client()` function calls back into Rust through the
//! `#[cxx::bridge]` to connect to a BatteryService and invoke GetVoltage.
//! This proves the full C++ -> Rust bridge path.
//!
//! Run with a battery service server already listening:
//! ```
//! RUST_LOG=info cargo run -p battery-service-example --bin server &
//! cargo run -p cxx-bridge-example
//! ```

fn main() {
    tracing_subscriber::fmt::init();

    let host = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let port: u16 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(30509);

    // Call into C++, which calls back into Rust via the bridge
    let result = cxx_bridge_example::ffi::run_cxx_client(&host, port);

    std::process::exit(result);
}
