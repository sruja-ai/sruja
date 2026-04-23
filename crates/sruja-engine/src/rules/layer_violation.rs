//! Layer violation validation rule
//!
//! Enforces strict layering principles in software architecture, preventing
//! dependencies that violate the designed architectural hierarchy.
//!
//! # Understanding Layer Violations
//!
//! A **layer violation** occurs when a higher-level layer depends on a lower-level
//! layer, breaking the intended unidirectional dependency flow. Proper layered
//! architectures enforce that dependencies flow "downwards" (from higher to lower
//! abstraction levels).
//!
//! # Standard Layer Hierarchy
//!
//! The rule enforces the following layer order (top to bottom):
//!
//! ```text
//! ┌─────────────┐
//! │   Web       │  ← Top: User-facing interfaces
//! ├─────────────┤
//! │   API       │  ← Service boundaries
//! ├─────────────┤
//! │   Service   │  ← Business logic
//! ├─────────────┤
//! │   Data      │  ← Data access layer
//! ├─────────────┤
//! │  Database   │  ← Bottom: Data storage
//! └─────────────┘
//!
//! Direction: ↑ Dependencies flow UP (lower layers depend on higher layers)
//!            ↑ Violations: Higher layers depending on lower layers
//! ```
//!
//! # Allowed Dependency Flows
//!
//! ✅ **Valid flows** (dependencies flow from higher to lower indices):
//! - `web -> api` (index 0 → 1): Web can depend on API
//! - `api -> service` (index 1 → 2): API can depend on Service
//! - `service -> data` (index 2 → 3): Service can depend on Data
//! - `data -> database` (index 3 → 4): Data can depend on Database
//! - `web -> database` (index 0 → 4): Web can directly depend on Database
//!
//! ❌ **Invalid flows** (dependencies flow from lower to higher indices):
//! - `service -> api` (index 2 → 1): Violation
//! - `database -> data` (index 4 → 3): Violation
//! - `api -> web` (index 1 → 0): Violation
//!
//! # Layer Resolution
//!
//! Layers are determined by:
//!
//! 1. **Explicit metadata** (highest priority):
//!    ```sruja
//!    myComponent = container "My Component" {
//!        metadata { layer "api" }
//!    }
//!    ```
//!
//! 2. **Name heuristics** (fallback):
//!    The element name is checked for known layer keywords:
//!    - Contains "web" → layer "web"
//!    - Contains "api" or "gateway" → layer "api"
//!    - Contains "service" → layer "service"
//!    - Contains "data" or "repository" → layer "data"
//!    - Contains "database" or "db" → layer "database"
//!
//! # Examples
//!
//! ## Valid Architecture
//!
//! ```sruja
//! // Explicit layers
//! frontend = container "Frontend" {
//!     metadata { layer "web" }
//! }
//! backend = container "Backend" {
//!     metadata { layer "service" }
//! }
//!
//! // Valid: web (0) → service (2)
//! frontend -> backend "calls"
//! ```
//!
//! ## Layer Violation (Error)
//!
//! ```sruja
//! // Name heuristics: contains "service"
//! coreService = container "Core Service"
//!
//! // Name heuristics: contains "web"
//! webInterface = container "Web Interface"
//!
//! // Invalid: service (2) → web (0)
//! // Error: Layer violation - service cannot depend on web
//! coreService -> webInterface "updates"
//! ```
//!
//! This produces:
//! ```text
//! [E206] Error: Layer violation: 'coreService' (service) cannot depend on 'webInterface' (web).
//! Dependencies must flow downwards (higher layers can only depend on lower layers).
//!   --> example.sruja:8:1
//!
//!   = Help: Reverse the dependency: 'webInterface -> coreService'
//!          Or restructure to follow proper layering (e.g., Web -> API -> Data)
//!          If this is intentional, consider documenting the exception
//! ```
//!
//! # When to Disable This Rule
//!
//! Layer violations may be intentional in certain cases:
//! - **Event-driven architectures**: Services may push events to web clients via websockets
//! - **Callback patterns**: Lower layers may invoke callbacks in higher layers
//! - **Pub/sub systems**: Publishers and subscribers can be at any layer
//! - **Legacy code**: Gradual refactoring may temporarily violate layering
//!
//! For intentional violations, consider:
//! 1. Documenting the rationale in the architecture
//! 2. Adding a metadata tag: `metadata { layerViolation "intentional" }`
//! 3. Using an anti-corruption layer to isolate the violation
//!
//! # Implementation Details
//!
//! ## Algorithm
//!
//! 1. Define the canonical layer ordering (web → api → service → data → database)
//! 2. For each relation, resolve the layer for both source and target elements
//! 3. Check that the source layer index is <= target layer index
//! 4. Generate an error if the dependency flows upward (violates layering)
//!
//! ## Performance
//!
//! - **Time Complexity**: O(m) where m = number of relations
//! - **Space Complexity**: O(1) (constant layer mapping)
//! - Uses efficient string operations and HashMap lookups
//!
//! ## Limitations
//!
//! - Does not support custom layer hierarchies (currently hard-coded)
//! - May flag intentional architectural patterns as violations
//! - Does not analyze transitive dependencies (only direct relations)
//!
//! # Future Enhancements
//!
//! - Support for custom layer definitions in metadata
//! - Configurable layer hierarchies per project
//! - Allow-list for intentional violations
//! - Detection of circular dependencies across layers
//! - Visualization of dependency graph with layer coloring

use crate::DomainSchema;
use std::collections::HashMap;

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{collect_elements, ElementDef, Program};

use crate::utils::resolve_layer;
use crate::validator::Rule;

/// Canonical layer hierarchy from highest (top) to lowest (bottom) abstraction.
///
/// Lower indices represent higher layers that should only depend on layers
/// with higher indices (lower in the hierarchy).
///
/// Index mapping:
/// - 0: Web (highest abstraction, user-facing)
/// - 1: API (service boundaries, contracts)
/// - 2: Service (business logic)
/// - 3: Data (data access, repositories)
/// - 4: Database (lowest abstraction, storage)
const LAYER_HIERARCHY: [&str; 5] = ["web", "api", "service", "data", "database"];

/// Rule that detects violations of strict layering principles.
///
/// This rule enforces that dependencies flow downward in the layer hierarchy,
/// preventing higher-level layers from depending on lower-level layers.
///
/// # Severity
///
/// This rule generates **Error** level diagnostics because layer violations
/// typically represent architectural mistakes that should be fixed.
///
/// # Configuration
///
/// Currently, this rule uses a fixed layer hierarchy. Future versions may support:
/// - Custom layer definitions via metadata
/// - Configurable layer orderings
/// - Exceptions for specific patterns (event-driven, callbacks)
pub struct LayerViolationRule;

impl Rule for LayerViolationRule {
    /// Returns the human-readable name of this validation rule.
    ///
    /// # Returns
    ///
    /// `"Layer Violation"`
    fn name(&self) -> &str {
        "Layer Violation"
    }

    /// Validates a program and returns diagnostics for layer violations.
    ///
    /// # Algorithm
    ///
    /// 1. Build a layer index map for O(1) layer lookups
    /// 2. Collect all elements and relations from the program
    /// 3. For each relation:
    ///    - Resolve the layer for the source element
    ///    - Resolve the layer for the target element
    ///    - Check if source_layer_index > target_layer_index (violation)
    ///    - Generate an error diagnostic if violation detected
    ///
    /// # Arguments
    ///
    /// * `program` - The architecture program to validate
    ///
    /// # Returns
    ///
    /// A vector of error diagnostics, one for each layer violation found
    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        // Build an efficient lookup map for layer indices
        // This transforms the layer hierarchy into O(1) lookup structure
        let layer_index_map = build_layer_index_map();

        // Collect all elements and relations from the program
        let (elements, relations) = collect_elements(program);

        // Validate each relation for layer violations
        let mut diagnostics = Vec::new();
        for relation in &relations {
            if let Some(diagnostic) =
                check_relation_for_violation(relation, &elements, &layer_index_map)
            {
                diagnostics.push(diagnostic);
            }
        }

        diagnostics
    }
}

/// Builds a HashMap mapping layer names to their hierarchical indices.
///
/// This function creates an efficient lookup structure where each layer name
/// maps to its position in the hierarchy. This enables O(1) index lookups
/// when checking layer violations.
///
/// # Returns
///
/// A HashMap where:
/// - Key: Layer name (e.g., "web", "api", "service")
/// - Value: Layer index (lower = higher in hierarchy)
///
/// # Example
///
/// Layer order is fixed: web (0), api (1), service (2), data (3), database (4).
/// The map is built internally; this function is not part of the public API.
fn build_layer_index_map() -> HashMap<String, usize> {
    LAYER_HIERARCHY
        .iter()
        .enumerate()
        .map(|(index, &layer_name)| (layer_name.to_string(), index))
        .collect()
}

/// Checks a single relation for layer violations.
///
/// This function validates that a relation does not violate layering principles
/// by ensuring the source element's layer is not "higher" (lower index) than
/// the target element's layer.
///
/// # Validation Logic
///
/// A relation `A -> B` is valid if `layer_index(A) <= layer_index(B)`.
/// This means dependencies flow from higher layers (lower indices) to lower
/// layers (higher indices).
///
/// # Arguments
///
/// * `relation` - The relation to validate
/// * `elements` - All elements in the program (for layer resolution)
/// * `layer_index_map` - Map of layer names to their hierarchical indices
///
/// # Returns
///
/// - `Some(Diagnostic)` if a layer violation is detected
/// - `None` if the relation respects layering principles
fn check_relation_for_violation(
    relation: &sruja_language::Relation,
    elements: &HashMap<String, ElementDef>,
    layer_index_map: &HashMap<String, usize>,
) -> Option<Diagnostic> {
    // Resolve layers for both ends of the relation
    let source_name = relation.from.as_string();
    let target_name = relation.to.as_string();

    let source_layer = resolve_layer(elements, &source_name, &LAYER_HIERARCHY);
    let target_layer = resolve_layer(elements, &target_name, &LAYER_HIERARCHY);

    // Skip validation if either layer couldn't be resolved
    // This can happen for elements with non-standard names or custom layers
    if source_layer.is_empty() || target_layer.is_empty() {
        return None;
    }

    // Get layer indices for comparison
    let source_index = match layer_index_map.get(&source_layer) {
        Some(&idx) => idx,
        None => return None, // Unknown layer, skip validation
    };

    let target_index = match layer_index_map.get(&target_layer) {
        Some(&idx) => idx,
        None => return None, // Unknown layer, skip validation
    };

    // Check for layer violation
    // Violation occurs when source is "higher" (lower index) than target
    if source_index > target_index {
        Some(create_layer_violation_diagnostic(
            &source_name,
            &source_layer,
            source_index,
            &target_name,
            &target_layer,
            target_index,
            relation,
        ))
    } else {
        None // No violation
    }
}

/// Creates a diagnostic for a layer violation.
///
/// This function generates a comprehensive error message that includes:
/// - Clear statement of the violation
/// - Layer information for both elements
/// - The problematic relation
/// - Actionable suggestions for resolution
///
/// # Diagnostic Structure
///
/// The diagnostic includes:
/// - **Message**: Explains which layers are involved and why it's a violation
/// - **Context**: Shows the relation that caused the violation
/// - **Suggestions**: Multiple options for fixing the violation
///
/// # Suggestions Provided
///
/// 1. **Reverse the dependency**: Change direction of the relation
/// 2. **Restructure architecture**: Introduce intermediate layers
/// 3. **Document the exception**: If intentional, document the rationale
/// 4. **Rename elements**: If layer detection was wrong due to naming
///
/// # Arguments
///
/// * `source_name` - Name of the source element
/// * `source_layer` - Resolved layer of the source element
/// * `source_index` - Hierarchical index of the source layer
/// * `target_name` - Name of the target element
/// * `target_layer` - Resolved layer of the target element
/// * `target_index` - Hierarchical index of the target layer
/// * `relation` - The relation that violated layering
///
/// # Returns
///
/// An error diagnostic with full context and actionable suggestions
fn create_layer_violation_diagnostic(
    source_name: &str,
    source_layer: &str,
    source_index: usize,
    target_name: &str,
    target_layer: &str,
    target_index: usize,
    relation: &sruja_language::Relation,
) -> Diagnostic {
    // Construct a clear, descriptive message
    let message = format!(
        "Layer violation: '{}' ({}) cannot depend on '{}' ({}). \
        Dependencies must flow downwards (higher layers can only depend on lower layers).",
        source_name, source_layer, target_name, target_layer
    );

    // Add context showing the layer indices
    let context = vec![
        format!("Layer hierarchy: {}", LAYER_HIERARCHY.join(" -> ")),
        format!("Source layer index: {} (higher layer)", source_index),
        format!("Target layer index: {} (lower layer)", target_index),
        format!(
            "Violation: {} (index {}) -> {} (index {})",
            source_name, source_index, target_name, target_index
        ),
    ];

    // Generate actionable suggestions
    let suggestions = vec![
        format!("Reverse the dependency: '{} -> {}'", target_name, source_name),
        format!(
            "Introduce an intermediate layer to mediate the interaction (e.g., use events or callbacks)"
        ),
        "Restructure to follow proper layering (e.g., Web -> API -> Service -> Data -> Database)".to_string(),
        "If this is intentional, document the rationale in architecture documentation".to_string(),
        "Check if element names correctly reflect their layer (consider renaming)".to_string(),
    ];

    Diagnostic::new(
        sruja_diagnostics::codes::CODE_LAYER_VIOLATION,
        Severity::Error,
        message,
        relation.location.clone(),
    )
    .with_context(context)
    .with_suggestions(suggestions)
}

#[cfg(test)]
mod tests {
    use crate::DomainSchema;
    use super::*;
    use sruja_language::Parser;

    /// Helper function to parse a program and run layer violation detection.
    ///
    /// Returns the validation diagnostics, or an empty vector if parsing fails.
    fn validate_program(input: &str) -> Vec<Diagnostic> {
        let parser = Parser::new("test.sruja".to_string());
        let program = match parser.parse(input) {
            Ok(p) => p,
            Err(_) => return vec![],
        };

        let rule = LayerViolationRule;
        rule.validate(&program, &DomainSchema::architecture())
    }

    #[test]
    fn test_valid_downward_dependencies() {
        let input = r#"
web = container "Web App"
api = container "API Gateway"
service = container "Core Service"
data = container "Data Access"
db = container "Database"

// All dependencies flow downward: valid
web -> api "calls"
api -> service "proxies"
service -> data "queries"
data -> db "persists"
"#;

        let diagnostics = validate_program(input);
        assert!(
            diagnostics.is_empty(),
            "Expected no violations for valid downward dependencies, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_single_layer_violation() {
        let input = r#"
web = container "Web App"
service = container "Core Service"

// Violation: service (index 2) -> web (index 0)
service -> web "updates"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            sruja_diagnostics::codes::CODE_LAYER_VIOLATION
        );
        assert!(diagnostics[0].message.contains("service"));
        assert!(diagnostics[0].message.contains("web"));
    }

    #[test]
    fn test_skip_direct_dependencies() {
        let input = r#"
web = container "Web App"
db = container "Database"

// Valid: can skip intermediate layers (web -> db is allowed)
web -> db "directly accesses"
"#;

        let diagnostics = validate_program(input);
        assert!(
            diagnostics.is_empty(),
            "Skipping intermediate layers should be allowed"
        );
    }

    #[test]
    fn test_explicit_layer_metadata() {
        let input = r#"
frontend = container "Frontend" {
    metadata { layer "web" }
}
backend = container "Backend" {
    metadata { layer "api" }
}

// Valid: web (0) -> api (1)
frontend -> backend "calls"
"#;

        let diagnostics = validate_program(input);
        assert!(
            diagnostics.is_empty(),
            "Explicit metadata should override name heuristics"
        );
    }

    #[test]
    fn test_name_heuristic_layer_detection() {
        let input = r#"
userInterface = container "User Interface"  // Contains "web"
apiGateway = container "API Gateway"        // Contains "api"

// Valid: web -> api
userInterface -> apiGateway "uses"
"#;

        let diagnostics = validate_program(input);
        assert!(
            diagnostics.is_empty(),
            "Name heuristics should correctly detect layers"
        );
    }

    #[test]
    fn test_violation_with_name_heuristics() {
        let input = r#"
coreService = container "Core Service"  // Detected as "service"
webFrontend = container "Web Frontend" // Detected as "web"

// Violation: service (2) -> web (0)
coreService -> webFrontend "pushes updates"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("service"));
        assert!(diagnostics[0].message.contains("web"));
    }

    #[test]
    fn test_multiple_layer_violations() {
        let input = r#"
myDatabase = container "Database"
myService = container "Service"
myWeb = container "Web"

// Multiple violations
myService -> myWeb "violation 1"
myDatabase -> myService "violation 2"
myDatabase -> myWeb "violation 3"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 3);
        assert!(diagnostics
            .iter()
            .all(|d| d.code == sruja_diagnostics::codes::CODE_LAYER_VIOLATION));
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
web = container "Web"
api = container "API"
"#;

        let diagnostics = validate_program(input);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_self_loops_are_valid() {
        let input = r#"
web = container "Web App"

// Self-loops don't violate layering (same layer)
web -> web "internal call"
"#;

        let diagnostics = validate_program(input);
        assert!(
            diagnostics.is_empty(),
            "Self-loops within the same layer should be valid"
        );
    }

    #[test]
    fn test_elements_with_unrecognized_layers() {
        let input = r#"
unknown1 = container "Component A"
unknown2 = container "Component B"

// Both have unrecognized layers, should be skipped
unknown1 -> unknown2 "relation"
"#;

        let diagnostics = validate_program(input);
        assert!(
            diagnostics.is_empty(),
            "Relations with unrecognized layers should be skipped"
        );
    }

    #[test]
    fn test_diagnostic_severity_is_error() {
        let input = r#"
service = container "Service"
web = container "Web"
service -> web "violation"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_diagnostic_includes_helpful_suggestions() {
        let input = r#"
service = container "Service"
web = container "Web"
service -> web "violation"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert!(!diagnostics[0].suggestions.is_empty());

        // Check for specific suggestion types
        let suggestions = &diagnostics[0].suggestions;
        assert!(suggestions
            .iter()
            .any(|s| s.contains("Reverse the dependency")));
        assert!(suggestions.iter().any(|s| s.contains("intermediate layer")));
        assert!(suggestions.iter().any(|s| s.contains("document")));
    }

    #[test]
    fn test_diagnostic_includes_context() {
        let input = r#"
service = container "Service"
web = container "Web"
service -> web "violation"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert!(!diagnostics[0].context.is_empty());

        // Check that context includes layer hierarchy
        let context = &diagnostics[0].context;
        assert!(context.iter().any(|c| c.contains("Layer hierarchy")));
        assert!(context.iter().any(|c| c.contains("Source layer index")));
        assert!(context.iter().any(|c| c.contains("Target layer index")));
    }

    #[test]
    fn test_build_layer_index_map() {
        let map = build_layer_index_map();

        assert_eq!(map.get("web"), Some(&0));
        assert_eq!(map.get("api"), Some(&1));
        assert_eq!(map.get("service"), Some(&2));
        assert_eq!(map.get("data"), Some(&3));
        assert_eq!(map.get("database"), Some(&4));
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn test_all_valid_layer_combinations() {
        let layers = ["web", "api", "service", "data", "database"];

        for (i, &from_layer) in layers.iter().enumerate() {
            for (j, &to_layer) in layers.iter().enumerate() {
                let input = format!(
                    r#"
{} = container "From" {{ metadata {{ layer "{}" }} }}
{} = container "To" {{ metadata {{ layer "{}" }} }}
{} -> {} "test"
"#,
                    from_layer, from_layer, to_layer, to_layer, from_layer, to_layer
                );

                let diagnostics = validate_program(&input);

                // Should only have violation if from > to (higher index depending on lower index)
                if i > j {
                    assert_eq!(
                        diagnostics.len(),
                        1,
                        "Expected violation: {} (index {}) -> {} (index {})",
                        from_layer,
                        i,
                        to_layer,
                        j
                    );
                } else {
                    assert!(
                        diagnostics.is_empty(),
                        "Expected no violation: {} (index {}) -> {} (index {})",
                        from_layer,
                        i,
                        to_layer,
                        j
                    );
                }
            }
        }
    }

    #[test]
    fn test_complex_nested_architecture() {
        let input = r#"
app = system "Application" {
    frontend = container "Frontend UI" {
        metadata { layer "web" }
    }
    gateway = container "API Gateway" {
        metadata { layer "api" }
    }
    auth = container "Auth Service" {
        metadata { layer "service" }
    }
    users = container "User Repository" {
        metadata { layer "data" }
    }
    postgres = container "PostgreSQL" {
        metadata { layer "database" }
    }
}

// Valid downward dependencies
app.frontend -> app.gateway "uses"
app.gateway -> app.auth "proxies"
app.auth -> app.users "queries"
app.users -> app.postgres "persists"

// Violation: service (2) -> web (0)
app.auth -> app.frontend "pushes updates"
"#;

        let diagnostics = validate_program(input);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("auth"));
        assert!(diagnostics[0].message.contains("frontend"));
    }
}
