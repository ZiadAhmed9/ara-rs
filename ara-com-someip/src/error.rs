//! SOME/IP-specific error types.

use thiserror::Error;

use ara_com::error::AraComError;

/// Errors specific to the SOME/IP transport layer.
///
/// A `From<SomeIpError> for AraComError` impl is provided so that methods
/// implementing the generic [`ara_com::transport::Transport`] trait can use `?`
/// to convert automatically. Some SOME/IP-specific APIs (e.g.
/// [`SomeIpTransport::bind`](crate::transport::SomeIpTransport::bind),
/// [`ServiceDiscovery::start`](crate::discovery::ServiceDiscovery::start))
/// return `SomeIpError` directly.
#[derive(Error, Debug)]
pub enum SomeIpError {
    /// The SOME/IP header could not be parsed or encoded.
    #[error("SOME/IP header error: {0}")]
    Header(String),

    /// Payload serialization failed at a specific byte offset.
    #[error("serialization error at offset {offset}: {message}")]
    Serialization { offset: usize, message: String },

    /// Payload deserialization failed at a specific byte offset.
    #[error("deserialization error at offset {offset}: {message}")]
    Deserialization { offset: usize, message: String },

    /// A service discovery operation failed.
    #[error("service discovery error: {0}")]
    Discovery(String),

    /// An endpoint configuration or binding error.
    #[error("endpoint error: {0}")]
    Endpoint(String),

    /// A pending request timed out waiting for a response.
    #[error("request timeout: session_id={session_id}")]
    Timeout { session_id: u16 },

    /// An underlying I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<SomeIpError> for AraComError {
    fn from(e: SomeIpError) -> Self {
        AraComError::Transport {
            message: e.to_string(),
        }
    }
}
