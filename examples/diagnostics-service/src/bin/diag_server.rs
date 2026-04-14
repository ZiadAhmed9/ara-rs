//! Diagnostics Service — skeleton (server) side.
//!
//! Demonstrates richer types than the battery-service example:
//! - Multiple methods (ReadDtc, ClearDtc, ReadEcuIdentification, ReadDataByIdentifier)
//! - Nested struct responses (DtcSnapshot, EcuInfo, DataRecord)
//! - Multiple input parameters (ReadDataByIdentifier)
//! - Fire-and-forget (ClearDtc)
//! - Two event groups (DtcStatusChanged, SessionChanged)
//!
//! ```
//! RUST_LOG=info cargo run -p diagnostics-service-example --bin diag_server
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

use diagnostics_service_example::generated::proxy::diagnostics_service::{
    ClearDtcRequest, ReadDataByIdentifierRequest, ReadDataByIdentifierResponse, ReadDtcRequest,
    ReadDtcResponse, ReadEcuIdentificationResponse,
};
use diagnostics_service_example::generated::skeleton::diagnostics_service::DiagnosticsServiceSkeleton;
use diagnostics_service_example::generated::types::{DataRecord, DtcSnapshot, EcuInfo};

/// Service ID 0x5000 (20480) — from diagnostics_service.arxml deployment.
const SERVICE_ID: ServiceId = ServiceId(0x5000);
const INSTANCE_ID: InstanceId = InstanceId(0x0001);
const SERVER_PORT: u16 = 30510;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting DiagnosticsService skeleton on port {SERVER_PORT}...");

    let config = SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0003,
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
            event_groups: vec![
                EventGroupConfig {
                    event_group_id: EventGroupId(1),
                    multicast: None,
                    events: vec![EventId(0x8001)], // DtcStatusChanged
                },
                EventGroupConfig {
                    event_group_id: EventGroupId(2),
                    multicast: None,
                    events: vec![EventId(0x8002)], // SessionChanged
                },
            ],
        }],
        remote_services: vec![],
        udp_threshold: 1400,
    };

    let mut transport = SomeIpTransport::new(config);
    transport.bind().await?;

    // Register the request handler
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
                            // MethodId(1) = ReadDtc
                            1 => {
                                let req = ReadDtcRequest::ara_deserialize(&payload)?;
                                info!("ReadDtc(dtc_id=0x{:06X})", req.dtc_id);
                                let resp = ReadDtcResponse {
                                    snapshot: DtcSnapshot {
                                        dtc_id: req.dtc_id,
                                        status_byte: 0x2F, // active + confirmed
                                        timestamp: 1700000000,
                                        occurrence_count: 3,
                                    },
                                };
                                let mut buf = Vec::new();
                                resp.ara_serialize(&mut buf)?;
                                Ok(Bytes::from(buf))
                            }
                            // MethodId(2) = ClearDtc (fire-and-forget)
                            2 => {
                                let req = ClearDtcRequest::ara_deserialize(&payload)?;
                                info!("ClearDtc(dtc_id=0x{:06X}) — cleared", req.dtc_id);
                                Ok(Bytes::new())
                            }
                            // MethodId(3) = ReadEcuIdentification
                            3 => {
                                info!("ReadEcuIdentification()");
                                let resp = ReadEcuIdentificationResponse {
                                    info: EcuInfo {
                                        serial_number: 0x00ABCDEF,
                                        hardware_version: 0x0201,
                                        software_version: 0x0310,
                                        uptime_seconds: 86400,
                                    },
                                };
                                let mut buf = Vec::new();
                                resp.ara_serialize(&mut buf)?;
                                Ok(Bytes::from(buf))
                            }
                            // MethodId(4) = ReadDataByIdentifier
                            4 => {
                                let req = ReadDataByIdentifierRequest::ara_deserialize(&payload)?;
                                info!(
                                    "ReadDataByIdentifier(id=0x{:04X}, sub=0x{:02X})",
                                    req.identifier, req.sub_function
                                );
                                let resp = ReadDataByIdentifierResponse {
                                    data: DataRecord {
                                        identifier: req.identifier,
                                        length: 4,
                                        value: 0xDEADBEEF,
                                    },
                                };
                                let mut buf = Vec::new();
                                resp.ara_serialize(&mut buf)?;
                                Ok(Bytes::from(buf))
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
    let skeleton = DiagnosticsServiceSkeleton::new(transport.clone(), INSTANCE_ID);
    skeleton.offer().await?;

    info!("DiagnosticsService offered via SD — waiting for requests...");

    // Periodically publish DtcStatusChanged events
    let event_skeleton = DiagnosticsServiceSkeleton::new(transport, INSTANCE_ID);
    tokio::spawn(async move {
        let mut dtc_id = 0x001234u32;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            info!("Publishing DtcStatusChanged event: DTC 0x{dtc_id:06X}");
            if let Err(e) = event_skeleton.notify_dtc_status_changed(&dtc_id).await {
                warn!("Failed to publish DtcStatusChanged: {e}");
            }
            dtc_id += 1;
        }
    });

    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    skeleton.stop_offer().await?;

    Ok(())
}
