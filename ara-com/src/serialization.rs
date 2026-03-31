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
// String — 4-byte big-endian length prefix followed by UTF-8 payload
// ---------------------------------------------------------------------------

impl AraSerialize for String {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        let bytes = self.as_bytes();
        let len = bytes.len() as u32;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(bytes);
        Ok(())
    }

    fn serialized_size(&self) -> usize {
        4 + self.len()
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
        if buf.len() < 4 + len {
            return Err(AraComError::Deserialization {
                message: format!(
                    "String payload truncated: need {} bytes, got {}",
                    len,
                    buf.len() - 4
                ),
            });
        }
        let s =
            std::str::from_utf8(&buf[4..4 + len]).map_err(|e| AraComError::Deserialization {
                message: format!("String is not valid UTF-8: {e}"),
            })?;
        Ok(s.to_owned())
    }
}

// ---------------------------------------------------------------------------
// Vec<T> — 4-byte big-endian element-count prefix followed by elements
// ---------------------------------------------------------------------------

impl<T: AraSerialize> AraSerialize for Vec<T> {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        let count = self.len() as u32;
        buf.extend_from_slice(&count.to_be_bytes());
        for item in self {
            item.ara_serialize(buf)?;
        }
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
                message: format!("need 4-byte count prefix for Vec, got {}", buf.len()),
            });
        }
        let count = u32::from_be_bytes(buf[..4].try_into().unwrap()) as usize;
        let mut offset = 4;
        let mut result = Vec::with_capacity(count);
        for _ in 0..count {
            let item = T::ara_deserialize(&buf[offset..])?;
            offset += item.serialized_size();
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
        assert_eq!(buf, [0x00, 0x00, 0x00, 0x00]);
        assert_eq!(String::ara_deserialize(&buf).unwrap(), "");
    }

    // --- Vec<T> ---

    #[test]
    fn test_vec_u32_round_trip() {
        let original: Vec<u32> = vec![1, 2, 3, 0xDEAD_BEEF];
        let mut buf = Vec::new();
        original.ara_serialize(&mut buf).unwrap();
        assert_eq!(buf.len(), original.serialized_size());
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
}
