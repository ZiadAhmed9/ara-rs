use async_trait::async_trait;
use bytes::Bytes;
use crate::types::*;
use crate::error::AraComError;

/// Message metadata for routing
#[derive(Debug, Clone)]
pub struct MessageHeader {
    pub service_id: ServiceId,
    pub method_id: MethodId,
    pub instance_id: InstanceId,
    pub session_id: u16,
    pub message_type: MessageType,
    pub return_code: ReturnCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Request,
    RequestNoReturn,
    Notification,
    Response,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReturnCode {
    Ok = 0x00,
    NotOk = 0x01,
    UnknownService = 0x02,
    UnknownMethod = 0x03,
    NotReady = 0x04,
    NotReachable = 0x05,
    Timeout = 0x06,
    WrongProtocolVersion = 0x07,
    WrongInterfaceVersion = 0x08,
    MalformedMessage = 0x09,
    WrongMessageType = 0x0A,
}

/// Serialization trait for AUTOSAR-compatible wire format encoding
pub trait AraSerialize: Send + Sync {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError>;
    fn serialized_size(&self) -> usize;
}

/// Deserialization trait for AUTOSAR-compatible wire format decoding
pub trait AraDeserialize: Sized + Send + Sync {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError>;
}

/// Transport backend trait — implemented by ara-com-someip and future backends
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
            dyn Fn(MessageHeader, Bytes) -> futures_core::future::BoxFuture<'static, Result<Bytes, AraComError>>
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
