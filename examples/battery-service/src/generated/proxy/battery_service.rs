use ara_com::error::AraComError;
use ara_com::proxy::ProxyBase;
use ara_com::transport::{AraDeserialize, AraSerialize, Transport};
use ara_com::types::{EventGroupId, InstanceId, MethodId, ServiceId};
use std::sync::Arc;
pub struct GetVoltageRequest {
    pub battery_id: u8,
}
impl AraSerialize for GetVoltageRequest {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.battery_id.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.battery_id.serialized_size()
    }
}
impl AraDeserialize for GetVoltageRequest {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        let battery_id = <u8 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        Ok(Self { battery_id })
    }
}
pub struct GetVoltageResponse {
    pub voltage: f64,
}
impl AraSerialize for GetVoltageResponse {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.voltage.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.voltage.serialized_size()
    }
}
impl AraDeserialize for GetVoltageResponse {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        let voltage = <f64 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        Ok(Self { voltage })
    }
}
pub struct SetChargeLimitRequest {
    pub limit: f64,
}
impl AraSerialize for SetChargeLimitRequest {
    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
        self.limit.ara_serialize(buf)?;
        Ok(())
    }
    fn serialized_size(&self) -> usize {
        self.limit.serialized_size()
    }
}
impl AraDeserialize for SetChargeLimitRequest {
    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
        #[allow(unused_mut)]
        let mut offset = 0usize;
        let limit = <f64 as AraDeserialize>::ara_deserialize(&buf[offset..])?;
        Ok(Self { limit })
    }
}
pub struct BatteryServiceProxy<T: Transport> {
    pub base: ProxyBase<T>,
}
impl<T: Transport> BatteryServiceProxy<T> {
    /// Create a new proxy connected to the given transport and instance.
    pub fn new(transport: Arc<T>, instance_id: InstanceId) -> Self {
        Self {
            base: ProxyBase::with_defaults(transport, ServiceId(4096), instance_id),
        }
    }
    pub async fn get_voltage(&self, battery_id: u8) -> Result<f64, AraComError> {
        let req = GetVoltageRequest { battery_id };
        let resp: GetVoltageResponse = self.base.call_method(MethodId(1), &req).await?;
        Ok(resp.voltage)
    }
    pub async fn set_charge_limit(&self, limit: f64) -> Result<(), AraComError> {
        let req = SetChargeLimitRequest { limit };
        self.base.call_fire_and_forget(MethodId(2), &req).await
    }
    pub async fn subscribe_voltage_changed(&self) -> Result<(), AraComError> {
        self.base
            .transport()
            .subscribe_event_group(
                self.base.service_id(),
                self.base.instance_id(),
                EventGroupId(1),
            )
            .await
    }
}
