use serde::{Deserialize, Serialize};

/// Root IR extracted from an ARXML project.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArxmlProject {
    pub services: Vec<ServiceInterface>,
    pub data_types: Vec<DataType>,
    /// Paths of source ARXML files that were loaded.
    pub source_files: Vec<String>,
}

/// A service interface (maps to a AUTOSAR `SERVICE-INTERFACE` element).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInterface {
    /// Human-readable display name (same as `short_name` unless overridden).
    pub name: String,
    /// AUTOSAR SHORT-NAME.
    pub short_name: String,
    /// Fully-qualified AUTOSAR path, e.g. `/services/BatteryService`.
    pub path: String,
    /// SOME/IP service identifier (optional — may not be present in pure AUTOSAR models).
    pub service_id: Option<u16>,
    pub major_version: u8,
    pub minor_version: u32,
    pub methods: Vec<Method>,
    pub events: Vec<Event>,
    pub fields: Vec<Field>,
    /// Text from the AUTOSAR `DESC` element, if present.
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    /// SOME/IP method identifier (optional).
    pub method_id: Option<u16>,
    pub input_params: Vec<Parameter>,
    pub output_params: Vec<Parameter>,
    /// `true` when the operation has no return value and no reply is expected.
    pub fire_and_forget: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    /// Autosar path of the referenced data type.
    pub type_ref: String,
    pub direction: ParamDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamDirection {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    /// SOME/IP event identifier (optional).
    pub event_id: Option<u16>,
    /// SOME/IP event group identifier (optional).
    pub event_group_id: Option<u16>,
    /// Autosar path of the data type carried by this event, if any.
    pub data_type_ref: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    /// Autosar path of the field's data type.
    pub data_type_ref: String,
    pub has_getter: bool,
    pub has_setter: bool,
    pub has_notifier: bool,
    /// SOME/IP method ID of the getter, if present.
    pub getter_method_id: Option<u16>,
    /// SOME/IP method ID of the setter, if present.
    pub setter_method_id: Option<u16>,
    /// SOME/IP event ID of the notifier, if present.
    pub notifier_event_id: Option<u16>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataType {
    pub name: String,
    /// Fully-qualified AUTOSAR path.
    pub path: String,
    pub kind: DataTypeKind,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataTypeKind {
    Primitive(PrimitiveType),
    Enumeration {
        variants: Vec<EnumVariant>,
    },
    Structure {
        fields: Vec<StructField>,
    },
    Array {
        element_type_ref: String,
        size: Option<usize>,
    },
    Vector {
        element_type_ref: String,
    },
    String {
        max_length: Option<usize>,
        encoding: StringEncoding,
    },
    TypeReference {
        target_ref: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimitiveType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub type_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringEncoding {
    Utf8,
    Utf16,
}
