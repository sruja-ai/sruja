//! Discovery commands: question bank for intelligent architecture capture.
//!
//! Use with the sruja-architecture-agent skill so the AI asks users these questions
//! before or during discovery. See skills/sruja-architecture-agent/SKILL.md.

use std::collections::HashSet;
use std::path::Path;

use sruja_scan::scan_repo;

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
Use with: npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-agent
Then in Cursor: run the agent and ask it to discover architecture; it will use this question bank.
"#;

/// Print the discovery question bank for use with the sruja-architecture-agent skill.
pub fn discover_questions() -> Result<(), CliError> {
    println!("{}", QUESTION_BANK);
    Ok(())
}

/// Machine-readable repo context for agents (JSON).
#[derive(serde::Serialize)]
pub struct DiscoverContextJson {
    pub repo: String,
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
    let languages = detect_languages(repo_path);
    let primary_language = languages
        .first()
        .map(|(l, _)| l.as_str())
        .unwrap_or("Unknown");
    let framework = detect_framework(repo_path, primary_language);
    let context = build_repo_context(repo_path, &graph);
    let (is_monolith, is_microservices) = detect_architecture_style(&graph);

    // Suggested areas: first segment of *repo-relative* paths (e.g. lib/, routes/, services/auth/).
    // Node paths from scan are often absolute; derive stable relative paths so suggestions are meaningful.
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
    let languages = detect_languages(repo_path);
    let primary_language = languages
        .first()
        .map(|(l, _)| l.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let framework = detect_framework(repo_path, languages.first().map(|(l, _)| l.as_str()).unwrap_or("")).map(String::from);
    let context = build_repo_context(repo_path, &graph);
    let (is_monolith, is_microservices) = detect_architecture_style(&graph);
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
        println!("{}", serde_json::to_string_pretty(&json).map_err(|e| CliError::Validation(e.to_string()))?);
        return Ok(());
    }
    let s = discover_context_string(repo)?;
    println!("{}", s);
    Ok(())
}
