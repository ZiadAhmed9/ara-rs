//! Fixed-length type serialization — SOME/IP big-endian byte order.
//!
//! Base `AraSerialize` / `AraDeserialize` impls for primitive types (u8, u16,
//! u32, u64, i8, i16, i32, i64, f32, f64, bool) live in `ara_com::serialization`
//! where the traits are defined, satisfying Rust's orphan rule.
//!
//! This module is reserved for **SOME/IP-specific** fixed-length helpers that
//! wrap or extend those base impls (e.g. little-endian overrides, alignment
//! padding, or SOME/IP TLV encodings).  Add them here as needed.
