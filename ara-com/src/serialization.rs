//! Standard-type impls for `AraSerialize` / `AraDeserialize`.
//!
//! These impls live in `ara-com` (where the traits are defined) so that the
//! orphan rule is satisfied.  Transport backends such as `ara-com-someip`
//! re-export these impls and add **only** backend-specific helpers on top.

use crate::error::AraComError;
use crate::transport::{AraDeserialize, AraSerialize};

// ---------------------------------------------------------------------------
// Helper macro — big-endian integer types
// ---------------------------------------------------------------------------

macro_rules! impl_int {
    ($t:ty) => {
        impl AraSerialize for $t {
            fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
                buf.extend_from_slice(&self.to_be_bytes());
                Ok(())
            }

            fn serialized_size(&self) -> usize {
                std::mem::size_of::<$t>()
            }
        }

        impl AraDeserialize for $t {
            fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
                const N: usize = std::mem::size_of::<$t>();
                if buf.len() < N {
                    return Err(AraComError::Deserialization {
                        message: format!(
                            "need {} bytes for {}, got {}",
                            N,
                            stringify!($t),
                            buf.len()
                        ),
                    });
                }
                let arr: [u8; N] = buf[..N].try_into().unwrap();
                Ok(<$t>::from_be_bytes(arr))
            }
        }
    };
}

impl_int!(u8);
impl_int!(u16);
impl_int!(u32);
impl_int!(u64);
impl_int!(i8);
impl_int!(i16);
impl_int!(i32);
impl_int!(i64);

// ---------------------------------------------------------------------------
// f32
// ---------------------------------------------------------------------------

impl AraSerialize for f32 {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        buf.extend_from_slice(&self.to_bits().to_be_bytes());
        Ok(())
    }

    fn serialized_size(&self) -> usize {
        4
    }
}

impl AraDeserialize for f32 {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        if buf.len() < 4 {
            return Err(AraComError::Deserialization {
                message: format!("need 4 bytes for f32, got {}", buf.len()),
            });
        }
        let arr: [u8; 4] = buf[..4].try_into().unwrap();
        Ok(f32::from_bits(u32::from_be_bytes(arr)))
    }
}

// ---------------------------------------------------------------------------
// f64
// ---------------------------------------------------------------------------

impl AraSerialize for f64 {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        buf.extend_from_slice(&self.to_bits().to_be_bytes());
        Ok(())
    }

    fn serialized_size(&self) -> usize {
        8
    }
}

impl AraDeserialize for f64 {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        if buf.len() < 8 {
            return Err(AraComError::Deserialization {
                message: format!("need 8 bytes for f64, got {}", buf.len()),
            });
        }
        let arr: [u8; 8] = buf[..8].try_into().unwrap();
        Ok(f64::from_bits(u64::from_be_bytes(arr)))
    }
}

// ---------------------------------------------------------------------------
// bool — 0x00 = false, 0x01 = true
// ---------------------------------------------------------------------------

impl AraSerialize for bool {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        buf.push(if *self { 0x01 } else { 0x00 });
        Ok(())
    }

    fn serialized_size(&self) -> usize {
        1
    }
}

impl AraDeserialize for bool {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        if buf.is_empty() {
            return Err(AraComError::Deserialization {
                message: "need 1 byte for bool, got 0".to_string(),
            });
        }
        match buf[0] {
            0x00 => Ok(false),
            0x01 => Ok(true),
            v => Err(AraComError::Deserialization {
                message: format!("invalid bool byte: 0x{v:02X}"),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// String — SOME/IP wire format:
//   4-byte big-endian total byte length (including BOM + NUL) +
//   UTF-8 BOM (0xEF 0xBB 0xBF) + UTF-8 bytes + NUL (0x00)
//
// Special case: empty string serializes as 4 zero bytes only (vsomeip compat).
// ---------------------------------------------------------------------------

const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

impl AraSerialize for String {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        if self.is_empty() {
            buf.extend_from_slice(&0u32.to_be_bytes());
            return Ok(());
        }
        let bytes = self.as_bytes();
        // total length = 3 (BOM) + string bytes + 1 (NUL)
        let total_len = (3 + bytes.len() + 1) as u32;
        buf.extend_from_slice(&total_len.to_be_bytes());
        buf.extend_from_slice(&UTF8_BOM);
        buf.extend_from_slice(bytes);
        buf.push(0x00);
        Ok(())
    }

    fn serialized_size(&self) -> usize {
        if self.is_empty() {
            4
        } else {
            4 + 3 + self.len() + 1
        }
    }
}

impl AraDeserialize for String {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        if buf.len() < 4 {
            return Err(AraComError::Deserialization {
                message: format!("need 4-byte length prefix for String, got {}", buf.len()),
            });
        }
        let len = u32::from_be_bytes(buf[..4].try_into().unwrap()) as usize;
        if len == 0 {
            return Ok(String::new());
        }
        if buf.len() < 4 + len {
            return Err(AraComError::Deserialization {
                message: format!(
                    "String payload truncated: need {} bytes, got {}",
                    len,
                    buf.len() - 4
                ),
            });
        }
        let payload = &buf[4..4 + len];
        // Skip UTF-8 BOM if present
        let content = if payload.starts_with(&UTF8_BOM) {
            &payload[3..]
        } else {
            payload
        };
        // Strip trailing NUL if present
        let content = if content.last() == Some(&0x00) {
            &content[..content.len() - 1]
        } else {
            content
        };
        let s = std::str::from_utf8(content).map_err(|e| AraComError::Deserialization {
            message: format!("String is not valid UTF-8: {e}"),
        })?;
        Ok(s.to_owned())
    }
}

// ---------------------------------------------------------------------------
// Vec<T> — SOME/IP wire format:
//   4-byte big-endian **byte length** of all serialized elements, then elements.
//   (NOT element count — SOME/IP uses byte length prefix per PRS_SOMEIP_00462)
// ---------------------------------------------------------------------------

impl<T: AraSerialize> AraSerialize for Vec<T> {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        let capacity: usize = self.iter().map(|item| item.serialized_size()).sum();
        let mut elements_buf: Vec<u8> = Vec::with_capacity(capacity);
        for item in self {
            item.ara_serialize(&mut elements_buf)?;
        }
        let byte_len = elements_buf.len() as u32;
        buf.extend_from_slice(&byte_len.to_be_bytes());
        buf.extend_from_slice(&elements_buf);
        Ok(())
    }

    fn serialized_size(&self) -> usize {
        4 + self
            .iter()
            .map(|item| item.serialized_size())
            .sum::<usize>()
    }
}

impl<T: AraDeserialize + AraSerialize> AraDeserialize for Vec<T> {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        if buf.len() < 4 {
            return Err(AraComError::Deserialization {
                message: format!("need 4-byte byte-length prefix for Vec, got {}", buf.len()),
            });
        }
        let byte_len = u32::from_be_bytes(buf[..4].try_into().unwrap()) as usize;
        if buf.len() < 4 + byte_len {
            return Err(AraComError::Deserialization {
                message: format!(
                    "Vec payload truncated: need {} bytes, got {}",
                    byte_len,
                    buf.len() - 4
                ),
            });
        }
        let payload = &buf[4..4 + byte_len];
        let mut offset = 0;
        let mut result = Vec::new();
        while offset < payload.len() {
            let item = T::ara_deserialize(&payload[offset..])?;
            offset += item.serialized_size();
            if offset > payload.len() {
                return Err(AraComError::Deserialization {
                    message: format!(
                        "Vec element overran payload: offset {} exceeds {} bytes",
                        offset,
                        payload.len()
                    ),
                });
            }
            result.push(item);
        }
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<T>(value: T) -> T
    where
        T: AraSerialize + AraDeserialize + Copy + std::fmt::Debug + PartialEq,
    {
        let mut buf = Vec::new();
        value.ara_serialize(&mut buf).unwrap();
        assert_eq!(buf.len(), value.serialized_size());
        T::ara_deserialize(&buf).unwrap()
    }

    // --- integer types ---

    #[test]
    fn test_u8_round_trip() {
        assert_eq!(round_trip(0u8), 0);
        assert_eq!(round_trip(255u8), 255);
    }

    #[test]
    fn test_u16_big_endian() {
        let mut buf = Vec::new();
        0x0102u16.ara_serialize(&mut buf).unwrap();
        assert_eq!(buf, [0x01, 0x02]);
        assert_eq!(u16::ara_deserialize(&buf).unwrap(), 0x0102);
    }

    #[test]
    fn test_u32_round_trip() {
        assert_eq!(round_trip(0xDEAD_BEEFu32), 0xDEAD_BEEF);
    }

    #[test]
    fn test_u64_round_trip() {
        assert_eq!(round_trip(u64::MAX), u64::MAX);
    }

    #[test]
    fn test_i32_negative() {
        assert_eq!(round_trip(-1i32), -1);
        assert_eq!(round_trip(i32::MIN), i32::MIN);
    }

    #[test]
    fn test_f32_round_trip() {
        assert_eq!(round_trip(1.5f32), 1.5f32);
        assert!(round_trip(f32::NAN).is_nan());
    }

    #[test]
    fn test_f64_round_trip() {
        assert_eq!(round_trip(std::f64::consts::PI), std::f64::consts::PI);
    }

    #[test]
    fn test_bool_round_trip() {
        assert!(!round_trip(false));
        assert!(round_trip(true));
    }

    #[test]
    fn test_bool_encoding() {
        let mut buf = Vec::new();
        true.ara_serialize(&mut buf).unwrap();
        assert_eq!(buf, [0x01]);
        let mut buf = Vec::new();
        false.ara_serialize(&mut buf).unwrap();
        assert_eq!(buf, [0x00]);
    }

    #[test]
    fn test_deserialize_insufficient_bytes() {
        let result = u32::ara_deserialize(&[0x00, 0x01]);
        assert!(result.is_err());
    }

    #[test]
    fn test_bool_invalid_byte() {
        let result = bool::ara_deserialize(&[0x02]);
        assert!(result.is_err());
    }

    // --- String ---

    #[test]
    fn test_string_round_trip() {
        let original = "hello SOME/IP".to_string();
        let mut buf = Vec::new();
        original.ara_serialize(&mut buf).unwrap();
        assert_eq!(buf.len(), original.serialized_size());
        let decoded = String::ara_deserialize(&buf).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_empty_string() {
        let original = String::new();
        let mut buf = Vec::new();
        original.ara_serialize(&mut buf).unwrap();
        // Empty string: just 4 zero bytes (vsomeip compat — no BOM/NUL)
        assert_eq!(buf, [0x00, 0x00, 0x00, 0x00]);
        assert_eq!(String::ara_deserialize(&buf).unwrap(), "");
    }

    #[test]
    fn test_string_bom_nul() {
        // "AB" => length=6 (3 BOM + 2 chars + 1 NUL), then BOM, "A", "B", NUL
        let s = "AB".to_string();
        let mut buf = Vec::new();
        s.ara_serialize(&mut buf).unwrap();
        assert_eq!(
            buf,
            [0x00, 0x00, 0x00, 0x06, 0xEF, 0xBB, 0xBF, 0x41, 0x42, 0x00]
        );
    }

    // --- Vec<T> ---

    #[test]
    fn test_vec_u32_round_trip() {
        let original: Vec<u32> = vec![1, 2, 3, 0xDEAD_BEEF];
        let mut buf = Vec::new();
        original.ara_serialize(&mut buf).unwrap();
        assert_eq!(buf.len(), original.serialized_size());
        assert_eq!(&buf[..4], &[0x00, 0x00, 0x00, 0x10]);
        let decoded: Vec<u32> = Vec::ara_deserialize(&buf).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_empty_vec() {
        let original: Vec<u8> = vec![];
        let mut buf = Vec::new();
        original.ara_serialize(&mut buf).unwrap();
        assert_eq!(buf, [0x00, 0x00, 0x00, 0x00]);
        let decoded: Vec<u8> = Vec::ara_deserialize(&buf).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_vec_u16_byte_length() {
        // 2 x u16 = 4 bytes of payload, so prefix should be 0x00000004, NOT element count 0x00000002
        let v: Vec<u16> = vec![1u16, 2u16];
        let mut buf = Vec::new();
        v.ara_serialize(&mut buf).unwrap();
        assert_eq!(&buf[..4], &[0x00, 0x00, 0x00, 0x04]);
    }
}
