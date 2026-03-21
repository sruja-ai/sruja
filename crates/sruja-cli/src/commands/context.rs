//! Context export commands: export architecture context for AI tools.

use std::fs;
use std::path::Path;

use sruja_scan::{scan_repo, Graph, NodeKind};

use super::CliError;
use crate::selection::compute_all_centrality;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ArchitectureContext {
    repo: String,
    summary: ContextSummary,
    layers: Vec<LayerInfo>,
    boundaries: Vec<BoundaryRule>,
    forbidden_patterns: Vec<String>,
    focus: Option<FocusContext>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct MultiRepoArchitectureContext {
    repos: Vec<ArchitectureContext>,
    combined_summary: ContextSummary,
    cross_repo_rules: Vec<String>,
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

pub async fn context_string(
    repo_root: &str,
    format: &str,
    file: Option<&str>,
    intent: Option<&str>,
    depth: usize,
) -> Result<String, CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo(repo_path)?;
    let context = build_architecture_context(&graph, repo_root, file, intent, depth)?;

    match format {
        "cursor-rules" => Ok(format_cursor_rules(&context)),
        "copilot-instructions" => Ok(format_copilot_instructions(&context)),
        "markdown" => Ok(format_markdown(&context)),
        "repomap" => Ok(format_repomap(&context, &graph)),
        "json" | "for-ai" => Ok(serde_json::to_string_pretty(&context)?),
        _ => Err(CliError::Validation(format!(
            "Unknown format: {}. Use: cursor-rules, copilot-instructions, markdown, repomap, json, for-ai",
            format
        ))),
    }
}

pub async fn context_string_multi(
    repo_roots: &[String],
    format: &str,
    file: Option<&str>,
    intent: Option<&str>,
    depth: usize,
) -> Result<String, CliError> {
    let repos: Vec<String> = if repo_roots.is_empty() {
        vec![".".to_string()]
    } else {
        repo_roots.to_vec()
    };

    if repos.len() == 1 {
        return context_string(&repos[0], format, file, intent, depth).await;
    }

    let mut contexts: Vec<ArchitectureContext> = Vec::with_capacity(repos.len());
    let mut repomaps: Vec<String> = Vec::new();
    for repo_root in &repos {
        let repo_path = Path::new(repo_root);
        if !repo_path.exists() {
            return Err(CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Repository not found: {}", repo_root),
            )));
        }
        let graph = scan_repo(repo_path)?;
        let context = build_architecture_context(&graph, repo_root, file, intent, depth)?;
        if format == "repomap" {
            repomaps.push(format_repomap(&context, &graph));
        }
        contexts.push(context);
    }

    if format == "repomap" {
        return Ok(repomaps.join("\n\n---\n\n"));
    }

    let combined_summary = ContextSummary {
        total_modules: contexts.iter().map(|c| c.summary.total_modules).sum(),
        total_services: contexts.iter().map(|c| c.summary.total_services).sum(),
        total_databases: contexts.iter().map(|c| c.summary.total_databases).sum(),
        total_external_apis: contexts.iter().map(|c| c.summary.total_external_apis).sum(),
    };

    let cross_repo_rules = vec![
        "Treat each repo as a hard boundary: avoid cross-repo imports; integrate via API/events"
            .to_string(),
        "If a change spans repos, update contracts (OpenAPI/SDK) and bump versions together"
            .to_string(),
        "Run checks in every touched repo (format, lint/typecheck, unit tests, integration tests)"
            .to_string(),
    ];

    let multi = MultiRepoArchitectureContext {
        repos: contexts,
        combined_summary,
        cross_repo_rules,
    };

    match format {
        "cursor-rules" => Ok(format_cursor_rules_multi(&multi)),
        "copilot-instructions" => Ok(format_copilot_instructions_multi(&multi)),
        "markdown" => Ok(format_markdown_multi(&multi)),
        "json" | "for-ai" => Ok(serde_json::to_string_pretty(&multi)?),
        _ => Err(CliError::Validation(format!(
            "Unknown format: {}. Use: cursor-rules, copilot-instructions, markdown, repomap, json, for-ai",
            format
        ))),
    }
}

pub async fn context_export(
    repo_roots: &[String],
    format: &str,
    output: Option<&str>,
    file: Option<&str>,
    intent: Option<&str>,
    depth: usize,
) -> Result<(), CliError> {
    let content = context_string_multi(repo_roots, format, file, intent, depth).await?;

    if let Some(path) = output {
        fs::write(path, &content)?;
        eprintln!("Written context to {}", path);
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn build_architecture_context(
    graph: &Graph,
    repo: &str,
    file: Option<&str>,
    intent: Option<&str>,
    depth: usize,
) -> Result<ArchitectureContext, CliError> {
    let modules = count_kind(graph, NodeKind::Module);
    let services = count_kind(graph, NodeKind::Service);
    let databases = count_kind(graph, NodeKind::Database);
    let external_apis = count_kind(graph, NodeKind::ExternalApi);

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

    Ok(ArchitectureContext {
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

fn count_kind(graph: &Graph, kind: NodeKind) -> usize {
    graph.nodes.iter().filter(|n| n.kind == kind).count()
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
    for c in &mut candidates {
        *c = normalize_path(c);
    }
    candidates.sort();
    candidates.dedup();

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

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn path_matches_any(node_path: &str, candidates: &[String]) -> bool {
    let node_norm = normalize_path(node_path);
    if candidates.contains(&node_norm) {
        return true;
    }
    candidates.iter().any(|c| node_norm.ends_with(c))
}

fn score_path_match(node_path: Option<&str>, candidates: &[String]) -> usize {
    let Some(p) = node_path else {
        return 0;
    };
    let p_norm = normalize_path(p);
    if candidates.contains(&p_norm) {
        return 3;
    }
    if candidates.iter().any(|c| p_norm.ends_with(c)) {
        return 2;
    }
    1
}

fn suggested_checks(intent: Option<&str>) -> Vec<String> {
    let mut checks: Vec<String> = vec![
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

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    checks
        .into_iter()
        .filter(|c| seen.insert(c.clone()))
        .collect()
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

    let mut layers: Vec<LayerInfo> = layer_counts
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
        .collect();
    layers.sort_by(|a, b| a.name.cmp(&b.name));
    layers
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

    let mut services: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Service)
        .collect();
    services.sort_by(|a, b| a.id.cmp(&b.id));

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

fn format_cursor_rules_multi(context: &MultiRepoArchitectureContext) -> String {
    let mut out = String::new();
    out.push_str("# Sruja Architecture Context (Multi-Repo)\n\n");
    out.push_str("This file is auto-generated by `sruja context -f cursor-rules -r <repo>...`\n\n");
    out.push_str("## Combined Summary\n\n");
    out.push_str(&format!(
        "- Repos: {}\n- Modules: {}\n- Services: {}\n- Databases: {}\n- External APIs: {}\n\n",
        context.repos.len(),
        context.combined_summary.total_modules,
        context.combined_summary.total_services,
        context.combined_summary.total_databases,
        context.combined_summary.total_external_apis
    ));

    if !context.cross_repo_rules.is_empty() {
        out.push_str("## Cross-Repo Rules\n\n");
        for rule in &context.cross_repo_rules {
            out.push_str(&format!("- {}\n", rule));
        }
        out.push('\n');
    }

    for repo in &context.repos {
        out.push_str("## Repo\n\n");
        out.push_str(&format!("- Path: {}\n\n", repo.repo));
        out.push_str(&format_cursor_rules(repo));
        out.push_str("\n---\n\n");
    }

    out
}

fn format_copilot_instructions_multi(context: &MultiRepoArchitectureContext) -> String {
    let mut out = String::new();
    out.push_str("# Architecture Context for GitHub Copilot (Multi-Repo)\n\n");
    out.push_str("Generated by `sruja context -f copilot-instructions -r <repo>...`\n\n");
    out.push_str("## Combined Summary\n\n");
    out.push_str(&format!(
        "Repos: {}. Total modules: {}. Total services: {}. Total databases: {}.\n\n",
        context.repos.len(),
        context.combined_summary.total_modules,
        context.combined_summary.total_services,
        context.combined_summary.total_databases
    ));

    if !context.cross_repo_rules.is_empty() {
        out.push_str("## Cross-Repo Rules\n\n");
        for rule in &context.cross_repo_rules {
            out.push_str(&format!("- {}\n", rule));
        }
        out.push('\n');
    }

    for repo in &context.repos {
        out.push_str("## Repo\n\n");
        out.push_str(&format!("Path: {}\n\n", repo.repo));
        out.push_str(&format_copilot_instructions(repo));
        out.push_str("\n---\n\n");
    }

    out
}

fn format_markdown_multi(context: &MultiRepoArchitectureContext) -> String {
    let mut out = String::new();
    out.push_str("# Architecture Context (Multi-Repo)\n\n");
    out.push_str("| Type | Count |\n|------|-------|\n");
    out.push_str(&format!("| Repos | {} |\n", context.repos.len()));
    out.push_str(&format!(
        "| Modules | {} |\n",
        context.combined_summary.total_modules
    ));
    out.push_str(&format!(
        "| Services | {} |\n",
        context.combined_summary.total_services
    ));
    out.push_str(&format!(
        "| Databases | {} |\n",
        context.combined_summary.total_databases
    ));
    out.push_str(&format!(
        "| External APIs | {} |\n\n",
        context.combined_summary.total_external_apis
    ));

    if !context.cross_repo_rules.is_empty() {
        out.push_str("## Cross-Repo Rules\n\n");
        for rule in &context.cross_repo_rules {
            out.push_str(&format!("- {}\n", rule));
        }
        out.push('\n');
    }

    for repo in &context.repos {
        out.push_str("---\n\n");
        out.push_str(&format_markdown(repo));
    }

    out
}

fn format_cursor_rules(context: &ArchitectureContext) -> String {
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

fn format_copilot_instructions(context: &ArchitectureContext) -> String {
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
    } else {
        instructions.push_str("No layers detected.\n");
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

fn format_markdown(context: &ArchitectureContext) -> String {
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

fn format_repomap(context: &ArchitectureContext, graph: &Graph) -> String {
    let mut out = String::new();
    out.push_str("# Sruja Repomap\n\n");
    out.push_str(&format!("Repo: {}\n", context.repo));
    out.push_str(&format!("Nodes: {}\n", graph.nodes.len()));
    out.push_str(&format!("Edges: {}\n", graph.edges.len()));

    let scores = compute_all_centrality(graph);
    let node_by_id: std::collections::HashMap<&str, &sruja_scan::Node> =
        graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    let mut outgoing: std::collections::HashMap<&str, std::collections::HashSet<&str>> =
        std::collections::HashMap::new();
    let mut incoming: std::collections::HashMap<&str, std::collections::HashSet<&str>> =
        std::collections::HashMap::new();
    for e in &graph.edges {
        let src = e.source.as_str();
        let tgt = e.target.as_str();
        if src == tgt {
            continue;
        }
        outgoing.entry(src).or_default().insert(tgt);
        incoming.entry(tgt).or_default().insert(src);
    }

    let repo_root = Path::new(&context.repo);
    let repo_prefix = repo_root
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")));

    out.push_str("\n## Top Nodes\n\n");

    let mut ranked: Vec<(&sruja_scan::Node, f64)> = graph
        .nodes
        .iter()
        .filter(|n| !n.id.contains('#'))
        .map(|n| {
            let pr = scores.get(&n.id).map(|s| s.pagerank).unwrap_or(0.0_f64);
            (n, pr)
        })
        .collect();

    ranked.sort_by(|left, right| {
        let (a, ap) = left;
        let (b, bp) = right;
        bp.partial_cmp(ap)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });

    let top_limit = 30usize;
    for (n, pr) in ranked
        .into_iter()
        .filter(|(n, _)| {
            matches!(
                n.kind,
                NodeKind::Service | NodeKind::Database | NodeKind::ExternalApi | NodeKind::Module
            )
        })
        .take(top_limit)
    {
        let kind = repomap_kind(n.kind);
        let path = n
            .path
            .as_deref()
            .and_then(|p| repomap_relative_path(p, repo_prefix.as_deref()));

        if let Some(p) = &path {
            out.push_str(&format!("- {}: {} ({}) [pr={:.3}]\n", kind, n.label, p, pr));
        } else {
            out.push_str(&format!("- {}: {} [pr={:.3}]\n", kind, n.label, pr));
        }

        let deps = outgoing
            .get(n.id.as_str())
            .map(|s| repomap_top_neighbors(s, &node_by_id, &scores, 5))
            .unwrap_or_default();
        if !deps.is_empty() {
            out.push_str(&format!("  deps: {}\n", deps.join(", ")));
        }

        let used_by = incoming
            .get(n.id.as_str())
            .map(|s| repomap_top_neighbors(s, &node_by_id, &scores, 5))
            .unwrap_or_default();
        if !used_by.is_empty() {
            out.push_str(&format!("  used_by: {}\n", used_by.join(", ")));
        }
    }

    if let Some(focus) = &context.focus {
        out.push_str("\n## Focus\n\n");
        out.push_str(&format!("- File: {}\n", focus.file));
        if let Some(intent) = &focus.intent {
            out.push_str(&format!("- Intent: {}\n", intent));
        }
        out.push_str(&format!("- Depth: {}\n", focus.depth));

        if !focus.matched_nodes.is_empty() {
            out.push_str("\nMatched nodes:\n");
            for n in &focus.matched_nodes {
                let path = n
                    .path
                    .as_deref()
                    .and_then(|p| repomap_relative_path(p, repo_prefix.as_deref()));
                if let Some(p) = path {
                    out.push_str(&format!(
                        "- {}: {} ({})\n",
                        repomap_kind(n.kind),
                        n.label,
                        p
                    ));
                } else {
                    out.push_str(&format!("- {}: {}\n", repomap_kind(n.kind), n.label));
                }
            }
        }

        if let Some(br) = &focus.blast_radius {
            out.push_str(&format!(
                "\nBlast radius: {} upstream, {} downstream\n",
                br.upstream.len(),
                br.downstream.len()
            ));
        }
    }

    out
}

fn repomap_kind(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Service => "service",
        NodeKind::Database => "database",
        NodeKind::ExternalApi => "external_api",
        NodeKind::Module => "module",
        _ => "node",
    }
}

fn repomap_relative_path(path: &str, repo_prefix: Option<&str>) -> Option<String> {
    let normalized = path.replace('\\', "/");
    let rel = if let Some(prefix) = repo_prefix {
        let prefix = prefix.trim_end_matches('/');
        normalized
            .strip_prefix(prefix)
            .unwrap_or(normalized.as_str())
            .trim_start_matches('/')
            .to_string()
    } else {
        normalized
            .trim_start_matches("./")
            .trim_start_matches('/')
            .to_string()
    };
    if rel.is_empty() {
        None
    } else {
        Some(rel)
    }
}

fn repomap_top_neighbors(
    neighbors: &std::collections::HashSet<&str>,
    node_by_id: &std::collections::HashMap<&str, &sruja_scan::Node>,
    scores: &std::collections::HashMap<String, crate::selection::ComponentImportance>,
    limit: usize,
) -> Vec<String> {
    let mut v: Vec<(&str, f64, &str)> = neighbors
        .iter()
        .filter_map(|id| {
            let n = node_by_id.get(id)?;
            let pr = scores.get(n.id.as_str()).map(|s| s.pagerank).unwrap_or(0.0);
            Some((n.label.as_str(), pr, n.id.as_str()))
        })
        .collect();

    v.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.2.cmp(b.2))
    });

    v.into_iter()
        .take(limit)
        .map(|(label, _, _)| label.to_string())
        .collect()
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
    fn score_path_match_prefers_exact_then_suffix() {
        let candidates = vec![
            normalize_path("/repo/src/lib.rs"),
            normalize_path("src/lib.rs"),
        ];
        assert_eq!(score_path_match(Some("/repo/src/lib.rs"), &candidates), 3);
        assert_eq!(
            score_path_match(Some("C:\\repo\\src\\lib.rs"), &candidates),
            2
        );
        assert_eq!(score_path_match(Some("other/file.rs"), &candidates), 1);
        assert_eq!(score_path_match(None, &candidates), 0);
    }

    #[test]
    fn suggested_checks_are_unique_and_intent_specific() {
        let checks = suggested_checks(Some("add-test"));
        assert!(checks.iter().any(|c| c.contains("cargo test -p <crate>")));
        let unique: std::collections::HashSet<_> = checks.iter().collect();
        assert_eq!(checks.len(), unique.len());
    }

    #[test]
    fn infer_layer_from_path_classifies_common_folders() {
        assert_eq!(infer_layer_from_path("src/api/routes.rs"), "api");
        assert_eq!(infer_layer_from_path("src/services/user.rs"), "services");
        assert_eq!(infer_layer_from_path("src/db/schema.sql"), "data");
        assert_eq!(infer_layer_from_path("src/models/user.rs"), "models");
        assert_eq!(infer_layer_from_path("src/utils/mod.rs"), "utils");
        assert_eq!(infer_layer_from_path("src/components/button.tsx"), "ui");
    }

    #[test]
    fn infer_layers_is_deterministic_and_counts_only_modules_with_paths() {
        let graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![
                node("m1", NodeKind::Module, Some("src/api/mod.rs")),
                node("m2", NodeKind::Module, Some("src/services/mod.rs")),
                node("m3", NodeKind::Module, None),
                node("svc", NodeKind::Service, Some("src/main.rs")),
            ],
            edges: vec![],
        };
        let layers = infer_layers(&graph);
        let names: Vec<&str> = layers.iter().map(|l| l.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
        let api_count = layers.iter().find(|l| l.name == "api").unwrap().modules;
        let svc_count = layers
            .iter()
            .find(|l| l.name == "services")
            .unwrap()
            .modules;
        assert_eq!(api_count, 1);
        assert_eq!(svc_count, 1);
    }

    #[test]
    fn infer_boundaries_adds_service_isolation_and_ui_rule() {
        let graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![
                node("b", NodeKind::Service, Some("b/main.rs")),
                node("a", NodeKind::Service, Some("a/main.rs")),
            ],
            edges: vec![],
        };
        let rules = infer_boundaries(&graph);
        assert!(rules
            .iter()
            .any(|r| r.from == "a" && r.to == "b" && !r.allowed));
        assert!(rules
            .iter()
            .any(|r| r.from == "b" && r.to == "a" && !r.allowed));
        assert!(rules
            .iter()
            .any(|r| r.from == "ui" && r.to == "data" && !r.allowed));
    }

    #[test]
    fn repomap_relative_path_handles_prefix_and_windows_separators() {
        assert_eq!(
            repomap_relative_path("/repo/src/lib.rs", Some("/repo")).as_deref(),
            Some("src/lib.rs")
        );
        assert_eq!(
            repomap_relative_path("C:\\repo\\src\\lib.rs", Some("C:/repo")).as_deref(),
            Some("src/lib.rs")
        );
        assert_eq!(
            repomap_relative_path("./src/lib.rs", None).as_deref(),
            Some("src/lib.rs")
        );
        assert_eq!(repomap_relative_path("/repo", Some("/repo")), None);
    }

    #[test]
    fn repomap_top_neighbors_orders_by_pagerank_then_id() {
        let n1 = node("a", NodeKind::Module, Some("a.rs"));
        let n2 = node("b", NodeKind::Module, Some("b.rs"));
        let node_by_id: std::collections::HashMap<&str, &sruja_scan::Node> = vec![&n1, &n2]
            .into_iter()
            .map(|n| (n.id.as_str(), n))
            .collect();
        let neighbors: std::collections::HashSet<&str> = ["a", "b"].into_iter().collect();
        let mut scores: std::collections::HashMap<String, crate::selection::ComponentImportance> =
            std::collections::HashMap::new();
        let mut a_score = crate::selection::ComponentImportance::default();
        a_score.pagerank = 0.1;
        let mut b_score = crate::selection::ComponentImportance::default();
        b_score.pagerank = 0.9;
        scores.insert("a".to_string(), a_score);
        scores.insert("b".to_string(), b_score);

        let ordered = repomap_top_neighbors(&neighbors, &node_by_id, &scores, 10);
        assert_eq!(ordered, vec!["b".to_string(), "a".to_string()]);
    }

    #[test]
    fn copilot_instructions_always_include_key_rule_newlines() {
        let context = ArchitectureContext {
            repo: "/repo".to_string(),
            summary: ContextSummary {
                total_modules: 1,
                total_services: 0,
                total_databases: 0,
                total_external_apis: 0,
            },
            layers: Vec::new(),
            boundaries: Vec::new(),
            forbidden_patterns: Vec::new(),
            focus: None,
        };

        let out = format_copilot_instructions(&context);
        assert!(out.contains("1. **Respect layers**: No layers detected.\n"));
        assert!(out.contains("\n2. **No cross-service imports**:"));
        assert!(out.contains("\n3. **UI -> Services -> Data**:"));
    }

    #[test]
    fn copilot_instructions_include_layer_names_when_present() {
        let context = ArchitectureContext {
            repo: "/repo".to_string(),
            summary: ContextSummary {
                total_modules: 2,
                total_services: 0,
                total_databases: 0,
                total_external_apis: 0,
            },
            layers: vec![LayerInfo {
                name: "api".to_string(),
                modules: 2,
                can_depend_on: vec!["services".to_string()],
            }],
            boundaries: Vec::new(),
            forbidden_patterns: Vec::new(),
            focus: None,
        };

        let out = format_copilot_instructions(&context);
        assert!(out.contains("Layers found: api\n"));
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

    #[test]
    fn repomap_includes_top_nodes_and_focus() {
        let graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![
                node("svc", NodeKind::Service, Some("/repo/svc/main.rs")),
                node("db", NodeKind::Database, Some("/repo/db/schema.sql")),
                node("mod", NodeKind::Module, Some("/repo/src/lib.rs")),
            ],
            edges: vec![edge("svc", "db"), edge("svc", "mod")],
        };

        let context =
            build_architecture_context(&graph, "/repo", Some("src/lib.rs"), Some("fix-bug"), 1)
                .expect("architecture context should build");
        let repomap = format_repomap(&context, &graph);
        assert!(repomap.contains("# Sruja Repomap"));
        assert!(repomap.contains("## Top Nodes"));
        assert!(repomap.contains("## Focus"));
        assert!(repomap.contains("File: src/lib.rs"));
    }
}
