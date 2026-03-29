//! Runtime analysis: analyze trace/span data for emergent cycles and hotspots.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::CliError;

#[derive(Debug, Clone, serde::Deserialize)]
struct Span {
    name: String,
    #[serde(default)]
    start: Option<String>,
    #[serde(default)]
    end: Option<String>,
    #[serde(default)]
    children: Vec<Span>,
}

#[derive(Debug, Serialize)]
struct RuntimeReport {
    trace_count: usize,
    total_spans: usize,
    max_depth: usize,
    emergent_cycles: Vec<EmergentCycle>,
    hotspots: Vec<Hotspot>,
    execution_graph: ExecutionGraph,
}

#[derive(Debug, Serialize)]
struct EmergentCycle {
    pattern: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Hotspot {
    name: String,
    count: usize,
    total_duration_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
struct ExecutionGraph {
    nodes: Vec<String>,
    edges: Vec<(String, String)>,
}

fn count_spans(spans: &[Span]) -> usize {
    spans
        .iter()
        .fold(0, |acc, s| acc + 1 + count_spans(&s.children))
}

fn max_depth(spans: &[Span], depth: usize) -> usize {
    spans
        .iter()
        .map(|s| max_depth(&s.children, depth + 1))
        .max()
        .unwrap_or(depth)
}

fn collect_paths(spans: &[Span], path: &mut Vec<String>, paths: &mut Vec<Vec<String>>) {
    for s in spans {
        path.push(s.name.clone());
        if s.children.is_empty() {
            paths.push(path.clone());
        } else {
            collect_paths(&s.children, path, paths);
        }
        path.pop();
    }
}

/// Detect cycles in a path: any contiguous subsequence where the same name appears twice
/// (e.g. [planner, executor, planner] -> pattern [planner, executor, planner]).
fn cycles_in_path(path: &[String]) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    for (i, name) in path.iter().enumerate() {
        if let Some(j) = path[i + 1..].iter().position(|n| n == name) {
            let j = i + 1 + j;
            let pattern: Vec<String> = path[i..=j].to_vec();
            if pattern.len() >= 2 {
                out.push(pattern);
            }
        }
    }
    out
}

fn analyze_spans(spans: &[Span]) -> RuntimeReport {
    let trace_count = spans.len();
    let total_spans = count_spans(spans);
    let depth = max_depth(spans, 0);
    let max_depth_val = if trace_count > 0 { depth + 1 } else { 0 };

    let mut all_paths = Vec::new();
    collect_paths(spans, &mut Vec::new(), &mut all_paths);

    let mut cycle_set: std::collections::HashSet<Vec<String>> = std::collections::HashSet::new();
    for path in &all_paths {
        for cycle in cycles_in_path(path) {
            cycle_set.insert(cycle);
        }
    }
    let emergent_cycles: Vec<EmergentCycle> = cycle_set
        .into_iter()
        .map(|pattern| EmergentCycle { pattern })
        .collect();

    let mut name_count: HashMap<String, usize> = HashMap::new();
    let mut name_duration: HashMap<String, i64> = HashMap::new();
    fn aggregate(
        spans: &[Span],
        counts: &mut HashMap<String, usize>,
        durations: &mut HashMap<String, i64>,
    ) {
        for s in spans {
            *counts.entry(s.name.clone()).or_default() += 1;
            if let (Some(start), Some(end)) = (&s.start, &s.end) {
                if let (Ok(st), Ok(en)) = (
                    chrono::DateTime::parse_from_rfc3339(start),
                    chrono::DateTime::parse_from_rfc3339(end),
                ) {
                    let ms = (en - st).num_milliseconds();
                    *durations.entry(s.name.clone()).or_default() += ms;
                }
            }
            aggregate(&s.children, counts, durations);
        }
    }
    aggregate(spans, &mut name_count, &mut name_duration);

    let mut hotspots: Vec<Hotspot> = name_count
        .iter()
        .map(|(name, &count)| {
            let total_duration_ms = name_duration.get(name).copied();
            Hotspot {
                name: name.clone(),
                count,
                total_duration_ms,
            }
        })
        .collect();
    hotspots.sort_by(|a, b| b.count.cmp(&a.count));
    hotspots.truncate(10);

    let mut nodes: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut edges: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    fn graph_from(
        spans: &[Span],
        parent: Option<&str>,
        nodes: &mut std::collections::HashSet<String>,
        edges: &mut std::collections::HashSet<(String, String)>,
    ) {
        for s in spans {
            nodes.insert(s.name.clone());
            if let Some(p) = parent {
                edges.insert((p.to_string(), s.name.clone()));
            }
            graph_from(&s.children, Some(&s.name), nodes, edges);
        }
    }
    graph_from(spans, None, &mut nodes, &mut edges);
    let execution_graph = ExecutionGraph {
        nodes: nodes.into_iter().collect(),
        edges: edges.into_iter().collect(),
    };

    RuntimeReport {
        trace_count,
        total_spans,
        max_depth: max_depth_val,
        emergent_cycles,
        hotspots,
        execution_graph,
    }
}

/// Run `sruja runtime analyze`: read traces JSON, analyze, output text or JSON.
pub async fn runtime_analyze(traces_path: &str, format: &str) -> Result<(), CliError> {
    let path = Path::new(traces_path);
    let content = fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("traces file not found: {}", traces_path),
            ))
        } else {
            CliError::Io(e)
        }
    })?;

    let spans: Vec<Span> = serde_json::from_str(&content).map_err(CliError::Json)?;

    let report = analyze_spans(&spans);

    match format {
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(CliError::Json)?
            );
        }
        _ => {
            println!("Root traces: {}", report.trace_count);
            println!("Total spans: {}", report.total_spans);
            println!("Max depth: {}", report.max_depth);
            if !report.emergent_cycles.is_empty() {
                println!("Emergent cycles: {}", report.emergent_cycles.len());
                for c in &report.emergent_cycles {
                    println!("  - {}", c.pattern.join(" → "));
                }
            }
            if !report.hotspots.is_empty() {
                println!("Hotspots (top by count):");
                for h in report.hotspots.iter().take(5) {
                    println!("  - {} (count: {})", h.name, h.count);
                }
            }
        }
    }

    Ok(())
}
