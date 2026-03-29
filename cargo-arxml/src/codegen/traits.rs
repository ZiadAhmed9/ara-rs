use crate::error::CargoArxmlError;
use crate::parser::ir::ArxmlProject;

/// Generate a `traits.rs` file declaring a Rust trait for each service
/// interface.
pub fn generate_traits(_project: &ArxmlProject) -> Result<String, CargoArxmlError> {
    // TODO(Week 2): emit `quote!`-generated trait definitions for each
    // ServiceInterface in the project.
    Ok(String::from(
        "// Auto-generated service interface traits — not yet implemented.\n",
    ))
}
