//! TCP loopback integration tests for SOME/IP transport.
//!
//! These tests mirror the UDP loopback tests but configure TCP endpoints.

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use bytes::Bytes;
use futures_core::future::BoxFuture;

use ara_com::error::AraComError;
use ara_com::transport::{MessageHeader, MessageType, ReturnCode, Transport};
use ara_com::types::{InstanceId, MethodId, ServiceId};

use ara_com_someip::config::{
    EndpointConfig, RemoteServiceConfig, SdConfig, ServiceConfig, SomeIpConfig,
};
use ara_com_someip::transport::SomeIpTransport;

const SERVICE_ID: ServiceId = ServiceId(0x5678);
const INSTANCE_ID: InstanceId = InstanceId(0x0001);
const METHOD_ID: MethodId = MethodId(0x0001);

/// Create a skeleton config with TCP endpoint.
fn tcp_skeleton_config(tcp_port: u16) -> SomeIpConfig {
    SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0010,
        discovery_mode: Default::default(),
        sd_config: SdConfig::default(),
        services: vec![ServiceConfig {
            service_id: SERVICE_ID,
            instance_id: INSTANCE_ID,
            endpoint: EndpointConfig {
                udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
                tcp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, tcp_port)),
                udp_threshold: 1400,
            },
            event_groups: vec![],
        }],
        remote_services: vec![],
        udp_threshold: 1400,
    }
}

/// Create a proxy config that targets the skeleton's TCP endpoint.
/// udp_threshold=0 forces all traffic to TCP.
fn tcp_proxy_config(tcp_port: u16) -> SomeIpConfig {
    SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0020,
        discovery_mode: Default::default(),
        sd_config: SdConfig::default(),
        services: vec![],
        remote_services: vec![RemoteServiceConfig {
            service_id: SERVICE_ID,
            instance_id: INSTANCE_ID,
            endpoint: EndpointConfig {
                udp: None,
                tcp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, tcp_port)),
                udp_threshold: 0,
            },
        }],
        udp_threshold: 1400,
    }
}

/// Helper: find the TCP listener port from a bound skeleton.
/// We bind the TCP listener to port 0 (random) in the config, but the
/// actual port is assigned by the OS. We extract it from the config's
/// tcp field since we bind to 0.0.0.0:{tcp_port}.
///
/// For these tests we use a fixed port 0 and re-create with the actual port.
async fn setup_tcp_pair() -> (SomeIpTransport, SomeIpTransport, u16) {
    // Bind skeleton with TCP on random port — we need the actual port
    // Use a TcpListener to find a free port first.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to find free port");
    let tcp_port = listener.local_addr().unwrap().port();
    drop(listener); // release the port

    let mut skeleton = SomeIpTransport::new(tcp_skeleton_config(tcp_port));
    skeleton.bind().await.expect("skeleton bind failed");

    let proxy = SomeIpTransport::new(tcp_proxy_config(tcp_port));
    // proxy doesn't need bind() for outgoing TCP connections

    (skeleton, proxy, tcp_port)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_tcp_request_response_round_trip() {
    let (skeleton, proxy, _port) = setup_tcp_pair().await;

    // Register echo handler
    skeleton
        .register_request_handler(
            SERVICE_ID,
            INSTANCE_ID,
            Box::new(
                |_hdr, payload| -> BoxFuture<'static, Result<Bytes, AraComError>> {
                    Box::pin(async move { Ok(payload) })
                },
            ),
        )
        .await
        .unwrap();

    // Send request via TCP
    let request_payload = Bytes::from(vec![0xDE, 0xAD, 0xBE, 0xEF]);
    let header = MessageHeader {
        service_id: SERVICE_ID,
        method_id: METHOD_ID,
        instance_id: INSTANCE_ID,
        session_id: 0,
        message_type: MessageType::Request,
        return_code: ReturnCode::Ok,
    };

    let (resp_hdr, resp_payload) = proxy
        .send_request(header, request_payload.clone())
        .await
        .expect("TCP request failed");

    assert_eq!(resp_hdr.message_type, MessageType::Response);
    assert_eq!(resp_hdr.return_code, ReturnCode::Ok);
    assert_eq!(resp_payload, request_payload);
}

#[tokio::test]
async fn test_tcp_fire_and_forget() {
    let (skeleton, proxy, _port) = setup_tcp_pair().await;

    let received = Arc::new(tokio::sync::Notify::new());
    let received_clone = received.clone();

    skeleton
        .register_request_handler(
            SERVICE_ID,
            INSTANCE_ID,
            Box::new(
                move |_hdr, _payload| -> BoxFuture<'static, Result<Bytes, AraComError>> {
                    let notify = received_clone.clone();
                    Box::pin(async move {
                        notify.notify_one();
                        Ok(Bytes::new())
                    })
                },
            ),
        )
        .await
        .unwrap();

    let header = MessageHeader {
        service_id: SERVICE_ID,
        method_id: METHOD_ID,
        instance_id: INSTANCE_ID,
        session_id: 0,
        message_type: MessageType::RequestNoReturn,
        return_code: ReturnCode::Ok,
    };

    proxy
        .send_fire_and_forget(header, Bytes::from(vec![0x01, 0x02]))
        .await
        .expect("TCP fire-and-forget failed");

    // Wait for handler to receive the message
    tokio::time::timeout(std::time::Duration::from_secs(2), received.notified())
        .await
        .expect("handler did not receive fire-and-forget within timeout");
}

#[tokio::test]
async fn test_tcp_concurrent_requests() {
    let (skeleton, proxy, _port) = setup_tcp_pair().await;
    let proxy = Arc::new(proxy);

    // Handler: echo the payload back with a small delay
    skeleton
        .register_request_handler(
            SERVICE_ID,
            INSTANCE_ID,
            Box::new(
                |_hdr, payload| -> BoxFuture<'static, Result<Bytes, AraComError>> {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        Ok(payload)
                    })
                },
            ),
        )
        .await
        .unwrap();

    let mut handles = Vec::new();
    for i in 0u8..5 {
        let proxy_clone = proxy.clone();
        handles.push(tokio::spawn(async move {
            let payload = Bytes::from(vec![i; 4]);
            let header = MessageHeader {
                service_id: SERVICE_ID,
                method_id: METHOD_ID,
                instance_id: INSTANCE_ID,
                session_id: 0,
                message_type: MessageType::Request,
                return_code: ReturnCode::Ok,
            };
            let (_resp_hdr, resp_payload) = proxy_clone
                .send_request(header, payload.clone())
                .await
                .expect("concurrent TCP request failed");
            assert_eq!(
                resp_payload, payload,
                "response payload mismatch for request {i}"
            );
        }));
    }

    for h in handles {
        h.await.unwrap();
    }
}

#[tokio::test]
async fn test_tcp_large_payload() {
    let (skeleton, proxy, _port) = setup_tcp_pair().await;

    // Handler: echo back
    skeleton
        .register_request_handler(
            SERVICE_ID,
            INSTANCE_ID,
            Box::new(
                |_hdr, payload| -> BoxFuture<'static, Result<Bytes, AraComError>> {
                    Box::pin(async move { Ok(payload) })
                },
            ),
        )
        .await
        .unwrap();

    // Send a payload larger than typical UDP MTU
    let large_payload = Bytes::from(vec![0xAB; 8000]);
    let header = MessageHeader {
        service_id: SERVICE_ID,
        method_id: METHOD_ID,
        instance_id: INSTANCE_ID,
        session_id: 0,
        message_type: MessageType::Request,
        return_code: ReturnCode::Ok,
    };

    let (_resp_hdr, resp_payload) = proxy
        .send_request(header, large_payload.clone())
        .await
        .expect("TCP large payload request failed");

    assert_eq!(resp_payload.len(), 8000);
    assert_eq!(resp_payload, large_payload);
}

#[tokio::test]
async fn test_tcp_connect_to_invalid_endpoint_returns_error() {
    // Proxy configured to connect to a port nothing is listening on
    let config = SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0030,
        discovery_mode: Default::default(),
        sd_config: SdConfig::default(),
        services: vec![],
        remote_services: vec![RemoteServiceConfig {
            service_id: SERVICE_ID,
            instance_id: INSTANCE_ID,
            endpoint: EndpointConfig {
                udp: None,
                tcp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1)), // port 1 = unlikely
                udp_threshold: 0,
            },
        }],
        udp_threshold: 1400,
    };

    let proxy = SomeIpTransport::new(config);

    let header = MessageHeader {
        service_id: SERVICE_ID,
        method_id: METHOD_ID,
        instance_id: INSTANCE_ID,
        session_id: 0,
        message_type: MessageType::Request,
        return_code: ReturnCode::Ok,
    };

    let result = proxy.send_request(header, Bytes::new()).await;
    assert!(
        result.is_err(),
        "should fail connecting to invalid endpoint"
    );
}

#[tokio::test]
async fn test_payload_routing_udp_vs_tcp() {
    // Configure proxy with BOTH UDP and TCP endpoints.
    // udp_threshold = 100: payloads < 100 use UDP, >= 100 use TCP.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to find free port");
    let tcp_port = listener.local_addr().unwrap().port();
    drop(listener);

    // Skeleton: has both UDP and TCP
    let mut skeleton = SomeIpTransport::new(SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0040,
        discovery_mode: Default::default(),
        sd_config: SdConfig::default(),
        services: vec![ServiceConfig {
            service_id: SERVICE_ID,
            instance_id: INSTANCE_ID,
            endpoint: EndpointConfig {
                udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
                tcp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, tcp_port)),
                udp_threshold: 100,
            },
            event_groups: vec![],
        }],
        remote_services: vec![],
        udp_threshold: 1400,
    });
    skeleton.bind().await.expect("skeleton bind failed");
    let udp_port = skeleton.local_addr().unwrap().port();

    // Echo handler
    skeleton
        .register_request_handler(
            SERVICE_ID,
            INSTANCE_ID,
            Box::new(
                |_hdr, payload| -> BoxFuture<'static, Result<Bytes, AraComError>> {
                    Box::pin(async move { Ok(payload) })
                },
            ),
        )
        .await
        .unwrap();

    // Proxy: knows both UDP and TCP endpoints with threshold=100
    let proxy = SomeIpTransport::new(SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0050,
        discovery_mode: Default::default(),
        sd_config: SdConfig::default(),
        services: vec![],
        remote_services: vec![RemoteServiceConfig {
            service_id: SERVICE_ID,
            instance_id: INSTANCE_ID,
            endpoint: EndpointConfig {
                udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, udp_port)),
                tcp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, tcp_port)),
                udp_threshold: 100,
            },
        }],
        udp_threshold: 1400,
    });
    // Proxy needs bind() for UDP receive loop (to get responses)
    // Actually for UDP requests the proxy needs its own socket
    let mut proxy = proxy;
    proxy.bind().await.expect("proxy bind failed");

    // Small payload (< 100 bytes) → should use UDP
    let small = Bytes::from(vec![0x01; 10]);
    let header = MessageHeader {
        service_id: SERVICE_ID,
        method_id: METHOD_ID,
        instance_id: INSTANCE_ID,
        session_id: 0,
        message_type: MessageType::Request,
        return_code: ReturnCode::Ok,
    };

    let (_resp_hdr, resp_payload) = proxy
        .send_request(header.clone(), small.clone())
        .await
        .expect("small payload request failed");
    assert_eq!(
        resp_payload, small,
        "small payload should round-trip via UDP"
    );

    // Large payload (>= 100 bytes) → should use TCP
    let large = Bytes::from(vec![0x02; 200]);
    let (_resp_hdr, resp_payload) = proxy
        .send_request(header, large.clone())
        .await
        .expect("large payload request failed");
    assert_eq!(
        resp_payload, large,
        "large payload should round-trip via TCP"
    );
}
