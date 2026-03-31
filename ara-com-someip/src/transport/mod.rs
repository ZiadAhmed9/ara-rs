//! SOME/IP transport implementation of the `ara_com::transport::Transport` trait.

use async_trait::async_trait;
use bytes::Bytes;
use futures_core::future::BoxFuture;

use ara_com::error::AraComError;
use ara_com::transport::{MessageHeader, Transport};
use ara_com::types::{
    EventGroupId, InstanceId, MajorVersion, MinorVersion, ServiceId, ServiceInstanceId,
};

use crate::config::SomeIpConfig;

/// SOME/IP transport implementation.
///
/// Wraps the per-service configuration and will hold UDP/TCP sockets, session
/// tracking tables, and the SD task handle once the full implementation lands
/// in Weeks 5–7.
#[allow(dead_code)]
pub struct SomeIpTransport {
    config: SomeIpConfig,
    // TODO (Week 5): UdpSocket for the unicast receive port
    // TODO (Week 5): pending_requests: DashMap<u16, oneshot::Sender<Bytes>>
    // TODO (Week 6): tcp_pool: TcpConnectionPool
    // TODO (Week 7): sd_handle: JoinHandle<()>
}

impl SomeIpTransport {
    /// Create a new transport with the given configuration.
    ///
    /// Sockets and the SD background task are not yet started here; that will
    /// happen in a dedicated `bind()` / `run()` method in Week 5.
    pub fn new(config: SomeIpConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Transport for SomeIpTransport {
    /// Serialize `payload`, prepend the SOME/IP header derived from `header`,
    /// send over UDP (or TCP if payload exceeds `udp_threshold`), and await
    /// the correlated `Response` or `Error` message.
    async fn send_request(
        &self,
        _header: MessageHeader,
        _payload: Bytes,
    ) -> Result<(MessageHeader, Bytes), AraComError> {
        // TODO (Week 5–6): allocate session_id, insert into pending_requests,
        // encode header + payload, pick UDP vs TCP, send, await oneshot channel.
        todo!("send_request: SOME/IP request/response correlation not yet implemented")
    }

    /// Serialize `payload`, prepend a RequestNoReturn SOME/IP header, and
    /// send fire-and-forget over UDP. No response is awaited.
    async fn send_fire_and_forget(
        &self,
        _header: MessageHeader,
        _payload: Bytes,
    ) -> Result<(), AraComError> {
        // TODO (Week 5): encode MessageType::RequestNoReturn header, send UDP datagram.
        todo!("send_fire_and_forget: fire-and-forget send not yet implemented")
    }

    /// Serialize `payload`, prepend a Notification SOME/IP header, and
    /// multicast/unicast to all active event group subscribers.
    async fn send_notification(
        &self,
        _header: MessageHeader,
        _payload: Bytes,
    ) -> Result<(), AraComError> {
        // TODO (Week 6–7): look up subscriber list for the event group, send
        // multicast or per-subscriber unicast datagrams.
        todo!("send_notification: event notification delivery not yet implemented")
    }

    /// Emit a SOME/IP-SD `OfferService` entry on the multicast SD socket and
    /// start the SD offer state machine (initial delay, repetition, main phase).
    async fn offer_service(
        &self,
        _service_id: ServiceId,
        _instance_id: InstanceId,
        _major_version: MajorVersion,
        _minor_version: MinorVersion,
    ) -> Result<(), AraComError> {
        // TODO (Week 7): build SD OfferService entry, schedule initial delay,
        // send on SD multicast socket, register in offered_services map.
        todo!("offer_service: SOME/IP-SD OfferService not yet implemented")
    }

    /// Emit a SOME/IP-SD `StopOfferService` entry and remove from the offered
    /// services table.
    async fn stop_offer_service(
        &self,
        _service_id: ServiceId,
        _instance_id: InstanceId,
    ) -> Result<(), AraComError> {
        // TODO (Week 7): send StopOfferService SD entry, remove from offered_services.
        todo!("stop_offer_service: SOME/IP-SD StopOfferService not yet implemented")
    }

    /// Send a SOME/IP-SD `FindService` entry on the multicast SD socket and
    /// wait until a matching `OfferService` is received (with timeout).
    async fn find_service(
        &self,
        _service_id: ServiceId,
        _instance_id: InstanceId,
        _major_version: MajorVersion,
        _minor_version: MinorVersion,
    ) -> Result<ServiceInstanceId, AraComError> {
        // TODO (Week 7): send FindService SD entry, register a oneshot channel
        // in the pending_finds table, await SD task notification.
        todo!("find_service: SOME/IP-SD FindService not yet implemented")
    }

    /// Register a handler that will be called when an incoming request arrives
    /// for the given service/instance. The skeleton side uses this to dispatch
    /// to user application logic.
    async fn register_request_handler(
        &self,
        _service_id: ServiceId,
        _instance_id: InstanceId,
        _handler: Box<
            dyn Fn(MessageHeader, Bytes) -> BoxFuture<'static, Result<Bytes, AraComError>>
                + Send
                + Sync,
        >,
    ) -> Result<(), AraComError> {
        // TODO (Week 6): store handler in a handler registry keyed by
        // (service_id, instance_id); the receive loop will dispatch into it.
        todo!("register_request_handler: request handler registry not yet implemented")
    }

    /// Send a SOME/IP-SD `SubscribeEventgroup` entry and wait for the
    /// corresponding `SubscribeEventgroupAck`.
    async fn subscribe_event_group(
        &self,
        _service_id: ServiceId,
        _instance_id: InstanceId,
        _event_group_id: EventGroupId,
    ) -> Result<(), AraComError> {
        // TODO (Week 7): send SubscribeEventgroup SD entry, await Ack on
        // pending_subscriptions channel.
        todo!("subscribe_event_group: SOME/IP-SD SubscribeEventgroup not yet implemented")
    }

    /// Send a SOME/IP-SD `StopSubscribeEventgroup` entry.
    async fn unsubscribe_event_group(
        &self,
        _service_id: ServiceId,
        _instance_id: InstanceId,
        _event_group_id: EventGroupId,
    ) -> Result<(), AraComError> {
        // TODO (Week 7): send StopSubscribeEventgroup SD entry, remove from
        // active subscriptions.
        todo!("unsubscribe_event_group: SOME/IP-SD StopSubscribeEventgroup not yet implemented")
    }
}
