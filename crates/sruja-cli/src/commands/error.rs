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
    #[error("Operation timed out: {message}")]
    Timeout {
        message: String,
    },
    #[error("Violations detected (--fail-on matched)")]
    FailOnViolations,
}

impl CliError {
    /// Creates a simple timeout error.
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
            message: if message.is_empty() { "Unknown parse error".into() } else { message },
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

    pub fn scan_with_help(message: impl Into<String>, help: impl Into<String>) -> Self {
        CliError::Scan {
            message: message.into(),
            help: Some(help.into()),
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
        }
    }

    /// Prints a pretty report of the error.
    pub fn report(&self) {
        use crate::utils::colors;
        match self {
            CliError::Io(e) => {
                eprintln!("{} {}: {}", colors::error("Error:"), "System IO error", e);
                eprintln!("  {} Ensure you have permissions and the path exists.", colors::info("💡 Help:"));
            }
            CliError::Parse {
                file,
                message,
                diagnostics,
                help,
                fix,
            } => {
                eprintln!("{} {} in {}: {}", colors::error("Error:"), "Parse error", file, message);
                for diag in diagnostics {
                    eprintln!("{}", sruja_diagnostics::format_diagnostic(diag));
                }
                if let Some(h) = help {
                    eprintln!("  {} {}", colors::info("💡 Help:"), h);
                }
                if let Some(f) = fix {
                    eprintln!("  {} {}", colors::success("🔧 Fix:"), f);
                }
            }
            CliError::Validation { message, help, fix } => {
                eprintln!("{} {}: {}", colors::error("Error:"), "Validation error", message);
                if let Some(h) = help {
                    eprintln!("  {} {}", colors::info("💡 Help:"), h);
                } else {
                    eprintln!("  {} Double check your arguments and configuration.", colors::info("💡 Help:"));
                }
                if let Some(f) = fix {
                    eprintln!("  {} {}", colors::success("🔧 Fix:"), f);
                }
            }
            CliError::Export(e) => {
                eprintln!("{} {}: {}", colors::error("Error:"), "Export error", e);
                eprintln!("  {} Ensure target directory exists and is writable.", colors::info("💡 Help:"));
            }
            CliError::Json(e) => {
                eprintln!("{} {}: {}", colors::error("Error:"), "JSON error", e);
                eprintln!("  {} The data structure in .sruja/ likely drift from expected schema.", colors::info("💡 Help:"));
                eprintln!("  {} Run 'sruja daily' to refresh context files.", colors::success("🔧 Fix:"));
            }
            CliError::Scan { message, help } => {
                eprintln!("{} {}: {}", colors::error("Error:"), "Scan error", message);
                if let Some(h) = help {
                    eprintln!("  {} {}", colors::info("💡 Help:"), h);
                } else {
                    eprintln!("  {} Try running 'sruja start -r .' to initialize Sruja.", colors::info("💡 Help:"));
                }
            }
            CliError::Timeout { message } => {
                eprintln!("{} {}: {}", colors::error("Error:"), "Operation timed out", message);
                eprintln!("  {} The project may be too large or there's a file system bottleneck.", colors::info("💡 Help:"));
                eprintln!("  {} Use .srujaignore to exclude large or irrelevant directories.", colors::success("🔧 Fix:"));
            }
            CliError::FailOnViolations => {
                eprintln!("{} {}", colors::error("Error:"), "Strict check failed: architecture violations detected.");
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
