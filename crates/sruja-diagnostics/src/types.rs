//! Core diagnostic types: severity, source location, and diagnostic message.

use serde::{Deserialize, Serialize};

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[derive(Default)]
pub enum Severity {
    /// Error that prevents compilation or execution
    #[default]
    Error,
    /// Warning about potential issues
    Warning,
    /// Informational message
    Info,
    /// Hint for improvements
    Hint,
}

impl Severity {
    /// Returns the string representation of the severity.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Info => "Info",
            Severity::Hint => "Hint",
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Severity::Error),
            "warning" => Ok(Severity::Warning),
            "info" => Ok(Severity::Info),
            "hint" => Ok(Severity::Hint),
            _ => Err(format!("Unknown Severity: {s}")),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a location in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SourceLocation {
    /// Path to the source file
    pub file: String,
    /// Line number (1-indexed)
    pub line: u32,
    /// Column number (1-indexed)
    pub column: u32,
}

impl SourceLocation {
    /// Creates a new source location.
    #[must_use]
    pub fn new(file: String, line: u32, column: u32) -> Self {
        Self { file, line, column }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// A diagnostic message (error, warning, or info).
///
/// Diagnostics contain all information needed to display rich error messages
/// to users, including source location, context, and actionable suggestions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Diagnostic {
    /// Unique error code (e.g., "E001", "W001")
    pub code: String,
    /// Severity level of the diagnostic
    pub severity: Severity,
    /// Main error/warning message
    pub message: String,
    /// Source location where the diagnostic occurred
    pub location: SourceLocation,
    /// Surrounding lines of code for context
    pub context: Vec<String>,
    /// Actionable suggestions for fixing the issue
    pub suggestions: Vec<String>,
}

impl Diagnostic {
    /// Creates a new diagnostic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
    ///
    /// let diag = Diagnostic::new(
    ///     "E101",
    ///     Severity::Error,
    ///     "unexpected token",
    ///     SourceLocation::new("test.sruja".to_string(), 1, 1),
    /// );
    /// ```
    #[must_use]
    pub fn new(
        code: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            location,
            context: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Creates a new diagnostic from a reference (for reuse).
    ///
    /// This is useful when you need to use the same diagnostic
    /// multiple times, such as in tests.
    #[must_use]
    pub fn new_from_ref(
        code: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        location: &SourceLocation,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            location: location.clone(),
            context: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Adds context lines to the diagnostic.
    #[must_use]
    pub fn with_context(mut self, context: Vec<String>) -> Self {
        self.context = context;
        self
    }

    /// Adds suggestions to the diagnostic.
    #[must_use]
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {} at {}",
            self.code, self.severity, self.message, self.location
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn severity_as_str_and_from_str() {
        assert_eq!(Severity::Error.as_str(), "Error");
        assert_eq!(Severity::from_str("warning").unwrap(), Severity::Warning);
        assert_eq!(Severity::from_str("INFO").unwrap(), Severity::Info);
        assert!(Severity::from_str("critical").is_err());
    }

    #[test]
    fn severity_display_matches_as_str() {
        assert_eq!(format!("{}", Severity::Hint), "Hint");
    }

    #[test]
    fn source_location_display() {
        let loc = SourceLocation::new("a.sruja".to_string(), 2, 3);
        assert_eq!(format!("{loc}"), "a.sruja:2:3");
    }

    #[test]
    fn diagnostic_builder_and_display() {
        let loc = SourceLocation::new("b.sruja".to_string(), 1, 1);
        let diag = Diagnostic::new("E001", Severity::Error, "oops", loc.clone())
            .with_context(vec!["line".to_string()])
            .with_suggestions(vec!["fix it".to_string()]);
        assert_eq!(diag.context.len(), 1);
        assert_eq!(diag.suggestions.len(), 1);
        let from_ref = Diagnostic::new_from_ref("W001", Severity::Warning, "warn", &loc);
        assert_eq!(from_ref.location.file, "b.sruja");
        assert!(format!("{diag}").contains("E001"));
        assert!(format!("{diag}").contains("oops"));
    }

    #[test]
    fn diagnostic_serde_roundtrip() {
        let loc = SourceLocation::new("c.sruja".to_string(), 5, 10);
        let diag = Diagnostic::new("E101", Severity::Info, "note", loc);
        let json = serde_json::to_string(&diag).expect("serialize");
        let back: Diagnostic = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.code, "E101");
        assert_eq!(back.severity, Severity::Info);
        assert_eq!(back.location.line, 5);
    }
}
