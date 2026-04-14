use super::types::*;
use ara_com::error::AraComError;
#[async_trait::async_trait]
pub trait DiagnosticsService: Send + Sync {
    async fn read_dtc(&self, dtc_id: u32) -> Result<DtcSnapshot, AraComError>;
    async fn clear_dtc(&self, dtc_id: u32) -> Result<(), AraComError>;
    async fn read_ecu_identification(&self) -> Result<EcuInfo, AraComError>;
    async fn read_data_by_identifier(
        &self,
        identifier: u16,
        sub_function: u8,
    ) -> Result<DataRecord, AraComError>;
}
