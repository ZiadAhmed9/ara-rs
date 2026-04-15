/**
 * vsomeip interop client — calls BatteryService::GetVoltage on an ara-rs server.
 *
 * Service: 0x4010, Instance: 0x0001, Method: 0x0001 (GetVoltage)
 * Request payload:  1 byte  — battery_id (u8)
 * Response payload: 8 bytes — voltage (f64, big-endian IEEE 754)
 *
 * Build: see Dockerfile or compile with:
 *   g++ -std=c++17 -o vsomeip_client vsomeip_client.cpp \
 *       $(pkg-config --cflags --libs vsomeip3)
 */

#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <condition_variable>
#include <mutex>
#include <thread>

#include <vsomeip/vsomeip.hpp>

static constexpr vsomeip::service_t SERVICE_ID   = 0x4010;
static constexpr vsomeip::instance_t INSTANCE_ID = 0x0001;
static constexpr vsomeip::method_t METHOD_ID     = 0x0001;  // GetVoltage
static constexpr vsomeip::major_version_t MAJOR   = 0x01;
static constexpr vsomeip::minor_version_t MINOR   = 0x00;

static std::shared_ptr<vsomeip::application> app;
static std::mutex mtx;
static std::condition_variable cv;
static bool service_available = false;
static bool response_received = false;
static bool interop_success = false;

static double decode_f64_be(const uint8_t* buf) {
    // Big-endian IEEE 754 f64
    uint64_t raw = 0;
    for (int i = 0; i < 8; ++i)
        raw = (raw << 8) | buf[i];
    double result;
    std::memcpy(&result, &raw, sizeof(result));
    return result;
}

static void on_availability(vsomeip::service_t, vsomeip::instance_t, bool available) {
    std::cout << "[vsomeip-client] Service 0x" << std::hex << SERVICE_ID
              << " is " << (available ? "AVAILABLE" : "UNAVAILABLE") << std::endl;
    if (available) {
        std::lock_guard<std::mutex> lk(mtx);
        service_available = true;
        cv.notify_one();
    }
}

static void on_response(const std::shared_ptr<vsomeip::message>& msg) {
    auto payload = msg->get_payload();
    if (!payload || payload->get_length() < 8) {
        std::cerr << "[vsomeip-client] ERROR: response payload too short ("
                  << (payload ? payload->get_length() : 0) << " bytes)" << std::endl;
        std::lock_guard<std::mutex> lk(mtx);
        response_received = true;
        cv.notify_one();
        return;
    }

    double voltage = decode_f64_be(payload->get_data());
    std::cout << "[vsomeip-client] GetVoltage response: " << std::fixed
              << std::setprecision(1) << voltage << "V" << std::endl;

    // Validate: the battery service returns 12.6 + battery_id * 0.1
    // We sent battery_id=1, so expect ~12.7
    if (voltage > 12.6 && voltage < 12.8) {
        std::cout << "[vsomeip-client] INTEROP SUCCESS — voltage matches expected value"
                  << std::endl;
        interop_success = true;
    } else {
        std::cerr << "[vsomeip-client] INTEROP FAILURE — unexpected voltage: "
                  << voltage << std::endl;
    }

    std::lock_guard<std::mutex> lk(mtx);
    response_received = true;
    cv.notify_one();
}

int main() {
    app = vsomeip::runtime::get()->create_application("vsomeip-client");
    if (!app->init()) {
        std::cerr << "[vsomeip-client] Failed to init vsomeip application" << std::endl;
        return 1;
    }

    app->register_availability_handler(SERVICE_ID, INSTANCE_ID, on_availability);
    app->register_message_handler(SERVICE_ID, INSTANCE_ID, METHOD_ID, on_response);
    app->request_service(SERVICE_ID, INSTANCE_ID, MAJOR, MINOR);

    // Run vsomeip in background thread
    std::thread runner([&]() { app->start(); });

    // Wait for service availability (timeout 15s)
    {
        std::unique_lock<std::mutex> lk(mtx);
        if (!cv.wait_for(lk, std::chrono::seconds(15), [] { return service_available; })) {
            std::cerr << "[vsomeip-client] Timeout waiting for service" << std::endl;
            app->stop();
            runner.join();
            return 1;
        }
    }

    // Send GetVoltage request: battery_id = 1
    auto request = vsomeip::runtime::get()->create_request();
    request->set_service(SERVICE_ID);
    request->set_instance(INSTANCE_ID);
    request->set_method(METHOD_ID);
    request->set_interface_version(MAJOR);

    auto payload = vsomeip::runtime::get()->create_payload();
    uint8_t data[] = { 0x01 };  // battery_id = 1
    payload->set_data(data, sizeof(data));
    request->set_payload(payload);

    std::cout << "[vsomeip-client] Sending GetVoltage(battery_id=1)..." << std::endl;
    app->send(request);

    // Wait for response (timeout 10s)
    {
        std::unique_lock<std::mutex> lk(mtx);
        if (!cv.wait_for(lk, std::chrono::seconds(10), [] { return response_received; })) {
            std::cerr << "[vsomeip-client] Timeout waiting for response" << std::endl;
            app->stop();
            runner.join();
            return 1;
        }
    }

    app->stop();
    runner.join();

    return interop_success ? 0 : 1;
}
