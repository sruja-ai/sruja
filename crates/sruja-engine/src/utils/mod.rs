//! Utilities module for common operations across validation rules.
//!
//! This module consolidates frequently used helper functions to eliminate
//! code duplication and ensure consistent behavior across all validation rules.
//!
//! # Examples
//!
//! ```rust
//! use sruja_engine::utils::{find_element, ElementFinder};
//! use sruja_language::{ElementDef, Program};
//! use std::collections::HashMap;
//!
//! # fn example(elements: &HashMap<String, ElementDef>) {
//! // Using the standalone function
//! if let Some(elem) = find_element(elements, "my.system.container") {
//!     println!("Found: {}", elem.assignment.name);
//! }
//!
//! // Using the builder for more control
//! let finder = ElementFinder::new(elements)
//!     .with_fuzzy_match(true)
//!     .with_case_insensitive(false);
//!
//! if let Some(elem) = finder.find("container") {
//!     println!("Found container via fuzzy match");
//! }
//! # }
//! ```

use sruja_language::ElementDef;
use std::collections::HashMap;

/// Finds an element by exact fully qualified name (FQN) or by leaf ID suffix match.
///
/// This helper function performs a flexible lookup that first attempts an exact
/// match on the FQN, then falls back to a suffix match for nested elements.
///
/// # Behavior
///
/// 1. **Exact Match**: Returns element if `name` exactly matches a key in the map
/// 2. **Suffix Match**: If exact match fails, returns the first element whose
///    FQN ends with `.name` (e.g., searching for "container" finds "system.container")
///
/// # Arguments
///
/// * `elements` - HashMap of fully qualified names to element definitions
/// * `name` - The name to search for (can be FQN or leaf ID)
///
/// # Returns
///
/// * `Some(&ElementDef)` if a matching element is found
/// * `None` if no match is found
///
/// # Examples
///
/// ```rust
/// use sruja_engine::utils::find_element;
/// use sruja_language::{ElementDef, ElementAssignment, ElementKind};
/// use sruja_diagnostics::SourceLocation;
/// use std::collections::HashMap;
///
/// # fn setup() -> HashMap<String, ElementDef> {
/// #     let mut elements = HashMap::new();
/// #     let elem = ElementDef {
/// #         assignment: ElementAssignment::new("container", ElementKind::Container),
/// #         location: SourceLocation::new(String::new(), 0, 0),
/// #     };
/// #     elements.insert("system.container".to_string(), elem);
/// #     elements
/// # }
/// let elements = setup();
///
/// // Exact FQN match
/// assert!(find_element(&elements, "system.container").is_some());
///
/// // Suffix match for leaf ID
/// assert!(find_element(&elements, "container").is_some());
///
/// // No match
/// assert!(find_element(&elements, "nonexistent").is_none());
/// ```
#[inline]
pub fn find_element<'a>(
    elements: &'a HashMap<String, ElementDef>,
    name: &str,
) -> Option<&'a ElementDef> {
    if name.is_empty() {
        return None;
    }

    // Fast path: exact FQN match
    if let Some(elem) = elements.get(name) {
        return Some(elem);
    }

    // Fallback: suffix match for nested elements (e.g., "container" -> "system.container")
    let suffix = format!(".{}", name);
    elements
        .iter()
        .find(|(fqn, _)| fqn.as_str() == name || fqn.ends_with(&suffix))
        .map(|(_, elem)| elem)
}

/// Checks if an element exists in the elements map using flexible lookup.
///
/// This is a convenience wrapper around [`find_element`] that returns a boolean
/// instead of the element reference.
///
/// # Arguments
///
/// * `elements` - HashMap of fully qualified names to element definitions
/// * `name` - The name to check for existence
///
/// # Returns
///
/// * `true` if the element exists (exact FQN or suffix match)
/// * `false` otherwise
///
/// # Examples
///
/// ```rust
/// use sruja_engine::utils::element_exists;
/// use sruja_language::{ElementDef, ElementAssignment, ElementKind};
/// use sruja_diagnostics::SourceLocation;
/// use std::collections::HashMap;
///
/// # fn setup() -> HashMap<String, ElementDef> {
/// #     let mut elements = HashMap::new();
/// #     elements.insert("system.service".to_string(), ElementDef {
/// #         assignment: ElementAssignment::new("service", ElementKind::Component),
/// #         location: SourceLocation::new(String::new(), 0, 0),
/// #     });
/// #     elements
/// # }
/// let elements = setup();
///
/// assert!(element_exists(&elements, "system.service"));
/// assert!(element_exists(&elements, "service"));
/// assert!(!element_exists(&elements, "missing"));
/// ```
#[inline]
pub fn element_exists(elements: &HashMap<String, ElementDef>, name: &str) -> bool {
    find_element(elements, name).is_some()
}

/// Builder for flexible element lookups with configurable matching strategies.
///
/// Use `ElementFinder` when you need more control over element lookup behavior,
/// such as case-insensitive matching or custom match predicates.
///
/// # Examples
///
/// ```rust
/// use sruja_engine::utils::ElementFinder;
/// use sruja_language::{ElementDef, ElementAssignment, ElementKind};
/// use sruja_diagnostics::SourceLocation;
/// use std::collections::HashMap;
///
/// # fn setup() -> HashMap<String, ElementDef> {
/// #     let mut elements = HashMap::new();
/// #     elements.insert("System.Container".to_string(), ElementDef {
/// #         assignment: ElementAssignment::new("Container", ElementKind::Container),
/// #         location: SourceLocation::new(String::new(), 0, 0),
/// #     });
/// #     elements
/// # }
/// let elements = setup();
///
/// // Case-insensitive lookup
/// let finder = ElementFinder::new(&elements)
///     .with_case_insensitive(true);
///
/// assert!(finder.find("system.container").is_some());
/// ```
pub struct ElementFinder<'a> {
    elements: &'a HashMap<String, ElementDef>,
    fuzzy_match: bool,
    case_insensitive: bool,
}

impl<'a> ElementFinder<'a> {
    /// Creates a new ElementFinder with default matching strategies.
    ///
    /// # Arguments
    ///
    /// * `elements` - HashMap of fully qualified names to element definitions
    ///
    /// # Default Behavior
    ///
    /// - `fuzzy_match`: true (allows suffix matching)
    /// - `case_insensitive`: false (exact case matching)
    pub fn new(elements: &'a HashMap<String, ElementDef>) -> Self {
        Self {
            elements,
            fuzzy_match: true,
            case_insensitive: false,
        }
    }

    /// Enables or disables fuzzy matching (suffix matching for nested elements).
    ///
    /// When enabled (default), searching for "container" can match "system.container".
    /// When disabled, only exact FQN matches are considered.
    pub fn with_fuzzy_match(mut self, enabled: bool) -> Self {
        self.fuzzy_match = enabled;
        self
    }

    /// Enables or disables case-insensitive matching.
    ///
    /// When enabled, "Container" will match "container" and "CONTAINER".
    pub fn with_case_insensitive(mut self, enabled: bool) -> Self {
        self.case_insensitive = enabled;
        self
    }

    /// Finds an element using the configured matching strategies.
    ///
    /// # Returns
    ///
    /// * `Some(&ElementDef)` if a matching element is found
    /// * `None` if no match is found
    pub fn find(&self, name: &str) -> Option<&'a ElementDef> {
        if name.is_empty() {
            return None;
        }

        let search_name = if self.case_insensitive {
            name.to_lowercase()
        } else {
            name.to_string()
        };

        // Try exact match first
        for (fqn, elem) in self.elements.iter() {
            let compare_fqn = if self.case_insensitive {
                fqn.to_lowercase()
            } else {
                fqn.clone()
            };

            if compare_fqn == search_name {
                return Some(elem);
            }
        }

        // Try fuzzy match if enabled
        if self.fuzzy_match {
            let suffix = format!(".{}", search_name);
            for (fqn, elem) in self.elements.iter() {
                let compare_fqn = if self.case_insensitive {
                    fqn.to_lowercase()
                } else {
                    fqn.clone()
                };

                if compare_fqn.ends_with(&suffix) {
                    return Some(elem);
                }
            }
        }

        None
    }
}

/// Extracts tags from an element definition.
///
/// Tags can be specified in multiple ways:
/// 1. Via tag references on assignment (e.g., `@tag`)
/// 2. Via metadata: `metadata { tags "api,external" }` or `metadata { tag "api" }`
///
/// # Arguments
///
/// * `elem` - The element definition to extract tags from
///
/// # Returns
///
/// A vector of tag strings (without the `#` or `@` prefix)
///
/// # Examples
///
/// ```rust
/// use sruja_engine::utils::extract_tags;
/// use sruja_language::{ElementDef, ElementAssignment, ElementKind};
/// use sruja_diagnostics::SourceLocation;
///
/// # fn setup() -> ElementDef {
/// #     let mut assignment = ElementAssignment::new("service", ElementKind::Component);
/// #     assignment.tag_refs = vec!["#api".to_string(), "#external".to_string()];
/// #     ElementDef {
/// #         assignment,
/// #         location: SourceLocation::new(String::new(), 0, 0),
/// #     }
/// # }
/// let elem = setup();
/// let tags = extract_tags(&elem);
///
/// assert!(tags.contains(&"api".to_string()));
/// assert!(tags.contains(&"external".to_string()));
/// ```
pub fn extract_tags(elem: &ElementDef) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();

    // Extract from tag references (e.g., @tag or #tag)
    for tag_ref in &elem.assignment.tag_refs {
        let tag = tag_ref
            .trim_start_matches('@')
            .trim_start_matches('#')
            .to_string();
        if !tag.is_empty() {
            tags.push(tag);
        }
    }

    // Extract from metadata
    if let Some(body) = &elem.assignment.body {
        for metadata in &body.metadata {
            if metadata.key == "tags" || metadata.key == "tag" {
                if let Some(value) = &metadata.value {
                    let value = value.trim().trim_matches('"');
                    tags.extend(
                        value
                            .split(',')
                            .map(|t| t.trim().to_string())
                            .filter(|t| !t.is_empty()),
                    );
                }
            }
        }
    }

    // Deduplicate tags while preserving order
    let mut seen = std::collections::HashSet::new();
    tags.retain(|tag| seen.insert(tag.to_lowercase()));

    tags
}

/// Checks if an element has a specific tag.
///
/// Tag matching is case-insensitive.
///
/// # Arguments
///
/// * `elem` - The element definition to check
/// * `tag_name` - The tag name to search for (case-insensitive)
///
/// # Returns
///
/// `true` if the element has the specified tag, `false` otherwise
///
/// # Examples
///
/// ```rust
/// use sruja_engine::utils::has_tag;
/// use sruja_language::{ElementDef, ElementAssignment, ElementKind};
/// use sruja_diagnostics::SourceLocation;
///
/// # fn setup() -> ElementDef {
/// #     let mut assignment = ElementAssignment::new("service", ElementKind::Component);
/// #     assignment.tag_refs = vec!["#api".to_string()];
/// #     ElementDef { assignment, location: SourceLocation::new(String::new(), 0, 0) }
/// # }
/// let elem = setup();
///
/// assert!(has_tag(&elem, "api"));
/// assert!(has_tag(&elem, "API")); // Case-insensitive
/// assert!(!has_tag(&elem, "external"));
/// ```
#[inline]
pub fn has_tag(elem: &ElementDef, tag_name: &str) -> bool {
    let tag_name_lower = tag_name.to_lowercase();
    extract_tags(elem)
        .iter()
        .any(|tag| tag.to_lowercase() == tag_name_lower)
}

/// Resolves the layer for an element based on metadata or name heuristics.
///
/// Layer resolution follows this priority:
/// 1. Explicit metadata: `metadata { layer "web" }`
/// 2. Name heuristics: checks if the element name contains known layer keywords
///
/// # Arguments
///
/// * `elements` - HashMap of all elements in the program
/// * `element_name` - The name of the element to resolve
/// * `known_layers` - List of known layer names (ordered by preference)
///
/// # Returns
///
/// The resolved layer name, or an empty string if no layer could be determined
///
/// # Examples
///
/// ```rust
/// use sruja_engine::utils::resolve_layer;
/// use sruja_language::{ElementDef, ElementAssignment, ElementKind};
/// use sruja_diagnostics::SourceLocation;
/// use std::collections::HashMap;
///
/// # fn setup() -> HashMap<String, ElementDef> {
/// #     let mut elements = HashMap::new();
/// #     elements.insert("web.server".to_string(), ElementDef {
/// #         assignment: ElementAssignment::new("server", ElementKind::Container),
/// #         location: SourceLocation::new(String::new(), 0, 0),
/// #     });
/// #     elements
/// # }
/// let elements = setup();
/// let layers = ["web", "api", "service", "data", "database"];
///
/// // Should match "web" layer from name heuristic
/// let layer = resolve_layer(&elements, "web.server", &layers);
/// assert_eq!(layer, "web");
/// ```
pub fn resolve_layer(
    elements: &HashMap<String, ElementDef>,
    element_name: &str,
    known_layers: &[&str],
) -> String {
    // Try explicit metadata first
    if let Some(elem) = find_element(elements, element_name) {
        if let Some(body) = &elem.assignment.body {
            for metadata in &body.metadata {
                if metadata.key == "layer" {
                    if let Some(value) = &metadata.value {
                        return value.trim().trim_matches('"').to_lowercase();
                    }
                }
            }
        }
    }

    // Fall back to name heuristics
    let name_lower = element_name.to_lowercase();
    for layer in known_layers {
        if name_lower.contains(layer) {
            return (*layer).to_string();
        }
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_diagnostics::SourceLocation;
    use sruja_language::{ElementAssignment, ElementDefBody, ElementKind, MetaEntry};

    /// Helper function to create a test element
    fn create_test_element(name: &str, kind: ElementKind) -> ElementDef {
        ElementDef {
            assignment: ElementAssignment {
                location: SourceLocation::new("test.sruja".to_string(), 1, 1),
                name: name.to_string(),
                kind,
                sub_kind: None,
                title: None,
                tag_refs: Vec::new(),
                body: None,
            },
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        }
    }

    fn create_nested_elements() -> HashMap<String, ElementDef> {
        let mut elements = HashMap::new();
        elements.insert(
            "system".to_string(),
            create_test_element("system", ElementKind::System),
        );
        elements.insert(
            "system.service".to_string(),
            create_test_element("service", ElementKind::System),
        );
        elements.insert(
            "system.service.container".to_string(),
            create_test_element("container", ElementKind::Container),
        );
        elements.insert(
            "database".to_string(),
            create_test_element("database", ElementKind::Database),
        );
        elements
    }

    #[test]
    fn test_find_element_exact_match() {
        let elements = create_nested_elements();

        assert!(find_element(&elements, "system").is_some());
        assert!(find_element(&elements, "system.service").is_some());
        assert!(find_element(&elements, "system.service.container").is_some());
    }

    #[test]
    fn test_find_element_suffix_match() {
        let elements = create_nested_elements();

        assert!(find_element(&elements, "service").is_some());
        assert!(find_element(&elements, "container").is_some());
        assert!(find_element(&elements, "database").is_some());
    }

    #[test]
    fn test_find_element_not_found() {
        let elements = create_nested_elements();

        assert!(find_element(&elements, "").is_none());
        assert!(find_element(&elements, "nonexistent").is_none());
        assert!(find_element(&elements, "system.nonexistent").is_none());
    }

    #[test]
    fn test_element_exists() {
        let elements = create_nested_elements();

        assert!(element_exists(&elements, "system"));
        assert!(element_exists(&elements, "service"));
        assert!(!element_exists(&elements, "missing"));
    }

    #[test]
    fn test_element_finder_exact_match() {
        let elements = create_nested_elements();
        let finder = ElementFinder::new(&elements).with_fuzzy_match(false);

        assert!(finder.find("system").is_some());
        assert!(finder.find("system.service").is_some());
        assert!(finder.find("service").is_none()); // No fuzzy match
    }

    #[test]
    fn test_element_finder_fuzzy_match() {
        let elements = create_nested_elements();
        let finder = ElementFinder::new(&elements).with_fuzzy_match(true);

        assert!(finder.find("system").is_some());
        assert!(finder.find("service").is_some());
        assert!(finder.find("container").is_some());
    }

    #[test]
    fn test_element_finder_case_insensitive() {
        let mut elements = HashMap::new();
        elements.insert(
            "System.Service".to_string(),
            create_test_element("Service", ElementKind::System),
        );

        let finder = ElementFinder::new(&elements).with_case_insensitive(true);

        assert!(finder.find("system.service").is_some());
        assert!(finder.find("SERVICE").is_some());
    }

    #[test]
    fn test_extract_tags_from_refs() {
        let assignment = ElementAssignment {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            name: "service".to_string(),
            kind: ElementKind::System,
            sub_kind: None,
            title: None,
            tag_refs: vec!["#api".to_string(), "@external".to_string(), "#".to_string()],
            body: None,
        };

        let elem = ElementDef {
            assignment,
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        };

        let tags = extract_tags(&elem);
        assert!(tags.contains(&"api".to_string()));
        assert!(tags.contains(&"external".to_string()));
        assert!(!tags.contains(&"".to_string()));
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_extract_tags_from_metadata() {
        let assignment = ElementAssignment {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            name: "service".to_string(),
            kind: ElementKind::System,
            sub_kind: None,
            title: None,
            tag_refs: Vec::new(),
            body: Some(ElementDefBody {
                metadata: vec![MetaEntry {
                    key: "tags".to_string(),
                    value: Some("api,external,pub".to_string()),
                }],

                ..Default::default()
            }),
        };

        let elem = ElementDef {
            assignment,
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        };

        let tags = extract_tags(&elem);
        assert!(tags.contains(&"api".to_string()));
        assert!(tags.contains(&"external".to_string()));
        assert!(tags.contains(&"pub".to_string()));
    }

    #[test]
    fn test_has_tag() {
        let assignment = ElementAssignment {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            name: "service".to_string(),
            kind: ElementKind::System,
            sub_kind: None,
            title: None,
            tag_refs: vec!["#api".to_string()],
            body: None,
        };

        let elem = ElementDef {
            assignment,
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        };

        assert!(has_tag(&elem, "api"));
        assert!(has_tag(&elem, "API"));
        assert!(!has_tag(&elem, "external"));
    }

    #[test]
    fn test_resolve_layer_from_metadata() {
        let mut elements = HashMap::new();
        let assignment = ElementAssignment {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            name: "myserver".to_string(),
            kind: ElementKind::Container,
            sub_kind: None,
            title: None,
            tag_refs: Vec::new(),
            body: Some(ElementDefBody {
                metadata: vec![MetaEntry {
                    key: "layer".to_string(),
                    value: Some("web".to_string()),
                }],
                ..Default::default()
            }),
        };

        elements.insert(
            "myserver".to_string(),
            ElementDef {
                assignment,
                location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            },
        );

        let layers = ["web", "api", "service", "data", "database"];
        assert_eq!(resolve_layer(&elements, "myserver", &layers), "web");
    }

    #[test]
    fn test_resolve_layer_from_name() {
        let elements = create_nested_elements();
        let layers = ["web", "api", "service", "data", "database"];

        assert_eq!(resolve_layer(&elements, "web.server", &layers), "web");
        assert_eq!(resolve_layer(&elements, "api.gateway", &layers), "api");
        assert_eq!(
            resolve_layer(&elements, "system.service", &layers),
            "service"
        );
    }
}
