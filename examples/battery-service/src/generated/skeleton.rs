use ara_com::error::AraComError;
use ara_com::skeleton::SkeletonBase;
use ara_com::transport::{AraSerialize, MessageHeader, MessageType, ReturnCode, Transport};
use ara_com::types::{InstanceId, MajorVersion, MethodId, MinorVersion, ServiceId};
use std::sync::Arc;
pub struct BatteryServiceSkeleton<T: Transport> {
    pub base: SkeletonBase<T>,
}
impl<T: Transport> BatteryServiceSkeleton<T> {
    pub fn new(transport: Arc<T>, instance_id: InstanceId) -> Self {
        Self {
            base: SkeletonBase::new(transport, ServiceId(4096), instance_id),
        }
    }
    pub async fn offer(&self) -> Result<(), AraComError> {
        self.base.offer(MajorVersion(1), MinorVersion(0)).await
    }
    pub async fn stop_offer(&self) -> Result<(), AraComError> {
        self.base.stop_offer().await
    }
    pub fn transport(&self) -> &Arc<T> {
        self.base.transport()
    }
    pub async fn notify_voltage_changed(
        &self,
        payload: &impl AraSerialize,
    ) -> Result<(), AraComError> {
        let mut buf = Vec::new();
        payload.ara_serialize(&mut buf)?;
        let header = MessageHeader {
            service_id: ServiceId(4096),
            method_id: MethodId(32769),
            instance_id: self.base.instance_id(),
            session_id: 0,
            message_type: MessageType::Notification,
            return_code: ReturnCode::Ok,
        };
        self.base
            .transport()
            .send_notification(header, bytes::Bytes::from(buf))
            .await
    }
}
