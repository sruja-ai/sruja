//! Scan and drift commands: scan, drift, quickstart.

use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

use sruja_scan::scan_scope::resolve_scan_scope;
use sruja_scan::{scan_repo, EdgeKind, Graph, NodeKind};

use super::CliError;
use crate::context_detection::{
    build_repo_context, detect_architecture_style, detect_framework, detect_languages,
};
use crate::utils::architecture_path;

fn should_fail_on_violations(fail_on: Option<&str>, violations: &[sruja_diff::Violation]) -> bool {
    if let Some(criteria) = fail_on {
        let criteria_lower = criteria.to_lowercase();
        let criteria_list: Vec<&str> = criteria_lower.split(',').map(|s| s.trim()).collect();

        for criterion in criteria_list {
            match criterion {
                "all" => {
                    if violations
                        .iter()
                        .any(|v| matches!(v.severity, sruja_diff::Severity::Error))
                    {
                        return true;
                    }
                }
                "cycles" | "circular" => {
                    if violations
                        .iter()
                        .any(|v| matches!(v.kind, sruja_diff::ViolationKind::CircularDependency))
                    {
                        return true;
                    }
                }
                "layer-violations" | "layer" => {
                    if violations
                        .iter()
                        .any(|v| matches!(v.kind, sruja_diff::ViolationKind::LayerViolation))
                    {
                        return true;
                    }
                }
                "god-modules" | "god" => {
                    if violations
                        .iter()
                        .any(|v| matches!(v.kind, sruja_diff::ViolationKind::GodModule))
                    {
                        return true;
                    }
                }
                "orphans" => {
                    if violations
                        .iter()
                        .any(|v| matches!(v.kind, sruja_diff::ViolationKind::OrphanComponent))
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    false
}

fn truth_status_from_baseline_compare(
    scanned: &Graph,
    baseline_path: &Path,
) -> Result<sruja_diff::TruthStatus, CliError> {
    let content = fs::read_to_string(baseline_path)?;
    let parser = sruja_language::Parser::new(baseline_path.to_string_lossy().as_ref());
    let program = parser.parse(&content).map_err(|diags| CliError::Parse {
        file: baseline_path.to_string_lossy().to_string(),
        message: diags
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("; "),
        diagnostics: diags,
    })?;
    let proposed_graph = sruja_diff::program_to_graph(&program);
    Ok(sruja_diff::compare_graphs(scanned, &proposed_graph).truth_status)
}

fn sanitize_identifier(raw: &str) -> String {
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

fn element_kind_for_node(node_kind: NodeKind) -> (sruja_language::ElementKind, Option<String>) {
    match node_kind {
        NodeKind::System => (sruja_language::ElementKind::System, None),
        NodeKind::Service => (
            sruja_language::ElementKind::Container,
            Some("service".to_string()),
        ),
        NodeKind::Container => (sruja_language::ElementKind::Container, None),
        NodeKind::Component => (sruja_language::ElementKind::Component, None),
        NodeKind::Database => (sruja_language::ElementKind::Database, None),
        NodeKind::Queue => (sruja_language::ElementKind::Queue, None),
        NodeKind::ExternalApi => (
            sruja_language::ElementKind::ExternalSystem,
            Some("api".to_string()),
        ),
        NodeKind::Frontend => (
            sruja_language::ElementKind::Container,
            Some("frontend".to_string()),
        ),
        NodeKind::Module => (
            sruja_language::ElementKind::Component,
            Some("module".to_string()),
        ),
    }
}

fn relation_label_for_edge(edge_kind: EdgeKind) -> &'static str {
    match edge_kind {
        EdgeKind::ReadsFrom => "reads from",
        EdgeKind::WritesTo => "writes to",
        EdgeKind::DependsOn => "depends on",
        EdgeKind::PublishesTo => "publishes to",
        EdgeKind::SubscribesTo => "subscribes to",
        EdgeKind::Owns => "owns",
        EdgeKind::Contains => "contains",
        EdgeKind::Uses => "uses",
        EdgeKind::Calls => "calls",
    }
}

fn qualified_ident_from_id(id: &str) -> sruja_language::QualifiedIdent {
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

fn build_draft_program_from_graph(graph: &Graph, filename: &str) -> sruja_language::Program {
    let mut id_map: std::collections::HashMap<&str, String> = std::collections::HashMap::new();
    let mut used: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for node in &graph.nodes {
        let mut base = sanitize_identifier(&node.id);
        let n = used.entry(base.clone()).or_insert(0);
        if *n > 0 {
            base = format!("{}_{}", base, *n + 1);
        }
        *n += 1;
        id_map.insert(node.id.as_str(), base);
    }

    let mut nodes = graph.nodes.clone();
    nodes.sort_by(|a, b| {
        let ida = id_map
            .get(a.id.as_str())
            .map(String::as_str)
            .unwrap_or(a.id.as_str());
        let idb = id_map
            .get(b.id.as_str())
            .map(String::as_str)
            .unwrap_or(b.id.as_str());
        ida.cmp(idb)
    });

    let mut items: Vec<sruja_language::TopLevelItem> = Vec::new();
    for node in &nodes {
        let name = id_map
            .get(node.id.as_str())
            .cloned()
            .unwrap_or_else(|| sanitize_identifier(&node.id));
        let (kind, sub_kind) = element_kind_for_node(node.kind);

        let body = sruja_language::ElementDefBody {
            description: node
                .path
                .as_ref()
                .map(|p| format!("Scanned from {}", p))
                .or_else(|| Some("Scanned from repository".to_string())),
            technology: match kind {
                sruja_language::ElementKind::Container | sruja_language::ElementKind::Database => {
                    Some(
                        node.technology
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string()),
                    )
                }
                _ => node.technology.clone(),
            },
            ..Default::default()
        };

        let assignment = sruja_language::ElementAssignment {
            location: sruja_diagnostics::SourceLocation::new(filename.to_string(), 1, 1),
            name,
            kind,
            sub_kind,
            title: Some(node.label.clone()),
            tag_refs: Vec::new(),
            body: Some(body),
        };
        let def = sruja_language::ElementDef {
            location: sruja_diagnostics::SourceLocation::new(filename.to_string(), 1, 1),
            assignment,
        };
        items.push(sruja_language::TopLevelItem::ElementDef(Box::new(def)));
    }

    let mut edges = graph.edges.clone();
    edges.sort_by(|a, b| {
        (a.source.as_str(), a.target.as_str(), a.kind.as_str()).cmp(&(
            b.source.as_str(),
            b.target.as_str(),
            b.kind.as_str(),
        ))
    });
    for edge in &edges {
        let from = id_map
            .get(edge.source.as_str())
            .cloned()
            .unwrap_or_else(|| sanitize_identifier(&edge.source));
        let to = id_map
            .get(edge.target.as_str())
            .cloned()
            .unwrap_or_else(|| sanitize_identifier(&edge.target));

        let rel = sruja_language::Relation {
            location: sruja_diagnostics::SourceLocation::new(filename.to_string(), 1, 1),
            from: qualified_ident_from_id(&from),
            to: qualified_ident_from_id(&to),
            label: Some(relation_label_for_edge(edge.kind).to_string()),
            description: None,
            technology: None,
            tags: Vec::new(),
        };
        items.push(sruja_language::TopLevelItem::Relation(rel));
    }

    sruja_language::Program::new().with_items(items)
}

fn write_draft_baseline(repo_root: &Path, graph: &Graph) -> Result<Option<PathBuf>, CliError> {
    let out_path = repo_root.join("repo.sruja");
    if out_path.exists() {
        return Ok(None);
    }
    let program = build_draft_program_from_graph(graph, out_path.to_string_lossy().as_ref());
    let printer = sruja_export::DslPrinter::new();
    let dsl = printer.print(&program);
    fs::write(&out_path, dsl)?;
    Ok(Some(out_path))
}

fn escape_github_actions_message(input: &str) -> String {
    input
        .replace('%', "%25")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
}

fn violation_kind_str(kind: sruja_diff::ViolationKind) -> &'static str {
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

fn violation_best_file_line(v: &sruja_diff::Violation) -> (Option<&str>, Option<u32>) {
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

fn print_violations_github_actions(violations: &[sruja_diff::Violation]) {
    for v in violations {
        let level = match v.severity {
            sruja_diff::Severity::Error => "error",
            sruja_diff::Severity::Warning => "warning",
            sruja_diff::Severity::Info => "notice",
        };
        let (file, line) = violation_best_file_line(v);
        let message = format!("{}: {}", violation_kind_str(v.kind), v.message);
        match (file, line) {
            (Some(f), Some(l)) => println!(
                "::{level} file={f},line={l}::{}",
                escape_github_actions_message(&message)
            ),
            (Some(f), None) => println!(
                "::{level} file={f}::{}",
                escape_github_actions_message(&message)
            ),
            (None, _) => println!("::{level}::{}", escape_github_actions_message(&message)),
        }
    }
}

pub async fn scan(repo_root: &str, output: &str) -> Result<(), CliError> {
    let graph =
        sruja_scan::scan_repo(Path::new(repo_root)).map_err(|e| CliError::Scan(e.to_string()))?;

    let json = serde_json::to_string_pretty(&graph)?;

    if output == "-" {
        println!("{}", json);
        return Ok(());
    }

    fs::write(output, json)?;
    println!("Wrote {}", output);
    Ok(())
}

pub async fn drift(
    repo_root: &str,
    architecture_path: Option<&str>,
    format: &str,
    _enrich: bool,
    violations_only: bool,
    fail_on: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let actual_graph = scan_repo(repo_path)?;

    let resolved = architecture_path::resolve_architecture_path(repo_path);
    let effective_arch = architecture_path.or_else(|| resolved.as_ref().and_then(|p| p.to_str()));

    if let Some(arch_path) = effective_arch {
        let arch_file = Path::new(arch_path);
        if !arch_file.exists() {
            return Err(CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Architecture file not found: {}", arch_path),
            )));
        }
        let content = fs::read_to_string(arch_file)?;
        let parser = sruja_language::Parser::new(arch_path);
        let program = parser.parse(&content).map_err(|diags| CliError::Parse {
            file: arch_path.to_string(),
            message: diags
                .iter()
                .map(|d| d.message.as_str())
                .collect::<Vec<_>>()
                .join("; "),
            diagnostics: diags,
        })?;
        let proposed_graph = sruja_diff::program_to_graph(&program);
        let diff_result = sruja_diff::compare_graphs(&actual_graph, &proposed_graph);

        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&diff_result)?);
            }
            "github" | "github-actions" => {
                print_violations_github_actions(&diff_result.violations);
            }
            _ => {
                print_diff_text(&diff_result, violations_only);
            }
        }

        if should_fail_on_violations(fail_on, &diff_result.violations) {
            std::process::exit(1);
        }
    } else {
        let drift_result = sruja_diff::detect_architectural_drift(&actual_graph);

        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&drift_result)?);
            }
            "github" | "github-actions" => {
                print_violations_github_actions(&drift_result.violations);
            }
            _ => {
                print_drift_text(&drift_result, violations_only);
            }
        }

        if should_fail_on_violations(fail_on, &drift_result.violations) {
            std::process::exit(1);
        }
    }

    Ok(())
}

pub async fn drift_json_string(
    repo_root: &str,
    architecture_path: Option<&str>,
    violations_only: bool,
) -> Result<String, CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let actual_graph = scan_repo(repo_path)?;

    let resolved = architecture_path::resolve_architecture_path(repo_path);
    let effective_arch = architecture_path.or_else(|| resolved.as_ref().and_then(|p| p.to_str()));

    if let Some(arch_path) = effective_arch {
        let arch_file = Path::new(arch_path);
        if !arch_file.exists() {
            return Err(CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Architecture file not found: {}", arch_path),
            )));
        }
        let content = fs::read_to_string(arch_file)?;
        let parser = sruja_language::Parser::new(arch_path);
        let program = parser.parse(&content).map_err(|diags| CliError::Parse {
            file: arch_path.to_string(),
            message: diags
                .iter()
                .map(|d| d.message.as_str())
                .collect::<Vec<_>>()
                .join("; "),
            diagnostics: diags,
        })?;
        let proposed_graph = sruja_diff::program_to_graph(&program);
        let diff_result = sruja_diff::compare_graphs(&actual_graph, &proposed_graph);

        if !violations_only {
            return Ok(serde_json::to_string_pretty(&diff_result)?);
        }

        let value = serde_json::to_value(&diff_result)?;
        let out = serde_json::json!({
            "truth_status": value.get("truth_status").cloned().unwrap_or(serde_json::Value::Null),
            "summary": value.get("summary").cloned().unwrap_or(serde_json::Value::Null),
            "violations": value.get("violations").cloned().unwrap_or(serde_json::Value::Null)
        });
        return Ok(serde_json::to_string_pretty(&out)?);
    }

    let drift_result = sruja_diff::detect_architectural_drift(&actual_graph);

    if !violations_only {
        return Ok(serde_json::to_string_pretty(&drift_result)?);
    }

    let value = serde_json::to_value(&drift_result)?;
    let out = serde_json::json!({
        "truth_status": value.get("truth_status").cloned().unwrap_or(serde_json::Value::Null),
        "health_score": value.get("health_score").cloned().unwrap_or(serde_json::Value::Null),
        "violations": value.get("violations").cloned().unwrap_or(serde_json::Value::Null)
    });
    Ok(serde_json::to_string_pretty(&out)?)
}

/// Result of `sruja status`: baseline path, truth status, and counts for extension/CI.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StatusOutput {
    /// Resolved architecture file path, or null if none found.
    pub baseline: Option<String>,
    /// "reviewed" | "drifted" | "unknown"
    pub truth_status: String,
    pub violations_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_score: Option<u8>,
    /// ISO8601 timestamp from .sruja/context.json if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_updated_at: Option<String>,
}

/// Compute status without printing: resolve baseline, run drift, return summary.
pub async fn status_result(repo_root: &str) -> Result<StatusOutput, CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let baseline = architecture_path::resolve_architecture_path(repo_path)
        .and_then(|p| p.to_str().map(String::from));

    let context_updated_at = std::fs::read_to_string(repo_path.join(".sruja/context.json"))
        .ok()
        .and_then(|s| {
            serde_json::from_str::<serde_json::Value>(&s)
                .ok()
                .and_then(|v| {
                    v.get("updated_at")
                        .and_then(|t| t.as_str())
                        .map(String::from)
                })
        });

    let graph = scan_repo(repo_path)?;

    if let Some(ref arch_path) = baseline {
        let content = fs::read_to_string(arch_path)?;
        let parser = sruja_language::Parser::new(arch_path);
        let program = parser.parse(&content).map_err(|diags| CliError::Parse {
            file: arch_path.clone(),
            message: diags
                .iter()
                .map(|d| d.message.as_str())
                .collect::<Vec<_>>()
                .join("; "),
            diagnostics: diags,
        })?;
        let proposed = sruja_diff::program_to_graph(&program);
        let diff = sruja_diff::compare_graphs(&graph, &proposed);
        let truth_status = match diff.truth_status {
            sruja_diff::TruthStatus::Reviewed => "reviewed",
            sruja_diff::TruthStatus::Drifted => "drifted",
            sruja_diff::TruthStatus::Unknown => "unknown",
        };
        return Ok(StatusOutput {
            baseline: Some(arch_path.clone()),
            truth_status: truth_status.to_string(),
            violations_count: diff.violations.len(),
            health_score: Some(diff.summary.health_score),
            context_updated_at,
        });
    }

    let drift = sruja_diff::detect_architectural_drift(&graph);
    let truth_status = match drift.truth_status {
        sruja_diff::TruthStatus::Reviewed => "reviewed",
        sruja_diff::TruthStatus::Drifted => "drifted",
        sruja_diff::TruthStatus::Unknown => "unknown",
    };
    Ok(StatusOutput {
        baseline: None,
        truth_status: truth_status.to_string(),
        violations_count: drift.violations.len(),
        health_score: Some(drift.health_score),
        context_updated_at,
    })
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct QuickstartResult {
    pub repo: String,
    /// Scan scope metadata (what was included/excluded).
    pub scan_scope: sruja_scan::scan_scope::ScanScope,
    /// Structural health only (cycles, layers, god modules, orphans).
    pub health_score: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_breakdown: Option<sruja_diff::HealthScoreBreakdown>,
    pub inventory: InventorySummary,
    pub top_findings: Vec<Finding>,
    pub actionable_fixes: Vec<ActionableFix>,
    /// No DSL baseline in quickstart: always "unknown".
    pub truth_status: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InventorySummary {
    pub modules: usize,
    pub services: usize,
    pub databases: usize,
    pub external_apis: usize,
    pub total_dependencies: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Finding {
    pub severity: String,
    pub kind: String,
    pub message: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ActionableFix {
    pub priority: String,
    pub description: String,
    pub impact: String,
    pub affected_components: Vec<String>,
}

impl QuickstartResult {
    fn from_drift_report(
        report: &sruja_diff::DriftReport,
        graph: &Graph,
        repo: &str,
        scan_scope: &sruja_scan::ScanScope,
    ) -> Self {
        let external_apis = graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::ExternalApi)
            .count();

        let mut all_violations: Vec<_> = report.violations.iter().collect();
        all_violations.sort_by(|a, b| {
            let severity_order = |s: &sruja_diff::Severity| match s {
                sruja_diff::Severity::Error => 0,
                sruja_diff::Severity::Warning => 1,
                sruja_diff::Severity::Info => 2,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        let top_findings: Vec<Finding> = all_violations
            .iter()
            .take(3)
            .map(|v| {
                let mut evidence: Vec<String> = v
                    .location
                    .as_ref()
                    .map(|s| vec![s.clone()])
                    .unwrap_or_default();
                for s in &v.sources {
                    evidence.push(sruja_diff::SourceRef::display_string(s));
                }
                Finding {
                    severity: match v.severity {
                        sruja_diff::Severity::Error => "error".to_string(),
                        sruja_diff::Severity::Warning => "warning".to_string(),
                        sruja_diff::Severity::Info => "info".to_string(),
                    },
                    kind: format!("{:?}", v.kind),
                    message: v.message.clone(),
                    evidence,
                }
            })
            .collect();

        let actionable_fixes = generate_actionable_fixes_from_violations(&report.violations);

        QuickstartResult {
            repo: repo.to_string(),
            scan_scope: scan_scope.clone(),
            health_score: report.health_score,
            health_breakdown: report.health_breakdown,
            inventory: InventorySummary {
                modules: report.total_modules,
                services: report.total_services,
                databases: report.total_databases,
                external_apis,
                total_dependencies: report.total_dependencies,
            },
            top_findings,
            actionable_fixes,
            truth_status: "unknown".to_string(),
        }
    }
}

fn generate_actionable_fixes_from_violations(
    violations: &[sruja_diff::Violation],
) -> Vec<ActionableFix> {
    use sruja_diff::ViolationKind;
    let mut fixes = Vec::new();

    let circular: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::CircularDependency))
        .collect();
    if !circular.is_empty() {
        let affected: Vec<String> = circular.iter().filter_map(|v| v.location.clone()).collect();
        fixes.push(ActionableFix {
            priority: "high".to_string(),
            description:
                "Break strong circular boundaries (Spaghetti Coupling) using Dependency Inversion or Event buses"
                    .to_string(),
            impact: "Cycles prevent modularity, cause cascading failures, and break independent testability/deployments.".to_string(),
            affected_components: affected,
        });
    }

    let layer: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::LayerViolation))
        .collect();
    if !layer.is_empty() {
        let affected: Vec<String> = layer.iter().filter_map(|v| v.location.clone()).collect();
        fixes.push(ActionableFix {
            priority: "medium".to_string(),
            description: "Introduce proper service layers to abstract direct database access"
                .to_string(),
            impact: "Improves separation of concerns and testability".to_string(),
            affected_components: affected,
        });
    }

    let god: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::GodModule))
        .collect();
    if !god.is_empty() {
        let affected: Vec<String> = god.iter().filter_map(|v| v.location.clone()).collect();
        fixes.push(ActionableFix {
            priority: "high".to_string(),
            description: "Decouple Bottlenecks (God Modules) to reduce fan-in/fan-out gravity".to_string(),
            impact: "High regression risk; modifying these components affects many distinct areas of the system, slowing down delivery.".to_string(),
            affected_components: affected,
        });
    }

    let orphans: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::OrphanComponent))
        .collect();
    if !orphans.is_empty() {
        let affected: Vec<String> = orphans.iter().filter_map(|v| v.location.clone()).collect();
        fixes.push(ActionableFix {
            priority: "low".to_string(),
            description: "Review orphan modules - integrate or remove unused code".to_string(),
            impact: "Reduces dead code and technical debt".to_string(),
            affected_components: affected,
        });
    }

    fixes.truncate(3);
    fixes
}

fn print_quickstart_summary(report: &sruja_diff::DriftReport, graph: &Graph, repo: &str) {
    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!("{}", "📊 Architecture Inventory".cyan().bold());
    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!("  Repository: {}", repo.green());
    println!();
    println!("  Components detected:");
    println!(
        "    • {} modules",
        report.total_modules.to_string().yellow()
    );
    println!(
        "    • {} services",
        report.total_services.to_string().yellow()
    );
    println!(
        "    • {} databases",
        report.total_databases.to_string().yellow()
    );
    let external_apis = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::ExternalApi)
        .count();
    println!("    • {} external APIs", external_apis.to_string().yellow());
    println!(
        "    • {} total dependencies",
        report.total_dependencies.to_string().yellow()
    );
    println!();

    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    let score_str = format!("{}/100", report.health_score);
    let colored_score = match report.health_score {
        80..=100 => score_str.green().bold(),
        60..=79 => score_str.yellow().bold(),
        _ => score_str.red().bold(),
    };
    println!(
        "💚 Architecture Health Score (structural only): {}",
        colored_score
    );
    println!("{}", "─".repeat(70).truecolor(100, 100, 100));

    let score_bar = match report.health_score {
        80..=100 => "████████████████████ ✓ Good".green(),
        60..=79 => "██████████████░░░░░░ ⚠ Fair".yellow(),
        40..=59 => "██████████░░░░░░░░░░ ⚠ Needs Work".truecolor(255, 140, 0),
        _ => "████░░░░░░░░░░░░░░░░ ✗ Critical".red(),
    };
    println!("  {}", score_bar);
    println!();

    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!("{}", "🔍 Top 3 Structural Findings".red().bold());
    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!(
        "  {}",
        "(Heuristic; no baseline. For actionable insights, add repo.sruja and run drift, or use the sruja-architecture skill.)"
            .truecolor(120, 120, 120)
    );
    println!();

    let mut sorted: Vec<_> = report.violations.iter().collect();
    sorted.sort_by(|a, b| {
        let severity_order = |s: &sruja_diff::Severity| match s {
            sruja_diff::Severity::Error => 0,
            sruja_diff::Severity::Warning => 1,
            sruja_diff::Severity::Info => 2,
        };
        severity_order(&a.severity).cmp(&severity_order(&b.severity))
    });

    for (i, v) in sorted.iter().take(3).enumerate() {
        let (icon, msg) = match v.severity {
            sruja_diff::Severity::Error => ("🚨", v.message.red().bold()),
            sruja_diff::Severity::Warning => ("⚠️", v.message.yellow().bold()),
            sruja_diff::Severity::Info => ("ℹ️", v.message.cyan().bold()),
        };
        println!();
        println!("  {}. {} {}", i + 1, icon, msg);
        if let Some(ref loc) = v.location {
            // Find actual path or sanitize ID to look like a path
            let display_loc = graph
                .nodes
                .iter()
                .find(|n| &n.id == loc)
                .map(|n| n.path.as_deref().unwrap_or(loc))
                .unwrap_or(loc)
                .replace("_", "/");
            println!(
                "     📍 Component: {}",
                display_loc.truecolor(180, 180, 180)
            );
        }
        if let Some(ref s) = v.suggestion {
            println!("     💡 Suggestion: {}", s.italic());
        }
    }

    if sorted.is_empty() {
        println!();
        println!("  ✓ No critical issues found!");
    }
    println!();

    let fixes = generate_actionable_fixes_from_violations(&report.violations);

    if !fixes.is_empty() {
        println!("{}", "─".repeat(70).truecolor(100, 100, 100));
        println!("{}", "🎯 Top Actionable Fixes".green().bold());
        println!("{}", "─".repeat(70).truecolor(100, 100, 100));

        for (i, fix) in fixes.iter().enumerate() {
            let (priority_icon, priority_color) = match fix.priority.as_str() {
                "high" => ("🔴", fix.priority.to_uppercase().red()),
                "medium" => ("🟡", fix.priority.to_uppercase().yellow()),
                _ => ("🟢", fix.priority.to_uppercase().cyan()),
            };

            println!();
            println!(
                "  {}. {} [{}] {}",
                i + 1,
                priority_icon,
                priority_color,
                fix.description.bold()
            );
            println!("     Impact: {}", fix.impact.italic());
            if !fix.affected_components.is_empty() {
                let display_affected: Vec<_> = fix
                    .affected_components
                    .iter()
                    .map(|c| c.replace("_", "/"))
                    .collect();
                println!(
                    "     Affected: {}",
                    display_affected.join(", ").truecolor(180, 180, 180)
                );
            }
        }
        println!();
    }

    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!("{}", "🗺️  High-Level Domain Map".magenta().bold());
    println!("{}", "─".repeat(70).truecolor(100, 100, 100));

    let mut domains: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for node in &graph.nodes {
        let path_str = node.path.as_deref().unwrap_or(&node.id);
        let normalized = path_str.replace(['\\', '_'], "/");
        let parts: Vec<&str> = normalized
            .split('/')
            .filter(|p| !p.is_empty() && *p != ".")
            .collect();
        if parts.is_empty() {
            continue;
        }

        let mut domain_name = parts[0].to_string();
        if (parts[0] == "crates"
            || parts[0] == "packages"
            || parts[0] == "src"
            || parts[0] == "internal")
            && parts.len() > 1
        {
            domain_name = format!("{}/{}", parts[0], parts[1]);
        }
        *domains.entry(domain_name).or_insert(0) += 1;
    }

    let mut sorted_domains: Vec<_> = domains.into_iter().collect();
    sorted_domains.sort_by(|a, b| b.1.cmp(&a.1));

    if sorted_domains.is_empty() {
        println!("\n  No clear domains identified.");
    } else {
        println!();
        let max_items = 10;
        let total = sorted_domains.len();
        for (i, (domain, count)) in sorted_domains.iter().take(max_items).enumerate() {
            let is_last = i == max_items - 1 || i == sorted_domains.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            println!(
                "  {} 📂 {} ({} components)",
                prefix.truecolor(100, 100, 100),
                domain.cyan().bold(),
                count.to_string().yellow()
            );
        }
        if total > max_items {
            println!(
                "  {} ... and {} more",
                "└──".truecolor(100, 100, 100),
                (total - max_items).to_string().truecolor(100, 100, 100)
            );
        }
    }

    println!();

    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!("{}", "🚀 Next Steps".blue().bold());
    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!();
    println!(
        "  1. {}",
        "Add a baseline for actionable drift: generate repo.sruja (e.g. use sruja-architecture skill), then run 'sruja drift -r . -a repo.sruja'".white()
    );
    println!(
        "  2. {}",
        "Run 'sruja drift -r . --format json' for full structural analysis (no baseline)".white()
    );
    println!(
        "  3. {}",
        "Run 'sruja scan . --output sruja.graph.json' to save the inferred graph".white()
    );
    println!(
        "  4. {}",
        "Run 'sruja impact <node> -r .' to explore change risk (blast radius)".white()
    );
    println!();
    println!("{}", "═".repeat(70).truecolor(100, 100, 100));
}

pub(crate) fn print_diff_text(result: &sruja_diff::DiffResult, violations_only: bool) {
    println!("{}", "═".repeat(60));
    println!("Baseline Drift: Scan vs DSL");
    println!("{}", "═".repeat(60));
    println!();

    if !violations_only {
        println!("📊 Summary");
        println!("{}", "-".repeat(40));
        let s = &result.summary;
        println!(
            "  Proposed: {} | Actual (scan): {}",
            s.proposed_components, s.existing_components
        );
        println!(
            "  New: {} | Missing: {} | Edges +{} -{}",
            s.new_components, s.missing_components, s.new_dependencies, s.removed_dependencies
        );
        println!("  Health Score: {}/100", s.health_score);
        println!();
    }

    let errors: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Error))
        .collect();
    let warnings: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Warning))
        .collect();
    let info: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Info))
        .collect();

    if !errors.is_empty() {
        println!("🚨 Errors ({})", errors.len());
        println!("{}", "-".repeat(40));
        for v in &errors {
            println!("  ✗ {}", v.message);
            if let Some(ref suggestion) = v.suggestion {
                println!("    → {}", suggestion);
            }
            print_violation_sources(v);
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("⚠️  Warnings ({})", warnings.len());
        println!("{}", "-".repeat(40));
        for v in &warnings {
            println!("  ⚠ {}", v.message);
            if let Some(ref suggestion) = v.suggestion {
                println!("    → {}", suggestion);
            }
            print_violation_sources(v);
        }
        println!();
    }

    if !violations_only && !info.is_empty() {
        println!("ℹ️  Info ({})", info.len());
        println!("{}", "-".repeat(40));
        for v in &info {
            println!("  ℹ {}", v.message);
            print_violation_sources(v);
        }
        println!();
    }

    if !violations_only && !result.suggestions.is_empty() {
        println!("💡 Suggestions");
        println!("{}", "-".repeat(40));
        for (i, s) in result.suggestions.iter().enumerate() {
            println!("  {}. {}", i + 1, s);
        }
        println!();
    }

    println!("{}", "═".repeat(60));
}

fn print_violation_sources(v: &sruja_diff::Violation) {
    if !v.sources.is_empty() {
        let refs: Vec<String> = v
            .sources
            .iter()
            .map(sruja_diff::SourceRef::display_string)
            .collect();
        println!("    📎 Sources: {}", refs.join(", "));
    }
}

pub(crate) fn print_drift_text(result: &sruja_diff::DriftReport, violations_only: bool) {
    println!("{}", "═".repeat(60));
    println!("Architecture Drift Detection");
    println!("{}", "═".repeat(60));
    println!();

    if !violations_only {
        println!("📊 Summary");
        println!("{}", "-".repeat(40));
        println!(
            "  Modules: {} | Services: {} | Databases: {}",
            result.total_modules, result.total_services, result.total_databases
        );
        println!("  Dependencies: {}", result.total_dependencies);
        println!(
            "  Health Score (structural only): {}/100",
            result.health_score
        );
        println!();
    }

    let errors: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Error))
        .collect();
    let warnings: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Warning))
        .collect();
    let info: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Info))
        .collect();

    if !errors.is_empty() {
        println!("🚨 Errors ({})", errors.len());
        println!("{}", "-".repeat(40));
        for v in &errors {
            println!("  ✗ {}", v.message);
            if let Some(ref suggestion) = v.suggestion {
                println!("    → {}", suggestion);
            }
            print_violation_sources(v);
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("⚠️  Warnings ({})", warnings.len());
        println!("{}", "-".repeat(40));
        for v in &warnings {
            println!("  ⚠ {}", v.message);
            if let Some(ref suggestion) = v.suggestion {
                println!("    → {}", suggestion);
            }
            print_violation_sources(v);
        }
        println!();
    }

    if !violations_only && !info.is_empty() {
        println!("ℹ️  Info ({})", info.len());
        println!("{}", "-".repeat(40));
        for v in &info {
            println!("  ℹ {}", v.message);
            print_violation_sources(v);
        }
        println!();
    }

    if !violations_only && !result.suggestions.is_empty() {
        println!("💡 Suggestions");
        println!("{}", "-".repeat(40));
        for (i, s) in result.suggestions.iter().enumerate() {
            println!("  {}. {}", i + 1, s);
        }
        println!();
    }

    println!("{}", "═".repeat(60));
}

pub async fn quickstart(
    repo_root: &str,
    format: &str,
    generate_baseline: bool,
    fail_on: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    eprintln!("{}", "═".repeat(70).truecolor(100, 100, 100));
    eprintln!(
        "{}",
        "🚀 Sruja Quickstart - Context Engineering".green().bold()
    );
    eprintln!("{}", "═".repeat(70).truecolor(100, 100, 100));
    eprintln!();

    eprintln!("📂 Scanning repository...");
    let graph = scan_repo(repo_path)?;
    eprintln!("   ✓ Found {} components", graph.nodes.len());
    let (_, scan_scope) = resolve_scan_scope(repo_path);

    let languages = detect_languages(repo_path);
    let primary_language = languages
        .first()
        .map(|(l, _)| l.as_str())
        .unwrap_or("Unknown");
    let framework = detect_framework(repo_path, primary_language);
    let (is_monolith, is_microservices) = detect_architecture_style(&graph);
    let context = build_repo_context(repo_path, &graph);

    eprintln!();
    eprintln!("📊 Repository Context");
    eprintln!("   • Primary Language: {}", primary_language.cyan());
    if let Some(ref fw) = framework {
        eprintln!("   • Framework: {}", fw.cyan());
    }
    if is_microservices {
        eprintln!("   • Architecture: {}", "Microservices".cyan());
    } else if is_monolith {
        eprintln!("   • Architecture: {}", "Monolith".cyan());
    }
    if let Some(ref domain) = context.domain {
        eprintln!("   • Domain: {}", domain.cyan());
    }
    eprintln!();

    eprintln!("🔍 Analyzing architecture health...");
    let drift_report = sruja_diff::detect_architectural_drift(&graph);
    eprintln!("   ✓ Analysis complete");
    eprintln!();

    // If a baseline exists, compute truth_status from scan vs DSL.
    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let truth_status = if let Some(ref p) = baseline_path {
        match truth_status_from_baseline_compare(&graph, p) {
            Ok(s) => match s {
                sruja_diff::TruthStatus::Reviewed => "reviewed",
                sruja_diff::TruthStatus::Drifted => "drifted",
                sruja_diff::TruthStatus::Unknown => "unknown",
            }
            .to_string(),
            Err(_) => "unknown".to_string(),
        }
    } else {
        "unknown".to_string()
    };

    if generate_baseline {
        match write_draft_baseline(repo_path, &graph)? {
            Some(p) => {
                eprintln!("📝 Wrote draft baseline: {}", p.to_string_lossy().cyan());
                eprintln!();
            }
            None => {
                eprintln!("📝 Baseline already exists (repo.sruja). Skipping write.");
                eprintln!();
            }
        }
    }

    match format {
        "json" => {
            let mut output =
                QuickstartResult::from_drift_report(&drift_report, &graph, repo_root, &scan_scope);
            output.truth_status = truth_status;
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            if baseline_path.is_some() {
                eprintln!("🧭 Truth (baseline present): {}", truth_status.cyan());
                eprintln!();
            }
            print_quickstart_summary(&drift_report, &graph, repo_root);
        }
    }

    if should_fail_on_violations(fail_on, &drift_report.violations) {
        std::process::exit(1);
    }

    Ok(())
}

/// PR-scoped drift: compare base and head refs to find NEW violations
pub async fn drift_pr(
    repo_root: &str,
    base_ref: Option<&str>,
    head_ref: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let base = base_ref.unwrap_or("origin/main");
    let head = head_ref.unwrap_or("HEAD");

    eprintln!("🔍 PR-Scoped Drift Detection");
    eprintln!("   Base: {} | Head: {}", base, head);
    eprintln!();

    // Check if git is available
    let git_ok = std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(repo_path)
        .output()
        .ok()
        .and_then(|o| o.status.success().then_some(()))
        .is_some();

    if !git_ok {
        return Err(CliError::Validation(
            "Not a git repository. PR-scoped drift requires git.".to_string(),
        ));
    }

    // Get list of changed files between base and head
    let changed_files_output = std::process::Command::new("git")
        .args(["diff", "--name-only", &format!("{}...{}", base, head)])
        .current_dir(repo_path)
        .output()
        .map_err(|e| {
            CliError::Io(std::io::Error::other(format!(
                "Failed to get changed files: {}",
                e
            )))
        })?;

    let changed_files: Vec<String> = String::from_utf8_lossy(&changed_files_output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .collect();

    if changed_files.is_empty() {
        eprintln!("✅ No changed files detected between {} in {}", base, head);
        return Ok(());
    }

    eprintln!("📝 Changed files: {}", changed_files.len());

    // Head graph: use cache by commit SHA if present (incremental / CI retry)
    let head_sha_output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .ok();
    let head_sha = head_sha_output
        .and_then(|o| {
            o.status
                .success()
                .then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_default();
    let cache_dir = repo_path.join(".sruja").join("cache");
    let _ = fs::create_dir_all(&cache_dir);
    let head_cache_path = if !head_sha.is_empty() {
        cache_dir.join(format!("head_{}.json", head_sha))
    } else {
        PathBuf::new() // sentinel: no cache
    };
    let head_graph = if !head_cache_path.as_os_str().is_empty() && head_cache_path.exists() {
        eprintln!(
            "📂 Using cached head graph ({})",
            &head_sha[..head_sha.len().min(8)]
        );
        let content = fs::read_to_string(&head_cache_path)?;
        serde_json::from_str(&content).map_err(CliError::Json)?
    } else {
        let g = scan_repo(repo_path)?;
        if !head_cache_path.as_os_str().is_empty() {
            if let Ok(json) = serde_json::to_string_pretty(&g) {
                let _ = fs::write(&head_cache_path, json);
            }
        }
        g
    };
    let head_drift = sruja_diff::detect_architectural_drift(&head_graph);

    // Base graph: from cache, or checkout base in a temp worktree and scan (so base != head)
    let cache_filename = base.replace(['/', '.'], "_");
    let cache_path = cache_dir.join(format!("{}.json", cache_filename));
    let base_graph = if cache_path.exists() {
        let content = fs::read_to_string(&cache_path)?;
        serde_json::from_str(&content).map_err(CliError::Json)?
    } else {
        // No cache: checkout base ref in a temp worktree and scan there
        let worktree_dir =
            std::env::temp_dir().join(format!("sruja-drift-base-{}", std::process::id()));
        if worktree_dir.exists() {
            let _ = fs::remove_dir_all(&worktree_dir);
        }
        let status = std::process::Command::new("git")
            .arg("worktree")
            .arg("add")
            .arg("--detach")
            .arg(worktree_dir.as_path())
            .arg(base)
            .current_dir(repo_path)
            .status()
            .map_err(|e| {
                CliError::Io(std::io::Error::other(format!(
                    "git worktree add failed (is '{}' a valid ref?): {}",
                    base, e
                )))
            })?;
        if !status.success() {
            return Err(CliError::Validation(format!(
                "Could not checkout base ref '{}'. Run a full scan on base and save to .sruja/cache/{}.json, or ensure the ref exists.",
                base, cache_filename
            )));
        }
        let base_graph = scan_repo(&worktree_dir).map_err(|e| {
            let _ = std::process::Command::new("git")
                .arg("worktree")
                .arg("remove")
                .arg("--force")
                .arg(worktree_dir.as_path())
                .current_dir(repo_path)
                .status();
            CliError::Scan(e.to_string())
        })?;
        let _ = std::process::Command::new("git")
            .arg("worktree")
            .arg("remove")
            .arg("--force")
            .arg(worktree_dir.as_path())
            .current_dir(repo_path)
            .status();
        base_graph
    };

    // Compare: what violations exist in head that don't exist in base
    let base_drift = sruja_diff::detect_architectural_drift(&base_graph);

    let new_violations: Vec<_> = head_drift
        .violations
        .iter()
        .filter(|hv| {
            !base_drift.violations.iter().any(|bv| {
                bv.kind == hv.kind && bv.message == hv.message && bv.location == hv.location
            })
        })
        .collect();

    let result = PrDriftResult {
        base_ref: base_ref.unwrap_or("origin/main").to_string(),
        head_ref: head_ref.unwrap_or("HEAD").to_string(),
        changed_files,
        base_health: base_drift.health_score,
        head_health: head_drift.health_score,
        new_violations: new_violations
            .iter()
            .map(|v| PrViolation {
                severity: format!("{:?}", v.severity),
                kind: format!("{:?}", v.kind),
                message: v.message.clone(),
                location: v.location.clone(),
                suggestion: v.suggestion.clone(),
            })
            .collect(),
        base_violations_count: base_drift.violations.len(),
        head_violations_count: head_drift.violations.len(),
    };

    match format {
        "json" => {
            let output = serde_json::to_string_pretty(&result)?;
            println!("{}", output);
        }
        "github-actions" => {
            print_github_actions_output(&result);
        }
        _ => {
            print_pr_drift_text(&result);
        }
    }

    // Exit with non-zero so CI fails when this PR introduces new violations
    if !result.new_violations.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}
fn print_pr_drift_text(result: &PrDriftResult) {
    println!("{}", "═".repeat(70));
    println!("🔍 PR-Scoped Drift Detection");
    println!("{}", "═".repeat(70));
    println!();
    println!("Base: {} → Head: {}", result.base_ref, result.head_ref);
    println!("Changed files: {}", result.changed_files.len());
    println!();

    println!("📊 Health Score Change");
    println!("{}", "-".repeat(40));
    if result.head_health < result.base_health {
        println!(
            "  {} → {} (⚠️ -{})",
            result.base_health,
            result.head_health,
            result.base_health - result.head_health
        );
    } else if result.head_health > result.base_health {
        println!(
            "  {} → {} (✓ +{})",
            result.base_health,
            result.head_health,
            result.head_health - result.base_health
        );
    } else {
        println!(
            "  {} → {} (no change)",
            result.base_health, result.head_health
        );
    }
    println!();

    if result.new_violations.is_empty() {
        println!("{}", "-".repeat(40));
        println!("✅ No NEW architectural violations introduced in this PR!");
        println!("{}", "-".repeat(40));
        println!();
        println!(
            "Existing violations: {} (base) → {} (head)",
            result.base_violations_count, result.head_violations_count
        );
    } else {
        println!(
            "🚨 NEW Violations Introduced in This PR ({})",
            result.new_violations.len()
        );
        println!("{}", "-".repeat(40));

        for v in &result.new_violations {
            let icon = match v.severity.as_str() {
                "Error" => "❌",
                "Warning" => "⚠️",
                _ => "ℹ️",
            };
            println!();
            println!("  {} [{}] {}", icon, v.severity.to_uppercase(), v.message);
            if let Some(ref loc) = v.location {
                println!("     📍 {}", loc);
            }
            if let Some(ref s) = v.suggestion {
                println!("     💡 {}", s);
            }
        }

        if result.new_violations.len() > 3 {
            println!();
            println!("     ... and {} more", result.new_violations.len() - 3);
        }

        println!();
        println!(
            "⚠️  This PR introduces {} new violation(s). Consider fixing before merge.",
            result.new_violations.len()
        );
        println!();
    }

    println!("{}", "═".repeat(70));
}
fn print_github_actions_output(result: &PrDriftResult) {
    for v in &result.new_violations {
        let level = match v.severity.as_str() {
            "Error" => "error",
            "Warning" => "warning",
            _ => "notice",
        };
        if let Some(ref loc) = v.location {
            println!(
                "::{} file={},title=Sruja {}::{}",
                level, loc, v.kind, v.message
            );
        } else {
            println!("::{} title=Sruja {}::{}", level, v.kind, v.message);
        }
    }

    if result.new_violations.is_empty() {
        println!(
            "::notice title=Sruja::✅ No new architectural violations. Health: {} → {}",
            result.base_health, result.head_health
        );
    } else {
        println!(
            "::error title=Sruja::🚨 {} new violation(s) introduced. Health: {} → {}",
            result.new_violations.len(),
            result.base_health,
            result.head_health
        );
    }
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PrDriftResult {
    base_ref: String,
    head_ref: String,
    changed_files: Vec<String>,
    base_health: u8,
    head_health: u8,
    new_violations: Vec<PrViolation>,
    base_violations_count: usize,
    head_violations_count: usize,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PrViolation {
    severity: String,
    kind: String,
    message: String,
    location: Option<String>,
    suggestion: Option<String>,
}
