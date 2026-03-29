use crate::error::CargoArxmlError;
use crate::parser::ir::ArxmlProject;

/// Generate a `types.rs` file containing all data type definitions extracted
/// from the ARXML model.
pub fn generate_types(_project: &ArxmlProject) -> Result<String, CargoArxmlError> {
    // TODO(Week 2): emit `quote!`-generated structs/enums for every
    // `DataType` in the project.
    Ok(String::from(
        "// Auto-generated data type definitions — not yet implemented.\n",
    ))
}
