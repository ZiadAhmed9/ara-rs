//! SOME/IP 16-byte header encoding and decoding.
//!
//! Wire layout (all fields big-endian):
//!
//! ```text
//! Bytes  0- 1   Service ID        (u16)
//! Bytes  2- 3   Method ID         (u16)
//! Bytes  4- 7   Length            (u32)  = payload_len + 8
//! Bytes  8- 9   Client ID         (u16)
//! Bytes 10-11   Session ID        (u16)
//! Byte  12      Protocol Version  (u8)   always 0x01
//! Byte  13      Interface Version (u8)
//! Byte  14      Message Type      (u8)
//! Byte  15      Return Code       (u8)
//! ```

use ara_com::transport::{MessageHeader, MessageType, ReturnCode};
use ara_com::types::{InstanceId, MethodId, ServiceId};

use crate::error::SomeIpError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// SOME/IP protocol version — always 0x01 per the spec.
pub const PROTOCOL_VERSION: u8 = 0x01;

/// Default interface version used when no specific version is configured.
pub const DEFAULT_INTERFACE_VERSION: u8 = 0x01;

/// Wire byte for `MessageType::Request`.
pub const MSG_TYPE_REQUEST: u8 = 0x00;
/// Wire byte for `MessageType::RequestNoReturn`.
pub const MSG_TYPE_REQUEST_NO_RETURN: u8 = 0x01;
/// Wire byte for `MessageType::Notification`.
pub const MSG_TYPE_NOTIFICATION: u8 = 0x02;
/// Wire byte for `MessageType::Response`.
pub const MSG_TYPE_RESPONSE: u8 = 0x80;
/// Wire byte for `MessageType::Error`.
pub const MSG_TYPE_ERROR: u8 = 0x81;

/// Fixed size of a SOME/IP base header in bytes.
pub const HEADER_LEN: usize = 16;

/// The eight bytes covered by the Length field itself (bytes 8-15).
const LENGTH_FIELD_OVERHEAD: u32 = 8;

// ---------------------------------------------------------------------------
// MessageType helpers
// ---------------------------------------------------------------------------

/// Convert our `MessageType` to its SOME/IP wire byte.
pub fn message_type_to_u8(msg_type: MessageType) -> u8 {
    match msg_type {
        MessageType::Request => MSG_TYPE_REQUEST,
        MessageType::RequestNoReturn => MSG_TYPE_REQUEST_NO_RETURN,
        MessageType::Notification => MSG_TYPE_NOTIFICATION,
        MessageType::Response => MSG_TYPE_RESPONSE,
        MessageType::Error => MSG_TYPE_ERROR,
    }
}

/// Convert a SOME/IP wire byte to our `MessageType`.
pub fn u8_to_message_type(byte: u8) -> Result<MessageType, SomeIpError> {
    match byte {
        MSG_TYPE_REQUEST => Ok(MessageType::Request),
        MSG_TYPE_REQUEST_NO_RETURN => Ok(MessageType::RequestNoReturn),
        MSG_TYPE_NOTIFICATION => Ok(MessageType::Notification),
        MSG_TYPE_RESPONSE => Ok(MessageType::Response),
        MSG_TYPE_ERROR => Ok(MessageType::Error),
        other => Err(SomeIpError::Header(format!(
            "unknown message type byte: 0x{other:02X}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// ReturnCode helpers
// ---------------------------------------------------------------------------

/// Convert our `ReturnCode` to its SOME/IP wire byte.
pub fn return_code_to_u8(rc: ReturnCode) -> u8 {
    rc as u8
}

/// Convert a SOME/IP wire byte to our `ReturnCode`.
pub fn u8_to_return_code(byte: u8) -> Result<ReturnCode, SomeIpError> {
    match byte {
        0x00 => Ok(ReturnCode::Ok),
        0x01 => Ok(ReturnCode::NotOk),
        0x02 => Ok(ReturnCode::UnknownService),
        0x03 => Ok(ReturnCode::UnknownMethod),
        0x04 => Ok(ReturnCode::NotReady),
        0x05 => Ok(ReturnCode::NotReachable),
        0x06 => Ok(ReturnCode::Timeout),
        0x07 => Ok(ReturnCode::WrongProtocolVersion),
        0x08 => Ok(ReturnCode::WrongInterfaceVersion),
        0x09 => Ok(ReturnCode::MalformedMessage),
        0x0A => Ok(ReturnCode::WrongMessageType),
        other => Err(SomeIpError::Header(format!(
            "unknown return code byte: 0x{other:02X}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// encode_header
// ---------------------------------------------------------------------------

/// Encode a SOME/IP 16-byte header.
///
/// # Arguments
///
/// * `header` – message metadata (service_id, method_id, session_id,
///   message_type, return_code; instance_id is not encoded in the base header)
/// * `client_id`         – SOME/IP client identifier (bytes 8-9)
/// * `payload_length`    – number of payload bytes that follow the header
/// * `interface_version` – interface version placed in byte 13
///
/// # Returns
///
/// A 16-byte array ready to prepend to the payload on the wire.
pub fn encode_header(
    header: &MessageHeader,
    client_id: u16,
    payload_length: u32,
    interface_version: u8,
) -> [u8; HEADER_LEN] {
    let mut buf = [0u8; HEADER_LEN];

    buf[0..2].copy_from_slice(&header.service_id.0.to_be_bytes());
    buf[2..4].copy_from_slice(&header.method_id.0.to_be_bytes());
    buf[4..8].copy_from_slice(&(payload_length + LENGTH_FIELD_OVERHEAD).to_be_bytes());
    buf[8..10].copy_from_slice(&client_id.to_be_bytes());
    buf[10..12].copy_from_slice(&header.session_id.to_be_bytes());
    buf[12] = PROTOCOL_VERSION;
    buf[13] = interface_version;
    buf[14] = message_type_to_u8(header.message_type);
    buf[15] = return_code_to_u8(header.return_code);

    buf
}

// ---------------------------------------------------------------------------
// decode_header
// ---------------------------------------------------------------------------

/// Decode a SOME/IP 16-byte header.
///
/// # Arguments
///
/// * `buf` – at least 16 bytes of raw data starting at the first byte of the
///   SOME/IP header
///
/// # Returns
///
/// `(MessageHeader, client_id, payload_length, interface_version)` where
/// `payload_length` is the actual number of payload bytes (i.e. `Length − 8`).
///
/// # Errors
///
/// Returns [`SomeIpError::Header`] if the buffer is shorter than 16 bytes, the
/// message type byte is unrecognised, or the return code byte is unrecognised.
pub fn decode_header(buf: &[u8]) -> Result<(MessageHeader, u16, u32, u8), SomeIpError> {
    if buf.len() < HEADER_LEN {
        return Err(SomeIpError::Header(format!(
            "buffer too short: need {HEADER_LEN} bytes, got {}",
            buf.len()
        )));
    }

    let service_id = ServiceId(u16::from_be_bytes([buf[0], buf[1]]));
    let method_id = MethodId(u16::from_be_bytes([buf[2], buf[3]]));
    let length = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
    let client_id = u16::from_be_bytes([buf[8], buf[9]]);
    let session_id = u16::from_be_bytes([buf[10], buf[11]]);
    // buf[12] is protocol version — we read but do not validate here; callers
    // that care about version checks can inspect the raw byte directly.
    let interface_version = buf[13];
    let message_type = u8_to_message_type(buf[14])?;
    let return_code = u8_to_return_code(buf[15])?;

    // Length field covers bytes 8-15 plus the payload; subtract the overhead.
    let payload_length = length.saturating_sub(LENGTH_FIELD_OVERHEAD);

    let header = MessageHeader {
        service_id,
        method_id,
        // instance_id is not carried in the base header; use a sentinel zero.
        instance_id: InstanceId(0),
        session_id,
        message_type,
        return_code,
    };

    Ok((header, client_id, payload_length, interface_version))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_header(
        service_id: u16,
        method_id: u16,
        session_id: u16,
        msg_type: MessageType,
        return_code: ReturnCode,
    ) -> MessageHeader {
        MessageHeader {
            service_id: ServiceId(service_id),
            method_id: MethodId(method_id),
            instance_id: InstanceId(0),
            session_id,
            message_type: msg_type,
            return_code,
        }
    }

    // -----------------------------------------------------------------------
    // Round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn round_trip_basic() {
        let hdr = make_header(0x1234, 0x0001, 0x0042, MessageType::Request, ReturnCode::Ok);
        let encoded = encode_header(&hdr, 0xABCD, 64, 1);
        let (decoded_hdr, client_id, payload_len, iface_ver) =
            decode_header(&encoded).expect("decode must succeed");

        assert_eq!(decoded_hdr.service_id, hdr.service_id);
        assert_eq!(decoded_hdr.method_id, hdr.method_id);
        assert_eq!(decoded_hdr.session_id, hdr.session_id);
        assert_eq!(decoded_hdr.message_type, hdr.message_type);
        assert_eq!(decoded_hdr.return_code, hdr.return_code);
        assert_eq!(client_id, 0xABCD);
        assert_eq!(payload_len, 64);
        assert_eq!(iface_ver, 1);
    }

    #[test]
    fn round_trip_zero_payload() {
        let hdr = make_header(
            0x0001,
            0x8001,
            0x0001,
            MessageType::Notification,
            ReturnCode::Ok,
        );
        let encoded = encode_header(&hdr, 0x0000, 0, DEFAULT_INTERFACE_VERSION);
        let (decoded_hdr, _, payload_len, _) =
            decode_header(&encoded).expect("decode must succeed");

        assert_eq!(payload_len, 0);
        assert_eq!(decoded_hdr.message_type, MessageType::Notification);
    }

    // -----------------------------------------------------------------------
    // Cross-validate with someip_parse
    // -----------------------------------------------------------------------

    #[test]
    fn cross_validate_with_someip_parse() {
        use someip_parse::SomeipHeader;
        use std::io::Cursor;

        let hdr = make_header(
            0x1234,
            0x0005,
            0x000A,
            MessageType::Response,
            ReturnCode::Ok,
        );
        let encoded = encode_header(&hdr, 0x0001, 32, 2);

        // someip_parse::SomeipHeader::read() consumes from a Read impl.
        let mut cursor = Cursor::new(&encoded[..]);
        let parsed = SomeipHeader::read(&mut cursor).expect("someip_parse must parse our header");

        assert_eq!(parsed.service_id(), 0x1234);
        assert_eq!(parsed.event_or_method_id(), 0x0005);
        // Length field = payload_len + 8 = 40
        assert_eq!(parsed.length, 40);
        assert_eq!(parsed.interface_version, 2);
        // return_code is stored as u8 in someip_parse
        assert_eq!(parsed.return_code, 0x00);

        // message_type comparison via discriminant
        assert_eq!(parsed.message_type, someip_parse::MessageType::Response);

        // request_id encodes client_id (high 16 bits) and session_id (low 16 bits)
        let client_from_rid = (parsed.request_id >> 16) as u16;
        let session_from_rid = (parsed.request_id & 0xFFFF) as u16;
        assert_eq!(client_from_rid, 0x0001);
        assert_eq!(session_from_rid, 0x000A);
    }

    // -----------------------------------------------------------------------
    // All MessageType variants
    // -----------------------------------------------------------------------

    #[test]
    fn all_message_types_round_trip() {
        let variants = [
            MessageType::Request,
            MessageType::RequestNoReturn,
            MessageType::Notification,
            MessageType::Response,
            MessageType::Error,
        ];

        for &mt in &variants {
            let hdr = make_header(0x0001, 0x0001, 0x0001, mt, ReturnCode::Ok);
            let encoded = encode_header(&hdr, 0, 0, DEFAULT_INTERFACE_VERSION);
            let (decoded, _, _, _) = decode_header(&encoded).unwrap_or_else(|e| {
                panic!("decode failed for {mt:?}: {e}");
            });
            assert_eq!(decoded.message_type, mt, "MessageType round-trip failed for {mt:?}");
        }
    }

    #[test]
    fn message_type_wire_values() {
        assert_eq!(message_type_to_u8(MessageType::Request), 0x00);
        assert_eq!(message_type_to_u8(MessageType::RequestNoReturn), 0x01);
        assert_eq!(message_type_to_u8(MessageType::Notification), 0x02);
        assert_eq!(message_type_to_u8(MessageType::Response), 0x80);
        assert_eq!(message_type_to_u8(MessageType::Error), 0x81);
    }

    // -----------------------------------------------------------------------
    // All ReturnCode variants
    // -----------------------------------------------------------------------

    #[test]
    fn all_return_codes_round_trip() {
        let variants = [
            ReturnCode::Ok,
            ReturnCode::NotOk,
            ReturnCode::UnknownService,
            ReturnCode::UnknownMethod,
            ReturnCode::NotReady,
            ReturnCode::NotReachable,
            ReturnCode::Timeout,
            ReturnCode::WrongProtocolVersion,
            ReturnCode::WrongInterfaceVersion,
            ReturnCode::MalformedMessage,
            ReturnCode::WrongMessageType,
        ];

        for &rc in &variants {
            let hdr = make_header(0x0001, 0x0001, 0x0001, MessageType::Response, rc);
            let encoded = encode_header(&hdr, 0, 0, DEFAULT_INTERFACE_VERSION);
            let (decoded, _, _, _) = decode_header(&encoded).unwrap_or_else(|e| {
                panic!("decode failed for {rc:?}: {e}");
            });
            assert_eq!(decoded.return_code, rc, "ReturnCode round-trip failed for {rc:?}");
        }
    }

    #[test]
    fn return_code_wire_values() {
        assert_eq!(return_code_to_u8(ReturnCode::Ok), 0x00);
        assert_eq!(return_code_to_u8(ReturnCode::NotOk), 0x01);
        assert_eq!(return_code_to_u8(ReturnCode::UnknownService), 0x02);
        assert_eq!(return_code_to_u8(ReturnCode::UnknownMethod), 0x03);
        assert_eq!(return_code_to_u8(ReturnCode::NotReady), 0x04);
        assert_eq!(return_code_to_u8(ReturnCode::NotReachable), 0x05);
        assert_eq!(return_code_to_u8(ReturnCode::Timeout), 0x06);
        assert_eq!(return_code_to_u8(ReturnCode::WrongProtocolVersion), 0x07);
        assert_eq!(return_code_to_u8(ReturnCode::WrongInterfaceVersion), 0x08);
        assert_eq!(return_code_to_u8(ReturnCode::MalformedMessage), 0x09);
        assert_eq!(return_code_to_u8(ReturnCode::WrongMessageType), 0x0A);
    }

    // -----------------------------------------------------------------------
    // Error cases
    // -----------------------------------------------------------------------

    #[test]
    fn decode_buffer_too_short_errors() {
        for len in 0..HEADER_LEN {
            let buf = vec![0u8; len];
            let err = decode_header(&buf).expect_err("must fail for short buffer");
            assert!(
                matches!(err, SomeIpError::Header(_)),
                "expected Header error for len={len}, got {err:?}"
            );
        }
    }

    #[test]
    fn decode_unknown_message_type_errors() {
        let hdr = make_header(0x0001, 0x0001, 0x0001, MessageType::Request, ReturnCode::Ok);
        let mut encoded = encode_header(&hdr, 0, 0, DEFAULT_INTERFACE_VERSION);
        // Overwrite byte 14 with an unknown message type.
        encoded[14] = 0x42;
        let err = decode_header(&encoded).expect_err("must fail for unknown message type");
        assert!(matches!(err, SomeIpError::Header(_)));
    }

    #[test]
    fn decode_unknown_return_code_errors() {
        let hdr = make_header(0x0001, 0x0001, 0x0001, MessageType::Request, ReturnCode::Ok);
        let mut encoded = encode_header(&hdr, 0, 0, DEFAULT_INTERFACE_VERSION);
        // Overwrite byte 15 with an unknown return code.
        encoded[15] = 0xFF;
        let err = decode_header(&encoded).expect_err("must fail for unknown return code");
        assert!(matches!(err, SomeIpError::Header(_)));
    }

    #[test]
    fn protocol_version_byte_is_always_one() {
        let hdr = make_header(0x0001, 0x0001, 0x0001, MessageType::Request, ReturnCode::Ok);
        let encoded = encode_header(&hdr, 0, 0, DEFAULT_INTERFACE_VERSION);
        assert_eq!(encoded[12], PROTOCOL_VERSION);
    }

    #[test]
    fn length_field_encodes_correctly() {
        let hdr = make_header(0x0001, 0x0001, 0x0001, MessageType::Request, ReturnCode::Ok);
        let encoded = encode_header(&hdr, 0, 100, DEFAULT_INTERFACE_VERSION);
        let length = u32::from_be_bytes([encoded[4], encoded[5], encoded[6], encoded[7]]);
        // Length = payload_length + 8
        assert_eq!(length, 108);
    }
}
