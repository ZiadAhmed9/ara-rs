//! Battery Service — skeleton (server) side.
//!
//! Offers a SOME/IP service on UDP that handles:
//!   - GetVoltage(battery_id: u8) → f64
//!   - SetChargeLimit(limit: f64) → () (fire-and-forget)
//!
//! Run this first, then start the client.
//!
//! ```
//! cargo run -p battery-service-example --bin server
//! ```

use std::net::{Ipv4Addr, SocketAddrV4};
use bytes::Bytes;
use futures_core::future::BoxFuture;
use tracing::{info, warn};

use ara_com::error::AraComError;
use ara_com::transport::{AraDeserialize, AraSerialize, MessageHeader, Transport};
use ara_com::types::{InstanceId, MajorVersion, MethodId, MinorVersion, ServiceId};

use ara_com_someip::config::{EndpointConfig, SdConfig, ServiceConfig, SomeIpConfig};
use ara_com_someip::transport::SomeIpTransport;

/// BatteryService IDs — would come from ARXML via cargo-arxml in production.
const SERVICE_ID: ServiceId = ServiceId(0x1234);
const INSTANCE_ID: InstanceId = InstanceId(0x0001);
const METHOD_GET_VOLTAGE: MethodId = MethodId(0x0001);
const METHOD_SET_CHARGE_LIMIT: MethodId = MethodId(0x0002);
const SERVER_PORT: u16 = 30509;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting BatteryService skeleton on port {SERVER_PORT}...");

    let config = SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0001,
        discovery_mode: Default::default(), // Static
        sd_config: SdConfig::default(),
        services: vec![ServiceConfig {
            service_id: SERVICE_ID,
            instance_id: INSTANCE_ID,
            endpoint: EndpointConfig {
                udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, SERVER_PORT)),
                tcp: None,
                udp_threshold: 1400,
            },
            event_groups: vec![],
        }],
        remote_services: vec![],
    };

    let mut transport = SomeIpTransport::new(config);
    transport.bind().await?;

    // Register the request handler
    transport
        .register_request_handler(
            SERVICE_ID,
            INSTANCE_ID,
            Box::new(
                |header: MessageHeader, payload: Bytes| -> BoxFuture<'static, Result<Bytes, AraComError>> {
                    Box::pin(async move {
                        match header.method_id {
                            METHOD_GET_VOLTAGE => handle_get_voltage(payload),
                            METHOD_SET_CHARGE_LIMIT => handle_set_charge_limit(payload),
                            other => {
                                warn!("Unknown method: {other}");
                                Err(AraComError::Transport {
                                    message: format!("unknown method: {other}"),
                                })
                            }
                        }
                    })
                },
            ),
        )
        .await?;

    // Offer the service
    transport
        .offer_service(SERVICE_ID, INSTANCE_ID, MajorVersion(1), MinorVersion(0))
        .await?;

    info!(
        "BatteryService offered on {}:{} — waiting for requests...",
        Ipv4Addr::LOCALHOST,
        SERVER_PORT
    );

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    transport
        .stop_offer_service(SERVICE_ID, INSTANCE_ID)
        .await?;

    Ok(())
}

fn handle_get_voltage(payload: Bytes) -> Result<Bytes, AraComError> {
    let battery_id = u8::ara_deserialize(&payload)?;
    let voltage = 12.6 + battery_id as f64 * 0.1;
    info!("GetVoltage(battery_id={battery_id}) → {voltage:.1}V");

    let mut buf = Vec::new();
    voltage.ara_serialize(&mut buf)?;
    Ok(Bytes::from(buf))
}

fn handle_set_charge_limit(payload: Bytes) -> Result<Bytes, AraComError> {
    let limit = f64::ara_deserialize(&payload)?;
    info!("SetChargeLimit(limit={limit:.1}A) — applied");
    Ok(Bytes::new())
}
