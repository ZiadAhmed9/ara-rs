//! Battery Service — skeleton (server) side using generated code.
//!
//! Uses the generated types (`GetVoltageRequest`, `GetVoltageResponse`, etc.)
//! from `cargo arxml generate` for request/response serialization.
//!
//! Offers the service via SOME/IP-SD and periodically publishes
//! `VoltageChanged` events to all subscribers.
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
use ara_com::types::{EventGroupId, EventId, InstanceId, ServiceId};

use ara_com_someip::config::{
    DiscoveryMode, EndpointConfig, EventGroupConfig, SdConfig, ServiceConfig, SomeIpConfig,
};
use ara_com_someip::transport::SomeIpTransport;

use battery_service_example::generated::proxy::battery_service::{
    GetVoltageRequest, GetVoltageResponse, SetChargeLimitRequest,
};
use battery_service_example::generated::skeleton::battery_service::BatteryServiceSkeleton;

/// Service ID 0x4010 — from SOMEIP-SERVICE-INTERFACE-DEPLOYMENT in battery_service.arxml.
const SERVICE_ID: ServiceId = ServiceId(0x4010);
const INSTANCE_ID: InstanceId = InstanceId(0x0001);
const SERVER_PORT: u16 = 30509;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting BatteryService skeleton on port {SERVER_PORT}...");

    let config = SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0001,
        discovery_mode: DiscoveryMode::SomeIpSd,
        sd_config: SdConfig::default(),
        services: vec![ServiceConfig {
            service_id: SERVICE_ID,
            instance_id: INSTANCE_ID,
            endpoint: EndpointConfig {
                udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, SERVER_PORT)),
                tcp: None,
                udp_threshold: 1400,
            },
            event_groups: vec![EventGroupConfig {
                event_group_id: EventGroupId(1),
                multicast: None,
                // VoltageChanged event (method_id 0x8001 on the wire)
                events: vec![EventId(0x8001)],
            }],
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
                |header: MessageHeader,
                 payload: Bytes|
                 -> BoxFuture<'static, Result<Bytes, AraComError>> {
                    Box::pin(async move {
                        match header.method_id.0 {
                            // MethodId(1) = GetVoltage
                            1 => {
                                let req = GetVoltageRequest::ara_deserialize(&payload)?;
                                let voltage = 12.6 + req.battery_id as f64 * 0.1;
                                info!("GetVoltage(battery_id={}) -> {voltage:.1}V", req.battery_id);
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

    // Offer the service using the generated skeleton (now via SD)
    let transport = Arc::new(transport);
    let skeleton = BatteryServiceSkeleton::new(transport.clone(), INSTANCE_ID);
    skeleton.offer().await?;

    info!(
        "BatteryService offered via SD on {}:{} — waiting for requests...",
        Ipv4Addr::LOCALHOST,
        SERVER_PORT
    );

    // Periodically publish VoltageChanged events
    let skeleton_for_events = BatteryServiceSkeleton::new(transport, INSTANCE_ID);
    tokio::spawn(async move {
        let mut voltage = 12.6_f64;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            voltage += 0.1;
            if voltage > 14.0 {
                voltage = 12.0;
            }
            info!("Publishing VoltageChanged event: {voltage:.1}V");
            if let Err(e) = skeleton_for_events.notify_voltage_changed(&voltage).await {
                warn!("Failed to publish event: {e}");
            }
        }
    });

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    skeleton.stop_offer().await?;

    Ok(())
}
