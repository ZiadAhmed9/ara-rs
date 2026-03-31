use ara_com::error::AraComError;
#[async_trait::async_trait]
pub trait BatteryService: Send + Sync {
    async fn get_voltage(&self, battery_id: u8) -> Result<f64, AraComError>;
    async fn set_charge_limit(&self, limit: f64) -> Result<(), AraComError>;
}
