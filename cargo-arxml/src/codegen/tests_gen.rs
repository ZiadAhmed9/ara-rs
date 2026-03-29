use crate::error::CargoArxmlError;
use crate::parser::ir::ArxmlProject;

/// Generate a `tests.rs` file with basic round-trip smoke tests for each
/// service.
pub fn generate_tests(_project: &ArxmlProject) -> Result<String, CargoArxmlError> {
    // TODO(Week 3): emit `quote!`-generated integration test scaffolding.
    Ok(String::from(
        "// Auto-generated test scaffolding — not yet implemented.\n",
    ))
}
