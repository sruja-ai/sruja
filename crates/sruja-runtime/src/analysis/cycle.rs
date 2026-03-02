//! Emergent cycle detection in execution traces.
//!
//! Detects repeating patterns in span sequences (e.g. A -> B -> C -> A) that
//! indicate workflow loops or agent re-planning cycles.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::trace::ExecutionTrace;

/// An emergent cycle detected in tool/workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentCycle {
    /// Cycle pattern (e.g. ["planner", "executor", "validator", "planner"])
    pub pattern: Vec<String>,
    /// Number of times this cycle was observed
    pub occurrences: usize,
    /// Severity based on occurrence count and pattern length
    pub severity: CycleSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CycleSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Detects emergent cycles in execution traces.
pub struct EmergentCycleDetector {
    /// Minimum occurrences to report (default 2)
    pub min_occurrences: usize,
    /// Maximum pattern length to consider (avoids runaway loops)
    pub max_pattern_len: usize,
}

impl Default for EmergentCycleDetector {
    fn default() -> Self {
        Self {
            min_occurrences: 2,
            max_pattern_len: 20,
        }
    }
}

impl EmergentCycleDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_occurrences(mut self, n: usize) -> Self {
        self.min_occurrences = n;
        self
    }

    /// Extract span names in depth-first order from a trace.
    fn dfs_names(t: &ExecutionTrace, out: &mut Vec<String>) {
        out.push(t.name.clone());
        for child in &t.children {
            Self::dfs_names(child, out);
        }
    }

    /// Find all cycles in a sequence of span names.
    /// A cycle is when name[i] == name[j] for j > i, with j - i <= max_pattern_len.
    fn find_cycles_in_sequence(names: &[String], max_len: usize) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        for i in 0..names.len() {
            for j in (i + 1)..names.len().min(i + max_len + 1) {
                if names[i] == names[j] {
                    let pattern: Vec<String> = names[i..=j].to_vec();
                    if pattern.len() >= 2 {
                        cycles.push(pattern);
                    }
                }
            }
        }
        cycles
    }

    /// Detect emergent cycles from execution traces.
    pub fn detect(&self, traces: &[ExecutionTrace]) -> Vec<EmergentCycle> {
        let mut pattern_counts: HashMap<String, (Vec<String>, usize)> = HashMap::new();

        for trace in traces {
            let mut names = Vec::new();
            Self::dfs_names(trace, &mut names);
            let cycles = Self::find_cycles_in_sequence(&names, self.max_pattern_len);
            for pattern in cycles {
                let key = pattern.join(" -> ");
                pattern_counts
                    .entry(key)
                    .or_insert_with(|| (pattern.clone(), 0))
                    .1 += 1;
            }
        }

        let mut result: Vec<EmergentCycle> = pattern_counts
            .into_iter()
            .filter(|(_, (_, count))| *count >= self.min_occurrences)
            .map(|(_, (pattern, occurrences))| {
                let severity = severity_for_occurrences(occurrences, pattern.len());
                EmergentCycle {
                    pattern,
                    occurrences,
                    severity,
                }
            })
            .collect();

        result.sort_by(|a, b| b.occurrences.cmp(&a.occurrences));
        result
    }
}

fn severity_for_occurrences(occurrences: usize, _pattern_len: usize) -> CycleSeverity {
    match occurrences {
        n if n >= 10 => CycleSeverity::Critical,
        n if n >= 5 => CycleSeverity::Error,
        n if n >= 2 => CycleSeverity::Warning,
        _ => CycleSeverity::Info,
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::trace::ExecutionTrace;

    fn make_trace(name: &str, children: Vec<ExecutionTrace>) -> ExecutionTrace {
        ExecutionTrace {
            id: name.to_string(),
            name: name.to_string(),
            start: Utc::now(),
            end: Utc::now(),
            attributes: vec![],
            children,
        }
    }

    #[test]
    fn detects_simple_cycle() {
        // A -> B -> A (two traces so occurrences >= 2)
        let trace = make_trace(
            "planner",
            vec![make_trace("executor", vec![make_trace("planner", vec![])])],
        );

        let detector = EmergentCycleDetector::new();
        let cycles = detector.detect(&[trace.clone(), trace]);

        assert!(!cycles.is_empty());
        assert!(cycles
            .iter()
            .any(|c| c.pattern == ["planner", "executor", "planner"]));
    }

    #[test]
    fn respects_min_occurrences() {
        let trace = make_trace("A", vec![make_trace("B", vec![make_trace("A", vec![])])]);
        let detector = EmergentCycleDetector::new().with_min_occurrences(2);
        let cycles = detector.detect(&[trace]);
        assert!(cycles.is_empty());
    }
}
