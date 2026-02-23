//! Runtime report types for analysis output and Phase 5 integration.

use serde::{Deserialize, Serialize};

use crate::analysis::{EmergentCycle, RuntimeHotspot};
use crate::trace::{ExecutionGraph, ExecutionTrace};

/// Aggregated runtime analysis report (traces, cycles, hotspots, execution graph).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeReport {
    pub trace_count: usize,
    pub total_spans: usize,
    pub max_depth: usize,
    pub total_duration_ms: i64,
    pub emergent_cycles: Vec<EmergentCycle>,
    pub hotspots: Vec<RuntimeHotspot>,
    pub execution_graph: ExecutionGraph,
}

/// Build a RuntimeReport from execution traces.
pub fn build_report(traces: &[ExecutionTrace]) -> RuntimeReport {
    let total_spans: usize = traces.iter().map(trace_span_count).sum();
    let max_depth = traces.iter().map(trace_max_depth).max().unwrap_or(0);
    let total_duration_ms: i64 = traces
        .iter()
        .map(|t| (t.end - t.start).num_milliseconds())
        .sum();

    let emergent_cycles = crate::analysis::EmergentCycleDetector::new().detect(traces);
    let hotspots = crate::analysis::HotspotDetector::new().detect(traces);
    let execution_graph = crate::trace::ExecutionGraphProcessor::process(traces);

    RuntimeReport {
        trace_count: traces.len(),
        total_spans,
        max_depth,
        total_duration_ms,
        emergent_cycles,
        hotspots,
        execution_graph,
    }
}

fn trace_span_count(t: &ExecutionTrace) -> usize {
    1 + t.children.iter().map(trace_span_count).sum::<usize>()
}

fn trace_max_depth(t: &ExecutionTrace) -> usize {
    let child_depth = t.children.iter().map(trace_max_depth).max().unwrap_or(0);
    1 + child_depth
}
