//! Discovery commands: question bank for intelligent architecture capture.
//!
//! Use with the sruja-architecture skill so the AI asks users these questions
//! before or during discovery. See skills/sruja-architecture/REFERENCE.md.

use std::collections::HashSet;
use std::path::Path;

use sruja_scan::scan_repo;
use sruja_scan::scan_scope::resolve_scan_scope;
use sruja_scan::{generate_repomap_from_graph, Graph, RepoMapOptions};

use super::CliError;
use crate::context_detection::{
    build_repo_context, detect_architecture_style, detect_framework, detect_languages,
};

const QUESTION_BANK: &str = r#"# Sruja discovery question bank

Ask the user 2–5 of these (adapt to context). Use answers to set scope, subpath, names, and externals.

## Context / shape
- Is this a single service, a monolith with modules, or several microservices?
- Should we capture one area first or the whole repo?

## Large repo
- The repo is big. Should we focus on a specific area (e.g. services/auth, apps/web) or the whole codebase? I can capture by subpath and we can stitch later.
- Which directory or service should we start with?

## Scope
- Do you want a minimal sketch (entry points + main deps), standard (10–30 components), or a deeper model (internal layers, error paths)?

## Boundaries
- What are your main bounded contexts or team-owned areas?
- Any external systems (payments, auth, notifications) that must appear in the diagram?

## Entry points and flows
- What's the main user-facing entry (web app, public API, CLI)?
- Any key flows (e.g. checkout, auth) I should make explicit?

## Refinement (after first draft)
- Does this match how you think about the system? Any services or boundaries missing?
- Prefer different names for systems or containers?

---
Use with: npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
Then in Cursor: run the agent and ask it to discover architecture; it will use this question bank.
"#;

/// Print the discovery question bank for use with the sruja-architecture skill.
pub fn discover_questions() -> Result<(), CliError> {
    println!("{}", QUESTION_BANK);
    Ok(())
}

/// Machine-readable repo context for agents (JSON).
#[derive(serde::Serialize)]
pub struct DiscoverContextJson {
    pub repo: String,
    pub scan_scope: sruja_scan::scan_scope::ScanScope,
    pub components: usize,
    pub edges: usize,
    pub primary_language: String,
    pub framework: Option<String>,
    pub architecture_style: String,
    pub domain: Option<String>,
    pub suggested_areas: Vec<String>,
}

/// Build repo context summary as a string (for prompts or discovery).
pub fn discover_context_string(repo: &str) -> Result<String, CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }

    let graph = scan_repo(repo_path).map_err(|e| CliError::Scan(e.to_string()))?;
    discover_context_string_from_graph(repo, repo_path, &graph)
}

/// Build repo context summary from a pre-scanned graph (includes actual structure).
pub fn discover_context_string_from_graph(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Result<String, CliError> {
    let languages = detect_languages(repo_path);
    let primary_language = languages
        .first()
        .map(|(l, _)| l.as_str())
        .unwrap_or("Unknown");
    let framework = detect_framework(repo_path, primary_language);
    let context = build_repo_context(repo_path, graph);
    let (is_monolith, is_microservices) = detect_architecture_style(graph);

    let repo_prefix = repo_path
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")));
    let repo_arg_norm = repo.replace('\\', "/").trim_end_matches('/').to_string();

    let mut areas: HashSet<String> = HashSet::new();
    for node in &graph.nodes {
        if let Some(ref path) = node.path {
            let normalized = path.replace('\\', "/");
            let rel = if let Some(ref prefix) = repo_prefix {
                normalized
                    .strip_prefix(prefix)
                    .or_else(|| normalized.strip_prefix(&format!("{}/", prefix)))
                    .unwrap_or(normalized.as_str())
                    .trim_start_matches('/')
            } else if !repo_arg_norm.is_empty() {
                normalized
                    .strip_prefix(&format!("{}/", repo_arg_norm))
                    .or_else(|| normalized.strip_prefix(&repo_arg_norm))
                    .unwrap_or(normalized.as_str())
                    .trim_start_matches('/')
            } else {
                normalized.as_str()
            };

            if rel.is_empty() {
                continue;
            }

            let first = rel.split('/').next().unwrap_or("");
            if !first.is_empty()
                && !first.starts_with('.')
                && first != "test-repos"
                && first != "evaluation"
            {
                areas.insert(first.to_string());
            }
        }
    }
    let mut areas: Vec<String> = areas.into_iter().collect();
    areas.sort();

    let arch_style = if is_microservices {
        "microservices"
    } else if is_monolith {
        "monolith"
    } else {
        "mixed/unclear"
    };

    let mut out = String::new();
    out.push_str("# Repo context (for contextual discovery questions)\n\n");
    out.push_str(&format!("**Repo:** {}\n", repo));
    out.push_str(&format!("**Components (scan):** {}\n", graph.nodes.len()));
    out.push_str(&format!("**Edges:** {}\n", graph.edges.len()));
    out.push_str(&format!("**Primary language:** {}\n", primary_language));
    if let Some(ref fw) = framework {
        out.push_str(&format!("**Framework:** {}\n", fw));
    }
    out.push_str(&format!("**Architecture style:** {}\n", arch_style));
    if let Some(ref domain) = context.domain {
        out.push_str(&format!("**Domain (inferred):** {}\n", domain));
    }
    let areas_str = if areas.is_empty() {
        "(none — single directory or flat structure)".to_string()
    } else {
        areas.join(", ")
    };
    out.push_str(&format!(
        "**Suggested areas (from paths):** {}\n",
        areas_str
    ));

    out.push_str("\n## Key Components\n\n");

    let mut file_nodes: Vec<_> = graph.nodes.iter().filter(|n| n.path.is_some()).collect();

    let mut incoming_count: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();
    for edge in &graph.edges {
        *incoming_count.entry(&edge.target).or_default() += 1;
    }

    file_nodes.sort_by(|a, b| {
        let a_count = incoming_count.get(a.id.as_str()).copied().unwrap_or(0);
        let b_count = incoming_count.get(b.id.as_str()).copied().unwrap_or(0);
        b_count.cmp(&a_count)
    });

    let top_files = file_nodes.iter().take(20);
    for node in top_files {
        if let Some(ref path) = node.path {
            let rel_path = if let Some(ref prefix) = repo_prefix {
                path.replace('\\', "/")
                    .strip_prefix(prefix)
                    .or_else(|| path.strip_prefix(&format!("{}/", prefix)))
                    .unwrap_or(path.as_str())
                    .trim_start_matches('/')
                    .to_string()
            } else {
                path.clone()
            };
            let import_count = incoming_count.get(node.id.as_str()).copied().unwrap_or(0);
            out.push_str(&format!(
                "- `{}` ({}, {} imports)\n",
                rel_path,
                node.kind.as_str(),
                import_count
            ));
        }
    }

    out.push_str("\n## Exported Interfaces\n\n");

    let export_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.id.contains('#'))
        .take(30)
        .collect();

    for node in export_nodes {
        out.push_str(&format!("- `{}` ({})\n", node.label, node.kind.as_str()));
    }

    out.push_str("\nUse this context to derive 2–5 questions tailored to this repo (see skill: contextual discovery).\n");
    Ok(out)
}

/// Build repo context as JSON for machine-readable consumption by agents.
pub fn discover_context_json(repo: &str) -> Result<DiscoverContextJson, CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }
    let graph = scan_repo(repo_path).map_err(|e| CliError::Scan(e.to_string()))?;
    discover_context_json_from_graph(repo, repo_path, &graph)
}

/// Build repo context as JSON from a pre-scanned graph (avoids rescanning).
pub fn discover_context_json_from_graph(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Result<DiscoverContextJson, CliError> {
    let (_, scan_scope) = resolve_scan_scope(repo_path);
    let languages = detect_languages(repo_path);
    let primary_language = languages
        .first()
        .map(|(l, _)| l.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let framework = detect_framework(
        repo_path,
        languages.first().map(|(l, _)| l.as_str()).unwrap_or(""),
    );
    let context = build_repo_context(repo_path, graph);
    let (is_monolith, is_microservices) = detect_architecture_style(graph);
    let architecture_style = if is_microservices {
        "microservices"
    } else if is_monolith {
        "monolith"
    } else {
        "mixed/unclear"
    }
    .to_string();
    let repo_prefix = repo_path
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")));
    let repo_arg_norm = repo.replace('\\', "/").trim_end_matches('/').to_string();
    let mut areas: HashSet<String> = HashSet::new();
    for node in &graph.nodes {
        if let Some(ref path) = node.path {
            let normalized = path.replace('\\', "/");
            let rel = if let Some(ref prefix) = repo_prefix {
                normalized
                    .strip_prefix(prefix)
                    .or_else(|| normalized.strip_prefix(&format!("{}/", prefix)))
                    .unwrap_or(normalized.as_str())
                    .trim_start_matches('/')
            } else if !repo_arg_norm.is_empty() {
                normalized
                    .strip_prefix(&format!("{}/", repo_arg_norm))
                    .or_else(|| normalized.strip_prefix(&repo_arg_norm))
                    .unwrap_or(normalized.as_str())
                    .trim_start_matches('/')
            } else {
                normalized.as_str()
            };
            if rel.is_empty() {
                continue;
            }
            let first = rel.split('/').next().unwrap_or("");
            if !first.is_empty()
                && !first.starts_with('.')
                && first != "test-repos"
                && first != "evaluation"
            {
                areas.insert(first.to_string());
            }
        }
    }
    let mut suggested_areas: Vec<String> = areas.into_iter().collect();
    suggested_areas.sort();
    Ok(DiscoverContextJson {
        repo: repo.to_string(),
        scan_scope,
        components: graph.nodes.len(),
        edges: graph.edges.len(),
        primary_language,
        framework,
        architecture_style,
        domain: context.domain.clone(),
        suggested_areas,
    })
}

/// Print repo context summary for the agent to derive contextual questions.
pub async fn discover_context(repo: &str, format: &str) -> Result<(), CliError> {
    if format == "json" {
        let json = discover_context_json(repo)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json).map_err(|e| CliError::Validation(e.to_string()))?
        );
        return Ok(());
    }
    let s = discover_context_string(repo)?;
    println!("{}", s);
    Ok(())
}

/// Generate a repository map with tree-sitter signatures for top files.
pub fn discover_repomap(
    repo: &str,
    max_files: usize,
    max_tokens: usize,
) -> Result<String, CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }

    let graph = scan_repo(repo_path).map_err(|e| CliError::Scan(e.to_string()))?;

    let options = RepoMapOptions {
        max_files,
        max_tokens,
        include_signatures: true,
    };

    generate_repomap_from_graph(repo_path, &graph, &options)
        .map_err(|e| CliError::Scan(e.to_string()))
}

/// Print repository map for LLM context.
pub async fn discover_repomap_cmd(
    repo: &str,
    max_files: usize,
    max_tokens: usize,
) -> Result<(), CliError> {
    let repomap = discover_repomap(repo, max_files, max_tokens)?;
    println!("{}", repomap);
    Ok(())
}
