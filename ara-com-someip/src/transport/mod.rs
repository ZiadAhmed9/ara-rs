//! SOME/IP transport implementation of the `ara_com::transport::Transport` trait.

pub mod header;

use std::net::SocketAddrV4;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use dashmap::DashMap;
use futures_core::future::BoxFuture;
use tokio::net::UdpSocket;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use ara_com::error::AraComError;
use ara_com::transport::{MessageHeader, MessageType, ReturnCode, Transport};
use ara_com::types::{
    EventGroupId, InstanceId, MajorVersion, MinorVersion, ServiceId,
    ServiceInstanceId,
};

use crate::config::{DiscoveryMode, SomeIpConfig};
use crate::error::SomeIpError;

use header::{decode_header, encode_header, DEFAULT_INTERFACE_VERSION, HEADER_LEN};

// ---------------------------------------------------------------------------
// Type aliases for the handler / pending request maps
// ---------------------------------------------------------------------------

type RequestHandler = Box<
    dyn Fn(MessageHeader, Bytes) -> BoxFuture<'static, Result<Bytes, AraComError>>
        + Send
        + Sync,
>;

type PendingResponse = oneshot::Sender<(MessageHeader, Bytes)>;

// ---------------------------------------------------------------------------
// SomeIpTransport
// ---------------------------------------------------------------------------

/// SOME/IP transport implementation.
///
/// Call [`SomeIpTransport::bind`] after construction to open the UDP socket and
/// start the background receive loop. Until `bind()` is called, all Transport
/// methods will return an error.
pub struct SomeIpTransport {
    config: SomeIpConfig,
    /// UDP socket for send/receive.
    udp_socket: Option<Arc<UdpSocket>>,
    /// Client ID for this application (bytes 8-9 of the SOME/IP header).
    client_id: u16,
    /// Monotonically increasing session counter for request correlation.
    session_counter: AtomicU16,
    /// Pending request→response correlation table: session_id → oneshot sender.
    pending_requests: Arc<DashMap<u16, PendingResponse>>,
    /// Registered request handlers: (service_id, instance_id) → handler fn.
    request_handlers: Arc<DashMap<(ServiceId, InstanceId), RequestHandler>>,
    /// Locally offered services for static mode.
    offered_services: Arc<DashMap<(ServiceId, InstanceId), (MajorVersion, MinorVersion)>>,
    /// Event subscribers: (service_id, event_group_id) → list of subscriber endpoints.
    event_subscribers: Arc<DashMap<(ServiceId, EventGroupId), Vec<SocketAddrV4>>>,
    /// Background receive loop handle (for shutdown).
    recv_handle: Option<JoinHandle<()>>,
}

impl SomeIpTransport {
    /// Create a new transport. Call [`bind`](Self::bind) to start the socket.
    pub fn new(config: SomeIpConfig) -> Self {
        let client_id = config.client_id;
        Self {
            config,
            udp_socket: None,
            client_id,
            session_counter: AtomicU16::new(1),
            pending_requests: Arc::new(DashMap::new()),
            request_handlers: Arc::new(DashMap::new()),
            offered_services: Arc::new(DashMap::new()),
            event_subscribers: Arc::new(DashMap::new()),
            recv_handle: None,
        }
    }

    /// Bind the UDP socket and start the background receive loop.
    ///
    /// Binds to `0.0.0.0:{port}` where port is taken from the first local
    /// service endpoint, or a random port if no services are configured.
    pub async fn bind(&mut self) -> Result<(), SomeIpError> {
        let bind_addr = self
            .config
            .services
            .first()
            .and_then(|s| s.endpoint.udp)
            .map(|addr| format!("0.0.0.0:{}", addr.port()))
            .unwrap_or_else(|| "0.0.0.0:0".to_string());

        let socket = UdpSocket::bind(&bind_addr).await?;
        debug!(addr = %socket.local_addr()?, "SOME/IP UDP socket bound");

        let socket = Arc::new(socket);
        self.udp_socket = Some(socket.clone());

        // Start background receive loop
        let pending = self.pending_requests.clone();
        let handlers = self.request_handlers.clone();
        let recv_socket = socket.clone();

        let handle = tokio::spawn(async move {
            let mut buf = vec![0u8; 65535];
            loop {
                let (len, src) = match recv_socket.recv_from(&mut buf).await {
                    Ok(r) => r,
                    Err(e) => {
                        error!(error = %e, "UDP recv_from failed");
                        continue;
                    }
                };

                if len < HEADER_LEN {
                    warn!(len, "received datagram shorter than SOME/IP header");
                    continue;
                }

                let (hdr, _client_id, payload_len, _iface_ver) =
                    match decode_header(&buf[..len]) {
                        Ok(r) => r,
                        Err(e) => {
                            warn!(error = %e, "failed to decode SOME/IP header");
                            continue;
                        }
                    };

                let payload_end = HEADER_LEN + payload_len as usize;
                if payload_end > len {
                    warn!(
                        expected = payload_end,
                        actual = len,
                        "truncated SOME/IP payload"
                    );
                    continue;
                }

                let payload = Bytes::copy_from_slice(&buf[HEADER_LEN..payload_end]);

                match hdr.message_type {
                    // --- Response / Error → correlate with pending request ---
                    MessageType::Response | MessageType::Error => {
                        if let Some((_, sender)) = pending.remove(&hdr.session_id) {
                            let _ = sender.send((hdr, payload));
                        } else {
                            warn!(
                                session_id = hdr.session_id,
                                "received response with no pending request"
                            );
                        }
                    }

                    // --- Request → dispatch to registered handler ---
                    MessageType::Request => {
                        // instance_id is not in the SOME/IP base header (always
                        // decoded as 0), so we look up by service_id with any
                        // registered instance_id.
                        let handler_ref = handlers
                            .iter()
                            .find(|entry| entry.key().0 == hdr.service_id);

                        if let Some(handler) = handler_ref {
                            let handler_fn = handler.value();
                            let resp_fut = handler_fn(hdr.clone(), payload);
                            let reply_socket = recv_socket.clone();
                            let reply_hdr = hdr;
                            let reply_src = src;

                            tokio::spawn(async move {
                                match resp_fut.await {
                                    Ok(resp_payload) => {
                                        let resp_header = MessageHeader {
                                            service_id: reply_hdr.service_id,
                                            method_id: reply_hdr.method_id,
                                            instance_id: reply_hdr.instance_id,
                                            session_id: reply_hdr.session_id,
                                            message_type: MessageType::Response,
                                            return_code: ReturnCode::Ok,
                                        };
                                        let wire_hdr = encode_header(
                                            &resp_header,
                                            0, // server doesn't need client_id
                                            resp_payload.len() as u32,
                                            DEFAULT_INTERFACE_VERSION,
                                        );
                                        let mut frame =
                                            Vec::with_capacity(HEADER_LEN + resp_payload.len());
                                        frame.extend_from_slice(&wire_hdr);
                                        frame.extend_from_slice(&resp_payload);
                                        if let Err(e) =
                                            reply_socket.send_to(&frame, reply_src).await
                                        {
                                            error!(error = %e, "failed to send response");
                                        }
                                    }
                                    Err(e) => {
                                        error!(error = %e, "request handler failed");
                                        // Send Error response
                                        let err_header = MessageHeader {
                                            service_id: reply_hdr.service_id,
                                            method_id: reply_hdr.method_id,
                                            instance_id: reply_hdr.instance_id,
                                            session_id: reply_hdr.session_id,
                                            message_type: MessageType::Error,
                                            return_code: ReturnCode::NotOk,
                                        };
                                        let wire_hdr = encode_header(
                                            &err_header,
                                            0,
                                            0,
                                            DEFAULT_INTERFACE_VERSION,
                                        );
                                        let _ =
                                            reply_socket.send_to(&wire_hdr, reply_src).await;
                                    }
                                }
                            });
                        } else {
                            warn!(
                                service_id = %hdr.service_id,
                                method_id = %hdr.method_id,
                                "no handler registered for incoming request"
                            );
                        }
                    }

                    // --- Fire-and-forget / Notification — no response expected ---
                    MessageType::RequestNoReturn | MessageType::Notification => {
                        let handler_ref = handlers
                            .iter()
                            .find(|entry| entry.key().0 == hdr.service_id);
                        if let Some(handler) = handler_ref {
                            let handler_fn = handler.value();
                            let fut = handler_fn(hdr, payload);
                            tokio::spawn(async move {
                                if let Err(e) = fut.await {
                                    warn!(error = %e, "fire-and-forget handler error");
                                }
                            });
                        }
                    }
                }
            }
        });

        self.recv_handle = Some(handle);
        Ok(())
    }

    /// Get the local address the UDP socket is bound to.
    pub fn local_addr(&self) -> Option<std::net::SocketAddr> {
        self.udp_socket
            .as_ref()
            .and_then(|s| s.local_addr().ok())
    }

    /// Allocate the next session ID.
    fn next_session_id(&self) -> u16 {
        let id = self.session_counter.fetch_add(1, Ordering::Relaxed);
        if id == 0 {
            // Session ID 0 is reserved in SOME/IP; skip it
            self.session_counter.fetch_add(1, Ordering::Relaxed)
        } else {
            id
        }
    }

    /// Get the UDP socket, or return an error if not yet bound.
    fn socket(&self) -> Result<&Arc<UdpSocket>, AraComError> {
        self.udp_socket.as_ref().ok_or(AraComError::Transport {
            message: "transport not bound — call bind() first".to_string(),
        })
    }

    /// Look up the remote endpoint for a service in static discovery mode.
    fn resolve_endpoint(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
    ) -> Option<SocketAddrV4> {
        self.config
            .remote_services
            .iter()
            .find(|rs| rs.service_id == service_id && rs.instance_id == instance_id)
            .and_then(|rs| rs.endpoint.udp)
    }

    /// Register a remote endpoint as a subscriber for skeleton-side event delivery.
    ///
    /// Called by SD or static config to add a consumer that should receive
    /// notifications for `(service_id, event_group_id)`.
    pub fn add_event_subscriber(
        &self,
        service_id: ServiceId,
        event_group_id: EventGroupId,
        endpoint: SocketAddrV4,
    ) {
        self.event_subscribers
            .entry((service_id, event_group_id))
            .or_default()
            .push(endpoint);
        debug!(%service_id, %event_group_id, %endpoint, "event subscriber added");
    }

    /// Remove a previously registered event subscriber.
    pub fn remove_event_subscriber(
        &self,
        service_id: ServiceId,
        event_group_id: EventGroupId,
        endpoint: &SocketAddrV4,
    ) {
        if let Some(mut subs) = self.event_subscribers.get_mut(&(service_id, event_group_id)) {
            subs.retain(|e| e != endpoint);
        }
        debug!(%service_id, %event_group_id, %endpoint, "event subscriber removed");
    }

    /// Construct and send a SOME/IP datagram.
    async fn send_datagram(
        &self,
        header: &MessageHeader,
        payload: &[u8],
        dest: SocketAddrV4,
    ) -> Result<(), AraComError> {
        let wire_hdr = encode_header(
            header,
            self.client_id,
            payload.len() as u32,
            DEFAULT_INTERFACE_VERSION,
        );
        let mut frame = Vec::with_capacity(HEADER_LEN + payload.len());
        frame.extend_from_slice(&wire_hdr);
        frame.extend_from_slice(payload);

        let socket = self.socket()?;
        socket
            .send_to(&frame, dest)
            .await
            .map_err(|e| AraComError::Transport {
                message: format!("UDP send failed: {e}"),
            })?;
        Ok(())
    }
}

#[async_trait]
impl Transport for SomeIpTransport {
    async fn send_request(
        &self,
        mut header: MessageHeader,
        payload: Bytes,
    ) -> Result<(MessageHeader, Bytes), AraComError> {
        let dest = self
            .resolve_endpoint(header.service_id, header.instance_id)
            .ok_or_else(|| AraComError::ServiceNotAvailable {
                service_id: header.service_id,
                instance_id: header.instance_id,
            })?;

        let session_id = self.next_session_id();
        header.session_id = session_id;
        header.message_type = MessageType::Request;

        // Set up the response correlation channel
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(session_id, tx);

        // Send the request
        if let Err(e) = self.send_datagram(&header, &payload, dest).await {
            self.pending_requests.remove(&session_id);
            return Err(e);
        }

        // Await the correlated response with a timeout
        let timeout = tokio::time::Duration::from_millis(5000);
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok((resp_header, resp_payload))) => Ok((resp_header, resp_payload)),
            Ok(Err(_)) => {
                // Channel closed without response
                Err(AraComError::Timeout {
                    timeout_ms: timeout.as_millis() as u64,
                })
            }
            Err(_) => {
                self.pending_requests.remove(&session_id);
                Err(AraComError::Timeout {
                    timeout_ms: timeout.as_millis() as u64,
                })
            }
        }
    }

    async fn send_fire_and_forget(
        &self,
        mut header: MessageHeader,
        payload: Bytes,
    ) -> Result<(), AraComError> {
        let dest = self
            .resolve_endpoint(header.service_id, header.instance_id)
            .ok_or_else(|| AraComError::ServiceNotAvailable {
                service_id: header.service_id,
                instance_id: header.instance_id,
            })?;

        header.session_id = self.next_session_id();
        header.message_type = MessageType::RequestNoReturn;

        self.send_datagram(&header, &payload, dest).await
    }

    async fn send_notification(
        &self,
        mut header: MessageHeader,
        payload: Bytes,
    ) -> Result<(), AraComError> {
        header.message_type = MessageType::Notification;
        // Notifications use session_id 0 per AUTOSAR SOME/IP spec (§4.2.1).
        header.session_id = 0;

        // Collect all subscribers for this service across all event groups.
        // At the transport layer the method_id already identifies the event;
        // event_group demux is handled at the SD / skeleton layer, so we
        // broadcast to every subscriber registered for this service_id.
        let destinations: Vec<SocketAddrV4> = self
            .event_subscribers
            .iter()
            .filter(|entry| entry.key().0 == header.service_id)
            .flat_map(|entry| entry.value().clone())
            .collect();

        if destinations.is_empty() {
            debug!(
                service_id = %header.service_id,
                method_id  = %header.method_id,
                "send_notification: no subscribers — dropping notification"
            );
            return Ok(());
        }

        info!(
            service_id   = %header.service_id,
            method_id    = %header.method_id,
            n_subscribers = destinations.len(),
            "sending SOME/IP notification"
        );

        for dest in destinations {
            if let Err(e) = self.send_datagram(&header, &payload, dest).await {
                warn!(error = %e, %dest, "failed to deliver notification to subscriber");
            }
        }

        Ok(())
    }

    async fn offer_service(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        major_version: MajorVersion,
        minor_version: MinorVersion,
    ) -> Result<(), AraComError> {
        match self.config.discovery_mode {
            DiscoveryMode::Static => {
                // In static mode, just register locally — no SD announcement needed
                self.offered_services
                    .insert((service_id, instance_id), (major_version, minor_version));
                debug!(
                    %service_id, %instance_id,
                    "service offered (static mode)"
                );
                Ok(())
            }
            DiscoveryMode::SomeIpSd => {
                // TODO (Week 7): SD OfferService state machine
                todo!("offer_service: SOME/IP-SD OfferService not yet implemented")
            }
        }
    }

    async fn stop_offer_service(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
    ) -> Result<(), AraComError> {
        match self.config.discovery_mode {
            DiscoveryMode::Static => {
                self.offered_services.remove(&(service_id, instance_id));
                debug!(
                    %service_id, %instance_id,
                    "service stopped (static mode)"
                );
                Ok(())
            }
            DiscoveryMode::SomeIpSd => {
                todo!("stop_offer_service: SOME/IP-SD StopOfferService not yet implemented")
            }
        }
    }

    async fn find_service(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        major_version: MajorVersion,
        minor_version: MinorVersion,
    ) -> Result<ServiceInstanceId, AraComError> {
        match self.config.discovery_mode {
            DiscoveryMode::Static => {
                // In static mode, check if endpoint is configured
                if self.resolve_endpoint(service_id, instance_id).is_some() {
                    Ok(ServiceInstanceId {
                        service_id,
                        instance_id,
                        major_version,
                        minor_version,
                    })
                } else {
                    Err(AraComError::ServiceNotAvailable {
                        service_id,
                        instance_id,
                    })
                }
            }
            DiscoveryMode::SomeIpSd => {
                todo!("find_service: SOME/IP-SD FindService not yet implemented")
            }
        }
    }

    async fn register_request_handler(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        handler: Box<
            dyn Fn(MessageHeader, Bytes) -> BoxFuture<'static, Result<Bytes, AraComError>>
                + Send
                + Sync,
        >,
    ) -> Result<(), AraComError> {
        self.request_handlers
            .insert((service_id, instance_id), handler);
        debug!(%service_id, %instance_id, "request handler registered");
        Ok(())
    }

    async fn subscribe_event_group(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        event_group_id: EventGroupId,
    ) -> Result<(), AraComError> {
        match self.config.discovery_mode {
            DiscoveryMode::Static => {
                // In static mode the subscription is implicitly configured.
                // Verify the remote service endpoint is known so we can surface
                // misconfigurations early.
                if self.resolve_endpoint(service_id, instance_id).is_none() {
                    return Err(AraComError::ServiceNotAvailable {
                        service_id,
                        instance_id,
                    });
                }
                debug!(
                    %service_id, %instance_id, %event_group_id,
                    "subscribed to event group (static mode — implicit subscription)"
                );
                Ok(())
            }
            DiscoveryMode::SomeIpSd => {
                // TODO (Week 7): SD SubscribeEventgroup
                todo!("subscribe_event_group: SOME/IP-SD SubscribeEventgroup not yet implemented")
            }
        }
    }

    async fn unsubscribe_event_group(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        event_group_id: EventGroupId,
    ) -> Result<(), AraComError> {
        match self.config.discovery_mode {
            DiscoveryMode::Static => {
                // Remove any subscriber entry keyed on this service + event group.
                self.event_subscribers
                    .remove(&(service_id, event_group_id));
                debug!(
                    %service_id, %instance_id, %event_group_id,
                    "unsubscribed from event group (static mode)"
                );
                Ok(())
            }
            DiscoveryMode::SomeIpSd => {
                // TODO (Week 7): SD StopSubscribeEventgroup
                todo!("unsubscribe_event_group: SOME/IP-SD StopSubscribeEventgroup not yet implemented")
            }
        }
    }
}

impl Drop for SomeIpTransport {
    fn drop(&mut self) {
        if let Some(handle) = self.recv_handle.take() {
            handle.abort();
        }
    }
}
