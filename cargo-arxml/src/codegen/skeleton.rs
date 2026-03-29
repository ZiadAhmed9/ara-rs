use crate::error::CargoArxmlError;
use crate::parser::ir::ServiceInterface;

/// Generate a skeleton stub for `svc`.
///
/// The skeleton is the server-side adapter that receives incoming calls and
/// dispatches them to a user-supplied implementation of the service trait.
pub fn generate_skeleton(_svc: &ServiceInterface) -> Result<String, CargoArxmlError> {
    // TODO(Week 2): emit a `quote!`-generated skeleton struct.
    Ok(String::from(
        "// Auto-generated service skeleton stub — not yet implemented.\n",
    ))
}
