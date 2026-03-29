use serde::Deserialize;

/// Top-level configuration read from `arxml.toml` (or the `[arxml]` section
/// of `Cargo.toml`).
#[derive(Debug, Deserialize, Default)]
pub struct ArxmlConfig {
    pub input: Option<InputConfig>,
    pub output: Option<OutputConfig>,
    pub naming: Option<NamingConfig>,
}

/// Input source configuration.
#[derive(Debug, Deserialize)]
pub struct InputConfig {
    /// Glob patterns or explicit paths to ARXML files / directories.
    pub paths: Vec<String>,
    /// Patterns to exclude (optional).
    pub exclude: Option<Vec<String>>,
}

/// Output directory / crate configuration.
#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    /// Directory to write generated Rust source files into.
    pub dir: String,
    /// Name of the generated crate (defaults to `generated`).
    pub crate_name: Option<String>,
}

/// Naming convention overrides for generated identifiers.
#[derive(Debug, Deserialize)]
pub struct NamingConfig {
    /// Convention for generated method names (default: `snake_case`).
    pub method_style: Option<NamingStyle>,
    /// Convention for generated type names (default: `snake_case`).
    pub type_style: Option<NamingStyle>,
}

#[derive(Debug, Deserialize)]
pub enum NamingStyle {
    #[serde(rename = "snake_case")]
    SnakeCase,
    #[serde(rename = "camelCase")]
    CamelCase,
}
