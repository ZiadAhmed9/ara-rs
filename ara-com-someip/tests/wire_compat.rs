//! Wire compatibility tests for SOME/IP.
//!
//! These tests decode hardcoded byte sequences that represent valid SOME/IP
//! frames as would be produced by vsomeip or other compliant implementations.
//! This ensures our header parsing, serialization, and framing are
//! wire-compatible.

use ara_com::transport::{AraDeserialize, AraSerialize, MessageHeader, MessageType, ReturnCode};
use ara_com::types::{MethodId, ServiceId};

use ara_com_someip::transport::header::{decode_header, encode_header, HEADER_LEN};

// ============================================================================
// Known-good SOME/IP header bytes (matches vsomeip wire format)
// ============================================================================

/// A SOME/IP Request frame for service 0x1234, method 0x0001,
/// client_id=0x0002, session_id=0x0001, interface_version=1,
/// with a 1-byte payload (0x42 = battery_id).
///
/// Wire layout:
/// ```text
/// 12 34       service_id
/// 00 01       method_id
/// 00 00 00 09 length = 8 (overhead) + 1 (payload)
/// 00 02       client_id
/// 00 01       session_id
/// 01          protocol_version
/// 01          interface_version
/// 00          message_type = Request
/// 00          return_code = Ok
/// 42          payload: battery_id = 0x42
/// ```
const VSOMEIP_REQUEST_FRAME: [u8; 17] = [
    0x12, 0x34, // service_id
    0x00, 0x01, // method_id
    0x00, 0x00, 0x00, 0x09, // length = 9
    0x00, 0x02, // client_id
    0x00, 0x01, // session_id
    0x01, // protocol_version
    0x01, // interface_version
    0x00, // message_type = Request
    0x00, // return_code = Ok
    0x42, // payload
];

/// A SOME/IP Response frame for the above request.
/// Returns an f64 voltage = 12.6 (IEEE 754 big-endian: 0x4029333333333333).
///
/// length = 8 + 8 (f64 payload) = 16
const VSOMEIP_RESPONSE_FRAME: [u8; 24] = [
    0x12, 0x34, // service_id
    0x00, 0x01, // method_id
    0x00, 0x00, 0x00, 0x10, // length = 16
    0x00, 0x02, // client_id
    0x00, 0x01, // session_id (matches request)
    0x01, // protocol_version
    0x01, // interface_version
    0x80, // message_type = Response
    0x00, // return_code = Ok
    // payload: f64 12.6 in big-endian IEEE 754
    0x40, 0x29, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33,
];

/// A SOME/IP RequestNoReturn (fire-and-forget) frame.
/// SetChargeLimit with f64 limit = 32.0 (0x4040000000000000).
const VSOMEIP_FIRE_AND_FORGET_FRAME: [u8; 24] = [
    0x12, 0x34, // service_id
    0x00, 0x02, // method_id (SetChargeLimit)
    0x00, 0x00, 0x00, 0x10, // length = 16
    0x00, 0x02, // client_id
    0x00, 0x03, // session_id
    0x01, // protocol_version
    0x01, // interface_version
    0x01, // message_type = RequestNoReturn
    0x00, // return_code = Ok
    // payload: f64 32.0 in big-endian IEEE 754
    0x40, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// A SOME/IP Notification (event) frame.
/// VoltageChanged event (method_id 0x8001) with f64 = 13.1.
const VSOMEIP_NOTIFICATION_FRAME: [u8; 24] = [
    0x12, 0x34, // service_id
    0x80, 0x01, // method_id = 0x8001 (event)
    0x00, 0x00, 0x00, 0x10, // length = 16
    0x00, 0x00, // client_id = 0 (notifications)
    0x00, 0x00, // session_id = 0 (notifications)
    0x01, // protocol_version
    0x01, // interface_version
    0x02, // message_type = Notification
    0x00, // return_code = Ok
    // payload: f64 13.1 in big-endian IEEE 754
    0x40, 0x2A, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33,
];

/// A SOME/IP Error response frame.
/// service_id=0x1234, method_id=0x0001, return_code=UnknownMethod (0x03).
const VSOMEIP_ERROR_FRAME: [u8; 16] = [
    0x12, 0x34, // service_id
    0x00, 0x01, // method_id
    0x00, 0x00, 0x00, 0x08, // length = 8 (no payload)
    0x00, 0x02, // client_id
    0x00, 0x05, // session_id
    0x01, // protocol_version
    0x01, // interface_version
    0x81, // message_type = Error
    0x03, // return_code = UnknownMethod
];

// ============================================================================
// Header decode tests
// ============================================================================

#[test]
fn decode_vsomeip_request_header() {
    let (hdr, client_id, payload_len, iface_ver) =
        decode_header(&VSOMEIP_REQUEST_FRAME).expect("decode failed");

    assert_eq!(hdr.service_id, ServiceId(0x1234));
    assert_eq!(hdr.method_id, MethodId(0x0001));
    assert_eq!(hdr.session_id, 0x0001);
    assert_eq!(hdr.message_type, MessageType::Request);
    assert_eq!(hdr.return_code, ReturnCode::Ok);
    assert_eq!(client_id, 0x0002);
    assert_eq!(payload_len, 1); // 9 - 8 = 1
    assert_eq!(iface_ver, 1);

    // Verify payload byte
    assert_eq!(VSOMEIP_REQUEST_FRAME[HEADER_LEN], 0x42);
}

#[test]
fn decode_vsomeip_response_header_and_payload() {
    let (hdr, client_id, payload_len, _) =
        decode_header(&VSOMEIP_RESPONSE_FRAME).expect("decode failed");

    assert_eq!(hdr.service_id, ServiceId(0x1234));
    assert_eq!(hdr.method_id, MethodId(0x0001));
    assert_eq!(hdr.session_id, 0x0001);
    assert_eq!(hdr.message_type, MessageType::Response);
    assert_eq!(hdr.return_code, ReturnCode::Ok);
    assert_eq!(client_id, 0x0002);
    assert_eq!(payload_len, 8);

    // Decode the f64 payload
    let voltage = f64::ara_deserialize(&VSOMEIP_RESPONSE_FRAME[HEADER_LEN..]).unwrap();
    assert!(
        (voltage - 12.6).abs() < 1e-10,
        "expected 12.6, got {voltage}"
    );
}

#[test]
fn decode_vsomeip_fire_and_forget() {
    let (hdr, _, payload_len, _) =
        decode_header(&VSOMEIP_FIRE_AND_FORGET_FRAME).expect("decode failed");

    assert_eq!(hdr.service_id, ServiceId(0x1234));
    assert_eq!(hdr.method_id, MethodId(0x0002));
    assert_eq!(hdr.message_type, MessageType::RequestNoReturn);
    assert_eq!(payload_len, 8);

    let limit = f64::ara_deserialize(&VSOMEIP_FIRE_AND_FORGET_FRAME[HEADER_LEN..]).unwrap();
    assert!((limit - 32.0).abs() < 1e-10, "expected 32.0, got {limit}");
}

#[test]
fn decode_vsomeip_notification() {
    let (hdr, client_id, payload_len, _) =
        decode_header(&VSOMEIP_NOTIFICATION_FRAME).expect("decode failed");

    assert_eq!(hdr.service_id, ServiceId(0x1234));
    assert_eq!(hdr.method_id, MethodId(0x8001));
    assert_eq!(hdr.session_id, 0x0000);
    assert_eq!(hdr.message_type, MessageType::Notification);
    assert_eq!(client_id, 0x0000);
    assert_eq!(payload_len, 8);

    let voltage = f64::ara_deserialize(&VSOMEIP_NOTIFICATION_FRAME[HEADER_LEN..]).unwrap();
    assert!(
        (voltage - 13.1).abs() < 1e-10,
        "expected 13.1, got {voltage}"
    );
}

#[test]
fn decode_vsomeip_error_frame() {
    let (hdr, _, payload_len, _) = decode_header(&VSOMEIP_ERROR_FRAME).expect("decode failed");

    assert_eq!(hdr.service_id, ServiceId(0x1234));
    assert_eq!(hdr.method_id, MethodId(0x0001));
    assert_eq!(hdr.session_id, 0x0005);
    assert_eq!(hdr.message_type, MessageType::Error);
    assert_eq!(hdr.return_code, ReturnCode::UnknownMethod);
    assert_eq!(payload_len, 0);
}

// ============================================================================
// Encode-then-verify: our encoder produces the exact same bytes
// ============================================================================

#[test]
fn encode_matches_vsomeip_request() {
    let hdr = MessageHeader {
        service_id: ServiceId(0x1234),
        method_id: MethodId(0x0001),
        instance_id: ara_com::types::InstanceId(0),
        session_id: 0x0001,
        message_type: MessageType::Request,
        return_code: ReturnCode::Ok,
    };
    let encoded = encode_header(&hdr, 0x0002, 1, 1);
    assert_eq!(
        encoded,
        VSOMEIP_REQUEST_FRAME[..HEADER_LEN],
        "our encoder doesn't match vsomeip wire format for Request"
    );
}

#[test]
fn encode_matches_vsomeip_response() {
    let hdr = MessageHeader {
        service_id: ServiceId(0x1234),
        method_id: MethodId(0x0001),
        instance_id: ara_com::types::InstanceId(0),
        session_id: 0x0001,
        message_type: MessageType::Response,
        return_code: ReturnCode::Ok,
    };
    let encoded = encode_header(&hdr, 0x0002, 8, 1);
    assert_eq!(
        encoded,
        VSOMEIP_RESPONSE_FRAME[..HEADER_LEN],
        "our encoder doesn't match vsomeip wire format for Response"
    );
}

#[test]
fn encode_matches_vsomeip_notification() {
    let hdr = MessageHeader {
        service_id: ServiceId(0x1234),
        method_id: MethodId(0x8001),
        instance_id: ara_com::types::InstanceId(0),
        session_id: 0x0000,
        message_type: MessageType::Notification,
        return_code: ReturnCode::Ok,
    };
    let encoded = encode_header(&hdr, 0x0000, 8, 1);
    assert_eq!(
        encoded,
        VSOMEIP_NOTIFICATION_FRAME[..HEADER_LEN],
        "our encoder doesn't match vsomeip wire format for Notification"
    );
}

#[test]
fn encode_matches_vsomeip_error() {
    let hdr = MessageHeader {
        service_id: ServiceId(0x1234),
        method_id: MethodId(0x0001),
        instance_id: ara_com::types::InstanceId(0),
        session_id: 0x0005,
        message_type: MessageType::Error,
        return_code: ReturnCode::UnknownMethod,
    };
    let encoded = encode_header(&hdr, 0x0002, 0, 1);
    assert_eq!(
        encoded, VSOMEIP_ERROR_FRAME,
        "our encoder doesn't match vsomeip wire format for Error"
    );
}

// ============================================================================
// Serialization wire format tests
// ============================================================================

#[test]
fn u32_big_endian_wire_format() {
    let mut buf = Vec::new();
    0xDEADBEEFu32.ara_serialize(&mut buf).unwrap();
    assert_eq!(buf, [0xDE, 0xAD, 0xBE, 0xEF]);
}

#[test]
fn f64_big_endian_wire_format() {
    // 12.6 in IEEE 754 double-precision big-endian
    let mut buf = Vec::new();
    12.6f64.ara_serialize(&mut buf).unwrap();
    assert_eq!(buf, [0x40, 0x29, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33]);
}

#[test]
fn bool_wire_format() {
    let mut buf = Vec::new();
    true.ara_serialize(&mut buf).unwrap();
    assert_eq!(buf, [0x01]);

    buf.clear();
    false.ara_serialize(&mut buf).unwrap();
    assert_eq!(buf, [0x00]);
}

#[test]
fn string_someip_wire_format() {
    // SOME/IP string: length(4) + BOM(3) + data + NUL(1)
    let mut buf = Vec::new();
    "Hi".to_string().ara_serialize(&mut buf).unwrap();
    assert_eq!(
        buf,
        [
            0x00, 0x00, 0x00, 0x06, // length = 6 (3 BOM + 2 chars + 1 NUL)
            0xEF, 0xBB, 0xBF, // UTF-8 BOM
            0x48, 0x69, // "Hi"
            0x00, // NUL terminator
        ]
    );
}

#[test]
fn vec_u16_someip_wire_format() {
    // SOME/IP array: byte-length prefix (not element count)
    let mut buf = Vec::new();
    vec![0x0001u16, 0x0002u16].ara_serialize(&mut buf).unwrap();
    assert_eq!(
        buf,
        [
            0x00, 0x00, 0x00, 0x04, // byte length = 4 (2 elements × 2 bytes each)
            0x00, 0x01, // element 0
            0x00, 0x02, // element 1
        ]
    );
}

#[test]
fn struct_sequential_field_layout() {
    // BatteryStatus-like struct: f64 voltage + f64 current + bool charging
    // SOME/IP structs are serialized as sequential fields with no padding/alignment
    let mut buf = Vec::new();
    12.6f64.ara_serialize(&mut buf).unwrap(); // voltage
    3.2f64.ara_serialize(&mut buf).unwrap(); // current
    true.ara_serialize(&mut buf).unwrap(); // charging

    assert_eq!(buf.len(), 8 + 8 + 1); // 17 bytes total, no padding
    assert_eq!(buf[16], 0x01); // charging = true at offset 16
}
