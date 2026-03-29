//! `ara-com` — Core traits and async abstractions for Adaptive AUTOSAR communication in Rust.
//!
//! This crate defines the vocabulary types and async abstractions that all
//! transport backends implement. It has **zero** transport dependencies; it is
//! the stable API surface that user-generated code (from `cargo-arxml`) depends on.
//!
//! # Module overview
//!
//! | Module | Contents |
//! |---|---|
//! | [`types`] | Newtype wrappers for SOME/IP / AUTOSAR identifiers |
//! | [`error`] | Unified [`AraComError`] type |
//! | [`transport`] | [`Transport`] backend trait + serialization traits |
//! | [`service`] | [`ServiceDefinition`] marker trait + [`ServiceHandle`] |
//! | [`method`] | [`MethodConfig`] + [`MethodResult`] |
//! | [`event`] | [`SubscriptionState`], [`EventConfig`], [`EventStream`] |
//! | [`field`] | [`FieldConfig`] + getter/setter/notifier traits |
//! | [`proxy`] | [`ProxyBase`] + [`Proxy`] trait |
//! | [`skeleton`] | [`SkeletonBase`] + [`Skeleton`] trait |

pub mod error;
pub mod event;
pub mod field;
pub mod method;
pub mod proxy;
pub mod serialization;
pub mod service;
pub mod skeleton;
pub mod transport;
pub mod types;

// --- Convenience re-exports ---

pub use error::AraComError;
pub use event::{EventConfig, EventStream, SubscriptionState};
pub use field::FieldConfig;
pub use method::{MethodConfig, MethodResult};
pub use proxy::{Proxy, ProxyBase};
pub use service::{AvailabilityHandler, ServiceDefinition, ServiceHandle, ServiceState};
pub use skeleton::{Skeleton, SkeletonBase};
pub use transport::{
    AraDeserialize, AraSerialize, MessageHeader, MessageType, ReturnCode, Transport,
};
pub use types::{
    EventGroupId, EventId, InstanceId, MajorVersion, MethodId, MinorVersion, ServiceId,
    ServiceInstanceId,
};
