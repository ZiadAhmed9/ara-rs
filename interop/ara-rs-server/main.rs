//! Interop server — BatteryService skeleton for Docker-based vsomeip testing.
//!
//! Reads the advertised address from `ARA_RS_UNICAST` (default: 172.20.0.10).
//! This is the routable IP embedded in SD offers so peers can reach us.
//! The socket itself always binds to 0.0.0.0 to accept traffic on any interface.

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

const SERVICE_ID: ServiceId = ServiceId(0x4010);
const INSTANCE_ID: InstanceId = InstanceId(0x0001);
const SERVER_PORT: u16 = 30509;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // The unicast address advertised in SD offers — must be routable by peers.
    // The socket binds to 0.0.0.0 regardless (handled by SomeIpTransport::bind).
    let advertise_ip: Ipv4Addr = std::env::var("ARA_RS_UNICAST")
        .unwrap_or_else(|_| "172.20.0.10".to_string())
        .parse()
        .expect("ARA_RS_UNICAST must be a valid IPv4 address");

    info!("Starting BatteryService interop server (advertise={advertise_ip}, port={SERVER_PORT})...");

    let config = SomeIpConfig {
        // unicast is the address embedded in SD offers for peer discovery
        unicast: advertise_ip,
        client_id: 0x0001,
        discovery_mode: DiscoveryMode::SomeIpSd,
        sd_config: SdConfig::default(),
        services: vec![ServiceConfig {
            service_id: SERVICE_ID,
            instance_id: INSTANCE_ID,
            endpoint: EndpointConfig {
                // The endpoint address in SD — must match the advertised IP
                udp: Some(SocketAddrV4::new(advertise_ip, SERVER_PORT)),
                tcp: None,
                udp_threshold: 1400,
            },
            event_groups: vec![EventGroupConfig {
                event_group_id: EventGroupId(1),
                multicast: None,
                events: vec![EventId(0x8001)],
            }],
        }],
        remote_services: vec![],
        udp_threshold: 1400,
    };

    let mut transport = SomeIpTransport::new(config);
    transport.bind().await?;

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
                            1 => {
                                let req = GetVoltageRequest::ara_deserialize(&payload)?;
                                let voltage = 12.6 + req.battery_id as f64 * 0.1;
                                info!("GetVoltage(battery_id={}) -> {voltage:.1}V", req.battery_id);
                                let resp = GetVoltageResponse { voltage };
                                let mut buf = Vec::new();
                                resp.ara_serialize(&mut buf)?;
                                Ok(Bytes::from(buf))
                            }
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

    let transport = Arc::new(transport);
    let skeleton = BatteryServiceSkeleton::new(transport.clone(), INSTANCE_ID);
    skeleton.offer().await?;

    info!("BatteryService offered via SD — waiting for interop requests...");

    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    skeleton.stop_offer().await?;

    Ok(())
}
