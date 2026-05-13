//! Standard error codes for diagnostics.
//!
//! Codes follow a hierarchical naming scheme so that parse, validation, and
//! architecture-context findings do not collide:
//!
//! - **E1xx: Parse/syntax** – Parser and lexer (unexpected token, invalid string, etc.).
//! - **E2xx: Semantic / structural** – Resolved AST (duplicate id, undefined ref, invalid relation).
//!   E204 (cycle), E205 (orphan), E206 (layer violation) are also used for
//!   architecture-context drift findings when emitting diagnostics.
//! - **E3xx: Validation rules** – Engine rule failures (missing field, property validation, timeouts).
//! - **E4xx: Policy / governance** – Policy and constraint violations.
//! - **W001+** – Warnings (best practice, style).
//!
//! When adding new codes, use the next free number in the reserved range (e.g. E107, E207).

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
/// Invalid relationship between elements (duplicate edges, self-references, or other structural relation issues)
pub const CODE_INVALID_RELATION: &str = "E203";
/// Cycle detected in dependency graph
pub const CODE_CYCLE_DETECTED: &str = "E204";
/// Element has no parent or connection
pub const CODE_ORPHAN_ELEMENT: &str = "E205";
/// Layer architecture constraint violation
pub const CODE_LAYER_VIOLATION: &str = "E206";
/// Container/component not nested in a system
pub const CODE_NESTING_VIOLATION: &str = "E207";

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

// State Machine Errors (E31x)
/// Initial state not found in any transition
pub const CODE_SM_INITIAL_NOT_FOUND: &str = "E311";
/// Terminal state has outgoing transitions
pub const CODE_SM_TERMINAL_HAS_OUTGOING: &str = "E312";

// Contract Errors (E32x)
/// Contract has no inputs, outputs or errors (empty)
pub const CODE_CONTRACT_EMPTY: &str = "E321";

// State Machine Warnings (W31x)
/// Unreachable state
pub const CODE_SM_UNREACHABLE_STATE: &str = "W311";
/// Dead state (non-terminal with no outgoing)
pub const CODE_SM_DEAD_STATE: &str = "W312";
/// Duplicate transition
pub const CODE_SM_DUPLICATE_TRANSITION: &str = "W313";
/// State machine has no terminal states
pub const CODE_SM_NO_TERMINAL: &str = "W314";

// Contract Warnings (W32x)
/// Contract has no input fields
pub const CODE_CONTRACT_NO_INPUTS: &str = "W321";
/// Contract has no error definitions
pub const CODE_CONTRACT_NO_ERRORS: &str = "W322";
/// Contract has no constraints
pub const CODE_CONTRACT_NO_CONSTRAINTS: &str = "W323";
/// Alias for duplicate identifier (same code as [`CODE_DUPLICATE_ID`])
pub const CODE_DUPLICATE_IDENTIFIER: &str = CODE_DUPLICATE_ID;
/// Back-compat alias for [`CODE_UNDEFINED_REF`] (undefined / unresolved reference)
pub const CODE_REFERENCE_NOT_FOUND: &str = CODE_UNDEFINED_REF;

// Warnings
/// Best practice suggestion
pub const CODE_BEST_PRACTICE: &str = "W001";

// Policy Errors (E4xx)
/// Policy constraint violation
pub const CODE_POLICY_VIOLATION: &str = "E401";
/// Custom DSL constraint violation
pub const CODE_CUSTOM_CONSTRAINT: &str = "E402";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_codes_are_e1xx() {
        assert!(CODE_SYNTAX_ERROR.starts_with('E') && CODE_SYNTAX_ERROR.len() >= 4);
        assert!(CODE_UNEXPECTED_TOKEN.starts_with('E'));
        assert!(CODE_MISSING_BRACE.starts_with('E'));
        assert!(CODE_INVALID_STRING.starts_with('E'));
    }

    #[test]
    fn semantic_codes_are_e2xx() {
        assert!(CODE_DUPLICATE_ID.starts_with('E'));
        assert!(CODE_UNDEFINED_REF.starts_with('E'));
        assert!(CODE_INVALID_RELATION.starts_with('E'));
        assert!(CODE_NESTING_VIOLATION.starts_with('E'));
        assert!(CODE_CYCLE_DETECTED.starts_with('E'));
        assert!(CODE_ORPHAN_ELEMENT.starts_with('E'));
        assert!(CODE_LAYER_VIOLATION.starts_with('E'));
    }

    #[test]
    fn validation_codes_are_e3xx() {
        assert!(CODE_INVALID_PROPERTY.starts_with('E'));
        assert!(CODE_MISSING_FIELD.starts_with('E'));
    }

    #[test]
    fn warning_code_is_w001() {
        assert_eq!(CODE_BEST_PRACTICE, "W001");
    }

    #[test]
    fn policy_code_is_e4xx() {
        assert!(CODE_POLICY_VIOLATION.starts_with('E'));
    }
}
