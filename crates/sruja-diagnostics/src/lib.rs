//! Diagnostics system for Sruja DSL.
//!
//! This module provides structured error and warning reporting with source locations,
//! context, and suggestions. It's designed to be compatible with the Go implementation
//! while leveraging Rust's type safety and performance.
//!
//! # Examples
//!
//! ```rust
//! use sruja_diagnostics::{Diagnostic, Severity, SourceLocation, BasicErrorReporter, ErrorReporter};
//!
//! let mut reporter = BasicErrorReporter::new();
//! let diag = Diagnostic::new(
//!     "E101",
//!     Severity::Error,
//!     "unexpected token",
//!     SourceLocation::new("test.sruja".to_string(), 1, 1),
//! );
//! reporter.report(diag);
//! ```

pub mod codes;
mod format;
mod reporter;
mod types;

#[cfg(test)]
mod tests;

pub use format::format_diagnostic;
pub use reporter::{BasicErrorReporter, ErrorReporter};
pub use types::{Diagnostic, Severity, SourceLocation};
