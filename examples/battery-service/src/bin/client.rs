//! Battery Service — proxy (client) side using generated code.
//!
//! Uses the generated `BatteryServiceProxy` from `cargo arxml generate`.
//! The proxy provides typed methods: `get_voltage(battery_id) -> f64`
//! and `set_charge_limit(limit)`.
//!
//! Start the server first:
//! ```
//! cargo run -p battery-service-example --bin server
//! ```
//! Then run:
//! ```
//! cargo run -p battery-service-example --bin client
//! ```

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use tracing::info;

use ara_com::transport::Transport;
use ara_com::types::{InstanceId, MajorVersion, MinorVersion};

use ara_com_someip::config::{
    EndpointConfig, RemoteServiceConfig, SdConfig, SomeIpConfig,
};
use ara_com_someip::transport::SomeIpTransport;

use battery_service_example::generated::proxy::BatteryServiceProxy;

const SERVER_PORT: u16 = 30509;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting BatteryService client (using generated proxy)...");

    let config = SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0002,
        discovery_mode: Default::default(),
        sd_config: SdConfig::default(),
        services: vec![],
        remote_services: vec![RemoteServiceConfig {
            service_id: ara_com::types::ServiceId(0x1000),
            instance_id: InstanceId(0x0001),
            endpoint: EndpointConfig {
                udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, SERVER_PORT)),
                tcp: None,
                udp_threshold: 1400,
            },
        }],
    };

    let mut transport = SomeIpTransport::new(config);
    transport.bind().await?;
    let transport = Arc::new(transport);

    // Find the service (static mode)
    let found = transport
        .find_service(
            ara_com::types::ServiceId(0x1000),
            InstanceId(0x0001),
            MajorVersion(1),
            MinorVersion(0),
        )
        .await?;
    info!("Found service: {found}");

    // Create the typed proxy — no manual header construction needed!
    let proxy = BatteryServiceProxy::new(transport, InstanceId(0x0001));

    // --- Call GetVoltage for battery IDs 0–3 ---
    for battery_id in 0u8..4 {
        let voltage = proxy.get_voltage(battery_id).await?;
        info!("GetVoltage(battery_id={battery_id}) → {voltage:.1}V");
    }

    // --- Fire-and-forget: SetChargeLimit ---
    proxy.set_charge_limit(32.0).await?;
    info!("SetChargeLimit(32.0A) → sent (fire-and-forget)");

    info!("All calls completed successfully!");
    Ok(())
}
