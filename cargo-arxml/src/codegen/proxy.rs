use crate::error::CargoArxmlError;
use crate::parser::ir::ServiceInterface;

/// Generate a proxy stub for `svc`.
///
/// The proxy is the client-side handle used to call methods and subscribe to
/// events on a remote service.
pub fn generate_proxy(_svc: &ServiceInterface) -> Result<String, CargoArxmlError> {
    // TODO(Week 2): emit a `quote!`-generated proxy struct that wraps an
    // `ara_com::proxy::ServiceProxy` and exposes typed async methods.
    Ok(String::from(
        "// Auto-generated service proxy stub — not yet implemented.\n",
    ))
}
