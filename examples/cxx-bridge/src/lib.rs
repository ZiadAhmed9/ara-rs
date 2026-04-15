//! CXX bridge example — exposes BatteryService proxy methods to C++.
//!
//! Demonstrates C++ calling into Rust to invoke a typed SOME/IP proxy method.
//! The async transport is hidden behind synchronous wrappers since `cxx` does
//! not support async functions.
//!
//! The bridge has two sides:
//! - `extern "Rust"`: Rust functions callable from C++ (`connect_battery_service`, `get_voltage`)
//! - `extern "C++"`: C++ function callable from Rust (`run_cxx_client`)
//!
//! The binary entry point (`main.rs`) calls `run_cxx_client()` in C++, which
//! calls back into the Rust bridge functions — proving the C++ -> Rust path.

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use ara_com::types::{InstanceId, ServiceId};
use ara_com_someip::config::{
    DiscoveryMode, EndpointConfig, RemoteServiceConfig, SdConfig, SomeIpConfig,
};
use ara_com_someip::transport::SomeIpTransport;
use battery_service_example::generated::proxy::battery_service::BatteryServiceProxy;

/// Simple error wrapper that implements Display for cxx Result compatibility.
#[derive(Debug)]
pub struct BridgeError(String);

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn to_err(e: impl std::fmt::Display) -> BridgeError {
    BridgeError(e.to_string())
}

#[cxx::bridge(namespace = "ara_rs")]
pub mod ffi {
    extern "Rust" {
        /// Opaque handle to a connected BatteryService proxy.
        type BatteryClient;

        /// Connect to a BatteryService at the given host and port.
        /// Returns an opaque client handle.
        fn connect_battery_service(host: &str, port: u16) -> Result<Box<BatteryClient>>;

        /// Call GetVoltage for the given battery ID. Blocks until the
        /// response arrives.
        fn get_voltage(client: &BatteryClient, battery_id: u8) -> Result<f64>;
    }

    unsafe extern "C++" {
        include!("cxx-bridge-example/src/cxx_client.h");

        /// C++ function that calls into the Rust bridge to exercise the
        /// connect + get_voltage path. Returns 0 on success, non-zero on failure.
        fn run_cxx_client(host: &str, port: u16) -> i32;
    }
}

/// Opaque handle holding the tokio runtime and typed proxy.
pub struct BatteryClient {
    runtime: tokio::runtime::Runtime,
    proxy: BatteryServiceProxy<SomeIpTransport>,
}

pub fn connect_battery_service(host: &str, port: u16) -> Result<Box<BatteryClient>, BridgeError> {
    let rt = tokio::runtime::Runtime::new().map_err(to_err)?;

    let addr: Ipv4Addr = host.parse().map_err(to_err)?;

    let proxy = rt.block_on(async {
        let config = SomeIpConfig {
            unicast: Ipv4Addr::LOCALHOST,
            client_id: 0x0099,
            discovery_mode: DiscoveryMode::Static,
            sd_config: SdConfig::default(),
            services: vec![],
            remote_services: vec![RemoteServiceConfig {
                service_id: ServiceId(0x4010),
                instance_id: InstanceId(0x0001),
                endpoint: EndpointConfig {
                    udp: Some(SocketAddrV4::new(addr, port)),
                    tcp: None,
                    udp_threshold: 1400,
                },
            }],
            udp_threshold: 1400,
        };

        let mut transport = SomeIpTransport::new(config);
        transport.bind().await.map_err(to_err)?;

        let transport = Arc::new(transport);
        let proxy = BatteryServiceProxy::new(transport, InstanceId(0x0001));
        Ok::<_, BridgeError>(proxy)
    })?;

    Ok(Box::new(BatteryClient { runtime: rt, proxy }))
}

pub fn get_voltage(client: &BatteryClient, battery_id: u8) -> Result<f64, BridgeError> {
    client
        .runtime
        .block_on(client.proxy.get_voltage(battery_id))
        .map_err(to_err)
}
