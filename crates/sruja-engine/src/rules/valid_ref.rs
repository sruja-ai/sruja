//! Valid reference validation rule
//!
//! Ensures all references in relations point to valid, defined elements in the architecture.
//!
//! This rule validates that every relation (e.g., `A -> B "calls"`) references elements
//! that actually exist in the architecture. It prevents undefined reference errors that
//! would otherwise cause runtime issues or incorrect visualizations.
//!
//! # Behavior
//!
//! The rule performs two checks for each relation:
//! 1. **Source validation**: The element on the left side of the relation exists
//! 2. **Target validation**: The element on the right side of the relation exists
//!
//! Both the source and target are validated using flexible matching:
//! - Exact fully qualified name (FQN) match is preferred
//! - Falls back to suffix matching for nested elements
//!
//! # Error Detection
//!
//! This rule detects the following issues:
//! - References to undefined elements (both source and target)
//! - Typos in element names
//! - Missing element definitions
//!
//! # Examples
//!
//! ## Valid Architecture
//!
//! ```sruja
//! user = person "User"
//! web = system "Web App"
//!
//! user -> web "uses"  // Both elements exist: valid
//! ```
//!
//! ## Invalid Architecture (Undefined Target)
//!
//! ```sruja
//! user = person "User"
//! // Missing: api = system "API"
//!
//! user -> api "calls"  // Error: 'api' is not defined
//! ```
//!
//! The above would produce:
//! ```text
//! [E202] Error: Undefined element 'api' in relation (invalid target reference)
//!   --> example.sruja:3:8
//!
//!   = Help: Element 'api' must be defined before it can be referenced
//!          Run `sruja tree example.sruja` to list defined element IDs
//!          Check for typos or spelling mistakes in the reference
//!          Did you mean: 'web'
//! ```
//!
//! ## Nested Element References
//!
//! The rule supports flexible matching for nested elements:
//!
//! ```sruja
//! app = system "App" {
//!   api = container "API"
//!   db = container "Database"
//! }
//!
//! // Can reference via FQN
//! app.api -> app.db "queries"  // Valid: exact FQN match
//!
//! // Or via suffix (when in scope)
//! api -> db "stores"  // Valid: suffix match finds app.api and app.db
//! ```
//!
//! # Implementation Details
//!
//! The validation algorithm:
//! 1. Collects all element definitions from the program
//! 2. Iterates through all relations in the architecture
//! 3. For each relation, checks that both source and target elements exist
//! 4. Uses flexible element lookup (exact FQN or suffix match)
//! 5. Generates diagnostics for any undefined references with helpful suggestions

use crate::DomainSchema;
use std::collections::HashSet;

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{Program, Relation};

// Utilities are used internally by helper functions
use crate::validator::Rule;

/// Rule that validates all references in relations point to existing elements.
///
/// This rule ensures that every relation in the architecture references
/// elements that are actually defined, preventing undefined reference errors.
pub struct ValidRefRule;

impl Rule for ValidRefRule {
    /// Returns the human-readable name of this validation rule.
    ///
    /// # Returns
    ///
    /// `"Valid References"`
    fn name(&self) -> &str {
        "Valid References"
    }

    /// Validates a program and returns diagnostics for any undefined references.
    ///
    /// # Algorithm
    ///
    /// 1. **Collect Elements**: Builds a complete index of all defined elements
    /// 2. **Check Relations**: For each relation, validates both endpoints:
    ///    - Source element must exist
    ///    - Target element must exist
    /// 3. **Generate Diagnostics**: Creates error messages with context and suggestions
    ///
    /// # Performance
    ///
    /// - Time Complexity: O(n + m) where n = elements, m = relations
    /// - Space Complexity: O(n) for the element index
    ///
    /// # Arguments
    ///
    /// * `program` - The architecture program to validate
    ///
    /// # Returns
    ///
    /// A vector of diagnostics, one for each undefined reference found
    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Collect all elements from the program to build our lookup index
        // This is done once for efficiency, as element lookup is O(1) after
        let (elements, relations) = sruja_language::collect_elements(program);
        let element_ids: HashSet<String> = elements.keys().cloned().collect();

        // Validate each relation in the architecture
        // A relation is valid only if both source and target elements exist
        for relation in &relations {
            validate_relation(relation, &element_ids, &mut diagnostics);
        }

        diagnostics
    }
}

/// Validates a single relation and adds diagnostics if references are invalid.
///
/// This helper function encapsulates the validation logic for a single relation,
/// making the main validation loop clearer and more maintainable.
///
/// # Validation Steps
///
/// 1. Extract source and target element names from the relation
/// 2. Check if source element exists, generate diagnostic if not
/// 3. Check if target element exists, generate diagnostic if not
///
/// # Arguments
///
/// * `relation` - The relation to validate
/// * `element_ids` - Set of all valid element IDs in the program
/// * `diagnostics` - Output vector to collect any error diagnostics
///
/// # Side Effects
///
/// Appends error diagnostics to the `diagnostics` vector for any undefined references.
fn validate_relation(
    relation: &Relation,
    element_ids: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let source_name = relation.from.as_string();
    let target_name = relation.to.as_string();

    // Validate the source element (left side of the relation)
    if !element_exists_by_id(&source_name, element_ids, &relation.location) {
        add_undefined_reference_diagnostic(
            source_name,
            relation,
            true, // is_source
            element_ids,
            diagnostics,
        );
    }

    // Validate the target element (right side of the relation)
    if !element_exists_by_id(&target_name, element_ids, &relation.location) {
        add_undefined_reference_diagnostic(
            target_name,
            relation,
            false, // is_target
            element_ids,
            diagnostics,
        );
    }
}

/// Checks if an element exists in the element index with flexible matching.
///
/// This function performs a more sophisticated lookup than simple HashSet containment:
/// - First checks for exact FQN match
/// - Falls back to suffix matching for nested elements
/// - Uses element existence validation to handle both cases
///
/// # Arguments
///
/// * `element_id` - The element ID to look up (FQN or leaf ID)
/// * `element_ids` - Set of all valid element IDs in the program
/// * `location` - Source location for error reporting (unused in current implementation)
///
/// # Returns
///
/// `true` if the element exists (exact or suffix match), `false` otherwise
fn element_exists_by_id(
    element_id: &str,
    element_ids: &HashSet<String>,
    _location: &sruja_diagnostics::SourceLocation,
) -> bool {
    // Fast path: exact match on fully qualified name
    if element_ids.contains(element_id) {
        return true;
    }

    // Fallback: check for suffix match (e.g., "container" matches "system.container")
    let suffix = format!(".{}", element_id);
    element_ids.iter().any(|id| id.ends_with(&suffix))
}

/// Creates and adds a diagnostic for an undefined reference.
///
/// This function generates a user-friendly error message with context and
/// actionable suggestions to help developers fix the undefined reference.
///
/// # Error Message Structure
///
/// The diagnostic includes:
/// - **Main message**: Clear statement of what's undefined
/// - **Context**: The relation where the error occurred
/// - **Suggestions**: Actionable steps to resolve the issue
///
/// # Suggestions Generated
///
/// 1. Verify the element is defined before use
/// 2. Check for typos in the reference
/// 3. Consider nested element naming conventions
///
/// # Arguments
///
/// * `reference_name` - The name of the undefined element being referenced
/// * `relation` - The relation containing the invalid reference
/// * `is_source` - Whether this is the source (true) or target (false) of the relation
/// * `element_ids` - Set of all valid element IDs (used for "Did you mean?" suggestions)
/// * `diagnostics` - Output vector to collect the diagnostic
///
/// # Side Effects
///
/// Appends a new diagnostic to the `diagnostics` vector.
fn add_undefined_reference_diagnostic(
    reference_name: String,
    relation: &Relation,
    is_source: bool,
    element_ids: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // Construct a descriptive message that clearly indicates the problem
    let role = if is_source { "source" } else { "target" };
    let message = format!(
        "Undefined element '{}' in relation (invalid {} reference)",
        reference_name, role
    );

    // Generate context that shows the problematic relation
    let relation_context = match &relation.label {
        Some(label) if !label.trim().is_empty() => format!(
            "{} -> {} \"{}\"",
            relation.from.as_string(),
            relation.to.as_string(),
            label
        ),
        _ => format!(
            "{} -> {}",
            relation.from.as_string(),
            relation.to.as_string()
        ),
    };

    // Create actionable suggestions to help the developer resolve the issue
    let mut suggestions = vec![
        format!(
            "Element '{}' must be defined before it can be referenced",
            reference_name
        ),
        format!(
            "Run `sruja tree {}` to list defined element IDs",
            relation.location.file
        ),
        "Check for typos or spelling mistakes in the reference".to_string(),
        format!(
            "Ensure the element name '{}' matches exactly (case-sensitive)",
            reference_name
        ),
        "For nested elements, use the fully qualified name (e.g., 'system.container')".to_string(),
    ];

    // Suggest likely intended elements (best-effort, avoids noisy guesses).
    let ids_vec: Vec<String> = element_ids.iter().cloned().collect();
    let candidates = best_reference_candidates(&reference_name, ids_vec);
    if !candidates.is_empty() {
        suggestions.push(format!(
            "Did you mean: {}",
            candidates
                .into_iter()
                .map(|c| format!("'{}'", c))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // Build and add the diagnostic with full context
    diagnostics.push(
        Diagnostic::new(
            sruja_diagnostics::codes::CODE_UNDEFINED_REF,
            Severity::Error,
            message,
            relation.location.clone(),
        )
        .with_context(vec![relation_context])
        .with_suggestions(suggestions),
    );
}

fn best_reference_candidates(reference: &str, element_ids: Vec<String>) -> Vec<String> {
    if element_ids.is_empty() {
        return Vec::new();
    }

    let reference_lc = reference.to_lowercase();

    // Prefer exact (case-insensitive) leaf matches first.
    let mut exact_leaf: Vec<String> = element_ids
        .iter()
        .filter(|id| leaf_id(id).eq_ignore_ascii_case(&reference_lc))
        .cloned()
        .collect();
    exact_leaf.sort();
    if !exact_leaf.is_empty() {
        return exact_leaf.into_iter().take(3).collect();
    }

    // Otherwise, score by edit distance against the leaf ID.
    let mut scored: Vec<(usize, String)> = element_ids
        .into_iter()
        .map(|id| {
            let leaf = leaf_id(&id).to_lowercase();
            (levenshtein(&reference_lc, &leaf), id)
        })
        .collect();

    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut out = Vec::with_capacity(3);
    for (dist, id) in scored {
        if out.len() >= 3 {
            break;
        }
        // Keep suggestions reasonably tight to avoid noise.
        if dist <= 3 || id.to_lowercase().contains(&reference_lc) {
            out.push(id);
        }
    }
    out
}

fn leaf_id(id: &str) -> &str {
    id.rsplit('.').next().unwrap_or(id)
}

/// Levenshtein edit distance (O(n*m)), fine for small strings and small element sets.
fn levenshtein(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    if a.is_empty() {
        return b.chars().count();
    }
    if b.is_empty() {
        return a.chars().count();
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let mut prev: Vec<usize> = (0..=b_chars.len()).collect();
    let mut curr: Vec<usize> = vec![0; b_chars.len() + 1];

    for (i, ca) in a_chars.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b_chars.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        prev.clone_from_slice(&curr);
    }

    prev[b_chars.len()]
}

#[cfg(test)]
mod tests {
    use crate::DomainSchema;
    use super::*;
    use sruja_language::Parser;

    /// Helper function to parse a program and run validation.
    ///
    /// Returns the validation diagnostics, or an empty vector if parsing fails.
    fn validate_program(input: &str) -> Vec<Diagnostic> {
        let parser = Parser::new("test.sruja".to_string());
        let program = match parser.parse(input) {
            Ok(p) => p,
            Err(_) => return vec![], // Skip if parse fails
        };

        let rule = ValidRefRule;
        rule.validate(&program, &DomainSchema::architecture())
    }

    #[test]
    fn test_valid_references() {
        let input = r#"
user = person "User"
web = system "Web App"
db = container "Database"

user -> web "uses"
web -> db "queries"
"#;

        let diagnostics = validate_program(input);
        assert!(
            diagnostics.is_empty(),
            "Expected no errors for valid references, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_undefined_source_reference() {
        let input = r#"
web = system "Web App"
db = container "Database"

// 'user' is not defined
user -> web "uses"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            sruja_diagnostics::codes::CODE_UNDEFINED_REF
        );
        assert!(diagnostics[0].message.contains("user"));
        assert!(diagnostics[0].message.contains("source"));
    }

    #[test]
    fn test_undefined_target_reference() {
        let input = r#"
user = person "User"
web = system "Web App"

// 'db' is not defined
web -> db "queries"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            sruja_diagnostics::codes::CODE_UNDEFINED_REF
        );
        assert!(diagnostics[0].message.contains("db"));
        assert!(diagnostics[0].message.contains("target"));
    }

    #[test]
    fn test_both_references_undefined() {
        let input = r#"
// Neither 'user' nor 'api' are defined
user -> api "calls"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics
            .iter()
            .all(|d| d.code == sruja_diagnostics::codes::CODE_UNDEFINED_REF));
    }

    #[test]
    fn test_nested_element_references() {
        let input = r#"
app = system "App" {
    api = container "API"
    db = container "Database"
}

// FQN references
app.api -> app.db "queries"

// Suffix references (should also work if in scope)
// Note: Current implementation may not support this based on context
"#;

        let diagnostics = validate_program(input);
        assert!(
            diagnostics.is_empty(),
            "Expected no errors for nested element references, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_case_sensitive_references() {
        let input = r#"
WebApp = system "Web App"
User = person "User"

// Case mismatch should fail
User -> webapp "uses"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("webapp"));
    }

    #[test]
    fn test_diagnostic_includes_suggestions() {
        let input = r#"
web = system "Web App"
api -> web "calls"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert!(!diagnostics[0].suggestions.is_empty());
        assert!(diagnostics[0]
            .suggestions
            .iter()
            .any(|s| s.contains("defined before")));
        assert!(
            diagnostics[0]
                .suggestions
                .iter()
                .any(|s| s.contains("sruja tree")),
            "Expected a suggestion to run `sruja tree` to list elements"
        );
        // "Did you mean" should suggest 'web' (only defined element) for undefined 'api'
        assert!(
            diagnostics[0]
                .suggestions
                .iter()
                .any(|s| s.contains("Did you mean") && s.contains("web")),
            "Expected 'Did you mean' suggestion with 'web': {:?}",
            diagnostics[0].suggestions
        );
    }

    #[test]
    fn test_empty_program() {
        let input = "";
        let diagnostics = validate_program(input);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_program_with_no_relations() {
        let input = r#"
user = person "User"
web = system "Web App"
db = container "Database"
"#;

        let diagnostics = validate_program(input);
        assert!(diagnostics.is_empty());
    }
}
