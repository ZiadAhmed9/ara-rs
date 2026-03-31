//! Battery Service — skeleton (server) side using generated code.
//!
//! Uses the generated types (`GetVoltageRequest`, `GetVoltageResponse`, etc.)
//! from `cargo arxml generate` for request/response serialization.
//!
//! ```
//! cargo run -p battery-service-example --bin server
//! ```

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use bytes::Bytes;
use futures_core::future::BoxFuture;
use tracing::{info, warn};

use ara_com::error::AraComError;
use ara_com::transport::{AraDeserialize, AraSerialize, MessageHeader, Transport};
use ara_com::types::{InstanceId, ServiceId};

use ara_com_someip::config::{EndpointConfig, SdConfig, ServiceConfig, SomeIpConfig};
use ara_com_someip::transport::SomeIpTransport;

use battery_service_example::generated::proxy::{
    GetVoltageRequest, GetVoltageResponse, SetChargeLimitRequest,
};
use battery_service_example::generated::skeleton::BatteryServiceSkeleton;

/// Service ID 0x1000 — auto-assigned by cargo-arxml codegen.
const SERVICE_ID: ServiceId = ServiceId(0x1000);
const INSTANCE_ID: InstanceId = InstanceId(0x0001);
const SERVER_PORT: u16 = 30509;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting BatteryService skeleton on port {SERVER_PORT}...");

    let config = SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0001,
        discovery_mode: Default::default(),
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

    // Register the request handler — dispatches by method_id
    transport
        .register_request_handler(
            SERVICE_ID,
            INSTANCE_ID,
            Box::new(
                |header: MessageHeader, payload: Bytes| -> BoxFuture<'static, Result<Bytes, AraComError>> {
                    Box::pin(async move {
                        match header.method_id.0 {
                            // MethodId(1) = GetVoltage
                            1 => {
                                let req = GetVoltageRequest::ara_deserialize(&payload)?;
                                let voltage = 12.6 + req.battery_id as f64 * 0.1;
                                info!("GetVoltage(battery_id={}) → {voltage:.1}V", req.battery_id);
                                let resp = GetVoltageResponse { voltage };
                                let mut buf = Vec::new();
                                resp.ara_serialize(&mut buf)?;
                                Ok(Bytes::from(buf))
                            }
                            // MethodId(2) = SetChargeLimit
                            2 => {
                                let req = SetChargeLimitRequest::ara_deserialize(&payload)?;
                                info!("SetChargeLimit(limit={:.1}A) — applied", req.limit);
                                Ok(Bytes::new())
                            }
                            other => {
                                warn!("Unknown method_id: {other}");
                                Err(AraComError::Transport {
                                    message: format!("unknown method_id: {other}"),
                                })
                            }
                        }
                    })
                },
            ),
        )
        .await?;

    // Offer the service using the generated skeleton
    let transport = Arc::new(transport);
    let skeleton = BatteryServiceSkeleton::new(transport, INSTANCE_ID);
    skeleton.offer().await?;

    info!(
        "BatteryService offered on {}:{} — waiting for requests...",
        Ipv4Addr::LOCALHOST,
        SERVER_PORT
    );

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    skeleton.stop_offer().await?;

    Ok(())
}
