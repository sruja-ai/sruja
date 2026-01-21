//! CLI commands implementation

use std::fs;
use std::path::Path;

use sruja_diagnostics::format_diagnostic;
use sruja_engine::Validator;
use sruja_export::json::{Exporter as JsonExporter, ExportError as JsonExportError};
use sruja_export::mermaid::{MermaidConfig, MermaidExporter};
use sruja_export::dot::{DotConfig, DotExporter};
use sruja_language::Parser;
use sruja_lsp::server::run_stdio;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Export error: {0}")]
    Export(#[from] JsonExportError),
}

/// Print version information
pub fn version() -> Result<(), CliError> {
    println!("sruja version 2.0.0");
    Ok(())
}

/// Lint a Sruja file
pub async fn lint(file: &str) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());
    
    // Parse the file
    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(diagnostics) => {
            // Print parse diagnostics
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!("Parsing failed with {} errors", diagnostics.len())));
        }
    };

    // Validate
    let mut validator = Validator::new();
    validator.register_default_rules();
    let diagnostics = validator.validate_sync(&program);

    if diagnostics.is_empty() {
        println!("✓ No issues found");
        return Ok(());
    }

    // Separate errors and warnings
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for diag in diagnostics {
        match diag.severity {
            sruja_diagnostics::Severity::Error => errors.push(diag),
            sruja_diagnostics::Severity::Warning => warnings.push(diag),
            _ => {}
        }
    }

    // Print warnings first
    for warning in &warnings {
        eprintln!("{}", format_diagnostic(warning));
    }

    // Print errors
    for error in &errors {
        eprintln!("{}", format_diagnostic(error));
    }

    if !errors.is_empty() {
        eprintln!("\n✗ Found {} error(s) and {} warning(s)", errors.len(), warnings.len());
        return Err(CliError::Validation(format!("Linting failed with {} errors", errors.len())));
    }

    if !warnings.is_empty() {
        println!("✓ Found {} warning(s) (no errors)", warnings.len());
    } else {
        println!("✓ No issues found");
    }

    Ok(())
}

/// Export a Sruja file to various formats
pub async fn export(
    format: &str,
    file: &str,
    extended: bool,
    view_level: u8,
    target: Option<&str>,
) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());
    
    // Parse the file
    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(diagnostics) => {
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!("Parsing failed with {} errors", diagnostics.len())));
        }
    };

    match format {
        "json" => {
            let exporter = JsonExporter::with_extended(extended);
            let json = exporter.export(&program)?;
            println!("{}", json);
        }
        "mermaid" => {
            let exporter = MermaidExporter::new(MermaidConfig {
                view_level,
                target_id: target.map(|s| s.to_string()),
                ..MermaidConfig::default()
            });
            let mmd = exporter.export(&program);
            println!("{}", mmd);
        }
        "dot" => {
            let exporter = DotExporter::new(DotConfig {
                view_level,
                target_id: target.map(|s| s.to_string()),
                ..DotConfig::default()
            });
            let dot = exporter.export(&program);
            println!("{}", dot);
        }
        _ => {
            return Err(CliError::Export(JsonExportError::Export(
                format!("Unsupported export format: {}", format)
            )));
        }
    }

    Ok(())
}

/// Format a Sruja file
pub async fn fmt(file: &str) -> Result<(), CliError> {
    // TODO: Implement formatting
    eprintln!("Formatting not yet implemented");
    Ok(())
}

/// Start LSP server
pub async fn lsp() -> Result<(), CliError> {
    run_stdio().await.map_err(|e| CliError::Validation(format!("LSP server error: {:?}", e)))?;
    Ok(())
}

/// Compile a Sruja file
pub async fn compile(file: &str) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());
    
    // Parse the file
    let program = match parser.parse(&content) {
        Ok(program) => {
            println!("✓ Parsing successful");
            program
        }
        Err(diagnostics) => {
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!("Compilation failed with {} errors", diagnostics.len())));
        }
    };

    // Validate
    let mut validator = Validator::new();
    validator.register_default_rules();
    let diagnostics = validator.validate_sync(&program);

    if diagnostics.is_empty() {
        println!("✓ Validation successful");
        println!("✓ Compilation successful");
        return Ok(());
    }

    // Print diagnostics
    for diag in &diagnostics {
        eprintln!("{}", format_diagnostic(diag));
    }

    let error_count = diagnostics.iter().filter(|d| d.severity == sruja_diagnostics::Severity::Error).count();
    if error_count > 0 {
        eprintln!("\n✗ Compilation failed with {} error(s)", error_count);
        return Err(CliError::Validation(format!("Compilation failed with {} errors", error_count)));
    }

    println!("✓ Compilation successful (with {} warning(s))", diagnostics.len());
    Ok(())
}
