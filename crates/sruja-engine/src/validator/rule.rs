//! Validation rule trait
//!
//! Defines the interface for all validation rules. Implement this trait to create
//! custom validation logic that can be registered with the [`super::Validator`].

use sruja_diagnostics::Diagnostic;
use sruja_language::Program;
use crate::DomainSchema;

/// Validation rule trait
///
/// This trait defines the interface for all validation rules. Implement this
/// trait to create custom validation logic that can be registered with the
/// [`Validator`](super::Validator).
///
/// # Thread Safety
///
/// Rules must be thread-safe (`Send + Sync`) to support parallel validation
/// across multiple architectures or parallel rule execution.
///
/// # Example
///
/// ```rust
/// use sruja_engine::validator::Rule;
/// use sruja_engine::DomainSchema;
/// use sruja_language::Program;
/// use sruja_diagnostics::{Diagnostic, Severity};
///
/// struct ExampleRule;
///
/// impl Rule for ExampleRule {
///     fn name(&self) -> &str {
///         "Example Rule"
///     }
///
///     fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
///         // Your validation logic here
///         vec![]
///     }
/// }
/// ```
pub trait Rule: Send + Sync {
    /// Get human-readable name of validation rule
    ///
    /// This name is used in diagnostic messages and rule exclusion configuration.
    /// It should be descriptive and unique across all registered rules.
    fn name(&self) -> &str;

    /// Validate program and return diagnostics
    ///
    /// This method is called by the validator to check the program against
    /// this rule's validation criteria. It should return an empty vector if
    /// no issues are found.
    ///
    /// # Guidelines
    ///
    /// - Be specific: Each diagnostic should point to the exact location of the issue
    /// - Be helpful: Include suggestions for fixing the issue
    /// - Be consistent: Use standard error codes and severity levels
    /// - Be performant: Avoid expensive operations in hot paths
    fn validate(&self, program: &Program, schema: &DomainSchema) -> Vec<Diagnostic>;
}
