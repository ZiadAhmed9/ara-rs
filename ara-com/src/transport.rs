use crate::error::AraComError;
use crate::types::*;
use async_trait::async_trait;
use bytes::Bytes;

/// SOME/IP message header carrying routing metadata.
///
/// Every message exchanged over a [`Transport`] is paired with a header that
/// identifies the target service, method, and instance. The transport layer
/// uses this information to route the message and correlate request/response
/// pairs via `session_id`.
#[derive(Debug, Clone)]
pub struct MessageHeader {
    /// Target service identifier.
    pub service_id: ServiceId,
    /// Target method or event identifier.
    pub method_id: MethodId,
    /// Target service instance.
    pub instance_id: InstanceId,
    /// Session identifier for request/response correlation.
    /// Transports typically assign this automatically; set to `0` in
    /// outgoing calls and let the transport fill it in.
    pub session_id: u16,
    /// Discriminates request, response, notification, and error frames.
    pub message_type: MessageType,
    /// SOME/IP return code (meaningful in responses and errors).
    pub return_code: ReturnCode,
}

/// SOME/IP message type discriminator (PRS_SOMEIP_00055).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Client-to-server request expecting a [`Response`](MessageType::Response).
    Request,
    /// Client-to-server fire-and-forget (no response expected).
    RequestNoReturn,
    /// Server-to-client event notification (unsolicited).
    Notification,
    /// Server-to-client response to a [`Request`](MessageType::Request).
    Response,
    /// Server-to-client error response.
    Error,
}

/// SOME/IP return code (PRS_SOMEIP_00058).
///
/// Carried in response and error messages to indicate success or the reason
/// for failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReturnCode {
    /// No error.
    Ok = 0x00,
    /// Unspecified error.
    NotOk = 0x01,
    /// The requested service is not known.
    UnknownService = 0x02,
    /// The requested method/event is not known.
    UnknownMethod = 0x03,
    /// The service is registered but not yet ready.
    NotReady = 0x04,
    /// The service instance is not reachable (network-level).
    NotReachable = 0x05,
    /// The request timed out on the server side.
    Timeout = 0x06,
    /// Protocol version mismatch.
    WrongProtocolVersion = 0x07,
    /// Interface version mismatch.
    WrongInterfaceVersion = 0x08,
    /// The message payload could not be parsed.
    MalformedMessage = 0x09,
    /// The message type is not valid in this context.
    WrongMessageType = 0x0A,
}

/// Serialization trait for AUTOSAR-compatible wire format encoding.
///
/// All primitive types, `String`, and `Vec<T>` implement this trait using
/// big-endian byte order and SOME/IP length-prefix conventions.
/// Generated struct types receive an implementation from `cargo-arxml`.
pub trait AraSerialize: Send + Sync {
    /// Append the wire-format bytes of `self` to `buf`.
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError>;
    /// Return the exact number of bytes that [`ara_serialize`](AraSerialize::ara_serialize)
    /// will append.
    fn serialized_size(&self) -> usize;
}

/// Deserialization trait for AUTOSAR-compatible wire format decoding.
///
/// Counterpart to [`AraSerialize`]. Implementations consume bytes from the
/// front of `buf` and return the decoded value.
pub trait AraDeserialize: Sized + Send + Sync {
    /// Decode a value from the beginning of `buf`.
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError>;
}

/// Async transport backend trait.
///
/// This is the primary extension point of `ara-com`. Transport backends
/// (such as `ara-com-someip`) implement this trait to provide the actual
/// network I/O. Generated proxy and skeleton code interacts with the
/// transport exclusively through this interface.
///
/// # Instance binding invariant
///
/// A transport enforces **one instance per service per transport**: once a
/// service ID is bound to an instance (via [`offer_service`](Transport::offer_service),
/// [`find_service`](Transport::find_service), or
/// [`subscribe_event_group`](Transport::subscribe_event_group)), attempting
/// to bind a *different* instance for the same service on the same transport
/// must return an error.
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    /// Send a request and wait for a response
    async fn send_request(
        &self,
        header: MessageHeader,
        payload: Bytes,
    ) -> Result<(MessageHeader, Bytes), AraComError>;

    /// Send a fire-and-forget message
    async fn send_fire_and_forget(
        &self,
        header: MessageHeader,
        payload: Bytes,
    ) -> Result<(), AraComError>;

    /// Send a notification/event
    async fn send_notification(
        &self,
        header: MessageHeader,
        payload: Bytes,
    ) -> Result<(), AraComError>;

    /// Offer a service instance (skeleton side)
    async fn offer_service(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        major_version: MajorVersion,
        minor_version: MinorVersion,
    ) -> Result<(), AraComError>;

    /// Stop offering a service
    async fn stop_offer_service(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
    ) -> Result<(), AraComError>;

    /// Find a service instance (proxy side)
    async fn find_service(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        major_version: MajorVersion,
        minor_version: MinorVersion,
    ) -> Result<ServiceInstanceId, AraComError>;

    /// Register a handler for incoming requests (skeleton side)
    async fn register_request_handler(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        handler: Box<
            dyn Fn(
                    MessageHeader,
                    Bytes,
                )
                    -> futures_core::future::BoxFuture<'static, Result<Bytes, AraComError>>
                + Send
                + Sync,
        >,
    ) -> Result<(), AraComError>;

    /// Subscribe to an event group
    async fn subscribe_event_group(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        event_group_id: EventGroupId,
    ) -> Result<(), AraComError>;

    /// Unsubscribe from an event group
    async fn unsubscribe_event_group(
        &self,
        service_id: ServiceId,
        instance_id: InstanceId,
        event_group_id: EventGroupId,
    ) -> Result<(), AraComError>;
}
