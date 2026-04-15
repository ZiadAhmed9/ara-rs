/**
 * C++ implementation of run_cxx_client — calls into the Rust bridge
 * to connect to a BatteryService and invoke GetVoltage.
 *
 * This code is compiled by cxx_build and linked into the Rust binary.
 * The Rust main.rs calls run_cxx_client(), which calls back into Rust
 * via connect_battery_service() and get_voltage() — proving the
 * C++ -> Rust bridge path end to end.
 */

#include "cxx-bridge-example/src/lib.rs.h"
#include "cxx-bridge-example/src/cxx_client.h"
#include <iostream>
#include <iomanip>

namespace ara_rs {

int32_t run_cxx_client(rust::Str host, uint16_t port) {
    std::cout << "[cxx-client] C++ calling into Rust bridge..." << std::endl;
    std::cout << "[cxx-client] Connecting to BatteryService at "
              << std::string(host) << ":" << port << std::endl;

    try {
        auto client = connect_battery_service(host, port);

        for (uint8_t id = 0; id < 4; ++id) {
            double voltage = get_voltage(*client, id);
            std::cout << "[cxx-client] GetVoltage(battery_id="
                      << static_cast<int>(id) << ") -> "
                      << std::fixed << std::setprecision(1)
                      << voltage << "V" << std::endl;
        }

        std::cout << "[cxx-client] CXX BRIDGE SUCCESS" << std::endl;
        return 0;

    } catch (const std::exception& e) {
        std::cerr << "[cxx-client] Error: " << e.what() << std::endl;
        return 1;
    }
}

} // namespace ara_rs
