use thiserror::Error;

use ara_com::error::AraComError;

#[derive(Error, Debug)]
pub enum SomeIpError {
    #[error("SOME/IP header error: {0}")]
    Header(String),

    #[error("serialization error at offset {offset}: {message}")]
    Serialization { offset: usize, message: String },

    #[error("deserialization error at offset {offset}: {message}")]
    Deserialization { offset: usize, message: String },

    #[error("service discovery error: {0}")]
    Discovery(String),

    #[error("endpoint error: {0}")]
    Endpoint(String),

    #[error("request timeout: session_id={session_id}")]
    Timeout { session_id: u16 },

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
