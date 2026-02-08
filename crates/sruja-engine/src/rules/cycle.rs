//! Cycle detection validation rule
//!
//! Detects circular dependencies in the architecture.
//! Relations inside causal_loop elements are excluded, since cycles are the
//! intended semantic for feedback loops in systems thinking models.

use std::collections::{HashMap, HashSet};

use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use sruja_language::{
    collect_elements, collect_relations_with_scope, resolve_relation_fqns, ElementKind, Program,
};

use crate::validator::Rule;

/// Returns true if the scope (parent FQN) refers to a causal_loop or feedback element.
/// Cycles are intentional in feedback loops (systems thinking).
fn is_scope_feedback_loop(
    scope: &str,
    elements: &HashMap<String, sruja_language::ElementDef>,
) -> bool {
    if scope.is_empty() {
        return false;
    }
    let elem = match elements.get(scope) {
        Some(e) => e,
        None => return false,
    };
    matches!(
        &elem.assignment.kind,
        ElementKind::Custom(k) if k == "causal_loop" || k == "feedback"
    )
}

/// Rule that detects circular dependencies
pub struct CycleDetectionRule;

impl Rule for CycleDetectionRule {
    fn name(&self) -> &str {
        "Cycle Detection"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Collect elements and relations with scope
        let (elements, _) = collect_elements(program);
        let relations_with_scope = collect_relations_with_scope(program);

        // Exclude relations inside causal_loop/feedback elements; cycles are intentional there
        let relations: Vec<_> = relations_with_scope
            .into_iter()
            .filter(|rws| !is_scope_feedback_loop(&rws.scope, &elements))
            .map(|rws| resolve_relation_fqns(rws.relation, &rws.scope, &elements))
            .collect();

        // Build adjacency list from filtered relations
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for rel in &relations {
            let from = rel.from.as_string();
            let to = rel.to.as_string();
            adj.entry(from).or_default().push(to);
        }

        // Detect cycles using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        fn dfs(
            node: &str,
            adj: &HashMap<String, Vec<String>>,
            elements: &HashMap<String, sruja_language::ElementDef>,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
            path: &mut Vec<String>,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            visited.insert(node.to_string());
            rec_stack.insert(node.to_string());
            path.push(node.to_string());

            if let Some(neighbors) = adj.get(node) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        dfs(
                            neighbor,
                            adj,
                            elements,
                            visited,
                            rec_stack,
                            path,
                            diagnostics,
                        );
                    } else if rec_stack.contains(neighbor) {
                        // Cycle detected
                        let cycle_start = path.iter().position(|x| x == neighbor).unwrap();
                        let cycle: Vec<String> = path[cycle_start..].to_vec();

                        // Skip cycles where all nodes are variables (causal/feedback loops)
                        let all_variables = cycle.iter().all(|node| {
                            elements.get(node).is_some_and(|e| {
                                matches!(
                                    &e.assignment.kind,
                                    ElementKind::Custom(k) if k == "variable"
                                )
                            })
                        });
                        if all_variables {
                            // Likely intentional causal loop; do not report
                            continue;
                        }

                        diagnostics.push(Diagnostic::new(
                            sruja_diagnostics::codes::CODE_CYCLE_DETECTED,
                            Severity::Error,
                            format!("Circular dependency detected: {}", cycle.join(" -> ")),
                            SourceLocation::new(String::new(), 0, 0),
                        ).with_suggestions(vec![
                            "Cycles are valid for feedback loops, event-driven patterns, or mutual dependencies".to_string(),
                            "If this is unintended, consider breaking the cycle by introducing an intermediate element".to_string(),
                        ]));
                    }
                }
            }

            rec_stack.remove(node);
            path.pop();
        }

        for node in adj.keys() {
            if !visited.contains(node) {
                dfs(
                    node,
                    &adj,
                    &elements,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut diagnostics,
                );
            }
        }

        diagnostics
    }
}
