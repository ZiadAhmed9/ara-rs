use std::collections::HashMap;

use crate::error::CargoArxmlError;
use crate::parser::ir::ArxmlProject;

pub mod proxy;
pub mod skeleton;
pub mod traits;
pub mod types;
pub mod tests_gen;

/// Top-level code generator.
///
/// Takes an [`ArxmlProject`] IR and produces a map of
/// `filename → Rust source code`.  Each value is a fully-formatted Rust
/// source file ready to be written to disk.
pub struct CodeGenerator<'a> {
    project: &'a ArxmlProject,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(project: &'a ArxmlProject) -> Self {
        Self { project }
    }

    /// Generate all Rust source files for the project.
    ///
    /// Returns a `HashMap<filename, source>`.  Filenames are relative paths
    /// (e.g. `"types.rs"`, `"proxy/battery_service.rs"`).
    pub fn generate(&self) -> Result<HashMap<String, String>, CargoArxmlError> {
        let mut output: HashMap<String, String> = HashMap::new();

        // Data type definitions
        let types_src = types::generate_types(self.project)?;
        output.insert("types.rs".to_string(), types_src);

        // Service traits
        let traits_src = traits::generate_traits(self.project)?;
        output.insert("traits.rs".to_string(), traits_src);

        // Per-service proxy and skeleton stubs
        for svc in &self.project.services {
            let proxy_src = proxy::generate_proxy(svc)?;
            let proxy_file = format!("proxy/{}.rs", snake_case(&svc.short_name));
            output.insert(proxy_file, proxy_src);

            let skeleton_src = skeleton::generate_skeleton(svc)?;
            let skeleton_file = format!("skeleton/{}.rs", snake_case(&svc.short_name));
            output.insert(skeleton_file, skeleton_src);
        }

        // Test scaffolding
        let tests_src = tests_gen::generate_tests(self.project)?;
        output.insert("tests.rs".to_string(), tests_src);

        Ok(output)
    }
}

// ---------------------------------------------------------------------------
// Helpers shared across submodules
// ---------------------------------------------------------------------------

/// Convert a PascalCase / camelCase name to snake_case.
pub(crate) fn snake_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.extend(ch.to_lowercase());
    }
    out
}
