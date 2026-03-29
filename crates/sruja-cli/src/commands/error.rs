//! CLI error type and conversions.

use sruja_diagnostics::Diagnostic;
use sruja_export::json::exporter::ExportError as JsonExportError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error in {file}")]
    Parse {
        file: String,
        message: String,
        diagnostics: Vec<Diagnostic>,
    },
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Export error: {0}")]
    Export(#[from] JsonExportError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Scan error: {0}")]
    Scan(String),
    #[error("Violations detected (--fail-on matched)")]
    FailOnViolations,
}

impl CliError {
    /// Creates a parse error with full diagnostic context.
    pub fn parse_with_diagnostics(file: impl Into<String>, diagnostics: Vec<Diagnostic>) -> Self {
        let message = diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        CliError::Parse {
            file: file.into(),
            message,
            diagnostics,
        }
    }

    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Io(_) => 2,
            CliError::Parse { .. } => 3,
            CliError::Validation(_) => 4,
            CliError::Export(_) => 5,
            CliError::Json(_) => 6,
            CliError::Scan(_) => 7,
            CliError::FailOnViolations => 1,
        }
    }
}

impl From<sruja_scan::ScanError> for CliError {
    fn from(e: sruja_scan::ScanError) -> Self {
        CliError::Scan(e.to_string())
    }
}
