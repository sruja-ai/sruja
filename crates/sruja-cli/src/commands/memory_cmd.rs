//! `sruja memory` — indexed search over learnings, context events, and decision records.

use crate::commands::CliError;
use sruja_memory::{MemoryStore, SearchMemoryOptions, TimelineOptions};
use std::collections::HashMap;
use std::path::Path;

pub fn memory_reindex(repo: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::validation(format!(
            "Repository not found: {repo}"
        )));
    }
    let mut store = MemoryStore::open(repo_path).map_err(map_memory_err)?;
    store.reindex().map_err(map_memory_err)?;
    println!(r#"{{"ok":true,"schema_version":"memory_index/v1"}}"#);
    Ok(())
}

pub fn memory_search(
    repo: &str,
    query: &str,
    element_id: Option<&str>,
    decision_id: Option<&str>,
    hitl_kind: Option<&str>,
    limit: usize,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let store = MemoryStore::open(repo_path).map_err(map_memory_err)?;
    let hits = store
        .search(SearchMemoryOptions {
            query,
            element_id,
            decision_id,
            hitl_kind,
            source: None,
            trust: None,
            limit,
        })
        .map_err(map_memory_err)?;
    let out = serde_json::json!({
        "schema_version": "memory_search/v1",
        "query": query,
        "count": hits.len(),
        "hits": hits,
        "note": "Results are hypothesis or reviewed_truth; never auto-merge into repo.sruja."
    });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

pub fn memory_timeline(
    repo: &str,
    anchor_id: Option<&str>,
    anchor_timestamp: Option<&str>,
    before: usize,
    after: usize,
    decision_id: Option<&str>,
    element_id: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let store = MemoryStore::open(repo_path).map_err(map_memory_err)?;
    let tl = store
        .timeline(TimelineOptions {
            anchor_id,
            anchor_timestamp,
            before,
            after,
            decision_id,
            element_id,
        })
        .map_err(map_memory_err)?;
    println!("{}", serde_json::to_string_pretty(&tl)?);
    Ok(())
}

/// Per-skill effectiveness stats derived from context events.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillStats {
    pub skill: String,
    pub total_uses: usize,
    pub successes: usize,
    pub failures: usize,
    pub success_rate: f64,
}

/// Computes skill effectiveness from context events that recorded `skills_used`.
pub fn memory_skill_stats(repo: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::validation(format!(
            "Repository not found: {repo}"
        )));
    }

    let events = crate::commands::context_events::read_context_events_query(
        repo_path,
        crate::commands::context_events::ContextEventQuery {
            limit: 1000,
            kind_filter: None,
            details_substring: None,
            decision_id: None,
            trace_id: None,
            run_id: None,
            element_id: None,
            decision_lineage_only: false,
        },
    )
    .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    // Map skill -> (successes, failures)
    let mut skill_map: HashMap<String, (usize, usize)> = HashMap::new();

    for ev in &events {
        if let Some(skills) = &ev.skills_used {
            let is_success = matches!(
                ev.outcome.as_str(),
                "ok" | "pass" | "passed" | "success" | "succeeded"
            );
            let is_failure = matches!(ev.outcome.as_str(), "fail" | "failed" | "error");
            if !is_success && !is_failure {
                continue;
            }
            for skill in skills {
                let entry = skill_map.entry(skill.clone()).or_insert((0, 0));
                if is_success {
                    entry.0 += 1;
                } else {
                    entry.1 += 1;
                }
            }
        }
    }

    let mut stats: Vec<SkillStats> = skill_map
        .into_iter()
        .map(|(skill, (successes, failures))| {
            let total = successes + failures;
            SkillStats {
                skill,
                total_uses: total,
                successes,
                failures,
                success_rate: if total > 0 {
                    successes as f64 / total as f64
                } else {
                    0.0
                },
            }
        })
        .collect();
    stats.sort_by_key(|b| std::cmp::Reverse(b.total_uses));

    if format == "json" {
        let out = serde_json::json!({
            "schema_version": "skill_stats/v1",
            "total_skills": stats.len(),
            "skills": stats,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!(
            "{}",
            crate::utils::colors::style("Skill Effectiveness").bold()
        );
        println!("{}", crate::utils::colors::dim("──────────────────"));
        if stats.is_empty() {
            println!("  No skill usage recorded in context events yet.");
        } else {
            for s in &stats {
                let rate = format!("{:.0}%", s.success_rate * 100.0);
                println!(
                    "  {} uses={} success={} fail={} rate={}",
                    crate::utils::colors::info(&s.skill),
                    s.total_uses,
                    s.successes,
                    s.failures,
                    rate
                );
            }
        }
    }

    Ok(())
}

/// Archives stale learnings based on decay score and age thresholds.
pub fn memory_archive(
    repo: &str,
    decay_threshold: f64,
    min_age_days: i64,
    force: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::validation(format!(
            "Repository not found: {repo}"
        )));
    }

    let mut memory = sruja_agent::AgenticMemory::load(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    if !force {
        // Dry-run: apply same filtering as auto_archive_stale for consistent preview.
        // Don't use curation_report's pre-filtered stale_entries since those use a
        // hardcoded threshold that may differ from the user-provided one.
        let candidates: Vec<_> = memory
            .learnings
            .iter()
            .filter(|e| {
                e.decay_score() < decay_threshold
                    && e.age_days() > min_age_days
                    && e.kind != Some(sruja_agent::LearningKind::Invariant)
            })
            .collect();
        if candidates.is_empty() {
            println!(
                "No entries meet the archive threshold (decay < {}, age > {} days).",
                decay_threshold, min_age_days
            );
        } else {
            println!(
                "{} stale entries would be archived (use --force to apply):",
                candidates.len()
            );
            for c in &candidates {
                println!(
                    "  {} age={}d decay={:.3} | {}",
                    &c.id[..c.id.len().min(12)],
                    c.age_days(),
                    c.decay_score(),
                    crate::utils::colors::dim(&c.context)
                );
            }
        }
        return Ok(());
    }

    let archived = memory.auto_archive_stale(decay_threshold, min_age_days);
    if archived.is_empty() {
        println!("No entries met the archive threshold.");
        return Ok(());
    }

    memory
        .save(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    println!("Archived {} stale learnings.", archived.len());
    for entry in &archived {
        println!(
            "  {} (age={}d, decay={:.3})",
            &entry.id[..entry.id.len().min(12)],
            entry.age_days(),
            entry.decay_score()
        );
    }
    Ok(())
}

fn map_memory_err(e: sruja_memory::MemoryStoreError) -> CliError {
    match e {
        sruja_memory::MemoryStoreError::Validation(msg) => CliError::validation(msg),
        other => CliError::Io(std::io::Error::other(other.to_string())),
    }
}
