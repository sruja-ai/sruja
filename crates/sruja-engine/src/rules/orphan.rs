//! Orphan element detection rule
//!
//! Detects elements in the architecture that are defined but not referenced
//! by any relations, which may indicate incomplete or unused architecture components.
//!
//! # Understanding Orphan Elements
//!
//! An **orphan element** is an element that:
//! - Is defined in the architecture
//! - Does not appear as the source OR target of any relation
//! - Is effectively disconnected from the rest of the architecture
//!
//! # When Orphan Elements Are OK
//!
//! Orphan elements are not always errors. They can be intentional:
//! - **Standalone elements**: Components that exist but have no dependencies yet
//! - **Future work**: Placeholder elements for planned features
//! - **Leaf nodes**: Top-level or bottom-level elements in specific use cases
//!
//! # When Orphan Elements Are Problems
//!
//! Orphan elements may indicate issues when:
//! - **Incomplete architecture**: Missing connections that should exist
//! - **Typos**: Element names that don't match intended references
//! - **Dead code**: Elements that are no longer needed
//! - **Forgotten components**: Elements that were added but never connected
//!
//! # Examples
//!
//! ## Valid Architecture (No Orphans)
//!
//! ```sruja
//! user = person "User"
//! web = system "Web App"
//! db = container "Database"
//!
//! user -> web "uses"
//! web -> db "queries"
//! ```
//!
//! All elements are connected through relations: no orphans.
//!
//! ## Architecture with Orphan
//!
//! ```sruja
//! user = person "User"
//! web = system "Web App"
//! cache = container "Redis Cache"  // Not referenced!
//!
//! user -> web "uses"
//! // Missing: web -> cache "caches"
//! ```
//!
//! This produces a warning:
//! ```text
//! [W001] Warning: Element 'cache' is defined but not referenced by any relation
//!   --> example.sruja:3:1
//!
//!   = Help: Orphan elements may indicate incomplete architecture
//!          Consider adding relations or removing unused elements
//! ```
//!
//! ## Nested Element Orphans
//!
//! ```sruja
//! app = system "App" {
//!   api = container "API"
//!   worker = container "Background Worker"  // Not referenced!
//!   db = container "Database"
//! }
//!
//! app.api -> app.db "queries"
//! // Missing: app.api -> app.worker or app.worker -> app.db
//! ```
//!
//! The nested element `app.worker` would be flagged as an orphan.
//!
//! # Implementation Details
//!
//! ## Detection Algorithm
//!
//! 1. **Collect all elements**: Build a complete index of defined elements
//! 2. **Collect all relations**: Identify all connections between elements
//! 3. **Build reference set**: Extract all source and target element names from relations
//! 4. **Find orphans**: Compare defined elements against referenced elements
//! 5. **Generate warnings**: Create diagnostics for unreferenced elements
//!
//! ## Reference Matching
//!
//! An element is considered "referenced" if:
//! - Its fully qualified name (FQN) appears in any relation
//! - OR its leaf ID appears in any relation (for nested elements)
//!
//! For example, `system.container` is considered referenced if a relation
//! contains either:
//! - `system.container` (exact FQN)
//! - `container` (leaf ID match, when appropriate)
//!
//! ## Performance
//!
//! - **Time Complexity**: O(n + m) where n = elements, m = relations
//! - **Space Complexity**: O(n) for element and reference sets
//! - Optimized for large architectures with many elements
//!
//! # Limitations
//!
//! - Does not consider implicit references (e.g., via grouping)
//! - Cannot distinguish between intentional and unintentional orphans
//! - Does not analyze directionality (source vs target)

use crate::DomainSchema;
use std::collections::HashSet;

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{collect_elements, ElementDef, Program};

use crate::validator::Rule;

/// Rule that detects orphan elements (elements defined but not referenced by relations).
///
/// This rule helps identify potentially incomplete or unused components in the
/// architecture by flagging elements that are defined but not connected through
/// any relations to other elements.
///
/// # Severity
///
/// This rule generates **Warning** level diagnostics because orphan elements
/// may be intentional (e.g., placeholder components for future implementation).
///
/// # Configuration
///
/// Currently, this rule always runs with default behavior. Future versions may
/// support configuration options such as:
/// - Ignoring specific element kinds (e.g., allow orphan `Person` elements)
/// - Configuring reference matching strictness
/// - Setting exemptions for specific elements via metadata
pub struct OrphanDetectionRule;

impl Rule for OrphanDetectionRule {
    /// Returns the human-readable name of this validation rule.
    ///
    /// # Returns
    ///
    /// `"Orphan Detection"`
    fn name(&self) -> &str {
        "Orphan Detection"
    }

    /// Validates a program and returns diagnostics for orphan elements.
    ///
    /// # Algorithm
    ///
    /// 1. Collect all defined elements from the program
    /// 2. Collect all relations between elements
    /// 3. Build a set of all elements that appear in any relation (source or target)
    /// 4. Find elements that are defined but do not appear in the reference set
    /// 5. Generate warning diagnostics for each orphan element
    ///
    /// # Arguments
    ///
    /// * `program` - The architecture program to validate
    ///
    /// # Returns
    ///
    /// A vector of warnings, one for each orphan element found
    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        // Collect all defined elements and relations from the program
        let (elements, relations) = collect_elements(program);

        // Build a comprehensive set of all referenced elements
        let referenced_elements = collect_referenced_elements(&relations);

        // Find and report orphan elements
        find_orphan_elements(&elements, &referenced_elements)
    }
}

/// Collects all elements that are referenced by any relation.
///
/// This function builds a complete set of element names that appear in any
/// relation, either as a source or a target. This set is then used to identify
/// orphan elements that are defined but never referenced.
///
/// # Collection Strategy
///
/// The function collects references using flexible matching:
/// - Exact fully qualified names (e.g., "system.container")
/// - Leaf IDs (e.g., "container" from "system.container")
///
/// This ensures that nested elements are correctly identified as referenced
/// even if the relation uses a simplified name.
///
/// # Arguments
///
/// * `relations` - All relations in the architecture
///
/// # Returns
///
/// A set of element names (FQNs and leaf IDs) that appear in relations
fn collect_referenced_elements(relations: &[sruja_language::Relation]) -> HashSet<String> {
    let mut referenced: HashSet<String> = HashSet::with_capacity(relations.len() * 2);

    for relation in relations {
        // Extract source and target element names
        let source_name = relation.from.as_string();
        let target_name = relation.to.as_string();

        // Add both fully qualified names to the referenced set
        referenced.insert(source_name.clone());
        referenced.insert(target_name.clone());

        // Only add leaf IDs for names that contain dots (i.e., are fully qualified)
        // This prevents false matches where leaf IDs accidentally match other elements
        if source_name.contains('.') {
            if let Some(leaf) = source_name.split('.').next_back() {
                referenced.insert(leaf.to_string());
            }
        }
        if target_name.contains('.') {
            if let Some(leaf) = target_name.split('.').next_back() {
                referenced.insert(leaf.to_string());
            }
        }
    }

    referenced
}

/// Finds orphan elements and generates diagnostics for each.
///
/// An orphan element is defined in the architecture but does not appear in
/// any relation. This function compares the set of defined elements against
/// the set of referenced elements and reports any elements that are missing
/// from the reference set.
///
/// # Matching Logic
///
/// An element is considered "referenced" if:
/// - Its exact FQN exists in the reference set
/// - OR its leaf ID exists in the reference set (for nested elements)
///
/// This allows for flexible reference matching while still catching true orphans.
///
/// # Arguments
///
/// * `elements` - Map of all defined elements (FQN -> ElementDef)
/// * `referenced_elements` - Set of all elements referenced by relations
///
/// # Returns
///
/// A vector of warning diagnostics for each orphan element found
fn find_orphan_elements(
    elements: &std::collections::HashMap<String, ElementDef>,
    referenced_elements: &HashSet<String>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Iterate through all defined elements to find orphans
    for (fully_qualified_name, element) in elements {
        // Check if this element is referenced by any relation
        let is_referenced = element_is_referenced(fully_qualified_name, referenced_elements);

        if !is_referenced {
            // Generate a warning for the orphan element
            diagnostics.push(create_orphan_diagnostic(fully_qualified_name, element));
        }
    }

    diagnostics
}

/// Checks if an element is referenced by any relation.
///
/// An element is considered referenced if either its exact FQN or its leaf ID
/// appears in the set of referenced elements. This flexible matching accommodates
/// both explicit and implicit reference styles.
///
/// # Arguments
///
/// * `fully_qualified_name` - The complete name of the element (e.g., "system.container")
/// * `referenced_elements` - Set of all elements referenced by relations
///
/// # Returns
///
/// `true` if the element is referenced, `false` otherwise
fn element_is_referenced(
    fully_qualified_name: &str,
    referenced_elements: &HashSet<String>,
) -> bool {
    // Check for exact FQN match first
    if referenced_elements.contains(fully_qualified_name) {
        return true;
    }

    // Check for leaf ID match (for nested elements)
    // e.g., "container" matches for "system.container"
    if let Some(leaf_id) = fully_qualified_name.split('.').next_back() {
        if referenced_elements.contains(leaf_id) {
            return true;
        }
    }

    false
}

/// Creates a diagnostic for an orphan element.
///
/// This function generates a user-friendly warning message with context and
/// actionable suggestions to help developers decide whether to connect the
/// orphan element or remove it.
///
/// # Diagnostic Components
///
/// - **Message**: Clear statement that the element is unreferenced
/// - **Context**: The element's fully qualified name
/// - **Suggestions**: Actionable steps to resolve the warning
///
/// # Suggestions Provided
///
/// 1. Connect the orphan to the rest of the architecture
/// 2. Remove the element if it's no longer needed
/// 3. Mark it as intentional (future enhancement)
///
/// # Arguments
///
/// * `fully_qualified_name` - The name of the orphan element
/// * `element` - The element definition (used for location)
///
/// # Returns
///
/// A warning diagnostic with helpful context and suggestions
fn create_orphan_diagnostic(fully_qualified_name: &str, element: &ElementDef) -> Diagnostic {
    let message = format!(
        "Element '{}' is defined but not referenced by any relation",
        fully_qualified_name
    );

    // Generate context showing the element definition
    let element_context = format!(
        "Element definition: {} = {} \"{}\"",
        element.assignment.name,
        element.assignment.kind.to_string().to_lowercase(),
        element.assignment.name
    );

    // Create actionable suggestions
    let suggestions = generate_orphan_suggestions(fully_qualified_name, element);

    Diagnostic::new(
        sruja_diagnostics::codes::CODE_ORPHAN_ELEMENT,
        Severity::Warning,
        message,
        element.location.clone(),
    )
    .with_context(vec![element_context])
    .with_suggestions(suggestions)
}

/// Generates actionable suggestions for handling an orphan element.
///
/// The suggestions are tailored based on the element's type and context,
/// providing relevant guidance for resolving the orphan warning.
///
/// # Suggestion Categories
///
/// 1. **Connection**: Add relations to connect the element
/// 2. **Removal**: Remove the element if it's unused
/// 3. **Documentation**: Add metadata to mark as intentional
/// 4. **Verification**: Check for typos in existing relations
///
/// # Arguments
///
/// * `fully_qualified_name` - The name of the orphan element
/// * `element` - The element definition (used for type-specific suggestions)
///
/// # Returns
///
/// A vector of actionable suggestions for resolving the orphan warning
fn generate_orphan_suggestions(fully_qualified_name: &str, element: &ElementDef) -> Vec<String> {
    let mut suggestions = Vec::new();

    // Suggestion 1: Connect the element to the architecture
    suggestions.push(format!(
        "Add relations to connect '{}' to other elements in your architecture",
        fully_qualified_name
    ));

    // Suggestion 2: Remove if unused
    suggestions.push(
        "If this element is no longer needed, consider removing it from your architecture"
            .to_string(),
    );

    // Suggestion 3: Check for typos
    suggestions.push(
        "Check for typos in existing relations that might be intended to reference this element"
            .to_string(),
    );

    // Type-specific suggestions
    let _kind_str = element.assignment.kind.to_string().to_lowercase();
    match element.assignment.kind {
        sruja_language::ElementKind::Person => {
            suggestions.push(format!(
                "Person '{}' may be an actor for a scenario - consider adding scenario definitions",
                fully_qualified_name
            ));
        }
        sruja_language::ElementKind::Database => {
            suggestions.push(format!(
                "Database '{}' should be connected to at least one service or container",
                fully_qualified_name
            ));
        }
        _ => {
            suggestions.push(format!(
                "Ensure '{}' participates in your architecture's data flow or service interactions",
                fully_qualified_name
            ));
        }
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use crate::DomainSchema;
    use super::*;
    use sruja_diagnostics::SourceLocation;
    use sruja_language::{Parser, QualifiedIdent, Relation};

    /// Helper function to parse a program and run orphan detection.
    ///
    /// Returns the validation diagnostics, or an empty vector if parsing fails.
    fn validate_program(input: &str) -> Vec<Diagnostic> {
        let parser = Parser::new("test.sruja".to_string());
        let program = match parser.parse(input) {
            Ok(p) => p,
            Err(_) => return vec![],
        };

        let rule = OrphanDetectionRule;
        rule.validate(&program, &DomainSchema::architecture())
    }

    #[test]
    fn test_fully_connected_architecture() {
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
            "Expected no orphans in fully connected architecture, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_single_orphan_element() {
        let input = r#"
user = person "User"
web = system "Web App"
cache = container "Redis Cache"

user -> web "uses"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            sruja_diagnostics::codes::CODE_ORPHAN_ELEMENT
        );
        assert!(diagnostics[0].message.contains("cache"));
    }

    #[test]
    fn test_multiple_orphan_elements() {
        let input = r#"
user = person "User"
web = system "Web App"
cache = container "Redis Cache"
queue = container "Message Queue"

user -> web "uses"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 2);

        let orphan_names: Vec<_> = diagnostics
            .iter()
            .filter_map(|d| d.message.split('\'').nth(1))
            .collect();

        assert!(orphan_names.contains(&"cache"));
        assert!(orphan_names.contains(&"queue"));
    }

    #[test]
    fn test_nested_element_orphan() {
        let input = r#"
app = system "App" {
    api = container "API"
    worker = container "Background Worker"
    db = container "Database"
}

app.api -> app.db "queries"
"#;

        let diagnostics = validate_program(input);
        // Currently finds both 'app' (top-level) and 'app.worker' (nested) as orphans
        // The parent 'app' is unreferenced because only its children are referenced
        // In the future, we may want to exclude parent elements with referenced children
        assert_eq!(diagnostics.len(), 2);

        let orphan_names: Vec<_> = diagnostics
            .iter()
            .filter_map(|d| d.message.split('\'').nth(1))
            .collect();

        assert!(orphan_names.contains(&"app.worker"));
        assert!(orphan_names.contains(&"app"));
    }

    #[test]
    fn test_orphan_container() {
        let input = r#"
web = system "Web App"
db = container "Database"

web -> web "self-loop"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("db"));

        // Check for container-specific suggestion
        assert!(diagnostics[0]
            .suggestions
            .iter()
            .any(|s| s.contains("db") && s.contains("participates")));
    }

    #[test]
    fn test_orphan_person() {
        let input = r#"
user = person "User"
admin = person "Admin"
web = system "Web App"

user -> web "uses"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("admin"));

        // Check for person-specific suggestion
        assert!(diagnostics[0]
            .suggestions
            .iter()
            .any(|s| s.contains("scenario")));
    }

    #[test]
    fn test_no_orphan_when_both_ends_referenced() {
        let input = r#"
user = person "User"
web = system "Web App"

web -> user "notifies"
"#;

        let diagnostics = validate_program(input);
        assert!(
            diagnostics.is_empty(),
            "Both elements should be referenced (bidirectional relation)"
        );
    }

    #[test]
    fn test_empty_program() {
        let input = "";
        let diagnostics = validate_program(input);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_program_with_only_definitions() {
        let input = r#"
user = person "User"
web = system "Web App"
db = container "Database"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 3); // All elements are orphans
    }

    #[test]
    fn test_diagnostic_severity_is_warning() {
        let input = r#"
web = system "Web App"
cache = container "Cache"

web -> web "self"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_diagnostic_includes_suggestions() {
        let input = r#"
web = system "Web App"
cache = container "Cache"

web -> web "self"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert!(!diagnostics[0].suggestions.is_empty());
        assert!(diagnostics[0]
            .suggestions
            .iter()
            .any(|s| s.contains("relations")));
    }

    #[test]
    fn test_collect_referenced_elements_simple() {
        let relations = vec![
            create_test_relation("a", "b"),
            create_test_relation("b", "c"),
        ];

        let referenced = collect_referenced_elements(&relations);

        assert!(referenced.contains("a"));
        assert!(referenced.contains("b"));
        assert!(referenced.contains("c"));
        assert_eq!(referenced.len(), 3);
    }

    #[test]
    fn test_collect_referenced_elements_nested() {
        let relations = vec![create_test_relation("system.api", "system.db")];

        let referenced = collect_referenced_elements(&relations);

        // Should contain both FQNs and leaf IDs
        assert!(referenced.contains("system.api"));
        assert!(referenced.contains("system.db"));
        assert!(referenced.contains("api"));
        assert!(referenced.contains("db"));
    }

    #[test]
    fn test_element_is_referenced_exact_match() {
        let mut referenced = HashSet::new();
        referenced.insert("system.container".to_string());

        assert!(element_is_referenced("system.container", &referenced));
        assert!(!element_is_referenced("other", &referenced));
    }

    #[test]
    fn test_element_is_referenced_leaf_match() {
        let mut referenced = HashSet::new();
        referenced.insert("container".to_string());

        assert!(element_is_referenced("system.container", &referenced));
        assert!(element_is_referenced("container", &referenced));
        assert!(!element_is_referenced("system.other", &referenced));
    }

    #[test]
    fn test_element_is_referenced_no_match() {
        let referenced = HashSet::new();

        assert!(!element_is_referenced("anything", &referenced));
        assert!(!element_is_referenced("", &referenced));
    }

    // Helper function to create test relations
    fn create_test_relation(from: &str, to: &str) -> sruja_language::Relation {
        Relation {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            from: QualifiedIdent::simple(from.to_string()),
            to: QualifiedIdent::simple(to.to_string()),
            label: None,
            description: Some("test".to_string()),
            technology: None,
            tags: Vec::new(),
        }
    }
}
