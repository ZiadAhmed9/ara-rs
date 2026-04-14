use super::super::types::*;
use ara_com::error::AraComError;
use ara_com::proxy::ProxyBase;
use ara_com::transport::{AraDeserialize, AraSerialize, Transport};
use ara_com::types::{EventGroupId, InstanceId, MethodId, ServiceId};
use std::sync::Arc;
pub struct ReadDtcRequest {
    pub dtc_id: u32,
}
impl AraSerialize for ReadDtcRequest {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.dtc_id.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.dtc_id.serialized_size()
    }
}
impl AraDeserialize for ReadDtcRequest {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        let dtc_id = <u32 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        Ok(Self { dtc_id })
    }
}
pub struct ReadDtcResponse {
    pub snapshot: DtcSnapshot,
}
impl AraSerialize for ReadDtcResponse {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.snapshot.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.snapshot.serialized_size()
    }
}
impl AraDeserialize for ReadDtcResponse {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        let snapshot = <DtcSnapshot as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        Ok(Self { snapshot })
    }
}
pub struct ClearDtcRequest {
    pub dtc_id: u32,
}
impl AraSerialize for ClearDtcRequest {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.dtc_id.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.dtc_id.serialized_size()
    }
}
impl AraDeserialize for ClearDtcRequest {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        let dtc_id = <u32 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        Ok(Self { dtc_id })
    }
}
pub struct ReadEcuIdentificationRequest {}
impl AraSerialize for ReadEcuIdentificationRequest {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        0
    }
}
impl AraDeserialize for ReadEcuIdentificationRequest {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        Ok(Self {})
    }
}
pub struct ReadEcuIdentificationResponse {
    pub info: EcuInfo,
}
impl AraSerialize for ReadEcuIdentificationResponse {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.info.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.info.serialized_size()
    }
}
impl AraDeserialize for ReadEcuIdentificationResponse {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        let info = <EcuInfo as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        Ok(Self { info })
    }
}
pub struct ReadDataByIdentifierRequest {
    pub identifier: u16,
    pub sub_function: u8,
}
impl AraSerialize for ReadDataByIdentifierRequest {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.identifier.ara_serialize(buf)?;
        self.sub_function.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.identifier.serialized_size() + self.sub_function.serialized_size()
    }
}
impl AraDeserialize for ReadDataByIdentifierRequest {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        let identifier = <u16 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        offset += identifier.serialized_size();
        let sub_function = <u8 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        Ok(Self {
            identifier,
            sub_function,
        })
    }
}
pub struct ReadDataByIdentifierResponse {
    pub data: DataRecord,
}
impl AraSerialize for ReadDataByIdentifierResponse {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.data.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.data.serialized_size()
    }
}
impl AraDeserialize for ReadDataByIdentifierResponse {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        let data = <DataRecord as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        Ok(Self { data })
    }
}
pub struct DiagnosticsServiceProxy<T: Transport> {
    pub base: ProxyBase<T>,
}
impl<T: Transport> DiagnosticsServiceProxy<T> {
    /// Create a new proxy connected to the given transport and instance.
    pub fn new(transport: Arc<T>, instance_id: InstanceId) -> Self {
        Self {
            base: ProxyBase::with_defaults(transport, ServiceId(20480), instance_id),
        }
    }
    pub async fn read_dtc(&self, dtc_id: u32) -> Result<DtcSnapshot, AraComError> {
        let req = ReadDtcRequest { dtc_id };
        let resp: ReadDtcResponse = self.base.call_method(MethodId(1), &req).await?;
        Ok(resp.snapshot)
    }
    pub async fn clear_dtc(&self, dtc_id: u32) -> Result<(), AraComError> {
        let req = ClearDtcRequest { dtc_id };
        self.base.call_fire_and_forget(MethodId(2), &req).await
    }
    pub async fn read_ecu_identification(&self) -> Result<EcuInfo, AraComError> {
        let req = ReadEcuIdentificationRequest {};
        let resp: ReadEcuIdentificationResponse = self.base.call_method(MethodId(3), &req).await?;
        Ok(resp.info)
    }
    pub async fn read_data_by_identifier(
        &self,
        identifier: u16,
        sub_function: u8,
    ) -> Result<DataRecord, AraComError> {
        let req = ReadDataByIdentifierRequest {
            identifier,
            sub_function,
        };
        let resp: ReadDataByIdentifierResponse = self.base.call_method(MethodId(4), &req).await?;
        Ok(resp.data)
    }
    pub async fn subscribe_dtc_status_changed(&self) -> Result<(), AraComError> {
        self.base
            .transport()
            .subscribe_event_group(
                self.base.service_id(),
                self.base.instance_id(),
                EventGroupId(1),
            )
            .await
    }
    pub async fn subscribe_session_changed(&self) -> Result<(), AraComError> {
        self.base
            .transport()
            .subscribe_event_group(
                self.base.service_id(),
                self.base.instance_id(),
                EventGroupId(2),
            )
            .await
    }
}
