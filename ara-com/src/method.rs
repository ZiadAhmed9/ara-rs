use crate::error::AraComError;
use std::time::Duration;

/// Configuration for method calls
#[derive(Debug, Clone)]
pub struct MethodConfig {
    /// Timeout for request-response
    pub timeout: Duration,
    /// Number of retries on transport failure
    pub retries: u32,
}

impl Default for MethodConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            retries: 0,
        }
    }
}

/// Result of a method call
pub type MethodResult<T> = Result<T, AraComError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_config_defaults() {
        let config = MethodConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.retries, 0);
    }
}
