use ara_com::transport::{AraSerialize, AraDeserialize};
use ara_com::error::AraComError;
#[derive(Debug, Clone, PartialEq)]
pub struct BatteryStatus {
    pub voltage: f64,
    pub current: f64,
    pub charging: bool,
}
impl AraSerialize for BatteryStatus {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.voltage.ara_serialize(buf)?;
        self.current.ara_serialize(buf)?;
        self.charging.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.voltage.serialized_size() + self.current.serialized_size()
            + self.charging.serialized_size()
    }
}
impl AraDeserialize for BatteryStatus {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        let mut offset = 0usize;
        let voltage = <f64 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        offset += voltage.serialized_size();
        let current = <f64 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        offset += current.serialized_size();
        let charging = <bool as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        offset += charging.serialized_size();
        Ok(Self { voltage, current, charging })
    }
}
