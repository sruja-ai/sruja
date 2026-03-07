//! Standard error codes for diagnostics.
//!
//! Codes follow a hierarchical naming scheme:
//! - E1xx: Syntax errors
//! - E2xx: Semantic errors
//! - E3xx: Validation errors
//! - E4xx: Policy errors
//! - W001: Best practice warnings

// Syntax Errors (E1xx)
/// Generic syntax error
pub const CODE_SYNTAX_ERROR: &str = "E101";
/// Token was not expected in this context
pub const CODE_UNEXPECTED_TOKEN: &str = "E102";
/// Missing closing brace or bracket
pub const CODE_MISSING_BRACE: &str = "E103";
/// Invalid or malformed string literal
pub const CODE_INVALID_STRING: &str = "E104";

// Semantic Errors (E2xx)
/// Duplicate identifier found in scope
pub const CODE_DUPLICATE_ID: &str = "E201";
/// Reference to undefined identifier
pub const CODE_UNDEFINED_REF: &str = "E202";
/// Invalid relationship between elements
pub const CODE_INVALID_RELATION: &str = "E203";
/// Cycle detected in dependency graph
pub const CODE_CYCLE_DETECTED: &str = "E204";
/// Element has no parent or connection
pub const CODE_ORPHAN_ELEMENT: &str = "E205";
/// Layer architecture constraint violation
pub const CODE_LAYER_VIOLATION: &str = "E206";

// Validation Errors (E3xx)
/// Property value fails validation
pub const CODE_INVALID_PROPERTY: &str = "E301";
/// Required field is missing
pub const CODE_MISSING_FIELD: &str = "E302";
/// Custom validation rule failed
pub const CODE_VALIDATION_RULE_ERROR: &str = "E303";
/// Validation operation timed out
pub const CODE_VALIDATION_TIMEOUT: &str = "E304";
/// Validation logic panicked
pub const CODE_VALIDATION_PANIC: &str = "E305";
/// Alias for duplicate identifier
pub const CODE_DUPLICATE_IDENTIFIER: &str = "E201";
/// Alias for reference not found
pub const CODE_REFERENCE_NOT_FOUND: &str = "E202";

// Warnings
/// Best practice suggestion
pub const CODE_BEST_PRACTICE: &str = "W001";

// Policy Errors (E4xx)
/// Policy constraint violation
pub const CODE_POLICY_VIOLATION: &str = "E401";
