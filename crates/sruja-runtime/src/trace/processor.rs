//! Trace processing: execution graph, span aggregation.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ExecutionTrace;

/// An edge in the execution graph (caller -> callee).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEdge {
    /// Caller span name
    pub caller: String,
    /// Callee span name
    pub callee: String,
    /// Number of times this edge was observed
    pub count: usize,
}

/// Execution graph: caller->callee relationships from parent-child spans.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionGraph {
    pub edges: Vec<ExecutionEdge>,
}

/// Builds an execution graph from traces.
pub struct ExecutionGraphProcessor;

impl ExecutionGraphProcessor {
    fn extract_edges(
        t: &ExecutionTrace,
        parent_name: Option<&str>,
        edges: &mut HashMap<(String, String), usize>,
    ) {
        let caller = parent_name.unwrap_or(&t.name);
        for child in &t.children {
            edges
                .entry((caller.to_string(), child.name.clone()))
                .and_modify(|n| *n += 1)
                .or_insert(1);
            Self::extract_edges(child, Some(&child.name), edges);
        }
    }

    /// Build execution graph from traces.
    pub fn process(traces: &[ExecutionTrace]) -> ExecutionGraph {
        let mut edge_counts: HashMap<(String, String), usize> = HashMap::new();
        for t in traces {
            Self::extract_edges(t, None, &mut edge_counts);
        }

        let edges: Vec<ExecutionEdge> = edge_counts
            .into_iter()
            .map(|((caller, callee), count)| ExecutionEdge {
                caller,
                callee,
                count,
            })
            .collect();

        ExecutionGraph { edges }
    }
}
