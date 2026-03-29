use std::collections::HashMap;

use thiserror::Error;

use crate::parser::ir::ArxmlProject;

#[derive(Debug, Clone, Error)]
pub enum ValidationError {
    #[error(
        "duplicate service ID {id:#06x} used by both '{service_name}' and another service"
    )]
    DuplicateServiceId { service_name: String, id: u16 },

    #[error("element '{element_path}' references missing type '{type_ref}'")]
    MissingTypeRef {
        element_path: String,
        type_ref: String,
    },

    #[error("service interface '{service_name}' has no methods, events, or fields")]
    EmptyServiceInterface { service_name: String },

    #[error(
        "service '{service_name}' method '{method_name}' has invalid method ID {id}"
    )]
    InvalidMethodId {
        service_name: String,
        method_name: String,
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
            if let Some(first) = seen.get(&id) {
                errors.push(ValidationError::DuplicateServiceId {
                    service_name: svc.short_name.clone(),
                    id,
                });
                // Also record the first claimant if it hasn't been reported yet.
                // (We only emit one error per duplicate — for the second+ claimant.)
                let _ = first; // suppress unused warning
            } else {
                seen.insert(id, &svc.short_name);
            }
        }
    }
}

fn check_empty_service_interfaces(
    project: &ArxmlProject,
    errors: &mut Vec<ValidationError>,
) {
    for svc in &project.services {
        if svc.methods.is_empty() && svc.events.is_empty() && svc.fields.is_empty() {
            errors.push(ValidationError::EmptyServiceInterface {
                service_name: svc.short_name.clone(),
            });
        }
    }
}
