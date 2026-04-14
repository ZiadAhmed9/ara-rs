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

        /// Treat auto-assigned IDs as errors instead of warnings.
        /// Use this to ensure all SOME/IP IDs come from ARXML deployments.
        #[arg(long)]
        strict: bool,
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
        Command::Generate {
            path,
            output_dir,
            strict,
        } => run_generate(&path, &output_dir, strict),
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
    strict: bool,
) -> Result<(), CargoArxmlError> {
    use cargo_arxml::codegen::assign_default_ids;

    eprintln!("Loading ARXML files from '{}'…", path.display());

    let parser = ArxmlParser::load(path)?;
    let mut project = parser.extract_ir()?;

    // Auto-assign missing IDs first, so validation catches collisions
    // between explicit IDs and auto-assigned ones.
    let auto_assignments = assign_default_ids(&mut project);

    // Surface auto-assigned IDs so they are never silent.
    if !auto_assignments.is_empty() {
        if strict {
            eprintln!(
                "error: {} SOME/IP ID(s) missing from ARXML (--strict mode):",
                auto_assignments.len()
            );
            for (i, a) in auto_assignments.iter().enumerate() {
                eprintln!("  [{}] {}", i + 1, a);
            }
            eprintln!();
            eprintln!("Add SOMEIP-SERVICE-INTERFACE-DEPLOYMENT entries to your ARXML,");
            eprintln!("or remove --strict to allow auto-assignment with warnings.");
            return Err(CargoArxmlError::CodeGen {
                message: format!(
                    "{} SOME/IP ID(s) missing — add deployment entries or remove --strict",
                    auto_assignments.len()
                ),
            });
        }

        eprintln!(
            "warning: {} SOME/IP ID(s) were auto-assigned (not from ARXML):",
            auto_assignments.len()
        );
        for (i, a) in auto_assignments.iter().enumerate() {
            eprintln!("  [{}] {}", i + 1, a);
        }
        eprintln!(
            "hint: add SOMEIP-SERVICE-INTERFACE-DEPLOYMENT entries to silence these warnings."
        );
    }

    // Validation runs after auto-assignment so it catches all ID collisions.
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
    let (files, _) = generator.generate()?;

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
