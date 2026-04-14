use ara_com::error::AraComError;
use ara_com::transport::{AraDeserialize, AraSerialize};
#[derive(Debug, Clone, PartialEq)]
pub struct DtcSnapshot {
    pub dtc_id: u32,
    pub status_byte: u8,
    pub timestamp: u64,
    pub occurrence_count: u16,
}
impl AraSerialize for DtcSnapshot {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.dtc_id.ara_serialize(buf)?;
        self.status_byte.ara_serialize(buf)?;
        self.timestamp.ara_serialize(buf)?;
        self.occurrence_count.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.dtc_id.serialized_size()
            + self.status_byte.serialized_size()
            + self.timestamp.serialized_size()
            + self.occurrence_count.serialized_size()
    }
}
impl AraDeserialize for DtcSnapshot {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        let dtc_id = <u32 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        offset += dtc_id.serialized_size();
        let status_byte = <u8 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        offset += status_byte.serialized_size();
        let timestamp = <u64 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        offset += timestamp.serialized_size();
        let occurrence_count = <u16 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        Ok(Self {
            dtc_id,
            status_byte,
            timestamp,
            occurrence_count,
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct EcuInfo {
    pub serial_number: u32,
    pub hardware_version: u16,
    pub software_version: u16,
    pub uptime_seconds: u64,
}
impl AraSerialize for EcuInfo {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.serial_number.ara_serialize(buf)?;
        self.hardware_version.ara_serialize(buf)?;
        self.software_version.ara_serialize(buf)?;
        self.uptime_seconds.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.serial_number.serialized_size()
            + self.hardware_version.serialized_size()
            + self.software_version.serialized_size()
            + self.uptime_seconds.serialized_size()
    }
}
impl AraDeserialize for EcuInfo {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        let serial_number = <u32 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        offset += serial_number.serialized_size();
        let hardware_version = <u16 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        offset += hardware_version.serialized_size();
        let software_version = <u16 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        offset += software_version.serialized_size();
        let uptime_seconds = <u64 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        Ok(Self {
            serial_number,
            hardware_version,
            software_version,
            uptime_seconds,
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub identifier: u16,
    pub length: u16,
    pub value: u32,
}
impl AraSerialize for DataRecord {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.identifier.ara_serialize(buf)?;
        self.length.ara_serialize(buf)?;
        self.value.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.identifier.serialized_size()
            + self.length.serialized_size()
            + self.value.serialized_size()
    }
}
impl AraDeserialize for DataRecord {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        let identifier = <u16 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        offset += identifier.serialized_size();
        let length = <u16 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        offset += length.serialized_size();
        let value = <u32 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        Ok(Self {
            identifier,
            length,
            value,
        })
    }
}
