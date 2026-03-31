//! Battery Service — proxy (client) side.
//!
//! Connects to the skeleton on localhost:30509 and calls:
//!   - GetVoltage for battery IDs 0–3
//!   - SetChargeLimit with 32.0A (fire-and-forget)
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

use bytes::Bytes;
use tracing::info;

use ara_com::transport::{
    AraDeserialize, AraSerialize, MessageHeader, MessageType, ReturnCode, Transport,
};
use ara_com::types::{InstanceId, MajorVersion, MethodId, MinorVersion, ServiceId};

use ara_com_someip::config::{
    EndpointConfig, RemoteServiceConfig, SdConfig, SomeIpConfig,
};
use ara_com_someip::transport::SomeIpTransport;

const SERVICE_ID: ServiceId = ServiceId(0x1234);
const INSTANCE_ID: InstanceId = InstanceId(0x0001);
const METHOD_GET_VOLTAGE: MethodId = MethodId(0x0001);
const METHOD_SET_CHARGE_LIMIT: MethodId = MethodId(0x0002);
const SERVER_PORT: u16 = 30509;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting BatteryService client...");

    let config = SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0002,
        discovery_mode: Default::default(), // Static
        sd_config: SdConfig::default(),
        services: vec![],
        remote_services: vec![RemoteServiceConfig {
            service_id: SERVICE_ID,
            instance_id: INSTANCE_ID,
            endpoint: EndpointConfig {
                udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, SERVER_PORT)),
                tcp: None,
                udp_threshold: 1400,
            },
        }],
    };

    let mut transport = SomeIpTransport::new(config);
    transport.bind().await?;

    // Find the service (static mode — just checks config)
    let found = transport
        .find_service(SERVICE_ID, INSTANCE_ID, MajorVersion(1), MinorVersion(0))
        .await?;
    info!("Found service: {found}");

    // --- Call GetVoltage for battery IDs 0–3 ---
    for battery_id in 0u8..4 {
        let mut req_buf = Vec::new();
        battery_id.ara_serialize(&mut req_buf)?;

        let header = MessageHeader {
            service_id: SERVICE_ID,
            method_id: METHOD_GET_VOLTAGE,
            instance_id: INSTANCE_ID,
            session_id: 0,
            message_type: MessageType::Request,
            return_code: ReturnCode::Ok,
        };

        let (resp_hdr, resp_payload) = transport
            .send_request(header, Bytes::from(req_buf))
            .await?;

        let voltage = f64::ara_deserialize(&resp_payload)?;
        info!(
            "GetVoltage(battery_id={battery_id}) → {voltage:.1}V [return_code={:?}]",
            resp_hdr.return_code
        );
    }

    // --- Fire-and-forget: SetChargeLimit ---
    let mut limit_buf = Vec::new();
    32.0f64.ara_serialize(&mut limit_buf)?;

    let header = MessageHeader {
        service_id: SERVICE_ID,
        method_id: METHOD_SET_CHARGE_LIMIT,
        instance_id: INSTANCE_ID,
        session_id: 0,
        message_type: MessageType::RequestNoReturn,
        return_code: ReturnCode::Ok,
    };

    transport
        .send_fire_and_forget(header, Bytes::from(limit_buf))
        .await?;
    info!("SetChargeLimit(32.0A) → sent (fire-and-forget)");

    info!("All calls completed successfully!");
    Ok(())
}
