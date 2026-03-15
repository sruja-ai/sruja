use sruja_language::{ElementDef, Program, Relation, TopLevelItem, ViewDef};

use std::collections::{HashMap, HashSet};

/// Result of resolving a view definition to concrete elements and relations
#[derive(Debug, Clone)]
pub struct ResolvedView {
    pub view_id: String,
    pub title: String,
    pub description: Option<String>,
    pub elements: HashMap<String, ElementDef>,
    pub relations: Vec<Relation>,
}

/// Resolve a single view definition to concrete elements and relations
pub fn resolve_view(
    view: &ViewDef,
    all_elements: &HashMap<String, ElementDef>,
    all_relations: &[Relation],
) -> ResolvedView {
    let mut visible_elements = HashSet::new();

    let first_rule = view.rules.first();

    if let Some(include) = first_rule.and_then(|r| r.include.as_ref()) {
        visible_elements.extend(apply_include_rule(
            include,
            all_elements,
            view.view_of.as_ref(),
        ));
    } else if let Some(scope) = &view.view_of {
        let scope_fqn = scope.as_string();
        visible_elements.insert(scope_fqn.clone());
        visible_elements.extend(get_descendants(&scope_fqn, all_elements));
    } else {
        visible_elements.extend(all_elements.keys().cloned());
    }

    if let Some(exclude) = first_rule.and_then(|r| r.exclude.as_ref()) {
        let excluded = apply_exclude_rule(exclude, all_elements, view.view_of.as_ref());
        for elem_id in excluded {
            visible_elements.remove(&elem_id);
        }
    }

    let filtered_relations: Vec<Relation> = all_relations
        .iter()
        .filter(|rel| {
            let from = rel.from.as_string();
            let to = rel.to.as_string();
            visible_elements.contains(&from) || visible_elements.contains(&to)
        })
        .cloned()
        .collect();

    for elem_id in visible_elements.clone() {
        add_parent_if_needed(&elem_id, all_elements, &mut visible_elements);
    }

    let elements: HashMap<String, ElementDef> = visible_elements
        .into_iter()
        .filter_map(|id| all_elements.get(&id).map(|e| (id.clone(), e.clone())))
        .collect();

    ResolvedView {
        view_id: view.id.clone(),
        title: view.title.clone(),
        description: view.description.clone(),
        elements,
        relations: filtered_relations,
    }
}

fn apply_include_rule(
    expr: &sruja_language::ViewRuleExpr,
    all_elements: &HashMap<String, ElementDef>,
    scope: Option<&sruja_language::QualifiedIdent>,
) -> HashSet<String> {
    let mut result = HashSet::new();

    if expr.wildcard {
        if let Some(scope_q) = scope {
            let scope_fqn = scope_q.as_string();
            result.insert(scope_fqn.clone());
            result.extend(get_descendants(&scope_fqn, all_elements));
        } else {
            result.extend(all_elements.keys().cloned());
        }
    } else {
        for pattern in &expr.elements {
            result.extend(match_pattern(pattern, all_elements));
        }
    }

    result
}

fn apply_exclude_rule(
    expr: &sruja_language::ViewRuleExpr,
    all_elements: &HashMap<String, ElementDef>,
    scope: Option<&sruja_language::QualifiedIdent>,
) -> HashSet<String> {
    apply_include_rule(expr, all_elements, scope)
}

fn match_pattern(pattern: &str, all_elements: &HashMap<String, ElementDef>) -> HashSet<String> {
    let mut result = HashSet::new();

    if let Some(prefix) = pattern.strip_suffix(".*") {
        for fqn in all_elements.keys() {
            if fqn == prefix || fqn.starts_with(&format!("{}.", prefix)) {
                result.insert(fqn.clone());
            }
        }
    } else if all_elements.contains_key(pattern) {
        result.insert(pattern.to_string());
    }

    result
}

fn get_descendants(fqn: &str, all_elements: &HashMap<String, ElementDef>) -> HashSet<String> {
    let mut result = HashSet::new();
    let prefix = format!("{}.", fqn);

    for elem_fqn in all_elements.keys() {
        if elem_fqn == fqn || elem_fqn.starts_with(&prefix) {
            result.insert(elem_fqn.clone());
        }
    }

    result
}

fn add_parent_if_needed(
    elem_id: &str,
    all_elements: &HashMap<String, ElementDef>,
    visible: &mut HashSet<String>,
) {
    if let Some(dot_idx) = elem_id.rfind('.') {
        let parent_id = &elem_id[..dot_idx];
        if all_elements.contains_key(parent_id) {
            visible.insert(parent_id.to_string());
            add_parent_if_needed(parent_id, all_elements, visible);
        }
    }
}

pub fn collect_views(program: &Program) -> Vec<ViewDef> {
    program
        .items
        .iter()
        .filter_map(|item| {
            if let TopLevelItem::View(view) = item {
                Some(view.clone())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mermaid::exporter::{MermaidConfig, MermaidExporter};
    use sruja_diagnostics::SourceLocation;
    use sruja_language::{ElementAssignment, ElementKind, QualifiedIdent};

    fn create_test_element(name: &str, kind: ElementKind) -> ElementDef {
        ElementDef {
            location: SourceLocation::new(String::new(), 0, 0),
            assignment: ElementAssignment::new(name, kind),
        }
    }

    #[test]
    fn test_wildcard_include() {
        let mut elements = HashMap::new();
        elements.insert(
            "Shop".to_string(),
            create_test_element("Shop", ElementKind::System),
        );
        elements.insert(
            "Shop.WebApp".to_string(),
            create_test_element("Shop.WebApp", ElementKind::Container),
        );
        elements.insert(
            "Shop.API".to_string(),
            create_test_element("Shop.API", ElementKind::Container),
        );

        let view = ViewDef {
            location: SourceLocation::new(String::new(), 0, 0),
            id: "test".to_string(),
            title: String::new(),
            description: None,
            view_of: None,
            tags: vec![],
            rules: vec![sruja_language::ViewRule {
                include: Some(sruja_language::ViewRuleExpr {
                    wildcard: true,
                    recursive: false,
                    elements: vec![],
                }),
                exclude: None,
            }],
        };

        let resolved = resolve_view(&view, &elements, &[]);
        assert_eq!(resolved.elements.len(), 3);
    }

    #[test]
    fn test_descendant_pattern() {
        let mut elements = HashMap::new();
        elements.insert(
            "Shop".to_string(),
            create_test_element("Shop", ElementKind::System),
        );
        elements.insert(
            "Shop.WebApp".to_string(),
            create_test_element("Shop.WebApp", ElementKind::Container),
        );
        elements.insert(
            "Shop.API".to_string(),
            create_test_element("Shop.API", ElementKind::Container),
        );

        let view = ViewDef {
            location: SourceLocation::new(String::new(), 0, 0),
            id: "test".to_string(),
            title: String::new(),
            description: None,
            view_of: None,
            tags: vec![],
            rules: vec![sruja_language::ViewRule {
                include: Some(sruja_language::ViewRuleExpr {
                    wildcard: false,
                    recursive: false,
                    elements: vec!["Shop.*".to_string()],
                }),
                exclude: None,
            }],
        };

        let resolved = resolve_view(&view, &elements, &[]);
        assert_eq!(resolved.elements.len(), 3);
    }

    #[test]
    fn test_specific_elements() {
        let mut elements = HashMap::new();
        elements.insert(
            "Shop".to_string(),
            create_test_element("Shop", ElementKind::System),
        );
        elements.insert(
            "Shop.WebApp".to_string(),
            create_test_element("Shop.WebApp", ElementKind::Container),
        );
        elements.insert(
            "Shop.API".to_string(),
            create_test_element("Shop.API", ElementKind::Container),
        );

        let view = ViewDef {
            location: SourceLocation::new(String::new(), 0, 0),
            id: "test".to_string(),
            title: String::new(),
            description: None,
            view_of: None,
            tags: vec![],
            rules: vec![sruja_language::ViewRule {
                include: Some(sruja_language::ViewRuleExpr {
                    wildcard: false,
                    recursive: false,
                    elements: vec!["Shop.API".to_string()],
                }),
                exclude: None,
            }],
        };

        let resolved = resolve_view(&view, &elements, &[]);
        assert!(resolved.elements.contains_key("Shop"));
        assert!(resolved.elements.contains_key("Shop.API"));
        assert!(!resolved.elements.contains_key("Shop.WebApp"));
    }

    #[test]
    fn test_exclude_rule() {
        let mut elements = HashMap::new();
        elements.insert(
            "Shop".to_string(),
            create_test_element("Shop", ElementKind::System),
        );
        elements.insert(
            "Shop.WebApp".to_string(),
            create_test_element("Shop.WebApp", ElementKind::Container),
        );
        elements.insert(
            "Shop.API".to_string(),
            create_test_element("Shop.API", ElementKind::Container),
        );

        let view = ViewDef {
            location: SourceLocation::new(String::new(), 0, 0),
            id: "test".to_string(),
            title: String::new(),
            description: None,
            view_of: None,
            tags: vec![],
            rules: vec![sruja_language::ViewRule {
                include: Some(sruja_language::ViewRuleExpr {
                    wildcard: true,
                    recursive: false,
                    elements: vec![],
                }),
                exclude: Some(sruja_language::ViewRuleExpr {
                    wildcard: false,
                    recursive: false,
                    elements: vec!["Shop.WebApp".to_string()],
                }),
            }],
        };

        let resolved = resolve_view(&view, &elements, &[]);
        assert!(resolved.elements.contains_key("Shop"));
        assert!(!resolved.elements.contains_key("Shop.WebApp"));
        assert!(resolved.elements.contains_key("Shop.API"));
    }

    #[test]
    fn test_scoped_view() {
        let mut elements = HashMap::new();
        elements.insert(
            "Shop".to_string(),
            create_test_element("Shop", ElementKind::System),
        );
        elements.insert(
            "Shop.WebApp".to_string(),
            create_test_element("Shop.WebApp", ElementKind::Container),
        );
        elements.insert(
            "Shop.API".to_string(),
            create_test_element("Shop.API", ElementKind::Container),
        );
        elements.insert(
            "User".to_string(),
            create_test_element("User", ElementKind::Person),
        );

        let view = ViewDef {
            location: SourceLocation::new(String::new(), 0, 0),
            id: "test".to_string(),
            title: String::new(),
            description: None,
            view_of: Some(QualifiedIdent::qualified(vec!["Shop".to_string()])),
            tags: vec![],
            rules: vec![sruja_language::ViewRule {
                include: Some(sruja_language::ViewRuleExpr {
                    wildcard: true,
                    recursive: false,
                    elements: vec![],
                }),
                exclude: None,
            }],
        };

        let resolved = resolve_view(&view, &elements, &[]);
        assert_eq!(resolved.elements.len(), 3);
        assert!(!resolved.elements.contains_key("User"));
    }

    #[test]
    fn test_export_from_resolved_view_produces_mermaid() {
        let mut elements = HashMap::new();
        elements.insert(
            "Shop".to_string(),
            create_test_element("Shop", ElementKind::System),
        );
        elements.insert(
            "Shop.Web".to_string(),
            create_test_element("Shop.Web", ElementKind::Container),
        );
        let relations = vec![];

        let resolved = ResolvedView {
            view_id: "test".to_string(),
            title: "Test View".to_string(),
            description: None,
            elements,
            relations,
        };

        let config = MermaidConfig::default();
        let exporter = MermaidExporter::new(config);
        let mermaid = exporter.export_from_resolved_view(&resolved);

        assert!(!mermaid.is_empty());
        assert!(mermaid.contains("graph"));
        assert!(mermaid.contains("Shop"));
    }
}
