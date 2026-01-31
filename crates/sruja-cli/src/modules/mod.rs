//! CLI modules
//!
//! This module contains reusable, modular components for CLI operations.

pub mod file_operations;
pub mod validation;

// Re-export commonly used types for convenience
pub use file_operations::{
    collect_sruja_files, is_directory, parse_file, read_file, write_file, ParseResult,
};
pub use sruja_diagnostics::format_diagnostic;
pub use validation::{
    validate_batch, validate_file, validate_program, BatchValidationResult, DiagnosticFormatter,
    ValidationConfig, ValidationResult,
};
