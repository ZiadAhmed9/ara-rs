//! `ara-com-someip` — SOME/IP transport backend for `ara-com`.
//!
//! This crate implements the [`ara_com::transport::Transport`] trait over
//! SOME/IP on UDP, providing:
//!
//! - **Request/response** with session-ID-based correlation and configurable
//!   timeout.
//! - **Fire-and-forget** (`RequestNoReturn`) for one-way calls.
//! - **Event notifications** with broadcast fan-out to subscribed proxies.
//! - **SOME/IP Service Discovery (SOME/IP-SD)** for dynamic offer/find and
//!   eventgroup subscription via multicast.
//! - **Static discovery** mode for pre-configured endpoints (useful in tests
//!   and constrained environments).
//!
//! # Quick start
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use ara_com_someip::config::{SomeIpConfig, DiscoveryMode};
//! use ara_com_someip::transport::SomeIpTransport;
//!
//! let config = SomeIpConfig { /* ... */ };
//! let mut transport = SomeIpTransport::new(config);
//! transport.bind().await?; // bind() requires &mut self
//! let transport = Arc::new(transport); // share after binding
//! // Now use the transport with generated proxy/skeleton code.
//! ```
//!
//! # Module overview
//!
//! | Module | Contents |
//! |---|---|
//! | [`transport`] | [`SomeIpTransport`](transport::SomeIpTransport) — the main entry point |
//! | [`config`] | Configuration structs ([`SomeIpConfig`](config::SomeIpConfig), [`SdConfig`](config::SdConfig), endpoint configs) |
//! | [`discovery`] | SOME/IP-SD state machine and message codec |
//! | [`error`] | [`SomeIpError`](error::SomeIpError) type |
//! | [`serialization`] | Wire-format context and re-exports of `ara-com` serialization traits |
//! | [`proxy`] | [`SomeIpProxy`](proxy::SomeIpProxy) runtime helper |
//! | [`skeleton`] | [`SomeIpSkeleton`](skeleton::SomeIpSkeleton) runtime helper |

pub mod config;
pub mod discovery;
pub mod error;
pub mod proxy;
pub mod serialization;
pub mod skeleton;
pub mod transport;
