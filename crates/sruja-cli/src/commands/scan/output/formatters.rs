use std::collections::HashMap;

use colored::Colorize;
use sruja_diff;
use sruja_scan::{Graph, NodeKind};

use super::helpers::{
    calculate_scan_quality_internal, escape_github_actions_message, violation_best_file_line,
    violation_kind_str,
};
use super::types::{ActionableFix, PrDriftResult};

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
            description: "Decouple Bottlenecks (God Modules) to reduce fan-in/fan-out gravity"
                .to_string(),
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
        .filter(|n| n.kind == NodeKind::EXTERNAL_API)
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
    sorted_domains.sort_by_key(|b| std::cmp::Reverse(b.1));

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
        "Author reviewed architecture: sruja quickstart -r . --generate-baseline writes repo.sruja.draft (structural evidence); use the sruja-architecture skill to shape repo.sruja, then 'sruja drift -r . -a repo.sruja'".white()
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

        let mode = s.baseline_mode.unwrap_or(sruja_diff::BaselineMode::Auto);
        if mode == sruja_diff::BaselineMode::Overview {
            let coverage = s.baseline_coverage_percent.unwrap_or(0.0);
            println!(
                "  Baseline Coverage: {:.1}% (overview baseline; not exhaustive)",
                coverage
            );
            println!(
                "  New: {} | Edges +{} -{}",
                s.new_components, s.new_dependencies, s.removed_dependencies
            );
            println!("  Health Score (structural only): {}/100", s.health_score);
            println!("  Note: to measure inventory drift, use an exhaustive baseline generated from scan output.");
        } else {
            println!(
                "  New: {} | Missing: {} | Edges +{} -{}",
                s.new_components, s.missing_components, s.new_dependencies, s.removed_dependencies
            );
            println!("  Health Score: {}/100", s.health_score);
        }
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

pub(crate) fn print_scan_scope_summary(scope: &sruja_scan::scan_scope::ScanScope) {
    println!("📂 Scan scope");
    println!("{}", "-".repeat(40));
    println!("  Files scanned: {}", scope.total_files);
    if !scope.included.is_empty() {
        let preview: Vec<_> = scope.included.iter().take(8).cloned().collect();
        let suffix = if scope.included.len() > preview.len() {
            format!(" (+{} more)", scope.included.len() - preview.len())
        } else {
            String::new()
        };
        println!("  Included areas: {}{}", preview.join(", "), suffix);
    }
    if !scope.excluded.is_empty() {
        println!(
            "  Excluded dirs: {} (see scan_scope in JSON for full list)",
            scope.excluded.len()
        );
    }
    println!();
}

pub(crate) fn print_could_not_infer_section(items: &[String]) {
    if items.is_empty() {
        return;
    }
    println!("Could not infer (limitations)");
    println!("{}", "-".repeat(40));
    for item in items {
        println!("  • {}", item);
    }
    println!();
}

pub(crate) fn print_drift_text(
    result: &sruja_diff::DriftReport,
    _graph: Option<&Graph>,
    violations_only: bool,
    advisory: bool,
    could_not_infer: &[String],
) {
    println!("{}", "═".repeat(60));
    println!("Structural scan (topology from code)");
    if advisory {
        println!("  Mode: advisory (orphan info suppressed)");
    }
    println!("{}", "═".repeat(60));
    println!();

    if !violations_only {
        print_scan_scope_summary(&result.scan_scope);
    }

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
        if let Some(ref breakdown) = result.health_breakdown {
            println!(
                "  Penalties — cycles: {} | layers: {} | god-modules: {} | orphans: {}",
                breakdown.cycle_penalty,
                breakdown.layer_penalty,
                breakdown.god_module_penalty,
                breakdown.orphan_penalty
            );
        }
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

    let print_violation = |v: &sruja_diff::Violation, icon: &str| {
        let (file, line) = violation_best_file_line(v);
        match (file, line) {
            (Some(f), Some(l)) => println!("  {} {} ({}:{})", icon, v.message, f, l),
            (Some(f), None) => println!("  {} {} ({})", icon, v.message, f),
            _ => println!("  {} {}", icon, v.message),
        }
        if let Some(ref suggestion) = v.suggestion {
            println!("    → {}", suggestion);
        }
        print_violation_sources(v);
    };

    if !errors.is_empty() {
        println!("🚨 Errors ({})", errors.len());
        println!("{}", "-".repeat(40));
        for v in &errors {
            print_violation(v, "✗");
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("⚠️  Warnings ({})", warnings.len());
        println!("{}", "-".repeat(40));
        for v in &warnings {
            print_violation(v, "⚠");
        }
        println!();
    }

    if !violations_only && !info.is_empty() {
        println!("ℹ️  Info ({})", info.len());
        println!("{}", "-".repeat(40));
        for v in &info {
            print_violation(v, "ℹ");
        }
        println!();
    }

    if !violations_only && errors.is_empty() && warnings.is_empty() && info.is_empty() {
        println!("✅ Clean structural scan");
        println!("{}", "-".repeat(40));
        println!("  No cycles, layer violations, or god-modules detected in scan evidence.");
        if advisory {
            println!("  (Orphan modules omitted in advisory mode.)");
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

    if !violations_only {
        print_could_not_infer_section(could_not_infer);
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
