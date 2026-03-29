use thiserror::Error;
use crate::types::*;

#[derive(Error, Debug)]
pub enum AraComError {
    #[error("transport error: {message}")]
    Transport { message: String },

    #[error("serialization error: {message}")]
    Serialization { message: String },

    #[error("deserialization error: {message}")]
    Deserialization { message: String },

    #[error("service {service_id:?} instance {instance_id:?} not available")]
    ServiceNotAvailable {
        service_id: ServiceId,
        instance_id: InstanceId,
    },

    #[error("method call timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("service discovery error: {message}")]
    Discovery { message: String },

    #[error("subscription error: {message}")]
    Subscription { message: String },

    #[error("application error: code={code}, message={message}")]
    Application { code: u8, message: String },

    #[error("protocol error: return_code={return_code:?}")]
    Protocol { return_code: crate::transport::ReturnCode },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
