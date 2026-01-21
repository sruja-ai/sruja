//! Layer violation validation rule
//!
//! Enforces strict layering (e.g., Web -> API -> Service -> Data -> Database),
//! mirroring the Go `LayerViolationRule`.

use std::collections::HashMap;

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{collect_elements, Program};

use crate::validator::Rule;

/// Rule that detects layer violations based on `metadata { layer "..." }` or name heuristics.
pub struct LayerViolationRule;

impl Rule for LayerViolationRule {
    fn name(&self) -> &str {
        "Layer Violation"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        let (elements, relations) = collect_elements(program);

        // Ordered from top to bottom.
        // NOTE: In Go this is currently hard-coded; keep parity.
        let layers: [&str; 5] = ["web", "api", "service", "data", "database"];
        let layer_map: HashMap<&str, usize> = layers.iter().copied().enumerate().map(|(i, l)| (l, i)).collect();

        let mut diags: Vec<Diagnostic> = Vec::with_capacity((relations.len() / 10).max(8));

        for rel in &relations {
            let from_name = rel.from.as_string();
            let to_name = rel.to.as_string();

            let from_layer = resolve_layer(&elements, &from_name, &layers);
            let to_layer = resolve_layer(&elements, &to_name, &layers);

            if from_layer.is_empty() || to_layer.is_empty() {
                continue;
            }

            // Unwrap is safe because we only ever return values from `layers`.
            let from_idx = *layer_map.get(from_layer.as_str()).unwrap_or(&usize::MAX);
            let to_idx = *layer_map.get(to_layer.as_str()).unwrap_or(&usize::MAX);

            if from_idx == usize::MAX || to_idx == usize::MAX {
                continue;
            }

            // Same logic as Go: dependency must flow "downwards" (from lower index to higher index).
            if from_idx > to_idx {
                let msg = format!(
                    "Layer violation: '{}' ({}) cannot depend on '{}' ({}). Dependencies must flow downwards (higher layers can only depend on lower layers).",
                    from_name, from_layer, to_name, to_layer
                );

                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_LAYER_VIOLATION,
                        Severity::Error,
                        msg,
                        rel.location.clone(),
                    )
                    .with_context(vec![format!("{} -> {}", from_name, to_name)])
                    .with_suggestions(vec![
                        format!("Reverse the dependency: '{} -> {}'", to_name, from_name),
                        "Or restructure to follow proper layering (e.g., Web -> API -> Data)".to_string(),
                        "If this is intentional, consider documenting the exception".to_string(),
                    ]),
                );
            }
        }

        diags
    }
}

fn resolve_layer(
    elements: &HashMap<String, sruja_language::ElementDef>,
    name: &str,
    layers: &[&str],
) -> String {
    // 1) Try metadata on matching element
    if let Some(elem) = find_element(elements, name) {
        if let Some(body) = &elem.assignment.body {
            for entry in &body.metadata {
                if entry.key == "layer" {
                    if let Some(v) = &entry.value {
                        return normalize_meta_value(v);
                    }
                }
            }
        }
    }

    // 2) Name heuristic
    let lower = name.to_lowercase();
    for l in layers {
        if lower.contains(l) {
            return (*l).to_string();
        }
    }
    String::new()
}

fn normalize_meta_value(v: &str) -> String {
    v.trim().trim_matches('"').to_lowercase()
}

fn find_element<'a>(
    elements: &'a HashMap<String, sruja_language::ElementDef>,
    name: &str,
) -> Option<&'a sruja_language::ElementDef> {
    // Exact FQN
    if let Some(e) = elements.get(name) {
        return Some(e);
    }
    // Exact leaf id match or suffix match: ".id"
    let suffix = format!(".{}", name);
    elements
        .iter()
        .find_map(|(fqn, e)| if fqn == name || fqn.ends_with(&suffix) { Some(e) } else { None })
}

