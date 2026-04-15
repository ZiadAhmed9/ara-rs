#pragma once
#include "rust/cxx.h"

namespace ara_rs {

/// Run the C++ client demo: connect to a BatteryService via the Rust
/// bridge and call GetVoltage for battery IDs 0-3.
/// Returns 0 on success, non-zero on failure.
int32_t run_cxx_client(rust::Str host, uint16_t port);

} // namespace ara_rs
