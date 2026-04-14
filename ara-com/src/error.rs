//! Unified error type for all `ara-com` operations.

use crate::types::*;
use thiserror::Error;

/// Error type returned by all `ara-com` and transport-backend operations.
///
/// Variants cover transport failures, serialization issues, service
/// discovery problems, and protocol-level errors. Transport backends
/// may convert their own error types into `AraComError` via `From` impls.
#[derive(Error, Debug)]
pub enum AraComError {
    /// A transport-level failure (e.g. socket closed, send failed).
    #[error("transport error: {message}")]
    Transport { message: String },

    /// A value could not be encoded to the SOME/IP wire format.
    #[error("serialization error: {message}")]
    Serialization { message: String },

    /// A received payload could not be decoded from the wire format.
    #[error("deserialization error: {message}")]
    Deserialization { message: String },

    /// The requested service instance was not found or is offline.
    #[error("service {service_id:?} instance {instance_id:?} not available")]
    ServiceNotAvailable {
        service_id: ServiceId,
        instance_id: InstanceId,
    },

    /// A request/response call exceeded the configured timeout.
    #[error("method call timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// A service-discovery operation failed (e.g. multicast unreachable).
    #[error("service discovery error: {message}")]
    Discovery { message: String },

    /// An event-group subscription was rejected or could not be established.
    #[error("subscription error: {message}")]
    Subscription { message: String },

    /// An application-level error returned by the server in a response.
    #[error("application error: code={code}, message={message}")]
    Application { code: u8, message: String },

    /// A SOME/IP protocol-level error indicated by the return code.
    #[error("protocol error: return_code={return_code:?}")]
    Protocol {
        return_code: crate::transport::ReturnCode,
    },

    /// An underlying I/O error (e.g. socket bind failure).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
