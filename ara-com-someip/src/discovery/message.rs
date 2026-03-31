//! SOME/IP-SD message encoding and decoding.
//!
//! SD messages are carried as regular SOME/IP messages addressed to the
//! well-known SD service (0xFFFF / method 0x8100). This module handles the
//! payload layer only — the outer SOME/IP header is managed by the transport.

use std::net::Ipv4Addr;

use ara_com::types::{EventGroupId, InstanceId, MajorVersion, MinorVersion, ServiceId};

use crate::error::SomeIpError;

// ---------------------------------------------------------------------------
// Well-known SD constants
// ---------------------------------------------------------------------------

/// SOME/IP Service ID used for all SD messages.
pub const SD_SERVICE_ID: u16 = 0xFFFF;
/// SOME/IP Method ID used for all SD messages.
pub const SD_METHOD_ID: u16 = 0x8100;

/// SD entry type: Find Service.
pub const SD_ENTRY_FIND_SERVICE: u8 = 0x00;
/// SD entry type: Offer Service.
pub const SD_ENTRY_OFFER_SERVICE: u8 = 0x01;
/// SD entry type: Subscribe Eventgroup.
pub const SD_ENTRY_SUBSCRIBE_EVENTGROUP: u8 = 0x06;
/// SD entry type: Stop Subscribe Eventgroup.
pub const SD_ENTRY_STOP_SUBSCRIBE_EVENTGROUP: u8 = 0x07;

/// SD option type: IPv4 Endpoint.
pub const SD_OPTION_IPV4_ENDPOINT: u8 = 0x04;

/// Transport protocol indicator: UDP.
pub const SD_PROTOCOL_UDP: u8 = 0x11;
/// Transport protocol indicator: TCP.
pub const SD_PROTOCOL_TCP: u8 = 0x06;

// ---------------------------------------------------------------------------
// Entry sizes and field offsets
// ---------------------------------------------------------------------------

/// Every SD entry is exactly 16 bytes.
const ENTRY_SIZE: usize = 16;
/// IPv4 Endpoint option total wire size (2-byte length field + 10 bytes body).
const IPV4_OPTION_WIRE_SIZE: usize = 12;

// ---------------------------------------------------------------------------
// Transport protocol
// ---------------------------------------------------------------------------

/// Transport-layer protocol carried in an SD endpoint option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportProtocol {
    Udp,
    Tcp,
}

impl TransportProtocol {
    fn to_wire(self) -> u8 {
        match self {
            TransportProtocol::Udp => SD_PROTOCOL_UDP,
            TransportProtocol::Tcp => SD_PROTOCOL_TCP,
        }
    }

    fn from_wire(byte: u8) -> Result<Self, SomeIpError> {
        match byte {
            SD_PROTOCOL_UDP => Ok(TransportProtocol::Udp),
            SD_PROTOCOL_TCP => Ok(TransportProtocol::Tcp),
            other => Err(SomeIpError::Discovery(format!(
                "unknown SD transport protocol byte: 0x{other:02X}"
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// SD Option
// ---------------------------------------------------------------------------

/// An SD option attached to one or more entries.
#[derive(Debug, Clone)]
pub enum SdOption {
    /// IPv4 endpoint option (type 0x04).
    Ipv4Endpoint {
        addr: Ipv4Addr,
        port: u16,
        protocol: TransportProtocol,
    },
}

impl SdOption {
    /// Encode the option into bytes (length field + body).
    ///
    /// The wire format for IPv4 is:
    /// ```text
    /// [0-1]  Length = 0x000A  (u16 BE, counts bytes after this field: 10 body bytes)
    /// [2]    Type   = 0x04
    /// [3]    Reserved = 0x00
    /// [4-7]  IPv4 address (4 bytes, big-endian)
    /// [8]    Reserved = 0x00
    /// [9]    Protocol (0x06 = TCP, 0x11 = UDP)
    /// [10-11] Port (u16 BE)
    /// Total: 12 bytes
    /// ```
    fn encode(&self) -> Vec<u8> {
        match self {
            SdOption::Ipv4Endpoint {
                addr,
                port,
                protocol,
            } => {
                let mut buf = vec![0u8; IPV4_OPTION_WIRE_SIZE];
                // Length field: 10 bytes follow the length field (body = type + reserved + addr + reserved + proto + port)
                buf[0] = 0x00;
                buf[1] = 0x0A;
                buf[2] = SD_OPTION_IPV4_ENDPOINT;
                buf[3] = 0x00; // reserved
                buf[4..8].copy_from_slice(&addr.octets());
                buf[8] = 0x00; // reserved
                buf[9] = protocol.to_wire();
                buf[10] = (port >> 8) as u8;
                buf[11] = *port as u8;
                buf
            }
        }
    }

    /// Decode one option starting at `buf[offset]`.
    ///
    /// Returns `(option, bytes_consumed)`.
    fn decode_one(buf: &[u8], offset: usize) -> Result<(Self, usize), SomeIpError> {
        // Need at least 2 bytes for the length field
        if offset + 2 > buf.len() {
            return Err(SomeIpError::Deserialization {
                offset,
                message: "SD option truncated: not enough bytes for length field".into(),
            });
        }

        let length = u16::from_be_bytes([buf[offset], buf[offset + 1]]) as usize;
        // total bytes consumed by this option = 2-byte length field + body
        let total = 2 + length;

        if length < 1 {
            return Err(SomeIpError::Deserialization {
                offset,
                message: "SD option length is zero — cannot read type byte".into(),
            });
        }

        if offset + total > buf.len() {
            return Err(SomeIpError::Deserialization {
                offset,
                message: format!(
                    "SD option body truncated: need {total} bytes, have {}",
                    buf.len() - offset
                ),
            });
        }

        let option_type = buf[offset + 2];
        match option_type {
            SD_OPTION_IPV4_ENDPOINT => {
                // Body = type(1) + reserved(1) + addr(4) + reserved(1) + proto(1) + port(2) = 10 bytes
                if length != 10 {
                    return Err(SomeIpError::Deserialization {
                        offset,
                        message: format!("IPv4 endpoint option: expected length 10, got {length}"),
                    });
                }
                // offset+2 = type, offset+3 = reserved, offset+4..8 = addr (4 bytes),
                // offset+8 = reserved, offset+9 = proto, offset+10..12 = port (2 bytes)
                let addr = Ipv4Addr::new(
                    buf[offset + 4],
                    buf[offset + 5],
                    buf[offset + 6],
                    buf[offset + 7],
                );
                let protocol = TransportProtocol::from_wire(buf[offset + 9])?;
                let port = u16::from_be_bytes([buf[offset + 10], buf[offset + 11]]);
                Ok((
                    SdOption::Ipv4Endpoint {
                        addr,
                        port,
                        protocol,
                    },
                    total,
                ))
            }
            other => Err(SomeIpError::Deserialization {
                offset,
                message: format!("unknown SD option type: 0x{other:02X}"),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// SD Entry
// ---------------------------------------------------------------------------

/// One SD entry — either a service announcement or an eventgroup subscription.
#[derive(Debug, Clone)]
pub enum SdEntry {
    /// Client searching for a service instance.
    FindService {
        service_id: ServiceId,
        instance_id: InstanceId,
        major_version: MajorVersion,
        minor_version: MinorVersion,
        /// Time-to-live in seconds. 0 = stop finding.
        ttl: u32,
    },
    /// Server announcing it offers a service instance.
    OfferService {
        service_id: ServiceId,
        instance_id: InstanceId,
        major_version: MajorVersion,
        minor_version: MinorVersion,
        /// Time-to-live in seconds. 0 = stop offer.
        ttl: u32,
    },
    /// Client subscribing to an eventgroup.
    SubscribeEventgroup {
        service_id: ServiceId,
        instance_id: InstanceId,
        major_version: MajorVersion,
        /// Time-to-live in seconds.
        ttl: u32,
        event_group_id: EventGroupId,
        /// Counter used to distinguish simultaneous subscriptions.
        counter: u8,
    },
    /// Client cancelling an eventgroup subscription (TTL = 0).
    StopSubscribeEventgroup {
        service_id: ServiceId,
        instance_id: InstanceId,
        event_group_id: EventGroupId,
    },
}

impl SdEntry {
    /// Encode the entry into a 16-byte array and return any associated SD option.
    ///
    /// The option index fields in the entry are left at 0 — the caller
    /// (`SdMessage::encode`) is responsible for patching them once the
    /// option array offset is known.
    pub fn encode(&self) -> ([u8; ENTRY_SIZE], Option<SdOption>) {
        let mut buf = [0u8; ENTRY_SIZE];
        match self {
            SdEntry::FindService {
                service_id,
                instance_id,
                major_version,
                minor_version,
                ttl,
            } => {
                buf[0] = SD_ENTRY_FIND_SERVICE;
                // bytes 1-3: option indices / counts — left at 0 (no options for Find)
                let [sh, sl] = service_id.0.to_be_bytes();
                buf[4] = sh;
                buf[5] = sl;
                let [ih, il] = instance_id.0.to_be_bytes();
                buf[6] = ih;
                buf[7] = il;
                buf[8] = major_version.0;
                encode_ttl(&mut buf[9..12], *ttl);
                let mv_bytes = minor_version.0.to_be_bytes();
                buf[12..16].copy_from_slice(&mv_bytes);
                (buf, None)
            }
            SdEntry::OfferService {
                service_id,
                instance_id,
                major_version,
                minor_version,
                ttl,
            } => {
                buf[0] = SD_ENTRY_OFFER_SERVICE;
                let [sh, sl] = service_id.0.to_be_bytes();
                buf[4] = sh;
                buf[5] = sl;
                let [ih, il] = instance_id.0.to_be_bytes();
                buf[6] = ih;
                buf[7] = il;
                buf[8] = major_version.0;
                encode_ttl(&mut buf[9..12], *ttl);
                let mv_bytes = minor_version.0.to_be_bytes();
                buf[12..16].copy_from_slice(&mv_bytes);
                (buf, None)
            }
            SdEntry::SubscribeEventgroup {
                service_id,
                instance_id,
                major_version,
                ttl,
                event_group_id,
                counter,
            } => {
                buf[0] = SD_ENTRY_SUBSCRIBE_EVENTGROUP;
                // bytes 1-3: option index fields — caller patches index1 if needed
                let [sh, sl] = service_id.0.to_be_bytes();
                buf[4] = sh;
                buf[5] = sl;
                let [ih, il] = instance_id.0.to_be_bytes();
                buf[6] = ih;
                buf[7] = il;
                buf[8] = major_version.0;
                encode_ttl(&mut buf[9..12], *ttl);
                buf[12] = 0x00; // reserved
                buf[13] = *counter;
                let [eh, el] = event_group_id.0.to_be_bytes();
                buf[14] = eh;
                buf[15] = el;
                (buf, None)
            }
            SdEntry::StopSubscribeEventgroup {
                service_id,
                instance_id,
                event_group_id,
            } => {
                buf[0] = SD_ENTRY_STOP_SUBSCRIBE_EVENTGROUP;
                let [sh, sl] = service_id.0.to_be_bytes();
                buf[4] = sh;
                buf[5] = sl;
                let [ih, il] = instance_id.0.to_be_bytes();
                buf[6] = ih;
                buf[7] = il;
                // major version = 0xFF per AUTOSAR spec for stop entries
                buf[8] = 0xFF;
                // TTL = 0 signals stop
                encode_ttl(&mut buf[9..12], 0);
                buf[12] = 0x00; // reserved
                buf[13] = 0x00; // counter = 0
                let [eh, el] = event_group_id.0.to_be_bytes();
                buf[14] = eh;
                buf[15] = el;
                (buf, None)
            }
        }
    }

    /// Decode one entry from `buf[offset..offset+16]`.
    fn decode_one(buf: &[u8], offset: usize) -> Result<Self, SomeIpError> {
        if offset + ENTRY_SIZE > buf.len() {
            return Err(SomeIpError::Deserialization {
                offset,
                message: format!(
                    "SD entry truncated: need {ENTRY_SIZE} bytes, have {}",
                    buf.len().saturating_sub(offset)
                ),
            });
        }

        let entry_type = buf[offset];
        let service_id = ServiceId(u16::from_be_bytes([buf[offset + 4], buf[offset + 5]]));
        let instance_id = InstanceId(u16::from_be_bytes([buf[offset + 6], buf[offset + 7]]));
        let major_version = MajorVersion(buf[offset + 8]);
        let ttl = decode_ttl(&buf[offset + 9..offset + 12]);

        match entry_type {
            SD_ENTRY_FIND_SERVICE => {
                let minor_version = MinorVersion(u32::from_be_bytes([
                    buf[offset + 12],
                    buf[offset + 13],
                    buf[offset + 14],
                    buf[offset + 15],
                ]));
                Ok(SdEntry::FindService {
                    service_id,
                    instance_id,
                    major_version,
                    minor_version,
                    ttl,
                })
            }
            SD_ENTRY_OFFER_SERVICE => {
                let minor_version = MinorVersion(u32::from_be_bytes([
                    buf[offset + 12],
                    buf[offset + 13],
                    buf[offset + 14],
                    buf[offset + 15],
                ]));
                Ok(SdEntry::OfferService {
                    service_id,
                    instance_id,
                    major_version,
                    minor_version,
                    ttl,
                })
            }
            SD_ENTRY_SUBSCRIBE_EVENTGROUP => {
                let counter = buf[offset + 13];
                let event_group_id =
                    EventGroupId(u16::from_be_bytes([buf[offset + 14], buf[offset + 15]]));
                Ok(SdEntry::SubscribeEventgroup {
                    service_id,
                    instance_id,
                    major_version,
                    ttl,
                    event_group_id,
                    counter,
                })
            }
            SD_ENTRY_STOP_SUBSCRIBE_EVENTGROUP => {
                let event_group_id =
                    EventGroupId(u16::from_be_bytes([buf[offset + 14], buf[offset + 15]]));
                Ok(SdEntry::StopSubscribeEventgroup {
                    service_id,
                    instance_id,
                    event_group_id,
                })
            }
            other => Err(SomeIpError::Deserialization {
                offset,
                message: format!("unknown SD entry type: 0x{other:02X}"),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// SD Message
// ---------------------------------------------------------------------------

/// A complete SOME/IP-SD message payload (flags + entries + options).
///
/// This covers the bytes **after** the standard 16-byte SOME/IP header.
#[derive(Debug, Clone)]
pub struct SdMessage {
    /// Reboot flag (bit 7 of flags byte 0 per AUTOSAR spec, modelled here as bool).
    pub reboot_flag: bool,
    /// Unicast flag (bit 6 of flags byte 0).
    pub unicast_flag: bool,
    /// SD entries.
    pub entries: Vec<SdEntry>,
    /// SD options referenced by entries.
    pub options: Vec<SdOption>,
}

impl SdMessage {
    /// Encode the SD message into a byte vector.
    ///
    /// Layout:
    /// ```text
    /// [0]     Flags byte: bit7=reboot, bit6=unicast
    /// [1-3]   Reserved (0x00)
    /// [4-7]   Entries array length (u32 BE)
    /// [8..]   Entries (16 bytes each)
    /// After entries:
    ///   [n..n+3]   Options array length (u32 BE)
    ///   [n+4..]    Options
    /// ```
    pub fn encode(&self) -> Vec<u8> {
        // --- encode options first so we know their offsets ---
        let mut options_bytes: Vec<u8> = Vec::new();
        for opt in &self.options {
            options_bytes.extend_from_slice(&opt.encode());
        }

        // --- encode entries ---
        let entries_len = self.entries.len() * ENTRY_SIZE;
        let mut entries_bytes: Vec<u8> = Vec::with_capacity(entries_len);
        for entry in &self.entries {
            let (raw, _opt) = entry.encode();
            entries_bytes.extend_from_slice(&raw);
        }

        // --- assemble ---
        let mut buf = Vec::with_capacity(8 + entries_bytes.len() + 4 + options_bytes.len());

        // Flags (1 byte) + 3 reserved bytes
        let mut flags: u8 = 0;
        if self.reboot_flag {
            flags |= 0b1000_0000; // bit 7
        }
        if self.unicast_flag {
            flags |= 0b0100_0000; // bit 6
        }
        buf.push(flags);
        buf.extend_from_slice(&[0x00, 0x00, 0x00]); // reserved

        // Entries array length
        buf.extend_from_slice(&(entries_bytes.len() as u32).to_be_bytes());
        buf.extend_from_slice(&entries_bytes);

        // Options array length
        buf.extend_from_slice(&(options_bytes.len() as u32).to_be_bytes());
        buf.extend_from_slice(&options_bytes);

        buf
    }

    /// Decode an SD message payload.
    ///
    /// `buf` should be the bytes **after** the standard SOME/IP header.
    pub fn decode(buf: &[u8]) -> Result<Self, SomeIpError> {
        // Minimum: 4 bytes flags/reserved + 4 bytes entries-len + 4 bytes options-len
        if buf.len() < 12 {
            return Err(SomeIpError::Deserialization {
                offset: 0,
                message: format!(
                    "SD message too short: need at least 12 bytes, got {}",
                    buf.len()
                ),
            });
        }

        let flags = buf[0];
        let reboot_flag = (flags & 0b1000_0000) != 0;
        let unicast_flag = (flags & 0b0100_0000) != 0;
        // bytes 1-3 are reserved — ignore

        let entries_len = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;

        if !entries_len.is_multiple_of(ENTRY_SIZE) {
            return Err(SomeIpError::Deserialization {
                offset: 4,
                message: format!(
                    "SD entries array length {entries_len} is not a multiple of {ENTRY_SIZE}"
                ),
            });
        }

        let entries_start = 8;
        let entries_end = entries_start + entries_len;

        if entries_end > buf.len() {
            return Err(SomeIpError::Deserialization {
                offset: 4,
                message: format!(
                    "SD entries array extends past buffer: need {entries_end}, have {}",
                    buf.len()
                ),
            });
        }

        // Options length field starts immediately after entries
        if entries_end + 4 > buf.len() {
            return Err(SomeIpError::Deserialization {
                offset: entries_end,
                message: "SD options length field truncated".into(),
            });
        }

        let options_len = u32::from_be_bytes([
            buf[entries_end],
            buf[entries_end + 1],
            buf[entries_end + 2],
            buf[entries_end + 3],
        ]) as usize;

        let options_start = entries_end + 4;
        let options_end = options_start + options_len;

        if options_end > buf.len() {
            return Err(SomeIpError::Deserialization {
                offset: options_start,
                message: format!(
                    "SD options array extends past buffer: need {options_end}, have {}",
                    buf.len()
                ),
            });
        }

        // Decode entries
        let entry_count = entries_len / ENTRY_SIZE;
        let mut entries = Vec::with_capacity(entry_count);
        for i in 0..entry_count {
            let entry_offset = entries_start + i * ENTRY_SIZE;
            entries.push(SdEntry::decode_one(buf, entry_offset)?);
        }

        // Decode options
        let mut options = Vec::new();
        let mut opt_cursor = options_start;
        while opt_cursor < options_end {
            let (opt, consumed) = SdOption::decode_one(buf, opt_cursor)?;
            options.push(opt);
            opt_cursor += consumed;
        }

        Ok(SdMessage {
            reboot_flag,
            unicast_flag,
            entries,
            options,
        })
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Encode a 24-bit TTL into three big-endian bytes.
#[inline]
fn encode_ttl(dst: &mut [u8], ttl: u32) {
    // ttl is logically 24-bit; clamp to 0x00FF_FFFF
    let ttl = ttl & 0x00FF_FFFF;
    dst[0] = (ttl >> 16) as u8;
    dst[1] = (ttl >> 8) as u8;
    dst[2] = ttl as u8;
}

/// Decode three big-endian bytes into a 24-bit TTL.
#[inline]
fn decode_ttl(src: &[u8]) -> u32 {
    ((src[0] as u32) << 16) | ((src[1] as u32) << 8) | (src[2] as u32)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a minimal SdMessage with a single entry and round-trip it.
    fn round_trip(msg: &SdMessage) -> SdMessage {
        let encoded = msg.encode();
        SdMessage::decode(&encoded).expect("decode should succeed")
    }

    #[test]
    fn test_round_trip_find_service() {
        let msg = SdMessage {
            reboot_flag: false,
            unicast_flag: false,
            entries: vec![SdEntry::FindService {
                service_id: ServiceId(0x1234),
                instance_id: InstanceId(0x0001),
                major_version: MajorVersion(1),
                minor_version: MinorVersion(5),
                ttl: 3,
            }],
            options: vec![],
        };

        let decoded = round_trip(&msg);

        assert!(!decoded.reboot_flag);
        assert!(!decoded.unicast_flag);
        assert_eq!(decoded.entries.len(), 1);
        assert!(decoded.options.is_empty());

        match &decoded.entries[0] {
            SdEntry::FindService {
                service_id,
                instance_id,
                major_version,
                minor_version,
                ttl,
            } => {
                assert_eq!(service_id.0, 0x1234);
                assert_eq!(instance_id.0, 0x0001);
                assert_eq!(major_version.0, 1);
                assert_eq!(minor_version.0, 5);
                assert_eq!(*ttl, 3);
            }
            other => panic!("unexpected entry variant: {other:?}"),
        }
    }

    #[test]
    fn test_round_trip_offer_service_with_ipv4_option() {
        let addr = Ipv4Addr::new(192, 168, 1, 10);
        let msg = SdMessage {
            reboot_flag: true,
            unicast_flag: true,
            entries: vec![SdEntry::OfferService {
                service_id: ServiceId(0xABCD),
                instance_id: InstanceId(0x0002),
                major_version: MajorVersion(2),
                minor_version: MinorVersion(99),
                ttl: 5,
            }],
            options: vec![SdOption::Ipv4Endpoint {
                addr,
                port: 30509,
                protocol: TransportProtocol::Udp,
            }],
        };

        let encoded = msg.encode();
        let decoded = SdMessage::decode(&encoded).expect("decode should succeed");

        assert!(decoded.reboot_flag);
        assert!(decoded.unicast_flag);
        assert_eq!(decoded.entries.len(), 1);
        assert_eq!(decoded.options.len(), 1);

        match &decoded.entries[0] {
            SdEntry::OfferService {
                service_id,
                instance_id,
                major_version,
                minor_version,
                ttl,
            } => {
                assert_eq!(service_id.0, 0xABCD);
                assert_eq!(instance_id.0, 0x0002);
                assert_eq!(major_version.0, 2);
                assert_eq!(minor_version.0, 99);
                assert_eq!(*ttl, 5);
            }
            other => panic!("unexpected entry variant: {other:?}"),
        }

        match &decoded.options[0] {
            SdOption::Ipv4Endpoint {
                addr: a,
                port,
                protocol,
            } => {
                assert_eq!(*a, addr);
                assert_eq!(*port, 30509);
                assert_eq!(*protocol, TransportProtocol::Udp);
            }
        }
    }

    #[test]
    fn test_round_trip_subscribe_eventgroup() {
        let msg = SdMessage {
            reboot_flag: false,
            unicast_flag: true,
            entries: vec![SdEntry::SubscribeEventgroup {
                service_id: ServiceId(0x0100),
                instance_id: InstanceId(0xFFFF),
                major_version: MajorVersion(1),
                ttl: 0xFFFFFF, // max 24-bit TTL
                event_group_id: EventGroupId(0x0010),
                counter: 3,
            }],
            options: vec![],
        };

        let decoded = round_trip(&msg);

        assert!(!decoded.reboot_flag);
        assert!(decoded.unicast_flag);
        assert_eq!(decoded.entries.len(), 1);

        match &decoded.entries[0] {
            SdEntry::SubscribeEventgroup {
                service_id,
                instance_id,
                major_version,
                ttl,
                event_group_id,
                counter,
            } => {
                assert_eq!(service_id.0, 0x0100);
                assert_eq!(instance_id.0, 0xFFFF);
                assert_eq!(major_version.0, 1);
                assert_eq!(*ttl, 0xFFFFFF);
                assert_eq!(event_group_id.0, 0x0010);
                assert_eq!(*counter, 3);
            }
            other => panic!("unexpected entry variant: {other:?}"),
        }
    }

    #[test]
    fn test_stop_subscribe_eventgroup_has_ttl_zero() {
        let entry = SdEntry::StopSubscribeEventgroup {
            service_id: ServiceId(0x0100),
            instance_id: InstanceId(0x0001),
            event_group_id: EventGroupId(0x0010),
        };

        let (raw, _opt) = entry.encode();

        // TTL occupies bytes 9-11 (3 bytes, big-endian 24-bit)
        let ttl = decode_ttl(&raw[9..12]);
        assert_eq!(ttl, 0, "StopSubscribeEventgroup must encode TTL = 0");
    }

    #[test]
    fn test_round_trip_multiple_entries() {
        let msg = SdMessage {
            reboot_flag: true,
            unicast_flag: false,
            entries: vec![
                SdEntry::FindService {
                    service_id: ServiceId(0x0001),
                    instance_id: InstanceId(0x0001),
                    major_version: MajorVersion(1),
                    minor_version: MinorVersion(0),
                    ttl: 10,
                },
                SdEntry::OfferService {
                    service_id: ServiceId(0x0002),
                    instance_id: InstanceId(0x0001),
                    major_version: MajorVersion(1),
                    minor_version: MinorVersion(0),
                    ttl: 30,
                },
                SdEntry::SubscribeEventgroup {
                    service_id: ServiceId(0x0002),
                    instance_id: InstanceId(0x0001),
                    major_version: MajorVersion(1),
                    ttl: 30,
                    event_group_id: EventGroupId(0x0001),
                    counter: 0,
                },
                SdEntry::StopSubscribeEventgroup {
                    service_id: ServiceId(0x0003),
                    instance_id: InstanceId(0x0001),
                    event_group_id: EventGroupId(0x0002),
                },
            ],
            options: vec![],
        };

        let encoded = msg.encode();
        let decoded = SdMessage::decode(&encoded).expect("decode should succeed");

        assert_eq!(decoded.entries.len(), 4);
        assert!(decoded.reboot_flag);
        assert!(!decoded.unicast_flag);

        assert!(matches!(decoded.entries[0], SdEntry::FindService { .. }));
        assert!(matches!(decoded.entries[1], SdEntry::OfferService { .. }));
        assert!(matches!(
            decoded.entries[2],
            SdEntry::SubscribeEventgroup { .. }
        ));
        assert!(matches!(
            decoded.entries[3],
            SdEntry::StopSubscribeEventgroup { .. }
        ));
    }

    #[test]
    fn test_decode_truncated_message_returns_error() {
        // A buffer too short to be a valid SD message
        let buf = [0u8; 6];
        let result = SdMessage::decode(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_flags_encoding() {
        let msg = SdMessage {
            reboot_flag: true,
            unicast_flag: false,
            entries: vec![],
            options: vec![],
        };
        let encoded = msg.encode();
        // bit 7 set, bit 6 clear
        assert_eq!(encoded[0], 0b1000_0000);

        let msg2 = SdMessage {
            reboot_flag: false,
            unicast_flag: true,
            entries: vec![],
            options: vec![],
        };
        let encoded2 = msg2.encode();
        assert_eq!(encoded2[0], 0b0100_0000);
    }
}
