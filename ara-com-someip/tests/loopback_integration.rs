//! Loopback integration tests for SOME/IP transport.
//!
//! Two `SomeIpTransport` instances communicate over UDP loopback:
//! one acts as a skeleton (server), the other as a proxy (client).

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use bytes::Bytes;
use futures_core::future::BoxFuture;

use ara_com::error::AraComError;
use ara_com::transport::{
    AraDeserialize, AraSerialize, MessageHeader, MessageType, ReturnCode, Transport,
};
use ara_com::types::{EventGroupId, InstanceId, MajorVersion, MethodId, MinorVersion, ServiceId};

use ara_com_someip::config::{
    EndpointConfig, RemoteServiceConfig, SdConfig, ServiceConfig, SomeIpConfig,
};
use ara_com_someip::transport::SomeIpTransport;

const SERVICE_ID: ServiceId = ServiceId(0x1234);
const INSTANCE_ID: InstanceId = InstanceId(0x0001);
const METHOD_ID: MethodId = MethodId(0x0001);
/// Notifications use method IDs in the 0x8000+ range per SOME/IP spec.
const EVENT_ID: MethodId = MethodId(0x8001);
const EVENT_GROUP_ID: EventGroupId = EventGroupId(0x0001);

/// Create a config for the skeleton side (binds to a specific port).
fn skeleton_config(port: u16) -> SomeIpConfig {
    SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0001,
        discovery_mode: Default::default(),
        sd_config: SdConfig::default(),
        services: vec![ServiceConfig {
            service_id: SERVICE_ID,
            instance_id: INSTANCE_ID,
            endpoint: EndpointConfig {
                udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)),
                tcp: None,
                udp_threshold: 1400,
            },
            event_groups: vec![],
        }],
        remote_services: vec![],
    }
}

/// Create a config for the proxy side (knows the skeleton's endpoint).
fn proxy_config(skeleton_port: u16) -> SomeIpConfig {
    SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0002,
        discovery_mode: Default::default(),
        sd_config: SdConfig::default(),
        services: vec![],
        remote_services: vec![RemoteServiceConfig {
            service_id: SERVICE_ID,
            instance_id: INSTANCE_ID,
            endpoint: EndpointConfig {
                udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, skeleton_port)),
                tcp: None,
                udp_threshold: 1400,
            },
        }],
    }
}

#[tokio::test]
async fn test_fire_and_forget() {
    // Skeleton side — bind to random port
    let mut skeleton = SomeIpTransport::new(skeleton_config(0));
    skeleton.bind().await.expect("skeleton bind failed");
    let skeleton_port = skeleton
        .local_addr()
        .expect("skeleton has no local addr")
        .port();

    // Register a handler that just returns Ok (fire-and-forget still dispatches)
    let received = Arc::new(tokio::sync::Notify::new());
    let received_clone = received.clone();

    skeleton
        .register_request_handler(
            SERVICE_ID,
            INSTANCE_ID,
            Box::new(move |_hdr, _payload| -> BoxFuture<'static, Result<Bytes, AraComError>> {
                let notify = received_clone.clone();
                Box::pin(async move {
                    notify.notify_one();
                    Ok(Bytes::new())
                })
            }),
        )
        .await
        .unwrap();

    // Proxy side
    let mut proxy = SomeIpTransport::new(proxy_config(skeleton_port));
    proxy.bind().await.expect("proxy bind failed");

    let header = MessageHeader {
        service_id: SERVICE_ID,
        method_id: METHOD_ID,
        instance_id: INSTANCE_ID,
        session_id: 0,
        message_type: MessageType::RequestNoReturn,
        return_code: ReturnCode::Ok,
    };

    proxy
        .send_fire_and_forget(header, Bytes::from_static(b"\x42"))
        .await
        .expect("fire_and_forget failed");

    // Wait for the skeleton to receive and process it
    tokio::time::timeout(std::time::Duration::from_secs(2), received.notified())
        .await
        .expect("skeleton did not receive fire-and-forget within 2s");
}

#[tokio::test]
async fn test_request_response_round_trip() {
    // Skeleton side — bind to random port
    let mut skeleton = SomeIpTransport::new(skeleton_config(0));
    skeleton.bind().await.expect("skeleton bind failed");
    let skeleton_port = skeleton
        .local_addr()
        .expect("skeleton has no local addr")
        .port();

    // Register handler: receives a u32, returns it doubled as u32
    skeleton
        .register_request_handler(
            SERVICE_ID,
            INSTANCE_ID,
            Box::new(|_hdr, payload| -> BoxFuture<'static, Result<Bytes, AraComError>> {
                Box::pin(async move {
                    let value = u32::ara_deserialize(&payload).map_err(|e| {
                        AraComError::Application {
                            code: 1,
                            message: e.to_string(),
                        }
                    })?;
                    let result = value * 2;
                    let mut buf = Vec::new();
                    result.ara_serialize(&mut buf)?;
                    Ok(Bytes::from(buf))
                })
            }),
        )
        .await
        .unwrap();

    // Proxy side
    let mut proxy = SomeIpTransport::new(proxy_config(skeleton_port));
    proxy.bind().await.expect("proxy bind failed");

    // Serialize the request payload: u32 = 21
    let mut req_buf = Vec::new();
    21u32.ara_serialize(&mut req_buf).unwrap();

    let header = MessageHeader {
        service_id: SERVICE_ID,
        method_id: METHOD_ID,
        instance_id: INSTANCE_ID,
        session_id: 0,
        message_type: MessageType::Request,
        return_code: ReturnCode::Ok,
    };

    let (resp_hdr, resp_payload) = proxy
        .send_request(header, Bytes::from(req_buf))
        .await
        .expect("send_request failed");

    // Verify response
    assert_eq!(resp_hdr.message_type, MessageType::Response);
    assert_eq!(resp_hdr.return_code, ReturnCode::Ok);
    assert_eq!(resp_hdr.service_id, SERVICE_ID);
    assert_eq!(resp_hdr.method_id, METHOD_ID);

    let result = u32::ara_deserialize(&resp_payload).expect("failed to deserialize response");
    assert_eq!(result, 42, "21 * 2 should be 42");
}

#[tokio::test]
async fn test_static_find_service() {
    let mut proxy = SomeIpTransport::new(proxy_config(30509));
    proxy.bind().await.expect("proxy bind failed");

    // Should find the statically configured service
    let found = proxy
        .find_service(
            SERVICE_ID,
            INSTANCE_ID,
            MajorVersion(1),
            MinorVersion(0),
        )
        .await
        .expect("find_service failed");

    assert_eq!(found.service_id, SERVICE_ID);
    assert_eq!(found.instance_id, INSTANCE_ID);
}

#[tokio::test]
async fn test_static_find_service_not_configured() {
    let mut proxy = SomeIpTransport::new(proxy_config(30509));
    proxy.bind().await.expect("proxy bind failed");

    // Unknown service should fail
    let result = proxy
        .find_service(
            ServiceId(0x9999),
            InstanceId(0x0001),
            MajorVersion(1),
            MinorVersion(0),
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_offer_and_stop_service_static() {
    let mut skeleton = SomeIpTransport::new(skeleton_config(0));
    skeleton.bind().await.expect("skeleton bind failed");

    skeleton
        .offer_service(SERVICE_ID, INSTANCE_ID, MajorVersion(1), MinorVersion(0))
        .await
        .expect("offer_service failed");

    skeleton
        .stop_offer_service(SERVICE_ID, INSTANCE_ID)
        .await
        .expect("stop_offer_service failed");
}

#[tokio::test]
async fn test_notification_delivery() {
    // --- Skeleton side (event producer) ---
    let mut skeleton = SomeIpTransport::new(skeleton_config(0));
    skeleton.bind().await.expect("skeleton bind failed");
    let skeleton_port = skeleton
        .local_addr()
        .expect("skeleton has no local addr")
        .port();

    // --- Proxy side (event consumer) ---
    // The proxy config points at the skeleton so subscribe_event_group can
    // validate the endpoint exists in static mode.
    let mut proxy = SomeIpTransport::new(proxy_config(skeleton_port));
    proxy.bind().await.expect("proxy bind failed");
    let proxy_addr = proxy
        .local_addr()
        .expect("proxy has no local addr");

    // Proxy registers a handler so it can receive incoming Notification messages.
    let received_payload = Arc::new(tokio::sync::Mutex::new(None::<Bytes>));
    let received_payload_clone = received_payload.clone();
    let notify_signal = Arc::new(tokio::sync::Notify::new());
    let notify_signal_clone = notify_signal.clone();

    proxy
        .register_request_handler(
            SERVICE_ID,
            INSTANCE_ID,
            Box::new(move |_hdr, payload| -> BoxFuture<'static, Result<Bytes, AraComError>> {
                let store = received_payload_clone.clone();
                let signal = notify_signal_clone.clone();
                Box::pin(async move {
                    *store.lock().await = Some(payload);
                    signal.notify_one();
                    Ok(Bytes::new())
                })
            }),
        )
        .await
        .unwrap();

    // Proxy subscribes (static mode — just validates config exists).
    proxy
        .subscribe_event_group(SERVICE_ID, INSTANCE_ID, EVENT_GROUP_ID)
        .await
        .expect("subscribe_event_group failed");

    // Skeleton registers the proxy as an event subscriber.
    // We need a SocketAddrV4 — extract it from the SocketAddr.
    let proxy_v4 = match proxy_addr {
        std::net::SocketAddr::V4(a) => a,
        std::net::SocketAddr::V6(_) => panic!("expected IPv4 loopback address"),
    };
    skeleton.add_event_subscriber(SERVICE_ID, EVENT_GROUP_ID, proxy_v4);

    // Skeleton sends a notification event.
    let event_payload = Bytes::from_static(b"\xDE\xAD\xBE\xEF");
    let notif_header = MessageHeader {
        service_id: SERVICE_ID,
        method_id: EVENT_ID,
        instance_id: INSTANCE_ID,
        session_id: 0,
        message_type: MessageType::Notification,
        return_code: ReturnCode::Ok,
    };

    skeleton
        .send_notification(notif_header, event_payload.clone())
        .await
        .expect("send_notification failed");

    // Proxy should receive the notification within 2 s.
    tokio::time::timeout(std::time::Duration::from_secs(2), notify_signal.notified())
        .await
        .expect("proxy did not receive notification within 2s");

    let received = received_payload
        .lock()
        .await
        .take()
        .expect("handler fired but payload was not stored");
    assert_eq!(received, event_payload, "notification payload mismatch");

    // Clean up: verify unsubscribe does not error.
    proxy
        .unsubscribe_event_group(SERVICE_ID, INSTANCE_ID, EVENT_GROUP_ID)
        .await
        .expect("unsubscribe_event_group failed");

    // After unsubscribe, skeleton should see the subscriber removed.
    skeleton.remove_event_subscriber(SERVICE_ID, EVENT_GROUP_ID, &proxy_v4);
    // Sending again to zero subscribers should succeed (not an error).
    let notif_header2 = MessageHeader {
        service_id: SERVICE_ID,
        method_id: EVENT_ID,
        instance_id: INSTANCE_ID,
        session_id: 0,
        message_type: MessageType::Notification,
        return_code: ReturnCode::Ok,
    };
    skeleton
        .send_notification(notif_header2, Bytes::new())
        .await
        .expect("send_notification with zero subscribers should return Ok");
}
