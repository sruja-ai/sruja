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
                eprintln!();
                eprintln!("{}", colors::style("Remediation:").bold());
                eprintln!("  1. Ensure you have permissions to the directory.");
                eprintln!("  2. Verify the path exists and is not a broken symlink.");
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
                eprintln!();
                eprintln!("{}", colors::style("Remediation:").bold());
                if let Some(f) = fix {
                    eprintln!("  1. {}", f);
                } else {
                    eprintln!("  1. Check syntax highlighting in your editor for errors.");
                }
                if let Some(h) = help {
                    eprintln!("  2. {}", h);
                }
                eprintln!(
                    "  3. Run the linter to verify fixes: {}",
                    colors::info(format!("sruja lint {}", file))
                );
            }
            CliError::Validation { message, help, fix } => {
                eprintln!("{} Validation error: {}", colors::error("Error:"), message);
                eprintln!();
                eprintln!("{}", colors::style("Remediation:").bold());
                if let Some(f) = fix {
                    eprintln!("  1. {}", f);
                } else {
                    eprintln!("  1. Double check your command arguments.");
                }
                if let Some(h) = help {
                    eprintln!("  2. {}", h);
                }
            }
            CliError::Export(e) => {
                eprintln!("{} Export error: {}", colors::error("Error:"), e);
                eprintln!();
                eprintln!("{}", colors::style("Remediation:").bold());
                eprintln!("  1. Ensure the target directory is writable.");
                eprintln!("  2. If exporting to a file, verify the parent path exists.");
            }
            CliError::Json(e) => {
                eprintln!("{} JSON error: {}", colors::error("Error:"), e);
                eprintln!();
                eprintln!("{}", colors::style("Remediation:").bold());
                eprintln!("  1. Your cached evidence in .sruja/ may be out of date.");
                eprintln!(
                    "  2. Run a full sync to refresh: {}",
                    colors::info("sruja daily")
                );
            }
            CliError::Scan { message, help } => {
                eprintln!("{} Scan error: {}", colors::error("Error:"), message);
                eprintln!();
                eprintln!("{}", colors::style("Remediation:").bold());
                if let Some(h) = help {
                    eprintln!("  1. {}", h);
                } else {
                    eprintln!(
                        "  1. If this is a new repo, run: {}",
                        colors::info("sruja start")
                    );
                }
                eprintln!("  2. Ensure your .srujaignore is not excluding all source files.");
            }
            CliError::Timeout { message } => {
                eprintln!(
                    "{} Operation timed out: {}",
                    colors::error("Error:"),
                    message
                );
                eprintln!();
                eprintln!("{}", colors::style("Remediation:").bold());
                eprintln!("  1. The project may be too large for a single scan pass.");
                eprintln!("  2. Use .srujaignore to narrow the scope.");
                eprintln!("  3. If on a slow disk, try increasing the timeout via SRUJA_TIMEOUT environment variable.");
            }
            CliError::FailOnViolations => {
                eprintln!(
                    "{} Strict check failed: architecture violations detected.",
                    colors::error("Error:")
                );
                eprintln!();
                eprintln!("{}", colors::style("Remediation:").bold());
                eprintln!("  1. Review violations in the output above.");
                eprintln!(
                    "  2. To ignore known violations, run: {}",
                    colors::info("sruja baseline")
                );
            }
            CliError::NotInitialized { path } => {
                eprintln!(
                    "{} Sruja is not initialized in {}",
                    colors::error("Error:"),
                    path
                );
                eprintln!();
                eprintln!("{}", colors::style("Remediation:").bold());
                eprintln!(
                    "  1. Initialize Sruja to get started: {}",
                    colors::success("sruja start")
                );
                eprintln!("  2. This will create .sruja/ and detect your architecture.");
            }
            CliError::ConfigCorrupted { message } => {
                eprintln!(
                    "{} Configuration corrupted: {}",
                    colors::error("Error:"),
                    message
                );
                eprintln!();
                eprintln!("{}", colors::style("Remediation:").bold());
                eprintln!(
                    "  1. Attempt to repair by running a sync: {}",
                    colors::info("sruja daily")
                );
                eprintln!(
                    "  2. If that fails, re-initialize (Caution: overwrites changes): {}",
                    colors::warning("sruja start --force")
                );
            }
            CliError::Discovery(message) => {
                eprintln!("{} Discovery error: {}", colors::error("Error:"), message);
                eprintln!();
                eprintln!("{}", colors::style("Remediation:").bold());
                eprintln!("  1. Ensure the repository root is correct.");
                eprintln!("  2. Check if the architecture file is valid and readable.");
            }
            CliError::CiGateExceeded { message } => {
                eprintln!(
                    "{} CI gate threshold exceeded: {}",
                    colors::error("Error:"),
                    message
                );
                eprintln!();
                eprintln!("{}", colors::style("Remediation:").bold());
                eprintln!("  1. Review the impact of your changes.");
                eprintln!("  2. Consider breaking the change into smaller PRs.");
                eprintln!("  3. To bypass, add the 'high-impact-approved' label to the PR.");
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
