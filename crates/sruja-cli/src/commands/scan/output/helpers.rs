use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use sruja_scan::{is_path_production_relevant, Graph, NodeKind};

use crate::commands::CliError;

use super::types::{DraftBaselineSkip, ScanQuality};

pub(crate) fn sanitize_identifier(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len().max(1));
    for (i, ch) in raw.chars().enumerate() {
        let ok = if i == 0 {
            ch.is_alphabetic() || ch == '_'
        } else {
            ch.is_alphanumeric() || ch == '_' || ch == '-'
        };
        if ok {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "_".to_string()
    } else if out.chars().next().map(|c| c.is_alphabetic() || c == '_') != Some(true) {
        format!("_{}", out)
    } else {
        out
    }
}

pub(crate) fn element_kind_for_node(
    node_kind: NodeKind,
) -> (sruja_language::ElementKind, Option<String>) {
    match node_kind.as_str() {
        NodeKind::SYSTEM => (sruja_language::ElementKind::System, None),
        NodeKind::SERVICE => (
            sruja_language::ElementKind::Container,
            Some("service".to_string()),
        ),
        NodeKind::CONTAINER => (sruja_language::ElementKind::Container, None),
        NodeKind::COMPONENT => (sruja_language::ElementKind::Component, None),
        NodeKind::DATABASE => (sruja_language::ElementKind::Database, None),
        NodeKind::QUEUE => (sruja_language::ElementKind::Queue, None),
        NodeKind::EXTERNAL_API => (
            sruja_language::ElementKind::ExternalSystem,
            Some("api".to_string()),
        ),
        NodeKind::FRONTEND => (
            sruja_language::ElementKind::Container,
            Some("frontend".to_string()),
        ),
        NodeKind::MODULE => (
            sruja_language::ElementKind::Component,
            Some("module".to_string()),
        ),
        other => (
            sruja_language::ElementKind::Component,
            Some(other.to_string()),
        ),
    }
}

pub(crate) fn qualified_ident_from_id(id: &str) -> sruja_language::QualifiedIdent {
    let parts = id
        .split('.')
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        sruja_language::QualifiedIdent {
            parts: vec!["_".to_string()],
        }
    } else {
        sruja_language::QualifiedIdent { parts }
    }
}

pub(crate) fn path_production_relevant(path: &str) -> bool {
    let normalized = path
        .trim_start_matches("./")
        .trim_start_matches(".\\")
        .replace('\\', "/");
    is_path_production_relevant(&normalized)
}

pub(crate) fn build_draft_program_from_graph(
    graph: &Graph,
    filename: &str,
) -> sruja_language::Program {
    super::super::draft_summary::build_summary_draft_program(graph, filename)
}

pub fn draft_baseline_skip_reason(repo_root: &Path) -> Option<DraftBaselineSkip> {
    if crate::utils::architecture_path::resolve_architecture_path(repo_root).is_some() {
        return Some(DraftBaselineSkip::ReviewedBaselineExists);
    }
    if super::super::draft_summary::draft_baseline_path(repo_root).exists() {
        return Some(DraftBaselineSkip::DraftExists);
    }
    None
}

pub fn write_draft_baseline(
    repo_root: &Path,
    graph: &Graph,
    force: bool,
) -> Result<Option<PathBuf>, CliError> {
    let out_path = super::super::draft_summary::draft_baseline_path(repo_root);
    if !force && draft_baseline_skip_reason(repo_root).is_some() {
        return Ok(None);
    }
    let program = build_draft_program_from_graph(graph, out_path.to_string_lossy().as_ref());
    let printer = sruja_export::DslPrinter::new();
    let dsl = printer.print(&program);
    let header = format!(
        r#"// Sruja architecture draft ({})
// Structural map from workspace manifests (Cargo/npm) when available — not domain architecture.
// Call/import graphs are intentionally omitted. Max {} containers, {} workspace relationships.
//
// Next steps:
// 1. Rename containers to match how your team describes the system.
// 2. Add actors, data stores, and runtime/data-flow relationships.
// 3. Copy or merge into repo.sruja when satisfied, then `sruja lint repo.sruja`.
// 4. Use `sruja drift -r . -a repo.sruja` in CI after promotion.

"#,
        super::super::draft_summary::DRAFT_BASELINE_FILE,
        super::super::draft_summary::MAX_SUMMARY_CONTAINERS,
        super::super::draft_summary::MAX_SUMMARY_EDGES,
    );
    fs::write(&out_path, format!("{}{}", header, dsl))?;
    Ok(Some(out_path))
}

pub(crate) fn escape_github_actions_message(input: &str) -> String {
    input
        .replace('%', "%25")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
}

pub(crate) fn violation_kind_str(kind: sruja_diff::ViolationKind) -> &'static str {
    match kind {
        sruja_diff::ViolationKind::LayerViolation => "layer-violation",
        sruja_diff::ViolationKind::MissingDependency => "missing-dependency",
        sruja_diff::ViolationKind::OrphanComponent => "orphan-component",
        sruja_diff::ViolationKind::CircularDependency => "circular-dependency",
        sruja_diff::ViolationKind::UndocumentedComponent => "undocumented-component",
        sruja_diff::ViolationKind::PatternMismatch => "pattern-mismatch",
        sruja_diff::ViolationKind::GodModule => "god-module",
    }
}

pub(crate) fn violation_best_file_line(v: &sruja_diff::Violation) -> (Option<&str>, Option<u32>) {
    if let Some(src) = v.sources.first() {
        if let (Some(ref file), Some(line)) = (&src.file, src.line) {
            return (Some(file.as_str()), Some(line));
        }
        if let Some(ref file) = src.file {
            return (Some(file.as_str()), None);
        }
    }

    if let Some(ref loc) = v.location {
        if let Some((file, line)) = loc.rsplit_once(':') {
            if let Ok(line_num) = line.parse::<u32>() {
                return (Some(file), Some(line_num));
            }
        }
        return (Some(loc.as_str()), None);
    }

    (None, None)
}

/// Static scan limitations always surfaced in structural drift output.
pub(crate) const INFERENCE_LIMITATIONS: &[&str] = &[
    "Dynamic imports and runtime plugin loading may be missing from the graph.",
    "Framework DI, reflection, and generated code may hide real dependencies.",
    "Layer rules are heuristic; confirm violations against your team's conventions.",
];

pub(crate) fn apply_advisory_violation_filter(report: &mut sruja_diff::DriftReport) {
    use sruja_diff::ViolationKind;
    report
        .violations
        .retain(|v| !matches!(v.kind, ViolationKind::OrphanComponent));
    report.orphan_modules = 0;
    report.health_score = sruja_diff::calculate_health_score_from_violations(
        &report.violations,
        sruja_diff::HealthScorePenalties::default(),
    );
    report.health_breakdown = None;
}

pub(crate) fn collect_could_not_infer(graph: &Graph) -> Vec<String> {
    let mut items: Vec<String> = INFERENCE_LIMITATIONS
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    let mut without_path = 0usize;
    for node in &graph.nodes {
        let has_path = node
            .metadata
            .get("path")
            .or_else(|| node.metadata.get("file"))
            .is_some();
        if !has_path && (node.kind == NodeKind::MODULE || node.kind == NodeKind::SERVICE) {
            without_path += 1;
        }
    }
    if without_path > 0 {
        items.push(format!(
            "{without_path} module(s)/service(s) have no file path in scan evidence (IDs only)."
        ));
    }

    if let Some(quality) = calculate_scan_quality_internal(graph) {
        if quality.coverage_percent < 60 {
            items.push(format!(
                "Low discovery coverage ({}% of nodes with confidence metadata).",
                quality.coverage_percent
            ));
        }
    }

    items
}

pub(crate) fn build_structural_drift_json_envelope(
    report: &sruja_diff::DriftReport,
    graph: &Graph,
    could_not_infer: &[String],
) -> serde_json::Value {
    let production_errors = report
        .violations
        .iter()
        .filter(|v| {
            matches!(v.severity, sruja_diff::Severity::Error)
                && v.production_relevant != Some(false)
        })
        .count();
    let production_warnings = report
        .violations
        .iter()
        .filter(|v| {
            matches!(v.severity, sruja_diff::Severity::Warning)
                && v.production_relevant != Some(false)
        })
        .count();
    let clean_scan = production_errors == 0 && production_warnings == 0;

    let mut value = serde_json::to_value(report).unwrap_or(serde_json::json!({}));
    let obj = value.as_object_mut();
    if let Some(map) = obj {
        map.insert("clean_scan".to_string(), serde_json::json!(clean_scan));
        map.insert(
            "could_not_infer".to_string(),
            serde_json::json!(could_not_infer),
        );
        if let Some(quality) = calculate_scan_quality_internal(graph) {
            map.insert(
                "scan_quality".to_string(),
                serde_json::to_value(quality).unwrap_or_default(),
            );
        }
    }
    crate::commands::remediation::wrap_deterministic_json(
        value,
        "structural_drift",
        "Deterministic structural scan (no repo.sruja baseline).",
    )
}

pub fn calculate_scan_quality_internal(graph: &Graph) -> Option<ScanQuality> {
    if graph.nodes.is_empty() {
        return None;
    }

    let mut has_incoming = HashSet::new();
    let mut has_outgoing = HashSet::new();
    for edge in &graph.edges {
        has_outgoing.insert(edge.source.as_str());
        has_incoming.insert(edge.target.as_str());
    }

    let entry_points = graph
        .nodes
        .iter()
        .filter(|n| !has_incoming.contains(n.id.as_str()))
        .count();
    let leaf_nodes = graph
        .nodes
        .iter()
        .filter(|n| !has_outgoing.contains(n.id.as_str()))
        .count();
    let orphans = graph
        .nodes
        .iter()
        .filter(|n| !has_incoming.contains(n.id.as_str()) && !has_outgoing.contains(n.id.as_str()))
        .count();

    let manifest_discoveries = graph
        .nodes
        .iter()
        .filter(|n| {
            n.metadata.contains_key("source_manifest")
                || n.id.contains("package_json")
                || n.id.contains("go_mod")
                || n.id.contains("dockerfile")
        })
        .count();

    // Confidence Score Calculation
    // Uses per-node confidence from ConfidenceScorer (technology, connectivity, path heuristics)
    // plus bonus for manifest discoveries and penalty for orphans.
    let nodes_with_confidence = graph
        .nodes
        .iter()
        .filter(|n| n.confidence.unwrap_or(0) > 0)
        .count();
    let coverage_percent = if !graph.nodes.is_empty() {
        ((nodes_with_confidence as f32 / graph.nodes.len() as f32) * 100.0) as u8
    } else {
        0
    };

    let graph_confidence = graph.confidence.unwrap_or(0) as i32;
    let mut score = (coverage_percent as i32 + graph_confidence) / 2;
    score += (manifest_discoveries * 5) as i32;
    score -= (orphans * 2) as i32;
    let confidence_score = score.clamp(0, 100) as u8;

    Some(ScanQuality {
        confidence_score,
        coverage_percent,
        manifest_discoveries,
        entry_point_count: entry_points,
        leaf_node_count: leaf_nodes,
        orphan_count: orphans,
    })
}
