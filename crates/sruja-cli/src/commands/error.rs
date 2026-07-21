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
        help: Option<String>,
        fix: Option<String>,
    },
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        help: Option<String>,
        fix: Option<String>,
    },
    #[error("Export error: {0}")]
    Export(#[from] JsonExportError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Scan error: {message}")]
    Scan {
        message: String,
        help: Option<String>,
    },
    #[error("Discovery error: {0}")]
    Discovery(String),
    #[error("Operation timed out: {message}")]
    Timeout { message: String },
    #[error("Violations detected (--fail-on matched)")]
    FailOnViolations,
    #[error("Sruja not initialized in this repository")]
    NotInitialized { path: String },
    #[error("Configuration directory .sruja/ is corrupted or files are missing")]
    ConfigCorrupted { message: String },
    #[error("CI gate threshold exceeded")]
    CiGateExceeded { message: String },
}

impl CliError {
    #[allow(dead_code)]
    pub fn timeout(message: impl Into<String>) -> Self {
        CliError::Timeout {
            message: message.into(),
        }
    }

    /// Creates a simple validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        CliError::Validation {
            message: message.into(),
            help: None,
            fix: None,
        }
    }

    /// Creates a parse error with full diagnostic context.
    pub fn parse_with_diagnostics(file: impl Into<String>, diagnostics: Vec<Diagnostic>) -> Self {
        let message = diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        CliError::Parse {
            file: file.into(),
            message: if message.is_empty() {
                "Unknown parse error".into()
            } else {
                message
            },
            diagnostics,
            help: Some("Run 'sruja lint' for detailed error information.".into()),
            fix: Some("Check syntax in the mentioned file.".into()),
        }
    }

    pub fn scan(message: impl Into<String>) -> Self {
        CliError::Scan {
            message: message.into(),
            help: None,
        }
    }

    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Io(_) => 2,
            CliError::Parse { .. } => 3,
            CliError::Validation { .. } => 4,
            CliError::Export(_) => 5,
            CliError::Json(_) => 6,
            CliError::Scan { .. } => 7,
            CliError::Timeout { .. } => 8,
            CliError::FailOnViolations => 1,
            CliError::NotInitialized { .. } => 9,
            CliError::ConfigCorrupted { .. } => 10,
            CliError::Discovery(_) => 11,
            CliError::CiGateExceeded { .. } => 12,
        }
    }

    /// Prints a pretty report of the error.
    pub fn report(&self) {
        use crate::utils::colors;
        match self {
            CliError::Io(e) => {
                eprintln!("{} System IO error: {}", colors::error("Error:"), e);
            }
            CliError::Parse {
                file,
                message,
                diagnostics,
                help,
                fix,
            } => {
                eprintln!(
                    "{} Parse error in {}: {}",
                    colors::error("Error:"),
                    file,
                    message
                );
                for diag in diagnostics {
                    eprintln!("{}", sruja_diagnostics::format_diagnostic(diag));
                }
                if let Some(f) = fix {
                    eprintln!("{} {}", colors::style("Fix:").bold(), f);
                }
                if let Some(h) = help {
                    eprintln!("{} {}", colors::style("Tip:").bold(), h);
                }
            }
            CliError::Validation { message, help, fix } => {
                eprintln!("{} {}", colors::error("Error:"), message);
                if let Some(f) = fix {
                    eprintln!("{} {}", colors::style("Fix:").bold(), f);
                }
                if let Some(h) = help {
                    eprintln!("{} {}", colors::style("Tip:").bold(), h);
                }
            }
            CliError::Export(e) => {
                eprintln!("{} Export error: {}", colors::error("Error:"), e);
            }
            CliError::Json(e) => {
                eprintln!("{} JSON error: {}", colors::error("Error:"), e);
            }
            CliError::Scan { message, help } => {
                eprintln!("{} Scan error: {}", colors::error("Error:"), message);
                if let Some(h) = help {
                    eprintln!("{} {}", colors::style("Tip:").bold(), h);
                }
            }
            CliError::Timeout { message } => {
                eprintln!(
                    "{} Operation timed out: {}",
                    colors::error("Error:"),
                    message
                );
            }
            CliError::FailOnViolations => {
                eprintln!(
                    "{} Strict check failed: architecture violations detected.",
                    colors::error("Error:")
                );
                eprintln!(
                    "To ignore known violations: {}",
                    colors::info("sruja baseline")
                );
            }
            CliError::NotInitialized { path } => {
                eprintln!(
                    "{} Sruja is not initialized in {}",
                    colors::error("Error:"),
                    path
                );
                eprintln!(
                    "Initialize to get started: {}",
                    colors::success("sruja start")
                );
            }
            CliError::ConfigCorrupted { message } => {
                eprintln!(
                    "{} Configuration corrupted: {}",
                    colors::error("Error:"),
                    message
                );
                eprintln!(
                    "Attempt repair: {}  |  Re-initialize: {}",
                    colors::info("sruja daily"),
                    colors::warning("sruja start --force")
                );
            }
            CliError::Discovery(message) => {
                eprintln!("{} Discovery error: {}", colors::error("Error:"), message);
            }
            CliError::CiGateExceeded { message } => {
                eprintln!(
                    "{} CI gate threshold exceeded: {}",
                    colors::error("Error:"),
                    message
                );
            }
        }
    }
}

impl From<sruja_scan::ScanError> for CliError {
    fn from(e: sruja_scan::ScanError) -> Self {
        CliError::Scan {
            message: e.to_string(),
            help: Some("Try running 'sruja start -r .' to initialize Sruja in the repo.".into()),
        }
    }
}

impl From<sruja_cache::CacheError> for CliError {
    fn from(e: sruja_cache::CacheError) -> Self {
        match e {
            sruja_cache::CacheError::Io(e) => CliError::Io(e),
            sruja_cache::CacheError::Json(e) => CliError::Json(e),
            sruja_cache::CacheError::Scan(e) => CliError::from(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_map_correctly() {
        let io_err = CliError::Io(std::io::Error::other("x"));
        assert_eq!(io_err.exit_code(), 2);

        let parse_err = CliError::parse_with_diagnostics("a.sruja", vec![]);
        assert_eq!(parse_err.exit_code(), 3);

        let val_err = CliError::validation("oops");
        assert_eq!(val_err.exit_code(), 4);

        let json_err =
            CliError::Json(serde_json::from_str::<serde_json::Value>("invalid").unwrap_err());
        assert_eq!(json_err.exit_code(), 6);

        let scan_err = CliError::scan("bad");
        assert_eq!(scan_err.exit_code(), 7);

        let fail_on = CliError::FailOnViolations;
        assert_eq!(fail_on.exit_code(), 1);
    }
}
