//! Error reporter trait and basic in-memory implementation.

use crate::types::{Diagnostic, Severity};

/// Error reporter trait for collecting and reporting diagnostics.
///
/// Implementations of this trait can collect diagnostics and provide
/// information about whether errors have been reported.
pub trait ErrorReporter {
    /// Reports a diagnostic.
    fn report(&mut self, diagnostic: Diagnostic);

    /// Reports a diagnostic from an owned value.
    ///
    /// This is useful for tests where you create diagnostics
    /// inline and want to pass them directly.
    fn report_owned(&mut self, diagnostic: Diagnostic) {
        self.report(diagnostic);
    }

    /// Returns `true` if any error-level diagnostics have been reported.
    #[must_use]
    fn has_errors(&self) -> bool;

    /// Returns a slice of all reported diagnostics.
    #[must_use]
    fn diagnostics(&self) -> &[Diagnostic];
}

/// Basic in-memory implementation of [`ErrorReporter`].
///
/// This reporter collects all diagnostics in memory and provides
/// simple methods to query the collected diagnostics.
#[derive(Debug, Default)]
pub struct BasicErrorReporter {
    diagnostics: Vec<Diagnostic>,
}

impl BasicErrorReporter {
    /// Creates a new empty error reporter.
    ///
    /// Pre-allocates space for 8 diagnostics to reduce allocations
    /// in common use cases.
    #[must_use]
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::with_capacity(8),
        }
    }

    /// Clears all reported diagnostics.
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    /// Returns the number of diagnostics reported.
    #[must_use]
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns `true` if no diagnostics have been reported.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

impl ErrorReporter for BasicErrorReporter {
    fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn report_owned(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    #[inline]
    fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    #[inline]
    fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Severity, SourceLocation};

    fn sample_diagnostic(code: &str, severity: Severity) -> Diagnostic {
        Diagnostic::new(
            code,
            severity,
            "message",
            SourceLocation::new("f.sruja".to_string(), 1, 1),
        )
    }

    #[test]
    fn basic_reporter_collects_and_reports_errors() {
        let mut reporter = BasicErrorReporter::new();
        assert!(reporter.is_empty());
        reporter.report(sample_diagnostic("E1", Severity::Warning));
        reporter.report_owned(sample_diagnostic("E2", Severity::Error));
        assert_eq!(reporter.len(), 2);
        assert!(reporter.has_errors());
        assert_eq!(reporter.diagnostics().len(), 2);
        reporter.clear();
        assert!(reporter.is_empty());
        assert!(!reporter.has_errors());
    }

    #[test]
    fn has_errors_false_when_only_warnings() {
        let mut reporter = BasicErrorReporter::default();
        reporter.report(sample_diagnostic("W1", Severity::Warning));
        assert!(!reporter.has_errors());
    }
}
