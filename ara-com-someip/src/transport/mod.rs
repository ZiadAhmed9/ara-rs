//! SOME/IP transport implementation of the `ara_com::transport::Transport` trait.
//!
//! Supports both UDP and TCP transports. UDP is used for small payloads and
//! notifications; TCP is used when payload size exceeds `udp_threshold` or
//! when only a TCP endpoint is configured.

pub mod header;

use std::net::SocketAddrV4;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use dashmap::DashMap;
use futures_core::future::BoxFuture;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{broadcast, oneshot};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use ara_com::error::AraComError;
use ara_com::transport::{MessageHeader, MessageType, ReturnCode, Transport};
use ara_com::types::{
    EventGroupId, EventId, InstanceId, MajorVersion, MethodId, MinorVersion, ServiceId,
    ServiceInstanceId,
};

use crate::config::{DiscoveryMode, SomeIpConfig};
use crate::discovery::{EventSubscriberCallback, FoundService, ServiceDiscovery};
use crate::error::SomeIpError;

use header::{decode_header, encode_header, DEFAULT_INTERFACE_VERSION, HEADER_LEN};

// ---------------------------------------------------------------------------
// Type aliases for the handler / pending request maps
// ---------------------------------------------------------------------------

type RequestHandler = Box<
    dyn Fn(MessageHeader, Bytes) -> BoxFuture<'static, Result<Bytes, AraComError>> + Send + Sync,
>;

type PendingResponse = oneshot::Sender<(MessageHeader, Bytes)>;

// ---------------------------------------------------------------------------
// Resolved endpoint
// ---------------------------------------------------------------------------

/// A resolved remote endpoint with protocol information.
#[derive(Debug, Clone, Copy)]
enum ResolvedEndpoint {
    Udp(SocketAddrV4),
    Tcp(SocketAddrV4),
}

// ---------------------------------------------------------------------------
// SomeIpTransport
// ---------------------------------------------------------------------------

/// SOME/IP transport implementation.
///
/// Call [`SomeIpTransport::bind`] after construction to open the UDP and/or
/// TCP sockets and start the background receive loops. Until `bind()` is
/// called, all Transport methods will return an error.
pub struct SomeIpTransport {
    config: SomeIpConfig,
    /// UDP socket for send/receive.
    udp_socket: Option<Arc<UdpSocket>>,
    /// TCP listener handles for incoming connections (skeleton side).
    /// One per distinct TCP port across all configured services.
    tcp_listener_handles: Vec<JoinHandle<()>>,
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
    /// Event subscribers: (service_id, instance_id, event_group_id) → list of subscriber endpoints.
    event_subscribers: Arc<DashMap<(ServiceId, InstanceId, EventGroupId), Vec<SocketAddrV4>>>,
    /// Notification channels for event reception (proxy side).
    /// (service_id, instance_id, event_method_id) → broadcast sender for raw payloads.
    notification_channels:
        Arc<DashMap<(ServiceId, InstanceId, MethodId), broadcast::Sender<Bytes>>>,
    /// Authoritative service_id → instance_id binding for this transport.
    /// Enforces the one-instance-per-service-per-transport invariant.
    /// Checked by every operation that creates per-instance state.
    instance_binding: DashMap<ServiceId, InstanceId>,
    /// Background receive loop handle (for shutdown).
    recv_handle: Option<JoinHandle<()>>,
    /// Service Discovery runtime (used when discovery_mode == SomeIpSd).
    sd: ServiceDiscovery,
}

impl SomeIpTransport {
    /// Create a new transport. Call [`bind`](Self::bind) to start the socket.
    pub fn new(config: SomeIpConfig) -> Self {
        let client_id = config.client_id;
        let unicast_addr = config.unicast;
        let sd_config = config.sd_config.clone();

        Self {
            config,
            udp_socket: None,
            tcp_listener_handles: Vec::new(),
            client_id,
            session_counter: AtomicU16::new(1),
            pending_requests: Arc::new(DashMap::new()),
            request_handlers: Arc::new(DashMap::new()),
            offered_services: Arc::new(DashMap::new()),
            event_subscribers: Arc::new(DashMap::new()),
            notification_channels: Arc::new(DashMap::new()),
            instance_binding: DashMap::new(),
            recv_handle: None,
            sd: ServiceDiscovery::new(sd_config, unicast_addr),
        }
    }

    /// Bind the UDP (and optionally TCP) sockets and start background receive
    /// loops.
    ///
    /// UDP binds to `0.0.0.0:{port}` where port is taken from the first local
    /// service endpoint, or a random port if no services are configured. If a
    /// TCP endpoint is configured, a TCP listener is also started on its port.
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

        // If SD mode is enabled, wire the event-subscriber callback and start
        // the SD background task.
        if self.config.discovery_mode == DiscoveryMode::SomeIpSd {
            let event_subscribers = self.event_subscribers.clone();
            let cb: EventSubscriberCallback = Arc::new(
                move |service_id, instance_id, event_group_id, endpoint, subscribe| {
                    if subscribe {
                        event_subscribers
                            .entry((service_id, instance_id, event_group_id))
                            .or_default()
                            .push(endpoint);
                        debug!(
                            %service_id, %instance_id, %event_group_id, %endpoint,
                            "SD: event subscriber added via callback"
                        );
                    } else {
                        if let Some(mut subs) =
                            event_subscribers.get_mut(&(service_id, instance_id, event_group_id))
                        {
                            subs.retain(|e| e != &endpoint);
                        }
                        debug!(
                            %service_id, %instance_id, %event_group_id, %endpoint,
                            "SD: event subscriber removed via callback"
                        );
                    }
                },
            );
            self.sd.set_event_subscriber_callback(cb);
            self.sd.start().await?;
        }

        // --- TCP listeners (server side) ---
        //
        // Bind a TCP listener for every distinct TCP port across all
        // configured local services. Each listener shares the same
        // handler/pending/notification maps.
        let mut bound_tcp_ports = std::collections::HashSet::new();
        for svc in &self.config.services {
            if let Some(tcp_addr) = svc.endpoint.tcp {
                if !bound_tcp_ports.insert(tcp_addr.port()) {
                    continue; // already bound this port
                }
                let listener = TcpListener::bind(format!("0.0.0.0:{}", tcp_addr.port())).await?;
                debug!(addr = %listener.local_addr()?, "SOME/IP TCP listener bound");

                let tcp_pending = self.pending_requests.clone();
                let tcp_handlers = self.request_handlers.clone();
                let tcp_notif_channels = self.notification_channels.clone();

                let tcp_handle = tokio::spawn(async move {
                    loop {
                        let (stream, peer) = match listener.accept().await {
                            Ok(r) => r,
                            Err(e) => {
                                error!(error = %e, "TCP accept failed");
                                continue;
                            }
                        };
                        debug!(%peer, "accepted TCP connection");

                        let conn_pending = tcp_pending.clone();
                        let conn_handlers = tcp_handlers.clone();
                        let conn_notif_channels = tcp_notif_channels.clone();

                        tokio::spawn(async move {
                            if let Err(e) = handle_tcp_connection(
                                stream,
                                conn_pending,
                                conn_handlers,
                                conn_notif_channels,
                            )
                            .await
                            {
                                debug!(error = %e, %peer, "TCP connection closed");
                            }
                        });
                    }
                });
                self.tcp_listener_handles.push(tcp_handle);
            }
        }

        // Start background UDP receive loop.
        //
        // DESIGN: a single SomeIpTransport serves at most ONE instance per
        // service_id.  This is a SOME/IP wire-format constraint: the base
        // header does not carry instance_id, so a socket bound to one
        // endpoint can only serve one instance.  Multiple instances of the
        // same service require separate transports on different ports.
        //
        // The handler and channel maps are keyed by (ServiceId, InstanceId,
        // ...) so each transport's entries are fully qualified, but the
        // receive loop matches only on service_id (+ method_id) because
        // instance_id is guaranteed unique per service within a transport.
        let pending = self.pending_requests.clone();
        let handlers = self.request_handlers.clone();
        let notif_channels = self.notification_channels.clone();
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

                let (hdr, _client_id, payload_len, _iface_ver) = match decode_header(&buf[..len]) {
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
                        // instance_id is not on the SOME/IP base header wire.
                        // The one-instance-per-service invariant guarantees at
                        // most one handler matches a given service_id.
                        let handler_ref = handlers.iter().find(|e| e.key().0 == hdr.service_id);

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
                                        let _ = reply_socket.send_to(&wire_hdr, reply_src).await;
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

                    // --- Notification → push to event channel or handler ---
                    MessageType::Notification => {
                        // One-instance-per-service invariant: at most one
                        // channel matches (service_id, *, method_id).
                        if let Some(entry) = notif_channels
                            .iter()
                            .find(|e| e.key().0 == hdr.service_id && e.key().2 == hdr.method_id)
                        {
                            let _ = entry.value().send(payload.clone());
                        }
                        // Also dispatch to registered handler (if any).
                        let handler_ref = handlers.iter().find(|e| e.key().0 == hdr.service_id);
                        if let Some(handler) = handler_ref {
                            let handler_fn = handler.value();
                            let fut = handler_fn(hdr, payload);
                            tokio::spawn(async move {
                                if let Err(e) = fut.await {
                                    warn!(error = %e, "notification handler error");
                                }
                            });
                        }
                    }

                    // --- Fire-and-forget — no response expected ---
                    MessageType::RequestNoReturn => {
                        let handler_ref = handlers.iter().find(|e| e.key().0 == hdr.service_id);
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
        self.udp_socket.as_ref().and_then(|s| s.local_addr().ok())
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

    /// Look up the remote endpoint for a service, choosing UDP or TCP based
    /// on payload size and endpoint availability.
    ///
    /// In static mode, checks the pre-configured remote_services list.
    /// In SD mode, additionally checks the dynamically discovered found_services.
    ///
    /// Routing rules:
    /// - If payload_size >= `udp_threshold` and a TCP endpoint is available, use TCP.
    /// - If only TCP is configured (no UDP), use TCP regardless of payload size.
    /// - Otherwise, use UDP.
    fn resolve_endpoint(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        payload_size: usize,
    ) -> Option<ResolvedEndpoint> {
        // Static config always takes precedence.
        if let Some(rs) = self
            .config
            .remote_services
            .iter()
            .find(|rs| rs.service_id == service_id && rs.instance_id == instance_id)
        {
            return Self::pick_protocol(&rs.endpoint, payload_size);
        }

        // In SD mode, fall back to dynamically discovered services.
        // The udp_threshold is a client-side decision: look it up from
        // local service config first (skeleton side), then fall back to
        // the configured default on SomeIpConfig.
        if self.config.discovery_mode == DiscoveryMode::SomeIpSd {
            let threshold = self
                .config
                .services
                .iter()
                .find(|s| s.service_id == service_id)
                .map(|s| s.endpoint.udp_threshold)
                .unwrap_or(self.config.udp_threshold);

            let found = self.sd.found_services();
            let result = found.get(&(service_id, instance_id)).and_then(|fs| {
                if fs.ttl_expires_at > std::time::Instant::now() {
                    let ep = crate::config::EndpointConfig {
                        udp: fs.udp_endpoint,
                        tcp: fs.tcp_endpoint,
                        udp_threshold: threshold,
                    };
                    Self::pick_protocol(&ep, payload_size)
                } else {
                    None
                }
            });
            if result.is_some() {
                return result;
            }
            found.remove_if(&(service_id, instance_id), |_, fs| {
                fs.ttl_expires_at <= std::time::Instant::now()
            });
        }

        None
    }

    /// Choose UDP or TCP based on endpoint availability and payload size.
    fn pick_protocol(
        ep: &crate::config::EndpointConfig,
        payload_size: usize,
    ) -> Option<ResolvedEndpoint> {
        match (ep.udp, ep.tcp) {
            (Some(udp), Some(tcp)) => {
                if payload_size >= ep.udp_threshold {
                    Some(ResolvedEndpoint::Tcp(tcp))
                } else {
                    Some(ResolvedEndpoint::Udp(udp))
                }
            }
            (Some(udp), None) => Some(ResolvedEndpoint::Udp(udp)),
            (None, Some(tcp)) => Some(ResolvedEndpoint::Tcp(tcp)),
            (None, None) => None,
        }
    }

    /// Check and record that `service_id` is bound to `instance_id` on this
    /// transport.  Returns `Ok(())` if the binding is new or matches an
    /// existing one.  Returns `Err` if a *different* instance_id is already
    /// bound for the same service_id.
    fn enforce_instance_binding(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        operation: &str,
    ) -> Result<(), AraComError> {
        match self.instance_binding.entry(service_id) {
            dashmap::mapref::entry::Entry::Occupied(e) => {
                let existing = *e.get();
                if existing != instance_id {
                    return Err(AraComError::Transport {
                        message: format!(
                            "cannot {operation} for {service_id} {instance_id}: \
                             {service_id} {existing} is already bound to this \
                             transport — use a separate transport for each instance"
                        ),
                    });
                }
                Ok(())
            }
            dashmap::mapref::entry::Entry::Vacant(e) => {
                e.insert(instance_id);
                Ok(())
            }
        }
    }

    /// Register a remote endpoint as a subscriber for skeleton-side event delivery.
    ///
    /// Register a remote endpoint as an event subscriber for
    /// `(service_id, instance_id, event_group_id)`.
    ///
    /// Called by the SD callback (skeleton side) and test harnesses.
    /// The caller must ensure the instance binding invariant holds —
    /// prefer [`Transport::subscribe_event_group`] for proxy-side use.
    pub fn add_event_subscriber(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        event_group_id: EventGroupId,
        endpoint: SocketAddrV4,
    ) {
        self.event_subscribers
            .entry((service_id, instance_id, event_group_id))
            .or_default()
            .push(endpoint);
        debug!(%service_id, %instance_id, %event_group_id, %endpoint, "event subscriber added");
    }

    /// Remove a previously registered event subscriber.
    pub fn remove_event_subscriber(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        event_group_id: EventGroupId,
        endpoint: &SocketAddrV4,
    ) {
        if let Some(mut subs) =
            self.event_subscribers
                .get_mut(&(service_id, instance_id, event_group_id))
        {
            subs.retain(|e| e != endpoint);
        }
        debug!(%service_id, %event_group_id, %endpoint, "event subscriber removed");
    }

    /// Subscribe to notification events for a specific
    /// `(service_id, instance_id, event_method_id)`.
    ///
    /// Returns a [`broadcast::Receiver<Bytes>`] that yields raw event payloads
    /// whenever the receive loop gets a Notification matching this key. The
    /// channel has a capacity of `buffer_size` entries; excess events are dropped
    /// for slow consumers (backpressure via `broadcast::Sender::send`).
    ///
    /// Call this **before** `subscribe_event_group` so the channel is ready
    /// before the first notification arrives.
    ///
    /// # Errors
    ///
    /// Returns an error if a channel for the same `service_id` with a
    /// **different** `instance_id` already exists.  A single transport can
    /// only track one instance per service (SOME/IP wire-format constraint).
    pub fn subscribe_notifications(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        event_method_id: MethodId,
        buffer_size: usize,
    ) -> Result<broadcast::Receiver<Bytes>, AraComError> {
        self.enforce_instance_binding(service_id, instance_id, "subscribe to notifications")?;

        let key = (service_id, instance_id, event_method_id);
        // If a channel already exists for this exact key, subscribe a new receiver.
        if let Some(tx) = self.notification_channels.get(&key) {
            return Ok(tx.subscribe());
        }
        let (tx, rx) = broadcast::channel(buffer_size);
        self.notification_channels.insert(key, tx);
        Ok(rx)
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

    /// Send a SOME/IP frame over a TCP connection.
    ///
    /// Opens a new TCP connection to `dest`, writes the length-prefixed frame,
    /// and returns the stream for reading the response (if needed).
    async fn send_tcp_frame(
        &self,
        header: &MessageHeader,
        payload: &[u8],
        dest: SocketAddrV4,
    ) -> Result<TcpStream, AraComError> {
        let wire_hdr = encode_header(
            header,
            self.client_id,
            payload.len() as u32,
            DEFAULT_INTERFACE_VERSION,
        );
        let mut frame = Vec::with_capacity(HEADER_LEN + payload.len());
        frame.extend_from_slice(&wire_hdr);
        frame.extend_from_slice(payload);

        let mut stream = TcpStream::connect(dest)
            .await
            .map_err(|e| AraComError::Transport {
                message: format!("TCP connect to {dest} failed: {e}"),
            })?;

        stream
            .write_all(&frame)
            .await
            .map_err(|e| AraComError::Transport {
                message: format!("TCP write failed: {e}"),
            })?;

        Ok(stream)
    }

    /// Send a frame via the appropriate protocol (UDP or TCP).
    async fn send_frame(
        &self,
        header: &MessageHeader,
        payload: &[u8],
        endpoint: ResolvedEndpoint,
    ) -> Result<Option<TcpStream>, AraComError> {
        match endpoint {
            ResolvedEndpoint::Udp(dest) => {
                self.send_datagram(header, payload, dest).await?;
                Ok(None)
            }
            ResolvedEndpoint::Tcp(dest) => {
                let stream = self.send_tcp_frame(header, payload, dest).await?;
                Ok(Some(stream))
            }
        }
    }

    /// Return the UDP endpoint for a locally offered service by service/instance ID.
    fn local_service_endpoint(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
    ) -> Option<SocketAddrV4> {
        self.config
            .services
            .iter()
            .find(|s| s.service_id == service_id && s.instance_id == instance_id)
            .and_then(|s| s.endpoint.udp)
    }
}

#[async_trait]
impl Transport for SomeIpTransport {
    async fn send_request(
        &self,
        mut header: MessageHeader,
        payload: Bytes,
    ) -> Result<(MessageHeader, Bytes), AraComError> {
        let endpoint = self
            .resolve_endpoint(header.service_id, header.instance_id, payload.len())
            .ok_or_else(|| AraComError::ServiceNotAvailable {
                service_id: header.service_id,
                instance_id: header.instance_id,
            })?;

        let session_id = self.next_session_id();
        header.session_id = session_id;
        header.message_type = MessageType::Request;

        let timeout_dur = tokio::time::Duration::from_millis(5000);

        match endpoint {
            ResolvedEndpoint::Udp(dest) => {
                // UDP: correlate via pending_requests map + receive loop
                let (tx, rx) = oneshot::channel();
                self.pending_requests.insert(session_id, tx);

                if let Err(e) = self.send_datagram(&header, &payload, dest).await {
                    self.pending_requests.remove(&session_id);
                    return Err(e);
                }

                match tokio::time::timeout(timeout_dur, rx).await {
                    Ok(Ok((resp_header, resp_payload))) => Ok((resp_header, resp_payload)),
                    Ok(Err(_)) => Err(AraComError::Timeout {
                        timeout_ms: timeout_dur.as_millis() as u64,
                    }),
                    Err(_) => {
                        self.pending_requests.remove(&session_id);
                        Err(AraComError::Timeout {
                            timeout_ms: timeout_dur.as_millis() as u64,
                        })
                    }
                }
            }
            ResolvedEndpoint::Tcp(dest) => {
                // TCP: send request and read response on the same stream
                let mut stream = self.send_tcp_frame(&header, &payload, dest).await?;

                match tokio::time::timeout(timeout_dur, read_tcp_frame(&mut stream)).await {
                    Ok(Ok((resp_header, resp_payload))) => Ok((resp_header, resp_payload)),
                    Ok(Err(e)) => Err(AraComError::Transport {
                        message: format!("TCP response read failed: {e}"),
                    }),
                    Err(_) => Err(AraComError::Timeout {
                        timeout_ms: timeout_dur.as_millis() as u64,
                    }),
                }
            }
        }
    }

    async fn send_fire_and_forget(
        &self,
        mut header: MessageHeader,
        payload: Bytes,
    ) -> Result<(), AraComError> {
        let endpoint = self
            .resolve_endpoint(header.service_id, header.instance_id, payload.len())
            .ok_or_else(|| AraComError::ServiceNotAvailable {
                service_id: header.service_id,
                instance_id: header.instance_id,
            })?;

        header.session_id = self.next_session_id();
        header.message_type = MessageType::RequestNoReturn;

        self.send_frame(&header, &payload, endpoint).await?;
        Ok(())
    }

    async fn send_notification(
        &self,
        mut header: MessageHeader,
        payload: Bytes,
    ) -> Result<(), AraComError> {
        header.message_type = MessageType::Notification;
        // Notifications use session_id 0 per AUTOSAR SOME/IP spec (§4.2.1).
        header.session_id = 0;

        // Resolve which event group(s) this event belongs to via config.
        // header.method_id is the event_id on the wire; we look up the
        // matching event_group_id(s) from the service's EventGroupConfig
        // and only fan out to subscribers of those groups.
        let event_id = EventId(header.method_id.0);
        let matching_groups: Vec<EventGroupId> = self
            .config
            .services
            .iter()
            .filter(|s| s.service_id == header.service_id)
            .flat_map(|s| &s.event_groups)
            .filter(|eg| eg.events.contains(&event_id))
            .map(|eg| eg.event_group_id)
            .collect();

        let destinations: Vec<SocketAddrV4> = if matching_groups.is_empty() {
            // No event-group config found — fall back to all subscribers for
            // this service_id + instance_id. This preserves behaviour for
            // services whose config does not populate event_groups (e.g. simple
            // tests).
            self.event_subscribers
                .iter()
                .filter(|entry| {
                    entry.key().0 == header.service_id && entry.key().1 == header.instance_id
                })
                .flat_map(|entry| entry.value().clone())
                .collect()
        } else {
            // Deliver only to subscribers whose event_group matches
            // for this specific instance.
            matching_groups
                .iter()
                .flat_map(|gid| {
                    self.event_subscribers
                        .get(&(header.service_id, header.instance_id, *gid))
                        .map(|v| v.value().clone())
                        .unwrap_or_default()
                })
                .collect()
        };

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
        self.enforce_instance_binding(service_id, instance_id, "offer service")?;

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
                // Register locally so FindService replies can find us.
                self.offered_services
                    .insert((service_id, instance_id), (major_version, minor_version));

                // Determine the UDP endpoint for this service.
                let udp_endpoint = self.local_service_endpoint(service_id, instance_id);

                self.sd
                    .offer_service(
                        service_id,
                        instance_id,
                        major_version,
                        minor_version,
                        udp_endpoint,
                    )
                    .await
                    .map_err(AraComError::from)
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
                self.offered_services.remove(&(service_id, instance_id));
                self.sd
                    .stop_offer_service(service_id, instance_id)
                    .await
                    .map_err(AraComError::from)
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
                if self.resolve_endpoint(service_id, instance_id, 0).is_some() {
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
                let found: FoundService = self
                    .sd
                    .find_service(service_id, instance_id, major_version, minor_version)
                    .await
                    .map_err(AraComError::from)?;

                Ok(ServiceInstanceId {
                    service_id: found.service_id,
                    instance_id: found.instance_id,
                    major_version: found.major_version,
                    minor_version: found.minor_version,
                })
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
        self.enforce_instance_binding(service_id, instance_id, "register handler")?;
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
        self.enforce_instance_binding(service_id, instance_id, "subscribe to event group")?;

        match self.config.discovery_mode {
            DiscoveryMode::Static => {
                // In static mode the subscription is implicitly configured.
                // Verify the remote service endpoint is known so we can surface
                // misconfigurations early.
                if self.resolve_endpoint(service_id, instance_id, 0).is_none() {
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
                // Determine our local endpoint to tell the skeleton where to send events.
                // The socket is bound to 0.0.0.0:port; we use the configured unicast
                // address as the IP so the skeleton can route event datagrams back to us.
                let data_port = self
                    .udp_socket
                    .as_ref()
                    .and_then(|s| s.local_addr().ok())
                    .map(|addr| addr.port())
                    .ok_or(AraComError::Transport {
                        message: "transport not bound — call bind() first".to_string(),
                    })?;
                let subscriber_endpoint = SocketAddrV4::new(self.config.unicast, data_port);

                // Use the major version from the found service if available.
                let major_version = self
                    .sd
                    .found_services()
                    .get(&(service_id, instance_id))
                    .map(|fs| fs.major_version)
                    .unwrap_or(MajorVersion(0));

                self.sd
                    .subscribe_eventgroup(
                        service_id,
                        instance_id,
                        event_group_id,
                        subscriber_endpoint,
                        major_version,
                    )
                    .await
                    .map_err(AraComError::from)
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
                    .remove(&(service_id, instance_id, event_group_id));
                debug!(
                    %service_id, %instance_id, %event_group_id,
                    "unsubscribed from event group (static mode)"
                );
                Ok(())
            }
            DiscoveryMode::SomeIpSd => self
                .sd
                .unsubscribe_eventgroup(service_id, instance_id, event_group_id)
                .await
                .map_err(AraComError::from),
        }
    }
}

impl Drop for SomeIpTransport {
    fn drop(&mut self) {
        if let Some(handle) = self.recv_handle.take() {
            handle.abort();
        }
        for handle in self.tcp_listener_handles.drain(..) {
            handle.abort();
        }
    }
}

// ---------------------------------------------------------------------------
// TCP helpers (free functions)
// ---------------------------------------------------------------------------

/// Read a single SOME/IP frame from a TCP stream.
///
/// SOME/IP over TCP uses the Length field in the 16-byte header for framing:
/// read 16 bytes (header), extract payload length, read that many bytes.
async fn read_tcp_frame(stream: &mut TcpStream) -> Result<(MessageHeader, Bytes), SomeIpError> {
    let mut hdr_buf = [0u8; HEADER_LEN];
    stream
        .read_exact(&mut hdr_buf)
        .await
        .map_err(|e| SomeIpError::Header(format!("TCP header read: {e}")))?;

    let (hdr, _client_id, payload_len, _iface_ver) = decode_header(&hdr_buf)?;
    let payload_len = payload_len as usize;

    let mut payload_buf = vec![0u8; payload_len];
    if payload_len > 0 {
        stream
            .read_exact(&mut payload_buf)
            .await
            .map_err(|e| SomeIpError::Header(format!("TCP payload read: {e}")))?;
    }

    Ok((hdr, Bytes::from(payload_buf)))
}

/// Handle a single accepted TCP connection on the server side.
///
/// Reads SOME/IP frames in a loop and dispatches them the same way the UDP
/// receive loop does. Responses are written back on the same TCP stream.
async fn handle_tcp_connection(
    mut stream: TcpStream,
    pending: Arc<DashMap<u16, PendingResponse>>,
    handlers: Arc<DashMap<(ServiceId, InstanceId), RequestHandler>>,
    notif_channels: Arc<DashMap<(ServiceId, InstanceId, MethodId), broadcast::Sender<Bytes>>>,
) -> Result<(), SomeIpError> {
    loop {
        let (hdr, payload) = read_tcp_frame(&mut stream).await?;

        match hdr.message_type {
            MessageType::Response | MessageType::Error => {
                if let Some((_, sender)) = pending.remove(&hdr.session_id) {
                    let _ = sender.send((hdr, payload));
                } else {
                    warn!(
                        session_id = hdr.session_id,
                        "TCP: response with no pending request"
                    );
                }
            }

            MessageType::Request => {
                let handler_ref = handlers.iter().find(|e| e.key().0 == hdr.service_id);
                if let Some(handler) = handler_ref {
                    let resp_fut = handler.value()(hdr.clone(), payload);
                    match resp_fut.await {
                        Ok(resp_payload) => {
                            let resp_header = MessageHeader {
                                service_id: hdr.service_id,
                                method_id: hdr.method_id,
                                instance_id: hdr.instance_id,
                                session_id: hdr.session_id,
                                message_type: MessageType::Response,
                                return_code: ReturnCode::Ok,
                            };
                            let wire_hdr = encode_header(
                                &resp_header,
                                0,
                                resp_payload.len() as u32,
                                DEFAULT_INTERFACE_VERSION,
                            );
                            let mut frame = Vec::with_capacity(HEADER_LEN + resp_payload.len());
                            frame.extend_from_slice(&wire_hdr);
                            frame.extend_from_slice(&resp_payload);
                            stream.write_all(&frame).await.map_err(|e| {
                                SomeIpError::Header(format!("TCP response write: {e}"))
                            })?;
                        }
                        Err(e) => {
                            error!(error = %e, "TCP request handler failed");
                            let err_header = MessageHeader {
                                service_id: hdr.service_id,
                                method_id: hdr.method_id,
                                instance_id: hdr.instance_id,
                                session_id: hdr.session_id,
                                message_type: MessageType::Error,
                                return_code: ReturnCode::NotOk,
                            };
                            let wire_hdr =
                                encode_header(&err_header, 0, 0, DEFAULT_INTERFACE_VERSION);
                            stream.write_all(&wire_hdr).await.map_err(|e| {
                                SomeIpError::Header(format!("TCP error write: {e}"))
                            })?;
                        }
                    }
                } else {
                    warn!(
                        service_id = %hdr.service_id,
                        "TCP: no handler for incoming request"
                    );
                }
            }

            MessageType::Notification => {
                if let Some(entry) = notif_channels
                    .iter()
                    .find(|e| e.key().0 == hdr.service_id && e.key().2 == hdr.method_id)
                {
                    let _ = entry.value().send(payload.clone());
                }
            }

            MessageType::RequestNoReturn => {
                let handler_ref = handlers.iter().find(|e| e.key().0 == hdr.service_id);
                if let Some(handler) = handler_ref {
                    let fut = handler.value()(hdr, payload);
                    tokio::spawn(async move {
                        if let Err(e) = fut.await {
                            warn!(error = %e, "TCP fire-and-forget handler error");
                        }
                    });
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;
    use crate::config::SdConfig;

    fn minimal_config() -> SomeIpConfig {
        SomeIpConfig {
            unicast: Ipv4Addr::LOCALHOST,
            client_id: 0x0001,
            discovery_mode: DiscoveryMode::Static,
            sd_config: SdConfig::default(),
            services: vec![],
            remote_services: vec![],
            udp_threshold: 1400,
        }
    }

    #[test]
    fn test_session_id_starts_at_1() {
        let transport = SomeIpTransport::new(minimal_config());
        let id = transport.next_session_id();
        assert_eq!(id, 1, "first session ID must be 1");
    }

    #[test]
    fn test_session_id_monotonic() {
        let transport = SomeIpTransport::new(minimal_config());
        let mut seen = std::collections::HashSet::new();
        for _ in 0..100 {
            let id = transport.next_session_id();
            assert_ne!(id, 0, "session ID 0 is reserved and must never be returned");
            assert!(seen.insert(id), "duplicate session ID {id} detected");
        }
    }

    #[test]
    fn test_socket_returns_error_before_bind() {
        let transport = SomeIpTransport::new(minimal_config());
        let result = transport.socket();
        assert!(
            result.is_err(),
            "socket() must return an error when transport is not yet bound"
        );
    }
}
