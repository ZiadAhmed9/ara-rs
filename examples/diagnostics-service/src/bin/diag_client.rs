//! Diagnostics Service — proxy (client) side.
//!
//! Discovers the DiagnosticsService via SOME/IP-SD, calls all four methods,
//! and subscribes to DtcStatusChanged events.
//!
//! Start the server first:
//! ```
//! RUST_LOG=info cargo run -p diagnostics-service-example --bin diag_server
//! ```
//! Then run:
//! ```
//! RUST_LOG=info cargo run -p diagnostics-service-example --bin diag_client
//! ```

use std::net::Ipv4Addr;
use std::sync::Arc;

use tracing::info;

use ara_com::transport::{AraDeserialize, Transport};
use ara_com::types::{InstanceId, MajorVersion, MethodId, MinorVersion, ServiceId};

use ara_com_someip::config::{DiscoveryMode, SdConfig, SomeIpConfig};
use ara_com_someip::transport::SomeIpTransport;

use diagnostics_service_example::generated::proxy::diagnostics_service::DiagnosticsServiceProxy;

const SERVICE_ID: ServiceId = ServiceId(0x5000);
const INSTANCE_ID: InstanceId = InstanceId(0x0001);
const DTC_STATUS_CHANGED_EVENT: MethodId = MethodId(0x8001);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting DiagnosticsService client...");

    let config = SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0004,
        discovery_mode: DiscoveryMode::SomeIpSd,
        sd_config: SdConfig::default(),
        services: vec![],
        remote_services: vec![],
        udp_threshold: 1400,
    };

    let mut transport = SomeIpTransport::new(config);
    transport.bind().await?;

    // Subscribe to DtcStatusChanged notifications before finding the service
    let mut event_rx =
        transport.subscribe_notifications(SERVICE_ID, INSTANCE_ID, DTC_STATUS_CHANGED_EVENT, 64)?;

    let transport = Arc::new(transport);

    // Discover the service via SD
    info!("Searching for DiagnosticsService via SD...");
    let found = transport
        .find_service(SERVICE_ID, INSTANCE_ID, MajorVersion(1), MinorVersion(0))
        .await?;
    info!("Found service: {found}");

    // Create the typed proxy
    let proxy = DiagnosticsServiceProxy::new(transport.clone(), INSTANCE_ID);

    // --- ReadDtc: nested struct response ---
    let snapshot = proxy.read_dtc(0x001234).await?;
    info!(
        "ReadDtc(0x001234) -> DtcSnapshot {{ dtc_id=0x{:06X}, status=0x{:02X}, \
         timestamp={}, count={} }}",
        snapshot.dtc_id, snapshot.status_byte, snapshot.timestamp, snapshot.occurrence_count
    );

    // --- ClearDtc: fire-and-forget ---
    proxy.clear_dtc(0x001234).await?;
    info!("ClearDtc(0x001234) -> sent (fire-and-forget)");

    // --- ReadEcuIdentification: no input params, struct output ---
    let ecu = proxy.read_ecu_identification().await?;
    info!(
        "ReadEcuIdentification() -> EcuInfo {{ serial=0x{:08X}, hw=0x{:04X}, \
         sw=0x{:04X}, uptime={}s }}",
        ecu.serial_number, ecu.hardware_version, ecu.software_version, ecu.uptime_seconds
    );

    // --- ReadDataByIdentifier: multiple inputs, struct output ---
    let record = proxy.read_data_by_identifier(0xF190, 0x01).await?;
    info!(
        "ReadDataByIdentifier(0xF190, 0x01) -> DataRecord {{ id=0x{:04X}, \
         len={}, value=0x{:08X} }}",
        record.identifier, record.length, record.value
    );

    // --- Subscribe to DtcStatusChanged events ---
    proxy.subscribe_dtc_status_changed().await?;
    info!("Subscribed to DtcStatusChanged — listening for 10 seconds...");

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut count = 0;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, event_rx.recv()).await {
            Ok(Ok(payload)) => {
                let dtc_id = u32::ara_deserialize(&payload)?;
                count += 1;
                info!("DtcStatusChanged event #{count}: DTC 0x{dtc_id:06X}");
            }
            Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(n))) => {
                info!("Missed {n} events");
            }
            Ok(Err(tokio::sync::broadcast::error::RecvError::Closed)) => {
                info!("Event channel closed");
                break;
            }
            Err(_timeout) => break,
        }
    }

    info!("Received {count} DtcStatusChanged events. Done!");
    Ok(())
}
