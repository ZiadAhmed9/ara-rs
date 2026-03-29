pub mod dynamic;
pub mod fixed;

/// SOME/IP byte order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

/// Serialization context carrying wire format configuration
#[derive(Debug, Clone)]
pub struct SerializationContext {
    pub byte_order: ByteOrder,
    /// String encoding: utf8 or utf16
    pub string_encoding: StringEncoding,
    /// Length field size for dynamic-length types (1, 2, or 4 bytes)
    pub length_field_size: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum StringEncoding {
    Utf8,
    Utf16Le,
    Utf16Be,
}

impl Default for SerializationContext {
    fn default() -> Self {
        Self {
            byte_order: ByteOrder::BigEndian,
            string_encoding: StringEncoding::Utf8,
            length_field_size: 4,
        }
    }
}

// Re-export the wire-format traits so callers have a single import point.
pub use ara_com::transport::{AraDeserialize, AraSerialize};
