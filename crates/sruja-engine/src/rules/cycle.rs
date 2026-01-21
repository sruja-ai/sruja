//! Cycle detection validation rule
//!
//! Detects circular dependencies in the architecture.

use std::collections::{HashMap, HashSet, VecDeque};

use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use sruja_language::Program;

use crate::validator::Rule;

/// Rule that detects circular dependencies
pub struct CycleDetectionRule;

impl Rule for CycleDetectionRule {
    fn name(&self) -> &str {
        "Cycle Detection"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Build adjacency list from relations
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        // TODO: Collect relations from program
        // For now, this is a placeholder

        // Detect cycles using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        fn dfs(
            node: &str,
            adj: &HashMap<String, Vec<String>>,
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
                        dfs(neighbor, adj, visited, rec_stack, path, diagnostics);
                    } else if rec_stack.contains(neighbor) {
                        // Cycle detected
                        let cycle_start = path.iter().position(|x| x == neighbor).unwrap();
                        let cycle: Vec<String> = path[cycle_start..].to_vec();
                        
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
                dfs(node, &adj, &mut visited, &mut rec_stack, &mut path, &mut diagnostics);
            }
        }

        diagnostics
    }
}
