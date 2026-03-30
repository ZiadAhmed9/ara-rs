use std::path::Path;

use autosar_data::{AutosarModel, ElementName, EnumItem};

use crate::error::CargoArxmlError;
use self::ir::{
    ArxmlProject, DataType, Event, Field, Method, Parameter, ParamDirection, ServiceInterface,
};

pub mod ir;

pub struct ArxmlParser {
    model: AutosarModel,
    /// Paths of files that were successfully loaded.
    source_files: Vec<String>,
}

impl ArxmlParser {
    /// Load an ARXML project from a directory (all `.arxml` files) or a single file.
    ///
    /// When `path` is a directory every file with the `.arxml` extension found
    /// directly inside it is loaded into one shared [`AutosarModel`].  When
    /// `path` is a file only that file is loaded.
    pub fn load(path: &Path) -> Result<Self, CargoArxmlError> {
        let model = AutosarModel::new();
        let mut source_files: Vec<String> = Vec::new();

        if path.is_dir() {
            let entries = std::fs::read_dir(path).map_err(|e| CargoArxmlError::Io {
                source: e,
                path: path.to_path_buf(),
            })?;

            let mut found_any = false;
            for entry in entries {
                let entry = entry.map_err(|e| CargoArxmlError::Io {
                    source: e,
                    path: path.to_path_buf(),
                })?;
                let file_path = entry.path();
                if file_path.extension().and_then(|e| e.to_str()) == Some("arxml") {
                    Self::load_single_file(&model, &file_path)?;
                    source_files.push(file_path.display().to_string());
                    found_any = true;
                }
            }

            if !found_any {
                return Err(CargoArxmlError::Config {
                    message: format!(
                        "no .arxml files found in directory '{}'",
                        path.display()
                    ),
                });
            }
        } else {
            Self::load_single_file(&model, path)?;
            source_files.push(path.display().to_string());
        }

        Ok(Self { model, source_files })
    }

    fn load_single_file(model: &AutosarModel, path: &Path) -> Result<(), CargoArxmlError> {
        // strict = false: recoverable parse errors are demoted to warnings
        model
            .load_file(path, false)
            .map_err(|e| CargoArxmlError::ArxmlLoad {
                source: e,
                path: path.to_path_buf(),
            })?;
        Ok(())
    }

    /// Extract the intermediate representation from the loaded model.
    ///
    /// Walks the element tree depth-first and collects every
    /// `SERVICE-INTERFACE` and `IMPLEMENTATION-DATA-TYPE` it finds.
    pub fn extract_ir(&self) -> Result<ArxmlProject, CargoArxmlError> {
        let mut project = ArxmlProject {
            source_files: self.source_files.clone(),
            ..Default::default()
        };

        for (_depth, element) in self.model.elements_dfs() {
            match element.element_name() {
                ElementName::ServiceInterface => {
                    if let Some(svc) = self.extract_service_interface(&element) {
                        project.services.push(svc);
                    }
                }
                ElementName::ImplementationDataType => {
                    if let Some(dt) = self.extract_data_type(&element) {
                        project.data_types.push(dt);
                    }
                }
                _ => {}
            }
        }

        Ok(project)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn extract_service_interface(
        &self,
        element: &autosar_data::Element,
    ) -> Option<ServiceInterface> {
        let short_name = element
            .get_sub_element(ElementName::ShortName)
            .and_then(|e| e.character_data())
            .and_then(|cd| cd.string_value())?;

        let path = element.path().ok()?;

        let description = element
            .get_sub_element(ElementName::Desc)
            .and_then(|e| e.character_data())
            .and_then(|cd| cd.string_value());

        // SOME/IP service ID — stored under SERVICE-IDENTIFIER in some schemas.
        let service_id: Option<u16> = element
            .get_sub_element(ElementName::ServiceIdentifier)
            .and_then(|e| e.character_data())
            .and_then(|cd| cd.unsigned_integer_value())
            .and_then(|v| u16::try_from(v).ok());

        let major_version: u8 = element
            .get_sub_element(ElementName::MajorVersion)
            .and_then(|e| e.character_data())
            .and_then(|cd| cd.unsigned_integer_value())
            .and_then(|v| u8::try_from(v).ok())
            .unwrap_or(1);

        let minor_version: u32 = element
            .get_sub_element(ElementName::MinorVersion)
            .and_then(|e| e.character_data())
            .and_then(|cd| cd.unsigned_integer_value())
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0);

        let methods = self.extract_methods(element);
        let events = self.extract_events(element);
        let fields = self.extract_fields(element);

        Some(ServiceInterface {
            name: short_name.clone(),
            short_name,
            path,
            service_id,
            major_version,
            minor_version,
            methods,
            events,
            fields,
            description,
        })
    }

    fn extract_methods(&self, service_element: &autosar_data::Element) -> Vec<Method> {
        let mut methods = Vec::new();

        let methods_container =
            match service_element.get_sub_element(ElementName::Methods) {
                Some(m) => m,
                None => return methods,
            };

        for child in methods_container.sub_elements() {
            if child.element_name() != ElementName::ClientServerOperation {
                continue;
            }

            let name = match child
                .get_sub_element(ElementName::ShortName)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.string_value())
            {
                Some(n) => n,
                None => continue,
            };

            let method_id: Option<u16> = child
                .get_sub_element(ElementName::MethodId)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.unsigned_integer_value())
                .and_then(|v| u16::try_from(v).ok());

            let description = child
                .get_sub_element(ElementName::Desc)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.string_value());

            let (input_params, output_params) = self.extract_parameters(&child);
            let fire_and_forget = output_params.is_empty();

            methods.push(Method {
                name,
                method_id,
                input_params,
                output_params,
                fire_and_forget,
                description,
            });
        }

        methods
    }

    fn extract_parameters(
        &self,
        operation: &autosar_data::Element,
    ) -> (Vec<Parameter>, Vec<Parameter>) {
        let mut input_params: Vec<Parameter> = Vec::new();
        let mut output_params: Vec<Parameter> = Vec::new();

        let params_container =
            match operation.get_sub_element(ElementName::Arguments) {
                Some(p) => p,
                None => return (input_params, output_params),
            };

        for param in params_container.sub_elements() {
            if param.element_name() != ElementName::ArgumentDataPrototype {
                continue;
            }

            let name = match param
                .get_sub_element(ElementName::ShortName)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.string_value())
            {
                Some(n) => n,
                None => continue,
            };

            let type_ref = param
                .get_sub_element(ElementName::TypeTref)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.string_value())
                .unwrap_or_default();

            // The DIRECTION element carries a CharacterData::Enum value.
            let direction = param
                .get_sub_element(ElementName::Direction)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.enum_value())
                .map(|ev| match ev {
                    EnumItem::Out => ParamDirection::Out,
                    EnumItem::Inout => ParamDirection::InOut,
                    _ => ParamDirection::In,
                })
                .unwrap_or(ParamDirection::In);

            let p = Parameter {
                name,
                type_ref,
                direction: direction.clone(),
            };

            match direction {
                ParamDirection::Out => output_params.push(p),
                ParamDirection::InOut => {
                    // InOut is included in both lists so callers can see it
                    // as both an input and an output.
                    input_params.push(p.clone());
                    output_params.push(p);
                }
                ParamDirection::In => input_params.push(p),
            }
        }

        (input_params, output_params)
    }

    fn extract_events(&self, service_element: &autosar_data::Element) -> Vec<Event> {
        let mut events = Vec::new();

        let events_container =
            match service_element.get_sub_element(ElementName::Events) {
                Some(e) => e,
                None => return events,
            };

        for child in events_container.sub_elements() {
            // Events in a ServiceInterface are VariableDataPrototype elements.
            if child.element_name() != ElementName::VariableDataPrototype {
                continue;
            }

            let name = match child
                .get_sub_element(ElementName::ShortName)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.string_value())
            {
                Some(n) => n,
                None => continue,
            };

            let event_id: Option<u16> = child
                .get_sub_element(ElementName::EventId)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.unsigned_integer_value())
                .and_then(|v| u16::try_from(v).ok());

            let event_group_id: Option<u16> = child
                .get_sub_element(ElementName::EventGroupIdentifier)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.unsigned_integer_value())
                .and_then(|v| u16::try_from(v).ok());

            let data_type_ref = child
                .get_sub_element(ElementName::TypeTref)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.string_value());

            let description = child
                .get_sub_element(ElementName::Desc)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.string_value());

            events.push(Event {
                name,
                event_id,
                event_group_id,
                data_type_ref,
                description,
            });
        }

        events
    }

    fn extract_fields(&self, service_element: &autosar_data::Element) -> Vec<Field> {
        let mut fields = Vec::new();

        let fields_container =
            match service_element.get_sub_element(ElementName::Fields) {
                Some(f) => f,
                None => return fields,
            };

        for child in fields_container.sub_elements() {
            if child.element_name() != ElementName::Field {
                continue;
            }

            let name = match child
                .get_sub_element(ElementName::ShortName)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.string_value())
            {
                Some(n) => n,
                None => continue,
            };

            let data_type_ref = child
                .get_sub_element(ElementName::TypeTref)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.string_value())
                .unwrap_or_default();

            let description = child
                .get_sub_element(ElementName::Desc)
                .and_then(|e| e.character_data())
                .and_then(|cd| cd.string_value());

            // HAS-GETTER / HAS-SETTER / HAS-NOTIFIER are boolean elements in AUTOSAR.
            let has_getter = child.get_sub_element(ElementName::HasGetter).is_some();
            let has_setter = child.get_sub_element(ElementName::HasSetter).is_some();
            let has_notifier = child.get_sub_element(ElementName::HasNotifier).is_some();

            // AUTOSAR does not expose numeric method/event IDs directly on the
            // FIELD element; those are carried on the referenced operations/events.
            // We leave these as None until a deeper reference-following pass is added.
            let getter_method_id: Option<u16> = None;
            let setter_method_id: Option<u16> = None;
            let notifier_event_id: Option<u16> = None;

            fields.push(Field {
                name,
                data_type_ref,
                has_getter,
                has_setter,
                has_notifier,
                getter_method_id,
                setter_method_id,
                notifier_event_id,
                description,
            });
        }

        fields
    }

    fn extract_data_type(&self, element: &autosar_data::Element) -> Option<DataType> {
        let short_name = element
            .get_sub_element(ElementName::ShortName)
            .and_then(|e| e.character_data())
            .and_then(|cd| cd.string_value())?;

        let path = element.path().ok()?;

        let description = element
            .get_sub_element(ElementName::Desc)
            .and_then(|e| e.character_data())
            .and_then(|cd| cd.string_value());

        // Derive the IR kind from the AUTOSAR CATEGORY attribute.
        let category = element
            .get_sub_element(ElementName::Category)
            .and_then(|e| e.character_data())
            .and_then(|cd| cd.string_value())
            .unwrap_or_default();

        let kind = match category.to_uppercase().as_str() {
            "VALUE" => {
                let base_type = element
                    .get_sub_element(ElementName::SwDataDefProps)
                    .and_then(|p| p.get_sub_element(ElementName::SwDataDefPropsVariants))
                    .and_then(|v| {
                        v.sub_elements().find(|e| {
                            e.element_name()
                                == ElementName::SwDataDefPropsConditional
                        })
                    })
                    .and_then(|c| c.get_sub_element(ElementName::BaseTypeRef))
                    .and_then(|e| e.character_data())
                    .and_then(|cd| cd.string_value())
                    .unwrap_or_default();

                map_primitive_type(&base_type)
                    .map(ir::DataTypeKind::Primitive)
                    .unwrap_or(ir::DataTypeKind::TypeReference {
                        target_ref: base_type,
                    })
            }
            "STRUCTURE" => {
                let fields = extract_struct_fields(element);
                ir::DataTypeKind::Structure { fields }
            }
            "ARRAY" => {
                let element_type_ref = element
                    .get_sub_element(ElementName::SubElements)
                    .and_then(|se| se.sub_elements().next())
                    .and_then(|sub| sub.get_sub_element(ElementName::TypeTref))
                    .and_then(|e| e.character_data())
                    .and_then(|cd| cd.string_value())
                    .unwrap_or_default();
                ir::DataTypeKind::Array {
                    element_type_ref,
                    size: None,
                }
            }
            "VECTOR" => {
                let element_type_ref = element
                    .get_sub_element(ElementName::SubElements)
                    .and_then(|se| se.sub_elements().next())
                    .and_then(|sub| sub.get_sub_element(ElementName::TypeTref))
                    .and_then(|e| e.character_data())
                    .and_then(|cd| cd.string_value())
                    .unwrap_or_default();
                ir::DataTypeKind::Vector { element_type_ref }
            }
            "STRING" => ir::DataTypeKind::String {
                max_length: None,
                encoding: ir::StringEncoding::Utf8,
            },
            _ => ir::DataTypeKind::TypeReference {
                target_ref: path.clone(),
            },
        };

        Some(DataType {
            name: short_name,
            path,
            kind,
            description,
        })
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

fn extract_struct_fields(element: &autosar_data::Element) -> Vec<ir::StructField> {
    let mut fields = Vec::new();

    let sub_elements = match element.get_sub_element(ElementName::SubElements) {
        Some(se) => se,
        None => return fields,
    };

    for sub in sub_elements.sub_elements() {
        let name = match sub
            .get_sub_element(ElementName::ShortName)
            .and_then(|e| e.character_data())
            .and_then(|cd| cd.string_value())
        {
            Some(n) => n,
            None => continue,
        };

        // Try TYPE-TREF first, then fall back to IMPLEMENTATION-DATA-TYPE-REF
        // inside SW-DATA-DEF-PROPS (the standard AUTOSAR structure).
        let type_ref = sub
            .get_sub_element(ElementName::TypeTref)
            .and_then(|e| e.character_data())
            .and_then(|cd| cd.string_value())
            .or_else(|| {
                sub.get_sub_element(ElementName::SwDataDefProps)
                    .and_then(|p| p.get_sub_element(ElementName::SwDataDefPropsVariants))
                    .and_then(|v| {
                        v.sub_elements().find(|e| {
                            e.element_name() == ElementName::SwDataDefPropsConditional
                        })
                    })
                    .and_then(|c| c.get_sub_element(ElementName::ImplementationDataTypeRef))
                    .and_then(|e| e.character_data())
                    .and_then(|cd| cd.string_value())
            })
            .unwrap_or_default();

        fields.push(ir::StructField { name, type_ref });
    }

    fields
}

/// Map an AUTOSAR base-type path to a [`ir::PrimitiveType`] by inspecting
/// its last path segment.
fn map_primitive_type(base_type_path: &str) -> Option<ir::PrimitiveType> {
    let name = base_type_path
        .split('/')
        .next_back()
        .unwrap_or("")
        .to_lowercase();
    match name.as_str() {
        "boolean" | "bool" => Some(ir::PrimitiveType::Bool),
        "uint8" | "u8" => Some(ir::PrimitiveType::U8),
        "uint16" | "u16" => Some(ir::PrimitiveType::U16),
        "uint32" | "u32" => Some(ir::PrimitiveType::U32),
        "uint64" | "u64" => Some(ir::PrimitiveType::U64),
        "sint8" | "int8" | "i8" => Some(ir::PrimitiveType::I8),
        "sint16" | "int16" | "i16" => Some(ir::PrimitiveType::I16),
        "sint32" | "int32" | "i32" => Some(ir::PrimitiveType::I32),
        "sint64" | "int64" | "i64" => Some(ir::PrimitiveType::I64),
        "float32" | "f32" => Some(ir::PrimitiveType::F32),
        "float64" | "f64" => Some(ir::PrimitiveType::F64),
        _ => None,
    }
}
