//! `cargo-arxml` — ARXML parser, validator, and Rust code generator for Adaptive AUTOSAR.
//!
//! This crate provides both a CLI binary (`cargo arxml`) and a library for
//! programmatic access to the ARXML-to-Rust pipeline.
//!
//! # Module overview
//!
//! | Module | Contents |
//! |---|---|
//! | [`parser`] | ARXML file loading, service interface extraction, intermediate representation |
//! | [`validator`] | Validation rules (duplicate IDs, missing type refs, method ID conflicts) |
//! | [`codegen`] | Rust code generation: types, traits, proxies, skeletons |
//! | [`config`] | `arxml.toml` configuration support |
//! | [`error`] | Unified error type for parser, validator, and codegen |

pub mod codegen;
pub mod config;
pub mod error;
pub mod parser;
pub mod validator;
