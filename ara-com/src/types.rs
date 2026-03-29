use std::fmt;

/// SOME/IP Service ID (16-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceId(pub u16);

/// SOME/IP Method ID (16-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MethodId(pub u16);

/// SOME/IP Event ID (16-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventId(pub u16);

/// SOME/IP Event Group ID (16-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventGroupId(pub u16);

/// Service Instance ID (16-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId(pub u16);

/// Major interface version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MajorVersion(pub u8);

/// Minor interface version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MinorVersion(pub u32);

/// Unique identifier for a service instance on the network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceInstanceId {
    pub service_id: ServiceId,
    pub instance_id: InstanceId,
    pub major_version: MajorVersion,
    pub minor_version: MinorVersion,
}

// --- Display impls ---

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServiceId(0x{:04X})", self.0)
    }
}

impl fmt::Display for MethodId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MethodId(0x{:04X})", self.0)
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EventId(0x{:04X})", self.0)
    }
}

impl fmt::Display for EventGroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EventGroupId(0x{:04X})", self.0)
    }
}

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InstanceId(0x{:04X})", self.0)
    }
}

impl fmt::Display for MajorVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MajorVersion({})", self.0)
    }
}

impl fmt::Display for MinorVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MinorVersion({})", self.0)
    }
}

impl fmt::Display for ServiceInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ServiceInstance(service={}, instance={}, version={}.{})",
            self.service_id, self.instance_id, self.major_version.0, self.minor_version.0
        )
    }
}

// --- From conversions ---

impl From<u16> for ServiceId {
    fn from(v: u16) -> Self {
        ServiceId(v)
    }
}

impl From<u16> for MethodId {
    fn from(v: u16) -> Self {
        MethodId(v)
    }
}

impl From<u16> for EventId {
    fn from(v: u16) -> Self {
        EventId(v)
    }
}

impl From<u16> for EventGroupId {
    fn from(v: u16) -> Self {
        EventGroupId(v)
    }
}

impl From<u16> for InstanceId {
    fn from(v: u16) -> Self {
        InstanceId(v)
    }
}

impl From<u8> for MajorVersion {
    fn from(v: u8) -> Self {
        MajorVersion(v)
    }
}

impl From<u32> for MinorVersion {
    fn from(v: u32) -> Self {
        MinorVersion(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_conversions() {
        assert_eq!(ServiceId::from(0x1234), ServiceId(0x1234));
        assert_eq!(MethodId::from(0x0001), MethodId(0x0001));
        assert_eq!(EventId::from(0x8001), EventId(0x8001));
        assert_eq!(EventGroupId::from(0x0010), EventGroupId(0x0010));
        assert_eq!(InstanceId::from(0x0001), InstanceId(0x0001));
        assert_eq!(MajorVersion::from(1u8), MajorVersion(1));
        assert_eq!(MinorVersion::from(0u32), MinorVersion(0));
    }

    #[test]
    fn test_display() {
        assert_eq!(ServiceId(0x1234).to_string(), "ServiceId(0x1234)");
        assert_eq!(MethodId(0x0001).to_string(), "MethodId(0x0001)");
        assert_eq!(MajorVersion(2).to_string(), "MajorVersion(2)");
        assert_eq!(MinorVersion(100).to_string(), "MinorVersion(100)");
    }

    #[test]
    fn test_service_instance_id_display() {
        let id = ServiceInstanceId {
            service_id: ServiceId(0x1234),
            instance_id: InstanceId(0x0001),
            major_version: MajorVersion(1),
            minor_version: MinorVersion(0),
        };
        assert_eq!(
            id.to_string(),
            "ServiceInstance(service=ServiceId(0x1234), instance=InstanceId(0x0001), version=1.0)"
        );
    }
}
