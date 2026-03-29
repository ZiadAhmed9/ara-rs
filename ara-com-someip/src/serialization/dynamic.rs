//! Dynamic-length type serialization — SOME/IP wire format.
//!
//! Base `AraSerialize` / `AraDeserialize` impls for `String` and `Vec<T>` live
//! in `ara_com::serialization` where the traits are defined, satisfying Rust's
//! orphan rule.
//!
//! This module is reserved for **SOME/IP-specific** dynamic-length helpers,
//! such as context-aware encoding that honours `SerializationContext`
//! (configurable length-field width, UTF-16 string encoding, etc.).
//! Add them here in Week 5 once `SerializationContext` is threaded through.
