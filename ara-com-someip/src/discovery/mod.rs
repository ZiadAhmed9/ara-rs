//! SOME/IP Service Discovery (SOME/IP-SD) runtime state machine.
//!
//! This module provides the [`ServiceDiscovery`] struct which manages:
//! - Locally offered services and their SD announcements
//! - Remote service discovery via FindService/OfferService exchange
//! - EventGroup subscription management
//!
//! The SD messages use a separate UDP socket bound to the SD multicast group
//! (default 239.224.224.224:30490). This socket is completely independent of
//! the main SOME/IP data sockets used for RPC and event delivery.

pub mod message;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::UdpSocket;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use ara_com::types::{EventGroupId, InstanceId, MajorVersion, MinorVersion, ServiceId};

/// Type alias for the find-service waiter map to satisfy clippy type_complexity.
type FindWaiterMap = Arc<DashMap<(ServiceId, InstanceId), Vec<oneshot::Sender<FoundService>>>>;

use crate::config::SdConfig;
use crate::error::SomeIpError;

use message::{SdEntry, SdMessage, SdOption, TransportProtocol, SD_METHOD_ID, SD_SERVICE_ID};

use crate::transport::header::{encode_header, DEFAULT_INTERFACE_VERSION, HEADER_LEN};
use ara_com::transport::{MessageHeader, MessageType, ReturnCode};
use ara_com::types::MethodId;

// ---------------------------------------------------------------------------
// SD constants
// ---------------------------------------------------------------------------

/// Default TTL used in FindService query entries (seconds).
const DEFAULT_FIND_TTL: u32 = 3;

/// Timeout for find_service requests.
const FIND_SERVICE_TIMEOUT: Duration = Duration::from_secs(5);

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// Metadata for a locally offered service instance.
#[derive(Debug, Clone)]
pub struct OfferedService {
    pub service_id: ServiceId,
    pub instance_id: InstanceId,
    pub major_version: MajorVersion,
    pub minor_version: MinorVersion,
    /// UDP endpoint that data-plane traffic should be directed to.
    pub udp_endpoint: Option<SocketAddrV4>,
    /// TCP endpoint (future use).
    pub tcp_endpoint: Option<SocketAddrV4>,
}

/// Metadata for a remotely discovered service instance.
#[derive(Debug, Clone)]
pub struct FoundService {
    pub service_id: ServiceId,
    pub instance_id: InstanceId,
    pub major_version: MajorVersion,
    pub minor_version: MinorVersion,
    /// UDP endpoint received via the SD IPv4 endpoint option.
    pub udp_endpoint: Option<SocketAddrV4>,
    /// TCP endpoint received via the SD IPv4 endpoint option.
    pub tcp_endpoint: Option<SocketAddrV4>,
    /// When the TTL expires (absolute monotonic time).
    pub ttl_expires_at: Instant,
}

/// State of an eventgroup subscription.
#[derive(Debug, Clone)]
pub struct SubscriptionEntry {
    pub service_id: ServiceId,
    pub instance_id: InstanceId,
    pub event_group_id: EventGroupId,
    /// Our local endpoint that the skeleton should send events to.
    pub subscriber_endpoint: SocketAddrV4,
}

// ---------------------------------------------------------------------------
// EventSubscriberCallback
// ---------------------------------------------------------------------------

/// Callback type for notifying the transport layer about new event subscribers
/// that were registered via SD (SubscribeEventgroup message from a remote proxy).
///
/// The SD background loop calls this when it receives a SubscribeEventgroup
/// entry for one of our offered services.
pub type EventSubscriberCallback =
    Arc<dyn Fn(ServiceId, InstanceId, EventGroupId, SocketAddrV4, bool) + Send + Sync>;

// ---------------------------------------------------------------------------
// ServiceDiscovery
// ---------------------------------------------------------------------------

/// SOME/IP Service Discovery runtime.
///
/// Manages a separate SD multicast socket and a background receive loop that
/// processes incoming SD messages and drives the discovery state machine.
///
/// # Lifecycle
///
/// 1. Construct with [`ServiceDiscovery::new`].
/// 2. Optionally set an event subscriber callback with
///    [`ServiceDiscovery::set_event_subscriber_callback`].
/// 3. Call [`ServiceDiscovery::start`] to bind the SD socket and spawn the
///    background task.
/// 4. Use [`offer_service`], [`stop_offer_service`], [`find_service`],
///    [`subscribe_eventgroup`], [`unsubscribe_eventgroup`] as needed.
pub struct ServiceDiscovery {
    config: SdConfig,
    /// Locally offered services: (service_id, instance_id) → metadata.
    offered_services: Arc<DashMap<(ServiceId, InstanceId), OfferedService>>,
    /// Remotely discovered services: (service_id, instance_id) → metadata.
    found_services: Arc<DashMap<(ServiceId, InstanceId), FoundService>>,
    /// Pending find_service waiters: resolved when we receive an OfferService.
    find_waiters: FindWaiterMap,
    /// Eventgroup subscriptions (proxy side): what we have subscribed to.
    subscriptions: Arc<DashMap<(ServiceId, InstanceId, EventGroupId), SubscriptionEntry>>,
    /// SD multicast UDP socket (separate from data-plane sockets).
    sd_socket: Option<Arc<UdpSocket>>,
    /// Background receive task handle.
    task_handle: Option<JoinHandle<()>>,
    /// Unicast address of this node, embedded in SD endpoint options.
    unicast_addr: Ipv4Addr,
    /// Callback invoked by the background loop to register/unregister event
    /// subscribers into the transport's event_subscribers map.
    event_subscriber_cb: Option<EventSubscriberCallback>,
}

impl ServiceDiscovery {
    /// Create a new `ServiceDiscovery` with the given SD configuration.
    ///
    /// `unicast_addr` is the IPv4 address of this node, embedded in SD
    /// endpoint option fields so remote nodes know how to reach us.
    pub fn new(config: SdConfig, unicast_addr: Ipv4Addr) -> Self {
        Self {
            config,
            offered_services: Arc::new(DashMap::new()),
            found_services: Arc::new(DashMap::new()),
            find_waiters: Arc::new(DashMap::new()),
            subscriptions: Arc::new(DashMap::new()),
            sd_socket: None,
            task_handle: None,
            unicast_addr,
            event_subscriber_cb: None,
        }
    }

    /// Register a callback that will be invoked when a remote proxy sends a
    /// `SubscribeEventgroup` or `StopSubscribeEventgroup` SD message for one of
    /// our offered services.
    ///
    /// The callback signature is `(service_id, event_group_id, endpoint, subscribe)`.
    /// When `subscribe` is `true` the endpoint should be added; when `false` it
    /// should be removed.
    pub fn set_event_subscriber_callback(&mut self, cb: EventSubscriberCallback) {
        self.event_subscriber_cb = Some(cb);
    }

    /// Return a shared reference to the found-services table.
    ///
    /// The transport uses this for endpoint resolution in SD mode.
    pub fn found_services(&self) -> Arc<DashMap<(ServiceId, InstanceId), FoundService>> {
        self.found_services.clone()
    }

    /// Bind the SD multicast socket and spawn the background receive loop.
    ///
    /// Must be called before any SD send operations.
    pub async fn start(&mut self) -> Result<(), SomeIpError> {
        // Use socket2 to set SO_REUSEADDR/SO_REUSEPORT before binding.
        // This is required for multicast: multiple processes (or transports in
        // the same test process) must bind to the same SD port.
        let raw_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        raw_socket.set_reuse_address(true)?;
        raw_socket.set_nonblocking(true)?;

        let bind_addr: std::net::SocketAddr =
            format!("0.0.0.0:{}", self.config.port).parse().unwrap();
        raw_socket.bind(&bind_addr.into())?;

        let std_socket: std::net::UdpSocket = raw_socket.into();
        let socket = UdpSocket::from_std(std_socket)?;

        // Join the multicast group on the loopback interface.
        // On Linux this requires specifying the interface address.
        socket.join_multicast_v4(self.config.multicast_group, Ipv4Addr::UNSPECIFIED)?;

        // Enable multicast loopback so both endpoints on the same host see the
        // messages (required for loopback integration tests).
        socket.set_multicast_loop_v4(true)?;

        debug!(
            multicast = %self.config.multicast_group,
            port = self.config.port,
            "SD socket bound"
        );

        let socket = Arc::new(socket);
        self.sd_socket = Some(socket.clone());

        // Clone all shared state for the background task.
        let offered_services = self.offered_services.clone();
        let found_services = self.found_services.clone();
        let find_waiters = self.find_waiters.clone();
        let event_subscriber_cb = self.event_subscriber_cb.clone();
        let sd_socket_bg = socket.clone();
        let unicast_addr = self.unicast_addr;
        let sd_ttl = self.config.ttl;

        let handle = tokio::spawn(async move {
            Self::receive_loop(
                sd_socket_bg,
                offered_services,
                found_services,
                find_waiters,
                event_subscriber_cb,
                unicast_addr,
                sd_ttl,
            )
            .await;
        });

        self.task_handle = Some(handle);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Skeleton-side operations
    // -----------------------------------------------------------------------

    /// Announce a locally offered service via SD multicast OfferService.
    pub async fn offer_service(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        major_version: MajorVersion,
        minor_version: MinorVersion,
        udp_endpoint: Option<SocketAddrV4>,
    ) -> Result<(), SomeIpError> {
        let offered = OfferedService {
            service_id,
            instance_id,
            major_version,
            minor_version,
            udp_endpoint,
            tcp_endpoint: None,
        };
        self.offered_services
            .insert((service_id, instance_id), offered);

        // Build the SD OfferService message using the configured TTL.
        let ttl = self.config.ttl;
        let entry = SdEntry::OfferService {
            service_id,
            instance_id,
            major_version,
            minor_version,
            ttl,
        };
        let mut options = vec![];
        if let Some(ep) = udp_endpoint {
            options.push(SdOption::Ipv4Endpoint {
                addr: *ep.ip(),
                port: ep.port(),
                protocol: TransportProtocol::Udp,
            });
        }
        let sd_msg = SdMessage {
            reboot_flag: false,
            unicast_flag: false,
            entries: vec![entry],
            options,
        };

        self.send_sd_multicast(&sd_msg).await?;

        info!(%service_id, %instance_id, "SD OfferService sent");
        Ok(())
    }

    /// Send a StopOffer (TTL=0) and remove from offered services.
    pub async fn stop_offer_service(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
    ) -> Result<(), SomeIpError> {
        let offered = self.offered_services.remove(&(service_id, instance_id));
        let (major_version, minor_version) = offered
            .map(|(_, o)| (o.major_version, o.minor_version))
            .unwrap_or((MajorVersion(0), MinorVersion(0)));

        let entry = SdEntry::OfferService {
            service_id,
            instance_id,
            major_version,
            minor_version,
            ttl: 0, // TTL=0 signals stop
        };
        let sd_msg = SdMessage {
            reboot_flag: false,
            unicast_flag: false,
            entries: vec![entry],
            options: vec![],
        };

        self.send_sd_multicast(&sd_msg).await?;

        info!(%service_id, %instance_id, "SD StopOfferService sent");
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Proxy-side operations
    // -----------------------------------------------------------------------

    /// Send a FindService SD message and wait for an OfferService reply.
    ///
    /// Returns the discovered [`FoundService`] or an error if no response
    /// arrives within [`FIND_SERVICE_TIMEOUT`].
    pub async fn find_service(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        major_version: MajorVersion,
        minor_version: MinorVersion,
    ) -> Result<FoundService, SomeIpError> {
        // If we already discovered this service, return it immediately.
        if let Some(entry) = self.found_services.get(&(service_id, instance_id)) {
            if entry.ttl_expires_at > Instant::now() {
                return Ok(entry.clone());
            }
        }

        // Register a waiter before sending FindService to avoid a race where the
        // OfferService response arrives before we've registered.
        let (tx, rx) = oneshot::channel::<FoundService>();
        self.find_waiters
            .entry((service_id, instance_id))
            .or_default()
            .push(tx);

        // Send the FindService multicast.
        let entry = SdEntry::FindService {
            service_id,
            instance_id,
            major_version,
            minor_version,
            ttl: DEFAULT_FIND_TTL,
        };
        let sd_msg = SdMessage {
            reboot_flag: false,
            unicast_flag: false,
            entries: vec![entry],
            options: vec![],
        };
        if let Err(e) = self.send_sd_multicast(&sd_msg).await {
            // Remove our waiter on send failure.
            if let Some(waiters) = self.find_waiters.get_mut(&(service_id, instance_id)) {
                // The sender was moved into the channel; we can't identify it precisely.
                // Drop the whole entry if there's only one waiter (common case).
                if waiters.len() <= 1 {
                    drop(waiters);
                    self.find_waiters.remove(&(service_id, instance_id));
                }
            }
            return Err(e);
        }

        debug!(%service_id, %instance_id, "SD FindService sent — waiting for OfferService");

        // Wait for the background loop to resolve our waiter.
        let result = tokio::time::timeout(FIND_SERVICE_TIMEOUT, rx).await;

        // On timeout or channel-close, clean up the dead waiter to prevent
        // accumulation of stale senders in the map.
        let cleanup = |key: (ServiceId, InstanceId)| {
            if let Some(mut waiters) = self.find_waiters.get_mut(&key) {
                // Remove senders whose receiver has been dropped (i.e. ours).
                waiters.retain(|w| !w.is_closed());
                if waiters.is_empty() {
                    drop(waiters);
                    self.find_waiters.remove(&key);
                }
            }
        };

        match result {
            Ok(Ok(found)) => {
                info!(%service_id, %instance_id, "SD FindService resolved");
                Ok(found)
            }
            Ok(Err(_)) => {
                cleanup((service_id, instance_id));
                Err(SomeIpError::Discovery(format!(
                    "find_service waiter channel closed for service {service_id} instance {instance_id}"
                )))
            }
            Err(_) => {
                cleanup((service_id, instance_id));
                Err(SomeIpError::Discovery(format!(
                    "find_service timed out after {}s for service {service_id} instance {instance_id}",
                    FIND_SERVICE_TIMEOUT.as_secs()
                )))
            }
        }
    }

    /// Send a SubscribeEventgroup SD message to the service's unicast endpoint.
    pub async fn subscribe_eventgroup(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        event_group_id: EventGroupId,
        subscriber_endpoint: SocketAddrV4,
        major_version: MajorVersion,
    ) -> Result<(), SomeIpError> {
        // Verify the service has been discovered.
        if !self.found_services.contains_key(&(service_id, instance_id)) {
            return Err(SomeIpError::Discovery(format!(
                "subscribe_eventgroup: service {service_id} instance {instance_id} not found"
            )));
        }

        let entry = SdEntry::SubscribeEventgroup {
            service_id,
            instance_id,
            major_version,
            ttl: self.config.ttl,
            event_group_id,
            counter: 0,
        };
        let options = vec![SdOption::Ipv4Endpoint {
            addr: *subscriber_endpoint.ip(),
            port: subscriber_endpoint.port(),
            protocol: TransportProtocol::Udp,
        }];
        let sd_msg = SdMessage {
            reboot_flag: false,
            unicast_flag: true,
            entries: vec![entry],
            options,
        };

        // Send via multicast so all nodes on the SD group receive the
        // subscription. This also works on loopback where unicast delivery
        // with SO_REUSEADDR is ambiguous.
        self.send_sd_multicast(&sd_msg).await?;

        self.subscriptions.insert(
            (service_id, instance_id, event_group_id),
            SubscriptionEntry {
                service_id,
                instance_id,
                event_group_id,
                subscriber_endpoint,
            },
        );

        info!(%service_id, %instance_id, %event_group_id, "SD SubscribeEventgroup sent");
        Ok(())
    }

    /// Send a StopSubscribeEventgroup SD message to the service's unicast endpoint.
    pub async fn unsubscribe_eventgroup(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        event_group_id: EventGroupId,
    ) -> Result<(), SomeIpError> {
        // Capture the subscription entry before removing it (we need the
        // subscriber endpoint to include in the StopSubscribeEventgroup options).
        let removed_sub = self
            .subscriptions
            .remove(&(service_id, instance_id, event_group_id));

        // Include the subscriber's own endpoint in the options so the
        // skeleton can identify which subscriber is unsubscribing.
        let subscriber_ep = removed_sub.map(|(_, s)| s.subscriber_endpoint);
        let options = if let Some(sub_ep) = subscriber_ep {
            vec![SdOption::Ipv4Endpoint {
                addr: *sub_ep.ip(),
                port: sub_ep.port(),
                protocol: TransportProtocol::Udp,
            }]
        } else {
            vec![]
        };

        let entry = SdEntry::StopSubscribeEventgroup {
            service_id,
            instance_id,
            event_group_id,
        };
        let sd_msg = SdMessage {
            reboot_flag: false,
            unicast_flag: false,
            entries: vec![entry],
            options,
        };
        self.send_sd_multicast(&sd_msg).await?;
        info!(%service_id, %instance_id, %event_group_id, "SD StopSubscribeEventgroup sent");

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn socket(&self) -> Result<&Arc<UdpSocket>, SomeIpError> {
        self.sd_socket.as_ref().ok_or_else(|| {
            SomeIpError::Discovery("SD not started — call start() first".to_string())
        })
    }

    async fn send_sd_multicast(&self, msg: &SdMessage) -> Result<(), SomeIpError> {
        let dest = SocketAddrV4::new(self.config.multicast_group, self.config.port);
        self.send_sd_unicast(msg, dest).await
    }

    async fn send_sd_unicast(
        &self,
        msg: &SdMessage,
        dest: SocketAddrV4,
    ) -> Result<(), SomeIpError> {
        let socket = self.socket()?;
        let payload = msg.encode();
        let frame = build_sd_frame(&payload);
        socket
            .send_to(&frame, SocketAddr::V4(dest))
            .await
            .map_err(SomeIpError::Io)?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Background receive loop (static method, runs inside tokio::spawn)
    // -----------------------------------------------------------------------

    async fn receive_loop(
        socket: Arc<UdpSocket>,
        offered_services: Arc<DashMap<(ServiceId, InstanceId), OfferedService>>,
        found_services: Arc<DashMap<(ServiceId, InstanceId), FoundService>>,
        find_waiters: FindWaiterMap,
        event_subscriber_cb: Option<EventSubscriberCallback>,
        unicast_addr: Ipv4Addr,
        sd_ttl: u32,
    ) {
        let mut buf = vec![0u8; 65535];
        loop {
            let (len, src) = match socket.recv_from(&mut buf).await {
                Ok(r) => r,
                Err(e) => {
                    error!(error = %e, "SD socket recv_from failed");
                    continue;
                }
            };

            // SD messages have a 16-byte SOME/IP header followed by the SD payload.
            if len < HEADER_LEN {
                warn!(len, "SD datagram too short");
                continue;
            }

            // Quick check: is this an SD message (service_id = 0xFFFF, method_id = 0x8100)?
            let svc_id = u16::from_be_bytes([buf[0], buf[1]]);
            let meth_id = u16::from_be_bytes([buf[2], buf[3]]);
            if svc_id != SD_SERVICE_ID || meth_id != SD_METHOD_ID {
                // Not an SD message — ignore
                continue;
            }

            let sd_payload = &buf[HEADER_LEN..len];
            let sd_msg = match SdMessage::decode(sd_payload) {
                Ok(m) => m,
                Err(e) => {
                    warn!(error = %e, "failed to decode SD message");
                    continue;
                }
            };

            // Process each entry in the SD message.
            for entry in &sd_msg.entries {
                match entry {
                    SdEntry::OfferService {
                        service_id,
                        instance_id,
                        major_version,
                        minor_version,
                        ttl,
                    } => {
                        // Extract UDP endpoint from options (first IPv4 UDP option).
                        let udp_ep = extract_udp_endpoint(&sd_msg.options);
                        let tcp_ep = extract_tcp_endpoint(&sd_msg.options);

                        let ttl_secs = *ttl;
                        if ttl_secs == 0 {
                            // StopOffer: remove from found_services.
                            if found_services
                                .remove(&(*service_id, *instance_id))
                                .is_some()
                            {
                                info!(
                                    %service_id, %instance_id,
                                    "SD StopOffer received — service removed"
                                );
                            }
                            continue;
                        }

                        let found = FoundService {
                            service_id: *service_id,
                            instance_id: *instance_id,
                            major_version: *major_version,
                            minor_version: *minor_version,
                            udp_endpoint: udp_ep,
                            tcp_endpoint: tcp_ep,
                            ttl_expires_at: Instant::now() + Duration::from_secs(ttl_secs as u64),
                        };

                        info!(
                            %service_id, %instance_id,
                            udp = ?udp_ep,
                            "SD OfferService received"
                        );

                        found_services.insert((*service_id, *instance_id), found.clone());

                        // Wake up any pending find_service waiters.
                        if let Some((_, waiters)) =
                            find_waiters.remove(&(*service_id, *instance_id))
                        {
                            for tx in waiters {
                                let _ = tx.send(found.clone());
                            }
                        }
                    }

                    SdEntry::FindService {
                        service_id,
                        instance_id,
                        major_version: _,
                        minor_version: _,
                        ttl: _,
                    } => {
                        // If we're offering this service, respond with OfferService unicast.
                        if let Some(offered) = offered_services.get(&(*service_id, *instance_id)) {
                            let offered = offered.clone();
                            let reply_entry = SdEntry::OfferService {
                                service_id: offered.service_id,
                                instance_id: offered.instance_id,
                                major_version: offered.major_version,
                                minor_version: offered.minor_version,
                                ttl: sd_ttl,
                            };
                            let mut reply_options = vec![];
                            if let Some(ep) = offered.udp_endpoint {
                                reply_options.push(SdOption::Ipv4Endpoint {
                                    addr: *ep.ip(),
                                    port: ep.port(),
                                    protocol: TransportProtocol::Udp,
                                });
                            } else {
                                // Embed unicast_addr with a placeholder port (0) if no
                                // explicit endpoint was configured.
                                reply_options.push(SdOption::Ipv4Endpoint {
                                    addr: unicast_addr,
                                    port: 0,
                                    protocol: TransportProtocol::Udp,
                                });
                            }
                            let reply_msg = SdMessage {
                                reboot_flag: false,
                                unicast_flag: true,
                                entries: vec![reply_entry],
                                options: reply_options,
                            };
                            let frame = build_sd_frame(&reply_msg.encode());
                            let reply_socket = socket.clone();
                            tokio::spawn(async move {
                                if let Err(e) = reply_socket.send_to(&frame, src).await {
                                    warn!(error = %e, "failed to send SD unicast OfferService reply");
                                }
                            });
                            debug!(%service_id, %instance_id, %src, "SD FindService — replied with OfferService");
                        }
                    }

                    SdEntry::SubscribeEventgroup {
                        service_id,
                        instance_id,
                        major_version: _,
                        ttl,
                        event_group_id,
                        counter: _,
                    } => {
                        if offered_services.contains_key(&(*service_id, *instance_id)) {
                            // Extract the subscriber's endpoint from the options.
                            if let Some(subscriber_ep) = extract_udp_endpoint(&sd_msg.options) {
                                if *ttl == 0 {
                                    // TTL=0 in a SubscribeEventgroup is a stop (should use
                                    // StopSubscribeEventgroup, but handle both).
                                    if let Some(ref cb) = event_subscriber_cb {
                                        cb(
                                            *service_id,
                                            *instance_id,
                                            *event_group_id,
                                            subscriber_ep,
                                            false,
                                        );
                                    }
                                } else {
                                    info!(
                                        %service_id, %instance_id, %event_group_id,
                                        subscriber = %subscriber_ep,
                                        "SD SubscribeEventgroup received"
                                    );
                                    if let Some(ref cb) = event_subscriber_cb {
                                        cb(
                                            *service_id,
                                            *instance_id,
                                            *event_group_id,
                                            subscriber_ep,
                                            true,
                                        );
                                    }
                                }
                            } else {
                                warn!(
                                    %service_id, %event_group_id,
                                    "SubscribeEventgroup missing IPv4 endpoint option"
                                );
                            }
                        }
                    }

                    SdEntry::StopSubscribeEventgroup {
                        service_id,
                        instance_id,
                        event_group_id,
                    } => {
                        if offered_services.contains_key(&(*service_id, *instance_id)) {
                            if let Some(subscriber_ep) = extract_udp_endpoint(&sd_msg.options) {
                                info!(
                                    %service_id, %instance_id, %event_group_id,
                                    subscriber = %subscriber_ep,
                                    "SD StopSubscribeEventgroup received"
                                );
                                if let Some(ref cb) = event_subscriber_cb {
                                    cb(
                                        *service_id,
                                        *instance_id,
                                        *event_group_id,
                                        subscriber_ep,
                                        false,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Drop for ServiceDiscovery {
    fn drop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Wrap an SD payload in a SOME/IP header and return the complete frame.
fn build_sd_frame(sd_payload: &[u8]) -> Vec<u8> {
    let hdr = MessageHeader {
        service_id: ServiceId(SD_SERVICE_ID),
        method_id: MethodId(SD_METHOD_ID),
        instance_id: ara_com::types::InstanceId(0xFFFF),
        session_id: 0,
        message_type: MessageType::Notification,
        return_code: ReturnCode::Ok,
    };
    let wire_hdr = encode_header(&hdr, 0, sd_payload.len() as u32, DEFAULT_INTERFACE_VERSION);
    let mut frame = Vec::with_capacity(HEADER_LEN + sd_payload.len());
    frame.extend_from_slice(&wire_hdr);
    frame.extend_from_slice(sd_payload);
    frame
}

/// Extract the first IPv4 UDP endpoint from a list of SD options.
fn extract_udp_endpoint(options: &[SdOption]) -> Option<SocketAddrV4> {
    options.iter().find_map(|opt| match opt {
        SdOption::Ipv4Endpoint {
            addr,
            port,
            protocol: TransportProtocol::Udp,
        } => Some(SocketAddrV4::new(*addr, *port)),
        _ => None,
    })
}

/// Extract the first IPv4 TCP endpoint from a list of SD options.
fn extract_tcp_endpoint(options: &[SdOption]) -> Option<SocketAddrV4> {
    options.iter().find_map(|opt| match opt {
        SdOption::Ipv4Endpoint {
            addr,
            port,
            protocol: TransportProtocol::Tcp,
        } => Some(SocketAddrV4::new(*addr, *port)),
        _ => None,
    })
}
