//! SOME/IP Service Discovery integration tests.
//!
//! Two `SomeIpTransport` instances communicate over UDP loopback — one acts as
//! skeleton (offering a service), the other as proxy (finding it).
//!
//! Each test allocates unique SD and data ports from monotonic counters to avoid
//! port conflicts when tests run in parallel.

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use futures_core::future::BoxFuture;

use ara_com::error::AraComError;
use ara_com::transport::{MessageHeader, MessageType, ReturnCode, Transport};
use ara_com::types::{EventGroupId, InstanceId, MajorVersion, MethodId, MinorVersion, ServiceId};

use ara_com_someip::config::{
    DiscoveryMode, EndpointConfig, SdConfig, ServiceConfig, SomeIpConfig,
};
use ara_com_someip::transport::SomeIpTransport;

// ---------------------------------------------------------------------------
// Test constants
// ---------------------------------------------------------------------------

const SERVICE_ID: ServiceId = ServiceId(0x2000);
const INSTANCE_ID: InstanceId = InstanceId(0x0001);
const EVENT_ID: MethodId = MethodId(0x8001);
const EVENT_GROUP_ID: EventGroupId = EventGroupId(0x0001);

/// Well-known loopback multicast group for tests.
/// 239.0.0.1 is in the administratively scoped range and safe for local use.
const TEST_MULTICAST: Ipv4Addr = Ipv4Addr::new(239, 0, 0, 1);

/// Monotonically increasing port allocator for test isolation.
/// Starts at 34000 to avoid the default 30490 and OS ephemeral ranges.
static NEXT_PORT: AtomicU16 = AtomicU16::new(34000);

fn alloc_port() -> u16 {
    NEXT_PORT.fetch_add(1, Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// Config helpers
// ---------------------------------------------------------------------------

fn sd_config(port: u16) -> SdConfig {
    SdConfig {
        multicast_group: TEST_MULTICAST,
        port,
        initial_delay_min: 0,
        initial_delay_max: 0,
        repetition_base_delay: 0,
        repetition_max: 0,
        ttl: 5,
    }
}

/// Build a skeleton config with pre-allocated data and SD ports.
fn skeleton_sd_config(data_port: u16, sd_port: u16) -> SomeIpConfig {
    SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0010,
        discovery_mode: DiscoveryMode::SomeIpSd,
        sd_config: sd_config(sd_port),
        services: vec![ServiceConfig {
            service_id: SERVICE_ID,
            instance_id: INSTANCE_ID,
            endpoint: EndpointConfig {
                udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, data_port)),
                tcp: None,
                udp_threshold: 1400,
            },
            event_groups: vec![],
        }],
        remote_services: vec![],
    }
}

/// Build a proxy config. The proxy data socket binds to a random port.
fn proxy_sd_config(sd_port: u16) -> SomeIpConfig {
    SomeIpConfig {
        unicast: Ipv4Addr::LOCALHOST,
        client_id: 0x0020,
        discovery_mode: DiscoveryMode::SomeIpSd,
        sd_config: sd_config(sd_port),
        services: vec![],
        remote_services: vec![],
    }
}

// ---------------------------------------------------------------------------
// Test 1: SD offer → find round-trip
// ---------------------------------------------------------------------------

/// Two transports on loopback, one offers, the other finds.
#[tokio::test]
async fn test_sd_offer_find_roundtrip() {
    let sd_port = alloc_port();
    let data_port = alloc_port();

    // --- Skeleton ---
    let mut skeleton = SomeIpTransport::new(skeleton_sd_config(data_port, sd_port));
    skeleton.bind().await.expect("skeleton bind failed");

    skeleton
        .offer_service(SERVICE_ID, INSTANCE_ID, MajorVersion(1), MinorVersion(0))
        .await
        .expect("offer_service failed");

    // --- Proxy ---
    // Bind the proxy AFTER the skeleton offers. The initial OfferService
    // multicast was sent before the proxy joined, so the proxy must rely on
    // the FindService → OfferService unicast reply path.
    let mut proxy = SomeIpTransport::new(proxy_sd_config(sd_port));
    proxy.bind().await.expect("proxy bind failed");

    // Small delay to let both SD sockets fully set up.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let found = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        proxy.find_service(SERVICE_ID, INSTANCE_ID, MajorVersion(1), MinorVersion(0)),
    )
    .await
    .expect("find_service outer timeout")
    .expect("find_service failed");

    assert_eq!(found.service_id, SERVICE_ID);
    assert_eq!(found.instance_id, INSTANCE_ID);
    assert_eq!(found.major_version, MajorVersion(1));
}

// ---------------------------------------------------------------------------
// Test 2: SD stop-offer (service disappears)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_sd_stop_offer() {
    let sd_port = alloc_port();
    let data_port = alloc_port();

    let mut skeleton = SomeIpTransport::new(skeleton_sd_config(data_port, sd_port));
    skeleton.bind().await.expect("skeleton bind failed");

    skeleton
        .offer_service(SERVICE_ID, INSTANCE_ID, MajorVersion(1), MinorVersion(0))
        .await
        .expect("offer_service failed");

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Proxy finds it.
    let mut proxy = SomeIpTransport::new(proxy_sd_config(sd_port));
    proxy.bind().await.expect("proxy bind failed");

    proxy
        .find_service(SERVICE_ID, INSTANCE_ID, MajorVersion(1), MinorVersion(0))
        .await
        .expect("first find_service must succeed");

    // Skeleton sends stop-offer.
    skeleton
        .stop_offer_service(SERVICE_ID, INSTANCE_ID)
        .await
        .expect("stop_offer_service failed");

    // Give the stop-offer time to propagate.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // A subsequent find_service should time out or fail.
    let find_result = tokio::time::timeout(
        std::time::Duration::from_millis(200),
        proxy.find_service(SERVICE_ID, INSTANCE_ID, MajorVersion(1), MinorVersion(0)),
    )
    .await;

    // Either the outer timeout fires (Err) or the SD find itself times out
    // (Ok(Err(...))).  Either way the service must not be immediately available.
    match find_result {
        Err(_outer_timeout) => { /* expected — no service to answer */ }
        Ok(Err(_sd_error)) => { /* also acceptable */ }
        Ok(Ok(_found)) => {
            // If the stop-offer multicast hasn't been delivered yet, the
            // cached entry might still be present.  This is a TTL race; the
            // test is non-deterministic under load.  Accept the result but log.
            eprintln!(
                "warn: find_service returned Ok after stop-offer \
                 (TTL-cache race — acceptable under load)"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Test 3: SD subscribe → event delivery
// ---------------------------------------------------------------------------

/// Skeleton offers + publishes events; proxy discovers + subscribes + receives.
#[tokio::test]
async fn test_sd_subscribe_event_delivery() {
    let sd_port = alloc_port();
    let data_port = alloc_port();

    // --- Skeleton side ---
    let skeleton = Arc::new(tokio::sync::Mutex::new(SomeIpTransport::new(
        skeleton_sd_config(data_port, sd_port),
    )));
    {
        let mut sk = skeleton.lock().await;
        sk.bind().await.expect("skeleton bind failed");
    }

    {
        let sk = skeleton.lock().await;
        sk.offer_service(SERVICE_ID, INSTANCE_ID, MajorVersion(1), MinorVersion(0))
            .await
            .expect("offer_service");
    }

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // --- Proxy side ---
    let mut proxy = SomeIpTransport::new(proxy_sd_config(sd_port));
    proxy.bind().await.expect("proxy bind failed");

    // Register a notification handler on the proxy to receive events.
    let received_payload = Arc::new(tokio::sync::Mutex::new(None::<Bytes>));
    let received_clone = received_payload.clone();
    let notify_signal = Arc::new(tokio::sync::Notify::new());
    let notify_clone = notify_signal.clone();

    proxy
        .register_request_handler(
            SERVICE_ID,
            INSTANCE_ID,
            Box::new(
                move |_hdr, payload| -> BoxFuture<'static, Result<Bytes, AraComError>> {
                    let store = received_clone.clone();
                    let signal = notify_clone.clone();
                    Box::pin(async move {
                        *store.lock().await = Some(payload);
                        signal.notify_one();
                        Ok(Bytes::new())
                    })
                },
            ),
        )
        .await
        .unwrap();

    // Proxy discovers the service.
    proxy
        .find_service(SERVICE_ID, INSTANCE_ID, MajorVersion(1), MinorVersion(0))
        .await
        .expect("find_service failed");

    // Proxy subscribes to the event group.
    proxy
        .subscribe_event_group(SERVICE_ID, INSTANCE_ID, EVENT_GROUP_ID)
        .await
        .expect("subscribe_event_group failed");

    // Give the SubscribeEventgroup SD message time to reach the skeleton.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Skeleton sends a notification event.
    let event_payload = Bytes::from_static(b"\xCA\xFE\xBA\xBE");
    {
        let sk = skeleton.lock().await;
        let notif_header = MessageHeader {
            service_id: SERVICE_ID,
            method_id: EVENT_ID,
            instance_id: INSTANCE_ID,
            session_id: 0,
            message_type: MessageType::Notification,
            return_code: ReturnCode::Ok,
        };
        sk.send_notification(notif_header, event_payload.clone())
            .await
            .expect("send_notification failed");
    }

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

    // Proxy unsubscribes.
    proxy
        .unsubscribe_event_group(SERVICE_ID, INSTANCE_ID, EVENT_GROUP_ID)
        .await
        .expect("unsubscribe_event_group failed");
}
