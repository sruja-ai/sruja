//! `sruja decision` — Decision Records as YAML-frontmatter Markdown under `.sruja/decisions/`.

use crate::commands::context_events::{
    append_context_event, policy_fingerprint, ContextEventRecord, CONTEXT_EVENTS_SCHEMA_V2,
};
use chrono::Utc;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sruja_graph::{Decision, DecisionStatus, SourceReference};
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

use crate::commands::CliError;

fn decisions_dir(repo: &Path) -> PathBuf {
    repo.join(".sruja").join("decisions")
}

/// YAML front matter for a Decision Record (generalized ADR).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionFrontmatter {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub status: String,
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(default)]
    pub elements: Vec<String>,
    #[serde(default)]
    pub actors: Vec<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
    pub supersedes: Vec<String>,
    /// Set when this record was superseded by another id.
    #[serde(default)]
    pub superseded_by: Option<String>,
}

fn default_scope() -> String {
    "repo".into()
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionListItem {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub status: String,
    pub title: String,
    pub path: String,
    #[serde(default)]
    pub elements: Vec<String>,
}

pub(crate) fn parse_decision_file(path: &Path) -> Result<(DecisionFrontmatter, String), CliError> {
    let raw = fs::read_to_string(path).map_err(CliError::Io)?;
    split_frontmatter(&raw).ok_or_else(|| {
        CliError::validation(format!("expected YAML front matter in {}", path.display()))
    })
}

fn split_frontmatter(raw: &str) -> Option<(DecisionFrontmatter, String)> {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after_first = trimmed.strip_prefix("---")?;
    let end = after_first.find("\n---\n")?;
    let yaml_part = after_first[..end].trim();
    let body = after_first[end + "\n---\n".len()..].to_string();
    let fm: DecisionFrontmatter = serde_yaml::from_str(yaml_part).ok()?;
    Some((fm, body))
}

/// Serialize front matter + body (internal helper; errors mapped by caller).
pub fn format_decision_file(fm: &DecisionFrontmatter, body: &str) -> Result<String, CliError> {
    let yaml = serde_yaml::to_string(fm).map_err(|e| CliError::validation(e.to_string()))?;
    Ok(format!("---\n{yaml}---\n{body}"))
}

/// Parse markdown body after front matter: title (`#`) and `##` sections used by the template.
pub(crate) fn parse_decision_sections(body: &str) -> (String, String, String, String, Vec<String>) {
    let mut title = String::new();
    let mut current_heading = String::new();
    let mut context = String::new();
    let mut decision = String::new();
    let mut consequences = String::new();
    let mut alternatives: Vec<String> = Vec::new();

    for line in body.lines() {
        let trimmed_end = line.trim_end();
        if trimmed_end.starts_with("# ") && title.is_empty() {
            title = trimmed_end[2..].trim().to_string();
            continue;
        }
        if let Some(rest) = trimmed_end.strip_prefix("## ") {
            current_heading = rest.trim().to_ascii_lowercase();
            continue;
        }
        match current_heading.as_str() {
            "context" => {
                context.push_str(line);
                context.push('\n');
            }
            "decision" => {
                decision.push_str(line);
                decision.push('\n');
            }
            "consequences" => {
                consequences.push_str(line);
                consequences.push('\n');
            }
            "alternatives considered" => {
                let l = line.trim();
                if let Some(rest) = l.strip_prefix("- ") {
                    alternatives.push(rest.trim().to_string());
                } else if let Some(rest) = l.strip_prefix("* ") {
                    alternatives.push(rest.trim().to_string());
                } else if !l.is_empty() {
                    alternatives.push(l.to_string());
                }
            }
            _ => {}
        }
    }

    (
        title,
        context.trim().to_string(),
        decision.trim().to_string(),
        consequences.trim().to_string(),
        alternatives,
    )
}

fn frontmatter_status_to_decision_status(s: &str) -> DecisionStatus {
    match s.to_lowercase().trim() {
        "accepted" => DecisionStatus::Accepted,
        "rejected" => DecisionStatus::Rejected,
        "superseded" => DecisionStatus::Superseded,
        "deprecated" => DecisionStatus::Deprecated,
        _ => DecisionStatus::Proposed,
    }
}

/// Build a [`KnowledgeGraph`](sruja_graph::KnowledgeGraph) decision row from a Decision Record file.
pub(crate) fn build_graph_decision(
    repo: &Path,
    md_path: &Path,
    fm: &DecisionFrontmatter,
    body: &str,
    previous: Option<&Decision>,
) -> Result<Decision, CliError> {
    let (parsed_title, context, decision_text, consequences, alternatives) =
        parse_decision_sections(body);
    let title = if parsed_title.is_empty() {
        fm.id.clone()
    } else {
        parsed_title
    };
    let rel = md_path
        .strip_prefix(repo)
        .unwrap_or(md_path)
        .to_string_lossy()
        .to_string();
    let status = frontmatter_status_to_decision_status(&fm.status);
    let now = Utc::now();
    let created_at = previous.map(|p| p.created_at).unwrap_or(now);
    let updated_at = now;
    let ratified_at = match status {
        DecisionStatus::Accepted => previous.and_then(|p| p.ratified_at).or(Some(now)),
        _ => previous.and_then(|p| p.ratified_at),
    };

    Ok(Decision {
        id: fm.id.clone(),
        title,
        status,
        context,
        decision: decision_text,
        consequences,
        alternatives,
        created_at,
        updated_at,
        ratified_at,
        author: None,
        source: SourceReference::adr_file(rel),
        affects: fm.elements.clone(),
    })
}

pub async fn decision_new(
    repo: &str,
    title: &str,
    record_type: &str,
    scope: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let id = create_decision_record(
        repo_path,
        title,
        record_type,
        scope,
        "sruja_decision_new",
        "human",
        "cli",
    )?;
    let path = decisions_dir(repo_path).join(format!("{id}.md"));
    eprintln!("Wrote {}", path.display());
    Ok(())
}

/// List Decision Record files (for CLI and MCP).
pub fn list_decisions(repo_path: &Path) -> Result<Vec<DecisionListItem>, CliError> {
    let dir = decisions_dir(repo_path);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut items: Vec<DecisionListItem> = Vec::new();
    for entry in fs::read_dir(&dir).map_err(CliError::Io)? {
        let entry = entry.map_err(CliError::Io)?;
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if let Ok((fm, body)) = parse_decision_file(&p) {
            let title = body
                .lines()
                .find(|l| l.starts_with("# "))
                .map(|l| l.trim_start_matches("# ").trim().to_string())
                .unwrap_or_else(|| fm.id.clone());
            items.push(DecisionListItem {
                id: fm.id,
                record_type: fm.record_type,
                status: fm.status,
                title,
                path: p
                    .strip_prefix(repo_path)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .to_string(),
                elements: fm.elements.clone(),
            });
        }
    }
    items.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(items)
}

/// Create a new proposed Decision Record; returns the assigned id.
pub fn create_decision_record(
    repo_path: &Path,
    title: &str,
    record_type: &str,
    scope: Option<&str>,
    source_tool: &str,
    actor: &str,
    source: &str,
) -> Result<String, CliError> {
    let dir = decisions_dir(repo_path);
    fs::create_dir_all(&dir).map_err(CliError::Io)?;
    let lock_path = dir.join(".id_alloc.lock");
    let lock_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(CliError::Io)?;
    lock_file.lock_exclusive().map_err(|e| {
        CliError::validation(format!(
            "could not lock {:?} for sequential decision ids: {}",
            lock_path, e
        ))
    })?;

    let year = Utc::now().format("%Y").to_string();
    let mut max_n = 0u32;
    for entry in fs::read_dir(&dir).map_err(CliError::Io)? {
        let entry = entry.map_err(CliError::Io)?;
        let name = entry.file_name().to_string_lossy().to_string();
        let prefix = format!("DR-{year}-");
        if let Some(rest) = name
            .strip_prefix(&prefix)
            .and_then(|s| s.strip_suffix(".md"))
        {
            if let Ok(n) = rest.parse::<u32>() {
                max_n = max_n.max(n);
            }
        }
    }
    let id = format!("DR-{}-{:03}", year, max_n + 1);

    let fm = DecisionFrontmatter {
        id: id.clone(),
        record_type: record_type.to_string(),
        status: "proposed".into(),
        scope: scope.unwrap_or("repo").to_string(),
        elements: vec![],
        actors: vec!["human".into()],
        sources: vec![],
        trace_id: None,
        supersedes: vec![],
        superseded_by: None,
    };
    let body = format!("# {title}\n\n## Context\n\n## Decision\n\n## Alternatives Considered\n\n## Evidence\n\n## Consequences\n\n## Follow-up Checks\n");
    let path = dir.join(format!("{id}.md"));
    if path.exists() {
        return Err(CliError::validation(format!(
            "decision file already exists: {}",
            path.display()
        )));
    }
    let content = format_decision_file(&fm, &body)?;
    fs::write(&path, content).map_err(CliError::Io)?;
    crate::graph_store::upsert_decision_record_in_stored_graph(repo_path, &path)?;
    append_context_event(
        repo_path,
        ContextEventRecord {
            schema_version: CONTEXT_EVENTS_SCHEMA_V2.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            kind: "decision_opened".into(),
            outcome: "ok".into(),
            policy_fingerprint: policy_fingerprint(repo_path),
            strict: None,
            details: serde_json::json!({ "decision_id": id }),
            trace_id: None,
            decision_id: Some(id.clone()),
            run_id: None,
            workflow_id: None,
            actor: Some(actor.to_string()),
            source: Some(source.to_string()),
            tool: Some(source_tool.to_string()),
            elements: None,
            subject_ids: None,
            evidence_refs: Some(vec![path.to_string_lossy().to_string()]),
            summary: Some(title.to_string()),
        },
    );
    Ok(id)
}

pub async fn decision_list(repo: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let items = list_decisions(repo_path)?;
    if items.is_empty() {
        if format == "json" {
            println!("[]");
        } else {
            println!("No decisions directory yet (.sruja/decisions/).");
        }
        return Ok(());
    }
    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&items)?);
    } else {
        for it in &items {
            println!(
                "{} [{}] {} — {}",
                it.id, it.status, it.record_type, it.title
            );
        }
    }
    Ok(())
}

fn resolve_decision_path(repo: &Path, id: &str) -> Result<PathBuf, CliError> {
    let dir = decisions_dir(repo);
    let p = dir.join(format!("{id}.md"));
    if p.is_file() {
        return Ok(p);
    }
    Err(CliError::validation(format!(
        "decision file not found for id {id} (expected {})",
        p.display()
    )))
}

pub async fn decision_show(repo: &str, id: &str) -> Result<(), CliError> {
    let p = resolve_decision_path(Path::new(repo), id)?;
    let raw = fs::read_to_string(&p).map_err(CliError::Io)?;
    print!("{raw}");
    Ok(())
}

pub async fn decision_trace(repo: &str, id: &str, limit: usize) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let events = crate::commands::context_events::read_context_events_query(
        repo_path,
        crate::commands::context_events::ContextEventQuery {
            limit,
            kind_filter: None,
            details_substring: None,
            decision_id: Some(id),
            trace_id: None,
            run_id: None,
            element_id: None,
            decision_lineage_only: false,
        },
    )
    .map_err(CliError::Io)?;
    println!("{}", serde_json::to_string_pretty(&events)?);
    Ok(())
}

pub async fn decision_link(repo: &str, id: &str, element: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let p = resolve_decision_path(repo_path, id)?;
    let (mut fm, body) = parse_decision_file(&p)?;
    let el = element.trim();
    if !el.is_empty() && !fm.elements.iter().any(|e| e == el) {
        fm.elements.push(el.to_string());
    }
    fs::write(&p, format_decision_file(&fm, &body)?).map_err(CliError::Io)?;
    crate::graph_store::upsert_decision_record_in_stored_graph(repo_path, &p)?;
    Ok(())
}

pub async fn decision_accept(repo: &str, id: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let p = resolve_decision_path(repo_path, id)?;
    let (mut fm, body) = parse_decision_file(&p)?;
    fm.status = "accepted".to_string();
    fs::write(&p, format_decision_file(&fm, &body)?).map_err(CliError::Io)?;
    crate::graph_store::upsert_decision_record_in_stored_graph(repo_path, &p)?;
    append_context_event(
        repo_path,
        ContextEventRecord {
            schema_version: CONTEXT_EVENTS_SCHEMA_V2.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            kind: "decision_accepted".into(),
            outcome: "ok".into(),
            policy_fingerprint: policy_fingerprint(repo_path),
            strict: None,
            details: serde_json::json!({ "decision_id": id }),
            trace_id: fm.trace_id.clone(),
            decision_id: Some(id.to_string()),
            run_id: None,
            workflow_id: None,
            actor: Some("human".into()),
            source: Some("cli".into()),
            tool: Some("sruja_decision_accept".into()),
            elements: if fm.elements.is_empty() {
                None
            } else {
                Some(fm.elements.clone())
            },
            subject_ids: None,
            evidence_refs: Some(vec![p.to_string_lossy().to_string()]),
            summary: Some(format!("Decision {id} accepted")),
        },
    );
    Ok(())
}

pub async fn decision_supersede(repo: &str, id: &str, by: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let p = resolve_decision_path(repo_path, id)?;
    let (mut fm, body) = parse_decision_file(&p)?;
    fm.status = "superseded".into();
    fm.superseded_by = Some(by.to_string());
    fs::write(&p, format_decision_file(&fm, &body)?).map_err(CliError::Io)?;
    crate::graph_store::upsert_decision_record_in_stored_graph(repo_path, &p)?;
    append_context_event(
        repo_path,
        ContextEventRecord {
            schema_version: CONTEXT_EVENTS_SCHEMA_V2.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            kind: "decision_superseded".into(),
            outcome: "ok".into(),
            policy_fingerprint: policy_fingerprint(repo_path),
            strict: None,
            details: serde_json::json!({ "decision_id": id, "superseded_by": by }),
            trace_id: fm.trace_id.clone(),
            decision_id: Some(id.to_string()),
            run_id: None,
            workflow_id: None,
            actor: Some("human".into()),
            source: Some("cli".into()),
            tool: Some("sruja_decision_supersede".into()),
            elements: if fm.elements.is_empty() {
                None
            } else {
                Some(fm.elements.clone())
            },
            subject_ids: None,
            evidence_refs: Some(vec![p.to_string_lossy().to_string()]),
            summary: Some(format!("Decision {id} superseded by {by}")),
        },
    );
    Ok(())
}
