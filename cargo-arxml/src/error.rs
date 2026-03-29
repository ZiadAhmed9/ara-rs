use std::path::PathBuf;
use thiserror::Error;

use crate::validator::ValidationError;

#[derive(Debug, Error)]
pub enum CargoArxmlError {
    /// Failed to load or parse an ARXML file.
    #[error("failed to load ARXML file '{path}': {source}")]
    ArxmlLoad {
        source: autosar_data::AutosarDataError,
        path: PathBuf,
    },

    /// One or more semantic validation errors were found in the model.
    #[error("validation failed with {} error(s)", errors.len())]
    Validation { errors: Vec<ValidationError> },

    /// Code generation failed.
    #[error("code generation error: {message}")]
    CodeGen { message: String },

    /// An I/O error occurred (e.g. reading a directory or writing output files).
    #[error("I/O error for path '{path}': {source}")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },

    /// A configuration error occurred (e.g. bad Cargo.toml / arxml.toml values).
    #[error("configuration error: {message}")]
    Config { message: String },
}
