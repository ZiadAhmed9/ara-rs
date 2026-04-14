//! Battery Service — proxy (client) side using generated code.
//!
//! Uses the generated `BatteryServiceProxy` from `cargo arxml generate`.
//! Discovers the service via SOME/IP-SD, calls typed methods, and subscribes
//! to `VoltageChanged` events.
//!
//! Start the server first:
//! ```
//! cargo run -p battery-service-example --bin server
//! ```
//! Then run:
//! ```
//! cargo run -p battery-service-example --bin client
//! ```

use std::net::Ipv4Addr;
use std::sync::Arc;

use tracing::info;

use ara_com::transport::{AraDeserialize, Transport};
use ara_com::types::{InstanceId, MajorVersion, MethodId, MinorVersion, ServiceId};

use ara_com_someip::config::{DiscoveryMode, SdConfig, SomeIpConfig};
use ara_com_someip::transport::SomeIpTransport;

use battery_service_example::generated::proxy::battery_service::BatteryServiceProxy;

const SERVICE_ID: ServiceId = ServiceId(0x4010);
const INSTANCE_ID: InstanceId = InstanceId(0x0001);
/// Event method ID for VoltageChanged (0x8001 from ARXML deployment).
const VOLTAGE_CHANGED_EVENT: MethodId = MethodId(0x8001);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting BatteryService client (SD discovery + events)...");

    let config = SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0002,
        discovery_mode: DiscoveryMode::SomeIpSd,
        sd_config: SdConfig::default(),
        services: vec![],
        remote_services: vec![],
        udp_threshold: 1400,
    };

    let mut transport = SomeIpTransport::new(config);
    transport.bind().await?;

    // Subscribe to VoltageChanged notifications BEFORE finding the service,
    // so the channel is ready when events start arriving.
    let mut event_rx =
        transport.subscribe_notifications(SERVICE_ID, INSTANCE_ID, VOLTAGE_CHANGED_EVENT, 64)?;

    let transport = Arc::new(transport);

    // Discover the service via SD
    info!("Searching for BatteryService via SD...");
    let found = transport
        .find_service(SERVICE_ID, INSTANCE_ID, MajorVersion(1), MinorVersion(0))
        .await?;
    info!("Found service: {found}");

    // Create the typed proxy
    let proxy = BatteryServiceProxy::new(transport.clone(), INSTANCE_ID);

    // --- Call GetVoltage for battery IDs 0-3 ---
    for battery_id in 0u8..4 {
        let voltage = proxy.get_voltage(battery_id).await?;
        info!("GetVoltage(battery_id={battery_id}) -> {voltage:.1}V");
    }

    // --- Fire-and-forget: SetChargeLimit ---
    proxy.set_charge_limit(32.0).await?;
    info!("SetChargeLimit(32.0A) -> sent (fire-and-forget)");

    // --- Subscribe to VoltageChanged events ---
    proxy.subscribe_voltage_changed().await?;
    info!("Subscribed to VoltageChanged events — listening for 10 seconds...");

    // Listen for events for 10 seconds
    let listen_deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut event_count = 0;
    loop {
        let remaining = listen_deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, event_rx.recv()).await {
            Ok(Ok(payload)) => {
                let voltage = f64::ara_deserialize(&payload)?;
                event_count += 1;
                info!("VoltageChanged event #{event_count}: {voltage:.1}V");
            }
            Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(n))) => {
                info!("Missed {n} events (slow consumer)");
            }
            Ok(Err(tokio::sync::broadcast::error::RecvError::Closed)) => {
                info!("Event channel closed");
                break;
            }
            Err(_timeout) => break,
        }
    }

    info!("Received {event_count} events. All done!");
    Ok(())
}
