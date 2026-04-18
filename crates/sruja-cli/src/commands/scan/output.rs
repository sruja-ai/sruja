use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::CliError;
use sruja_scan::{is_path_production_relevant, EdgeKind, Graph, NodeKind};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct QuickstartResult {
    pub repo: String,
    pub scan_scope: sruja_scan::scan_scope::ScanScope,
    pub health_score: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_breakdown: Option<sruja_diff::HealthScoreBreakdown>,
    pub inventory: InventorySummary,
    pub top_findings: Vec<Finding>,
    pub actionable_fixes: Vec<ActionableFix>,
    pub truth_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_quality: Option<ScanQuality>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScanQuality {
    pub confidence_score: u8,
    pub coverage_percent: u8,
    pub manifest_discoveries: usize,
    pub entry_point_count: usize,
    pub leaf_node_count: usize,
    pub orphan_count: usize,
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StatusOutput {
    pub baseline: Option<String>,
    pub truth_status: String,
    pub violations_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_score: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_findings: Vec<Finding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub health_history: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PrDriftResult {
    pub base_ref: String,
    pub head_ref: String,
    pub changed_files: Vec<String>,
    pub base_health: u8,
    pub head_health: u8,
    pub new_violations: Vec<PrViolation>,
    pub base_violations_count: usize,
    pub head_violations_count: usize,
    pub component_diffs: Vec<sruja_diff::ComponentDiff>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PrViolation {
    pub severity: String,
    pub kind: String,
    pub message: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
}

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

pub(crate) fn relation_label_for_edge(edge_kind: EdgeKind) -> &'static str {
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
    let mut nodes = graph
        .nodes
        .iter()
        .filter(|n| {
            // Filter by production relevance
            if let Some(p) = n.path.as_deref() {
                if !path_production_relevant(p) {
                    return false;
                }
            }

            // Filter out low-level noise:
            // - Symbols starting with lowercase (likely helper functions)
            // - Very specific internal types if they follow a pattern
            // - For now, we'll keep mostly "high-level" kinds
            match n.kind {
                NodeKind::Service
                | NodeKind::Database
                | NodeKind::ExternalApi
                | NodeKind::Frontend
                | NodeKind::Container => true,
                NodeKind::Module => {
                    // If it looks like a module-level item but not a deeply nested symbol
                    // Heuristic: skip if it contains common noisy suffixes
                    let noise = [
                        "Summary", "Output", "Baseline", "Config", "Options", "Result",
                    ];
                    !noise.iter().any(|&s| n.id.ends_with(s))
                }
                _ => false, // Skip Raw/Component/etc if too noisy
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    let allowed_ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    let mut edges = graph
        .edges
        .iter()
        .filter(|e| {
            allowed_ids.contains(e.source.as_str()) && allowed_ids.contains(e.target.as_str())
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut id_map: HashMap<String, String> = HashMap::new();
    let mut used: HashMap<String, usize> = HashMap::new();

    for node in &nodes {
        let mut base = sanitize_identifier(&node.id);
        let n = used.entry(base.clone()).or_insert(0);
        if *n > 0 {
            base = format!("{}_{}", base, *n + 1);
        }
        *n += 1;
        id_map.insert(node.id.clone(), base);
    }

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
    let repo_name = if let Some(parent) = Path::new(filename).parent() {
        if parent.as_os_str().is_empty() || parent == Path::new(".") {
            std::env::current_dir()
                .ok()
                .and_then(|d| {
                    d.file_name()
                        .and_then(|n| n.to_str().map(|s| s.to_string()))
                })
                .unwrap_or_else(|| "MySystem".to_string())
        } else {
            parent
                .file_name()
                .and_then(|n| n.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "MySystem".to_string())
        }
    } else {
        "MySystem".to_string()
    };
    let system_name = sanitize_identifier(&repo_name);

    let mut system_body = sruja_language::ElementDefBody {
        description: Some(format!("The {} system architecture", repo_name)),
        ..Default::default()
    };
    let mut system_items = Vec::new();

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
                    let mut tech = node
                        .technology
                        .clone()
                        .unwrap_or_else(|| "Unknown".to_string());
                    if tech == "Unknown" {
                        if let Some(path) = &node.path {
                            let normalized = path.replace('\\', "/");
                            if normalized.contains("package.json") {
                                tech = "Node.js".to_string();
                            } else if normalized.ends_with(".rs") {
                                tech = "Rust".to_string();
                            } else if normalized.ends_with(".go") {
                                tech = "Go".to_string();
                            }
                        }
                    }
                    Some(tech)
                }
                _ => node.technology.clone(),
            },
            ..Default::default()
        };

        let assignment = sruja_language::ElementAssignment {
            location: sruja_diagnostics::SourceLocation::new(filename.to_string(), 1, 1),
            name,
            kind: match kind {
                sruja_language::ElementKind::System => sruja_language::ElementKind::Container,
                k => k,
            },
            sub_kind,
            title: Some(node.label.clone()),
            tag_refs: Vec::new(),
            body: Some(body),
        };
        let def = sruja_language::ElementDef {
            location: sruja_diagnostics::SourceLocation::new(filename.to_string(), 1, 1),
            assignment,
        };

        system_items.push(sruja_language::ElementDefBodyItem::ElementDef(Box::new(
            def,
        )));
    }

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
        system_items.push(sruja_language::ElementDefBodyItem::Relation(rel));
    }

    system_body.items = system_items;
    let system_def = sruja_language::ElementDef {
        location: sruja_diagnostics::SourceLocation::new(filename.to_string(), 1, 1),
        assignment: sruja_language::ElementAssignment {
            location: sruja_diagnostics::SourceLocation::new(filename.to_string(), 1, 1),
            name: system_name,
            kind: sruja_language::ElementKind::System,
            sub_kind: None,
            title: Some(repo_name.to_string()),
            tag_refs: Vec::new(),
            body: Some(system_body),
        },
    };
    items.push(sruja_language::TopLevelItem::ElementDef(Box::new(
        system_def,
    )));

    sruja_language::Program::new().with_items(items)
}

pub fn write_draft_baseline(
    repo_root: &Path,
    graph: &Graph,
    force: bool,
) -> Result<Option<PathBuf>, CliError> {
    let out_path = repo_root.join("repo.sruja");
    if out_path.exists() && !force {
        return Ok(None);
    }
    let program = build_draft_program_from_graph(graph, out_path.to_string_lossy().as_ref());
    let printer = sruja_export::DslPrinter::new();
    let dsl = printer.print(&program);
    let header = r#"// Sruja Architecture Baseline
// Generated automatically from codebase analysis.
//
// Next Steps:
// 1. Review the elements and relationships below.
// 2. Refine the names, descriptions, and technologies.
// 3. Add 'source' bindings to your OpenAPI/Docs/K8s manifests.
// 4. Run 'sruja lint repo.sruja' to validate.

"#;
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

pub(crate) fn print_violations_github_actions(violations: &[sruja_diff::Violation]) {
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

pub(crate) fn generate_actionable_fixes_from_violations(
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

pub(crate) fn print_quickstart_summary(
    report: &sruja_diff::DriftReport,
    graph: &Graph,
    repo: &str,
) {
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

    if let Some(quality) = calculate_scan_quality_internal(graph) {
        println!("{}", "─".repeat(70).truecolor(100, 100, 100));
        let quality_score_str = format!("{}/100", quality.confidence_score);
        let colored_quality = match quality.confidence_score {
            80..=100 => quality_score_str.green().bold(),
            60..=79 => quality_score_str.yellow().bold(),
            _ => quality_score_str.red().bold(),
        };
        println!(
            "💎 Scanner Confidence & Trust Score:     {}",
            colored_quality
        );
        println!("{}", "─".repeat(70).truecolor(100, 100, 100));
        println!(
            "  Discovery Coverage: {} | Manifests: {} | Entrypoints: {}",
            format!("{}%", quality.coverage_percent).cyan(),
            quality.manifest_discoveries.to_string().cyan(),
            quality.entry_point_count.to_string().cyan()
        );
        println!(
            "  Structural Integrity: Nodes: {} | Leaves: {} | Orphans: {}",
            graph.nodes.len().to_string().cyan(),
            quality.leaf_node_count.to_string().cyan(),
            quality.orphan_count.to_string().red()
        );
        println!();
    }

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

    let mut domains: HashMap<String, usize> = HashMap::new();
    for node in &graph.nodes {
        let path_str = node.path.as_deref().unwrap_or(&node.id);

        // Strip repo root or leading /tmp/ junk
        let mut relative_path = path_str;
        if let Some(stripped) = path_str.strip_prefix(repo) {
            relative_path = stripped;
        }
        let normalized = relative_path.replace(['\\', '_'], "/");
        let parts: Vec<&str> = normalized
            .split('/')
            .filter(|p| {
                !p.is_empty() && *p != "." && *p != "tmp" && *p != "node_modules" && *p != ".git"
            })
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

pub(crate) fn print_violation_sources(v: &sruja_diff::Violation) {
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

pub(crate) fn print_pr_drift_text(result: &PrDriftResult) {
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

    if !result.component_diffs.is_empty() {
        println!("🏗️  Component Impact");
        println!("{}", "-".repeat(40));
        for diff in &result.component_diffs {
            let files = if diff.files_changed.len() == 1 {
                "file"
            } else {
                "files"
            };
            println!(
                "  {} [{} {}, +{}, -{}]",
                diff.component_id.yellow().bold(),
                diff.files_changed.len(),
                files,
                diff.lines_added.to_string().green(),
                diff.lines_deleted.to_string().red()
            );
        }
        println!();
    }

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

        for (i, v) in result.new_violations.iter().enumerate() {
            let icon = match v.severity.as_str() {
                "Error" => "❌",
                "Warning" => "⚠️",
                "Info" | "Notice" => "ℹ️",
                _ => "ℹ️",
            };
            println!();
            println!(
                "  {}. {} [{}] {}",
                i + 1,
                icon,
                v.severity.to_uppercase(),
                v.message
            );
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

pub(crate) fn print_github_actions_output(result: &PrDriftResult) {
    for v in &result.new_violations {
        let level = match v.severity.as_str() {
            "Error" => "error",
            "Warning" => "warning",
            "Info" | "Notice" => "notice",
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

    // Confidence Score Calculation (Heuristic)
    // 1. Coverage: Ratio of nodes with evidence vs total
    let nodes_with_evidence = graph.nodes.iter().filter(|n| !n.sources.is_empty()).count();
    let coverage_percent = if !graph.nodes.is_empty() {
        ((nodes_with_evidence as f32 / graph.nodes.len() as f32) * 100.0) as u8
    } else {
        0
    };

    // 2. Score: coverage + bonus for manifests - penalty for orphans
    let mut score = coverage_percent as i32;
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
