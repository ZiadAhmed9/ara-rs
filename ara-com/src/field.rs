use crate::error::AraComError;
use crate::event::EventStream;

/// Configuration for field access
#[derive(Debug, Clone)]
pub struct FieldConfig {
    pub has_getter: bool,
    pub has_setter: bool,
    pub has_notifier: bool,
}

impl Default for FieldConfig {
    fn default() -> Self {
        Self {
            has_getter: true,
            has_setter: true,
            has_notifier: true,
        }
    }
}

/// Trait for a readable field value
pub trait FieldGetter<T>: Send + Sync {
    /// Retrieve the current field value from the service.
    fn get(&self) -> impl std::future::Future<Output = Result<T, AraComError>> + Send;
}

/// Trait for a writable field value
pub trait FieldSetter<T>: Send + Sync {
    /// Set the field value on the service.
    fn set(&self, value: T) -> impl std::future::Future<Output = Result<(), AraComError>> + Send;
}

/// Trait for a field that emits change notifications
pub trait FieldNotifier<T>: Send + Sync {
    /// Subscribe to notifications for this field's value changes.
    fn subscribe(&self) -> EventStream<T>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_config_default() {
        let cfg = FieldConfig::default();
        assert!(cfg.has_getter);
        assert!(cfg.has_setter);
        assert!(cfg.has_notifier);
    }

    #[test]
    fn test_field_config_read_only() {
        let cfg = FieldConfig {
            has_getter: true,
            has_setter: false,
            has_notifier: false,
        };
        assert!(cfg.has_getter);
        assert!(!cfg.has_setter);
        assert!(!cfg.has_notifier);
    }
}
