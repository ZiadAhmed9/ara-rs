use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::parser::ir::ArxmlProject;

#[derive(Debug, Clone, Error)]
pub enum ValidationError {
    #[error("duplicate service ID {id:#06x} used by both '{service_name}' and another service")]
    DuplicateServiceId { service_name: String, id: u16 },

    #[error("element '{element_path}' references missing type '{type_ref}'")]
    MissingTypeRef {
        element_path: String,
        type_ref: String,
    },

    #[error("service interface '{service_name}' has no methods, events, or fields")]
    EmptyServiceInterface { service_name: String },

    #[error("service '{service_name}' method '{method_name}' has invalid method ID {id:#06x}")]
    InvalidMethodId {
        service_name: String,
        method_name: String,
        id: u16,
    },

    #[error("service '{service_name}' has duplicate method ID {id:#06x} (methods: '{first}', '{second}')")]
    DuplicateMethodId {
        service_name: String,
        first: String,
        second: String,
        id: u16,
    },
}

/// Run all semantic checks over `project` and return any errors found.
///
/// This does **not** return a `Result` — callers decide whether errors are
/// fatal.  The list is empty when the project is valid.
pub fn validate(project: &ArxmlProject) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    check_duplicate_service_ids(project, &mut errors);
    check_empty_service_interfaces(project, &mut errors);
    check_missing_type_refs(project, &mut errors);
    check_invalid_method_ids(project, &mut errors);

    errors
}

// ---------------------------------------------------------------------------
// Individual checks
// ---------------------------------------------------------------------------

fn check_duplicate_service_ids(project: &ArxmlProject, errors: &mut Vec<ValidationError>) {
    // Map service_id → first service name that claimed it.
    let mut seen: HashMap<u16, &str> = HashMap::new();

    for svc in &project.services {
        if let Some(id) = svc.service_id {
            if let Some(_first) = seen.get(&id) {
                errors.push(ValidationError::DuplicateServiceId {
                    service_name: svc.short_name.clone(),
                    id,
                });
            } else {
                seen.insert(id, &svc.short_name);
            }
        }
    }
}

fn check_empty_service_interfaces(project: &ArxmlProject, errors: &mut Vec<ValidationError>) {
    for svc in &project.services {
        if svc.methods.is_empty() && svc.events.is_empty() && svc.fields.is_empty() {
            errors.push(ValidationError::EmptyServiceInterface {
                service_name: svc.short_name.clone(),
            });
        }
    }
}

/// Check that every type_ref in method parameters, event data types, field
/// data types, and composite data type fields resolves to a known DataType path
/// or a recognized primitive pattern.
fn check_missing_type_refs(project: &ArxmlProject, errors: &mut Vec<ValidationError>) {
    // Build set of known type paths.
    let known_types: HashSet<&str> = project
        .data_types
        .iter()
        .map(|dt| dt.path.as_str())
        .collect();

    for svc in &project.services {
        // Method parameters
        for method in &svc.methods {
            for param in method
                .input_params
                .iter()
                .chain(method.output_params.iter())
            {
                if !is_resolvable_type_ref(&param.type_ref, &known_types) {
                    errors.push(ValidationError::MissingTypeRef {
                        element_path: format!("{}/{}/{}", svc.short_name, method.name, param.name),
                        type_ref: param.type_ref.clone(),
                    });
                }
            }
        }

        // Event data types
        for event in &svc.events {
            if let Some(ref type_ref) = event.data_type_ref {
                if !is_resolvable_type_ref(type_ref, &known_types) {
                    errors.push(ValidationError::MissingTypeRef {
                        element_path: format!("{}/{}", svc.short_name, event.name),
                        type_ref: type_ref.clone(),
                    });
                }
            }
        }

        // Field data types
        for field in &svc.fields {
            if !is_resolvable_type_ref(&field.data_type_ref, &known_types) {
                errors.push(ValidationError::MissingTypeRef {
                    element_path: format!("{}/{}", svc.short_name, field.name),
                    type_ref: field.data_type_ref.clone(),
                });
            }
        }
    }

    // Composite data type internal references (struct fields, array/vector element types)
    for dt in &project.data_types {
        match &dt.kind {
            crate::parser::ir::DataTypeKind::Structure { fields } => {
                for field in fields {
                    if !is_resolvable_type_ref(&field.type_ref, &known_types) {
                        errors.push(ValidationError::MissingTypeRef {
                            element_path: format!("{}/{}", dt.name, field.name),
                            type_ref: field.type_ref.clone(),
                        });
                    }
                }
            }
            crate::parser::ir::DataTypeKind::Array {
                element_type_ref, ..
            }
            | crate::parser::ir::DataTypeKind::Vector {
                element_type_ref, ..
            } => {
                if !is_resolvable_type_ref(element_type_ref, &known_types) {
                    errors.push(ValidationError::MissingTypeRef {
                        element_path: dt.name.clone(),
                        type_ref: element_type_ref.clone(),
                    });
                }
            }
            crate::parser::ir::DataTypeKind::TypeReference { target_ref } => {
                if !is_resolvable_type_ref(target_ref, &known_types) {
                    errors.push(ValidationError::MissingTypeRef {
                        element_path: dt.name.clone(),
                        type_ref: target_ref.clone(),
                    });
                }
            }
            _ => {}
        }
    }
}

/// A type reference is resolvable if it matches a known DataType path, or if
/// it is a primitive type path (e.g. `/AUTOSAR/uint8`, `/primitives/uint16`).
///
/// Matching is case-insensitive on the last path segment, consistent with the
/// parser's `map_primitive_type` which lowercases before matching.
fn is_resolvable_type_ref(type_ref: &str, known_types: &HashSet<&str>) -> bool {
    if known_types.contains(type_ref) {
        return true;
    }

    // Primitive types are referenced by their last path segment in codegen,
    // and the parser produces paths like `/AUTOSAR/uint8` or just `uint8`.
    // Case-insensitive to match the parser's behavior.
    let last_segment = type_ref
        .rsplit('/')
        .next()
        .unwrap_or(type_ref)
        .to_lowercase();
    matches!(
        last_segment.as_str(),
        "boolean"
            | "bool"
            | "uint8"
            | "u8"
            | "uint16"
            | "u16"
            | "uint32"
            | "u32"
            | "uint64"
            | "u64"
            | "sint8"
            | "int8"
            | "i8"
            | "sint16"
            | "int16"
            | "i16"
            | "sint32"
            | "int32"
            | "i32"
            | "sint64"
            | "int64"
            | "i64"
            | "float32"
            | "f32"
            | "float64"
            | "f64"
            | "string"
    )
}

/// Check for invalid and duplicate method IDs within each service.
///
/// SOME/IP method IDs must be in range 0x0001..=0x7FFF. The range 0x8000..=0xFFFE
/// is reserved for events, and 0x0000 and 0xFFFF are reserved by the protocol.
fn check_invalid_method_ids(project: &ArxmlProject, errors: &mut Vec<ValidationError>) {
    for svc in &project.services {
        let mut seen_ids: HashMap<u16, &str> = HashMap::new();

        for method in &svc.methods {
            if let Some(id) = method.method_id {
                // Check reserved/invalid ranges
                if id == 0x0000 || id == 0xFFFF || id >= 0x8000 {
                    errors.push(ValidationError::InvalidMethodId {
                        service_name: svc.short_name.clone(),
                        method_name: method.name.clone(),
                        id,
                    });
                }

                // Check for duplicates within the same service
                if let Some(first_name) = seen_ids.get(&id) {
                    errors.push(ValidationError::DuplicateMethodId {
                        service_name: svc.short_name.clone(),
                        first: first_name.to_string(),
                        second: method.name.clone(),
                        id,
                    });
                } else {
                    seen_ids.insert(id, &method.name);
                }
            }
        }
    }
}
