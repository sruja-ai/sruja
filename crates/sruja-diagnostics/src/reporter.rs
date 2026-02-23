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
