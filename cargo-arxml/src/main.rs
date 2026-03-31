use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

use cargo_arxml::{codegen::CodeGenerator, error::CargoArxmlError, parser::ArxmlParser, validator};

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

/// cargo-arxml — ARXML parser, validator, and Rust code generator for
/// Adaptive AUTOSAR.
#[derive(Parser)]
#[command(name = "cargo-arxml", version, author)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Load and validate ARXML files, reporting semantic errors.
    Validate {
        /// Path to an ARXML file or a directory containing ARXML files.
        path: PathBuf,
    },

    /// Parse ARXML files and generate Rust source code.
    Generate {
        /// Path to an ARXML file or a directory containing ARXML files.
        path: PathBuf,

        /// Directory to write the generated Rust source files into.
        #[arg(long)]
        output_dir: PathBuf,
    },

    /// Parse ARXML files and dump the extracted IR as JSON.
    Inspect {
        /// Path to an ARXML file or a directory containing ARXML files.
        path: PathBuf,
    },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Validate { path } => run_validate(&path),
        Command::Generate { path, output_dir } => run_generate(&path, &output_dir),
        Command::Inspect { path } => run_inspect(&path),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

fn run_validate(path: &std::path::Path) -> Result<(), CargoArxmlError> {
    eprintln!("Loading ARXML files from '{}'…", path.display());

    let parser = ArxmlParser::load(path)?;
    let project = parser.extract_ir()?;

    eprintln!(
        "Loaded {} service interface(s) and {} data type(s).",
        project.services.len(),
        project.data_types.len()
    );

    let errors = validator::validate(&project);

    if errors.is_empty() {
        println!("Validation passed — no errors found.");
        Ok(())
    } else {
        eprintln!("Validation failed with {} error(s):", errors.len());
        for (i, e) in errors.iter().enumerate() {
            eprintln!("  [{}] {}", i + 1, e);
        }
        Err(CargoArxmlError::Validation { errors })
    }
}

fn run_generate(
    path: &std::path::Path,
    output_dir: &std::path::Path,
) -> Result<(), CargoArxmlError> {
    eprintln!("Loading ARXML files from '{}'…", path.display());

    let parser = ArxmlParser::load(path)?;
    let project = parser.extract_ir()?;

    // Validation is a prerequisite for code generation.
    let errors = validator::validate(&project);
    if !errors.is_empty() {
        eprintln!(
            "Aborting code generation: {} validation error(s):",
            errors.len()
        );
        for (i, e) in errors.iter().enumerate() {
            eprintln!("  [{}] {}", i + 1, e);
        }
        return Err(CargoArxmlError::Validation { errors });
    }

    let generator = CodeGenerator::new(&project);
    let files = generator.generate()?;

    // Create output directory if it does not exist.
    std::fs::create_dir_all(output_dir).map_err(|e| CargoArxmlError::Io {
        source: e,
        path: output_dir.to_path_buf(),
    })?;

    for (filename, source) in &files {
        let dest = output_dir.join(filename);

        // Create any intermediate directories (e.g. `proxy/`, `skeleton/`).
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CargoArxmlError::Io {
                source: e,
                path: parent.to_path_buf(),
            })?;
        }

        std::fs::write(&dest, source).map_err(|e| CargoArxmlError::Io {
            source: e,
            path: dest.clone(),
        })?;

        println!("  wrote {}", dest.display());
    }

    println!(
        "Generated {} file(s) into '{}'.",
        files.len(),
        output_dir.display()
    );
    Ok(())
}

fn run_inspect(path: &std::path::Path) -> Result<(), CargoArxmlError> {
    eprintln!("Loading ARXML files from '{}'…", path.display());

    let parser = ArxmlParser::load(path)?;
    let project = parser.extract_ir()?;

    let json = serde_json::to_string_pretty(&project).map_err(|e| CargoArxmlError::CodeGen {
        message: format!("JSON serialization failed: {e}"),
    })?;

    println!("{json}");
    Ok(())
}
