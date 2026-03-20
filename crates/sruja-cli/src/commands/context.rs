//! Context export commands: export architecture context for AI tools.

use std::fs;
use std::path::Path;

use sruja_scan::{scan_repo, Graph, NodeKind};

use super::CliError;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AiContext {
    repo: String,
    summary: ContextSummary,
    layers: Vec<LayerInfo>,
    boundaries: Vec<BoundaryRule>,
    forbidden_patterns: Vec<String>,
    focus: Option<FocusContext>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FocusContext {
    file: String,
    intent: Option<String>,
    depth: usize,
    matched_nodes: Vec<FocusNode>,
    blast_radius: Option<sruja_scan::BlastRadiusResult>,
    suggested_checks: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FocusNode {
    id: String,
    kind: NodeKind,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ContextSummary {
    total_modules: usize,
    total_services: usize,
    total_databases: usize,
    total_external_apis: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct LayerInfo {
    name: String,
    modules: usize,
    can_depend_on: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct BoundaryRule {
    from: String,
    to: String,
    allowed: bool,
    reason: String,
}

pub async fn context_export(
    repo_root: &str,
    format: &str,
    output: Option<&str>,
    file: Option<&str>,
    intent: Option<&str>,
    depth: usize,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo(repo_path)?;
    let context = build_ai_context(&graph, repo_root, file, intent, depth)?;

    let content = match format {
        "cursor-rules" => format_cursor_rules(&context),
        "copilot-instructions" => format_copilot_instructions(&context),
        "markdown" => format_markdown(&context),
        "json" | "for-ai" => serde_json::to_string_pretty(&context)?,
        _ => {
            return Err(CliError::Validation(format!(
            "Unknown format: {}. Use: cursor-rules, copilot-instructions, markdown, json, for-ai",
            format
        )))
        }
    };

    if let Some(path) = output {
        fs::write(path, &content)?;
        eprintln!("Written context to {}", path);
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn build_ai_context(
    graph: &Graph,
    repo: &str,
    file: Option<&str>,
    intent: Option<&str>,
    depth: usize,
) -> Result<AiContext, CliError> {
    let modules = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Module)
        .count();
    let services = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Service)
        .count();
    let databases = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Database)
        .count();
    let external_apis = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::ExternalApi)
        .count();

    let layers = infer_layers(graph);
    let boundaries = infer_boundaries(graph);

    let forbidden_patterns = vec![
        "Avoid direct database access from routes/handlers - use a service layer".to_string(),
        "Do not import internal modules from other services directly".to_string(),
        "UI components should not directly call database layers".to_string(),
    ];

    let focus = file
        .map(|f| build_focus_context(graph, repo, f, intent, depth))
        .transpose()?;

    Ok(AiContext {
        repo: repo.to_string(),
        summary: ContextSummary {
            total_modules: modules,
            total_services: services,
            total_databases: databases,
            total_external_apis: external_apis,
        },
        layers,
        boundaries,
        forbidden_patterns,
        focus,
    })
}

fn build_focus_context(
    graph: &Graph,
    repo_root: &str,
    file: &str,
    intent: Option<&str>,
    depth: usize,
) -> Result<FocusContext, CliError> {
    let repo_path = Path::new(repo_root);
    let repo_canon = repo_path
        .canonicalize()
        .unwrap_or_else(|_| repo_path.to_path_buf());

    let requested_path = Path::new(file);
    let absolute = if requested_path.is_absolute() {
        requested_path.to_path_buf()
    } else {
        repo_path.join(requested_path)
    };

    let absolute_canon = absolute.canonicalize().unwrap_or(absolute.clone());
    let rel = absolute_canon
        .strip_prefix(&repo_canon)
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let absolute_str = absolute.to_string_lossy().to_string();
    let absolute_canon_str = absolute_canon.to_string_lossy().to_string();
    let mut candidates: Vec<String> = vec![absolute_str, absolute_canon_str];
    if let Some(r) = &rel {
        candidates.push(r.clone());
    }

    let mut matched: Vec<&sruja_scan::Node> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.path
                .as_ref()
                .is_some_and(|p| path_matches_any(p, &candidates))
        })
        .collect();

    matched.sort_by(|a, b| {
        score_path_match(a.path.as_deref(), &candidates)
            .cmp(&score_path_match(b.path.as_deref(), &candidates))
            .reverse()
            .then_with(|| a.id.cmp(&b.id))
    });

    let matched_nodes: Vec<FocusNode> = matched
        .iter()
        .take(10)
        .map(|n| FocusNode {
            id: n.id.clone(),
            kind: n.kind,
            label: n.label.clone(),
            path: n.path.clone(),
        })
        .collect();

    let blast_target = matched
        .iter()
        .find(|n| !n.id.contains('#'))
        .or_else(|| matched.first())
        .map(|n| n.id.as_str());

    let blast_radius = blast_target
        .filter(|_| depth > 0)
        .map(|id| graph.blast_radius(id, depth));

    let suggested_checks = suggested_checks(intent);

    Ok(FocusContext {
        file: file.to_string(),
        intent: intent.map(|s| s.to_string()),
        depth,
        matched_nodes,
        blast_radius,
        suggested_checks,
    })
}

fn path_matches_any(node_path: &str, candidates: &[String]) -> bool {
    candidates.iter().any(|c| path_matches(node_path, c))
}

fn path_matches(node_path: &str, candidate: &str) -> bool {
    if node_path == candidate {
        return true;
    }
    let node_norm = node_path.replace('\\', "/");
    let cand_norm = candidate.replace('\\', "/");
    node_norm.ends_with(&cand_norm)
}

fn score_path_match(node_path: Option<&str>, candidates: &[String]) -> usize {
    let Some(p) = node_path else {
        return 0;
    };
    if candidates.iter().any(|c| p == c) {
        return 3;
    }
    if candidates
        .iter()
        .any(|c| p.replace('\\', "/").ends_with(&c.replace('\\', "/")))
    {
        return 2;
    }
    1
}

fn suggested_checks(intent: Option<&str>) -> Vec<String> {
    let mut checks = vec![
        "cargo fmt --all".to_string(),
        "cargo clippy -- -D warnings".to_string(),
        "cargo test --workspace".to_string(),
        "sruja drift -r .".to_string(),
    ];

    match intent {
        Some("add-test") => {
            checks.insert(0, "cargo test -p <crate> <test_name>".to_string());
        }
        Some("fix-bug") => {
            checks.insert(0, "cargo test --workspace".to_string());
        }
        Some("refactor") => {
            checks.insert(0, "cargo test --workspace".to_string());
        }
        Some("add-feature") => {
            checks.insert(0, "cargo test --workspace".to_string());
        }
        _ => {}
    }

    checks.dedup();
    checks
}

fn infer_layers(graph: &Graph) -> Vec<LayerInfo> {
    let mut layer_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for node in &graph.nodes {
        if node.kind == NodeKind::Module {
            if let Some(path) = &node.path {
                let layer = infer_layer_from_path(path);
                *layer_counts.entry(layer).or_default() += 1;
            }
        }
    }

    layer_counts
        .into_iter()
        .map(|(name, count)| {
            let can_depend_on = match name.as_str() {
                "api" | "routes" | "handlers" => vec!["services".to_string()],
                "services" => vec!["data".to_string(), "models".to_string()],
                "data" | "db" | "repository" => vec!["models".to_string()],
                "models" | "entities" => vec![],
                "utils" | "lib" | "common" => vec![],
                _ => vec![],
            };
            LayerInfo {
                name,
                modules: count,
                can_depend_on,
            }
        })
        .collect()
}

fn infer_layer_from_path(path: &str) -> String {
    let path_lower = path.to_lowercase();
    let parts: Vec<&str> = path_lower.split('/').collect();

    for part in &parts {
        match *part {
            "api" | "routes" | "handlers" | "controllers" | "endpoints" => {
                return "api".to_string()
            }
            "services" | "service" => return "services".to_string(),
            "data" | "db" | "database" | "repository" | "repos" | "dal" => {
                return "data".to_string()
            }
            "models" | "model" | "entities" | "entity" | "domain" => return "models".to_string(),
            "utils" | "lib" | "common" | "shared" | "helpers" => return "utils".to_string(),
            "components" | "ui" | "views" | "pages" => return "ui".to_string(),
            _ => {}
        }
    }

    "other".to_string()
}

fn infer_boundaries(graph: &Graph) -> Vec<BoundaryRule> {
    let mut boundaries = Vec::new();

    let services: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Service)
        .collect();

    if services.len() > 1 {
        for s1 in &services {
            for s2 in &services {
                if s1.id != s2.id {
                    boundaries.push(BoundaryRule {
                        from: s1.id.clone(),
                        to: s2.id.clone(),
                        allowed: false,
                        reason: "Services should communicate via APIs/events, not direct imports"
                            .to_string(),
                    });
                }
            }
        }
    }

    boundaries.push(BoundaryRule {
        from: "ui".to_string(),
        to: "data".to_string(),
        allowed: false,
        reason: "UI should not directly access data layer - use services".to_string(),
    });

    boundaries
}

fn format_cursor_rules(context: &AiContext) -> String {
    let mut rules = String::new();
    rules.push_str("# Sruja Architecture Context\n\n");
    rules.push_str("This file is auto-generated by `sruja context export -f cursor-rules`\n\n");
    rules.push_str("## Architecture Overview\n\n");
    rules.push_str(&format!(
        "- Modules: {}\n- Services: {}\n- Databases: {}\n- External APIs: {}\n\n",
        context.summary.total_modules,
        context.summary.total_services,
        context.summary.total_databases,
        context.summary.total_external_apis
    ));

    if !context.layers.is_empty() {
        rules.push_str("## Layers\n\n");
        for layer in &context.layers {
            rules.push_str(&format!(
                "### {} ({} modules)\n",
                layer.name.to_uppercase().replace("_", " "),
                layer.modules
            ));
            if !layer.can_depend_on.is_empty() {
                rules.push_str(&format!(
                    "Can depend on: {}\n\n",
                    layer.can_depend_on.join(", ")
                ));
            } else {
                rules.push_str("No external dependencies allowed.\n\n");
            }
        }
    }

    if !context.boundaries.is_empty() {
        rules.push_str("## Boundary Rules\n\n");
        for boundary in &context.boundaries {
            if !boundary.allowed {
                rules.push_str(&format!(
                    "- **{} -> {}**: {}\n",
                    boundary.from, boundary.to, boundary.reason
                ));
            }
        }
        rules.push('\n');
    }

    if !context.forbidden_patterns.is_empty() {
        rules.push_str("## Forbidden Patterns\n\n");
        for pattern in &context.forbidden_patterns {
            rules.push_str(&format!("- {}\n", pattern));
        }
        rules.push('\n');
    }

    rules.push_str("## When suggesting code\n\n");
    rules.push_str("1. Respect layer boundaries - check imports before suggesting\n");
    rules.push_str("2. Use existing patterns in the codebase\n");
    rules.push_str("3. If adding a new dependency, verify it does not violate boundaries\n");
    rules.push_str("4. Run `sruja drift -r .` after changes to verify architecture health\n");

    if let Some(focus) = &context.focus {
        rules.push_str("\n## Current Task Focus\n\n");
        rules.push_str(&format!("- File: {}\n", focus.file));
        if let Some(intent) = &focus.intent {
            rules.push_str(&format!("- Intent: {}\n", intent));
        }
        if let Some(br) = &focus.blast_radius {
            rules.push_str(&format!(
                "- Blast radius: {} upstream, {} downstream (depth {})\n",
                br.upstream.len(),
                br.downstream.len(),
                focus.depth
            ));
        }
    }

    rules
}

fn format_copilot_instructions(context: &AiContext) -> String {
    let mut instructions = String::new();
    instructions.push_str("# Architecture Context for GitHub Copilot\n\n");
    instructions.push_str("Generated by `sruja context export -f copilot-instructions`\n\n");

    instructions.push_str("## Summary\n");
    instructions.push_str(&format!(
        "This codebase has {} modules, {} services, {} databases.\n\n",
        context.summary.total_modules,
        context.summary.total_services,
        context.summary.total_databases
    ));

    instructions.push_str("## Key Rules\n\n");
    instructions.push_str("1. **Respect layers**: ");
    if !context.layers.is_empty() {
        let layer_names: Vec<_> = context.layers.iter().map(|l| l.name.clone()).collect();
        instructions.push_str(&format!("Layers found: {}\n", layer_names.join(", ")));
    }

    instructions
        .push_str("2. **No cross-service imports**: Services should communicate via API/event\n");
    instructions.push_str("3. **UI -> Services -> Data**: Follow this flow, never skip layers\n\n");

    if !context.boundaries.is_empty() {
        instructions.push_str("## Forbidden Dependencies\n");
        for boundary in context.boundaries.iter().filter(|b| !b.allowed).take(5) {
            instructions.push_str(&format!(
                "- {} -> {} is not allowed\n",
                boundary.from, boundary.to
            ));
        }
        instructions.push('\n');
    }

    instructions.push_str("## Before Committing\n");
    instructions.push_str("Run: `sruja drift -r .` to check for architectural violations.\n");

    if let Some(focus) = &context.focus {
        instructions.push_str("\n## Current Task Focus\n\n");
        instructions.push_str(&format!("- File: {}\n", focus.file));
        if let Some(intent) = &focus.intent {
            instructions.push_str(&format!("- Intent: {}\n", intent));
        }
    }

    instructions
}

fn format_markdown(context: &AiContext) -> String {
    let mut md = String::new();
    md.push_str("# Architecture Context\n\n");
    md.push_str(&format!(
        "> Generated by `sruja context export -f markdown` for {}\n\n",
        context.repo
    ));

    md.push_str("## Overview\n\n");
    md.push_str("| Type | Count |\n|------|-------|\n");
    md.push_str(&format!(
        "| Modules | {} |\n",
        context.summary.total_modules
    ));
    md.push_str(&format!(
        "| Services | {} |\n",
        context.summary.total_services
    ));
    md.push_str(&format!(
        "| Databases | {} |\n",
        context.summary.total_databases
    ));
    md.push_str(&format!(
        "| External APIs | {} |\n\n",
        context.summary.total_external_apis
    ));

    if !context.layers.is_empty() {
        md.push_str("## Layers\n\n");
        for layer in &context.layers {
            md.push_str(&format!(
                "### {}\n\n**Can depend on:** {}\n\n**Modules:** {}\n\n",
                layer.name,
                if layer.can_depend_on.is_empty() {
                    "None".to_string()
                } else {
                    layer.can_depend_on.join(", ")
                },
                layer.modules
            ));
        }
    }

    if !context.forbidden_patterns.is_empty() {
        md.push_str("## Rules to Follow\n\n");
        for pattern in &context.forbidden_patterns {
            md.push_str(&format!("- {}\n", pattern));
        }
        md.push('\n');
    }

    if let Some(focus) = &context.focus {
        md.push_str("## Current Task Focus\n\n");
        md.push_str(&format!("- File: {}\n", focus.file));
        if let Some(intent) = &focus.intent {
            md.push_str(&format!("- Intent: {}\n", intent));
        }
        if let Some(br) = &focus.blast_radius {
            md.push_str(&format!(
                "- Blast radius: {} upstream, {} downstream (depth {})\n",
                br.upstream.len(),
                br.downstream.len(),
                focus.depth
            ));
        }
        if !focus.suggested_checks.is_empty() {
            md.push_str("\n### Suggested checks\n\n");
            for c in &focus.suggested_checks {
                md.push_str(&format!("- `{}`\n", c));
            }
            md.push('\n');
        }
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn node(id: &str, kind: NodeKind, path: Option<&str>) -> sruja_scan::Node {
        sruja_scan::Node {
            id: id.to_string(),
            kind,
            label: id.to_string(),
            technology: None,
            path: path.map(|p| p.to_string()),
            metadata: HashMap::new(),
        }
    }

    fn edge(source: &str, target: &str) -> sruja_scan::Edge {
        sruja_scan::Edge {
            source: source.to_string(),
            target: target.to_string(),
            kind: sruja_scan::EdgeKind::Calls,
            evidence: Vec::new(),
        }
    }

    #[test]
    fn focus_context_matches_relative_path_suffix() {
        let graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![
                node("module:src", NodeKind::Module, Some("src")),
                node("src_lib_rs", NodeKind::Module, Some("/repo/src/lib.rs")),
            ],
            edges: vec![edge("module:src", "src_lib_rs")],
        };

        let focus = build_focus_context(&graph, "/repo", "src/lib.rs", Some("fix-bug"), 2)
            .expect("focus context should build");
        assert_eq!(focus.file, "src/lib.rs");
        assert!(focus.matched_nodes.iter().any(|n| n.id == "src_lib_rs"));
        assert!(focus.blast_radius.is_some());
    }
}
