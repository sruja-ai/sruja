//! Timeline commands: suggest architecture-significant refs via LLM from git log.
//!
//! Used by capture_timeline.sh when no refs are given. Reuses shared LLM module.

use std::path::Path;
use std::process::Command;

use super::llm::call_llm;
use super::CliError;

const MAX_COMMITS: usize = 300;
const MAX_SUGGESTED_REFS: usize = 25;

const SUGGEST_REFS_SYSTEM: &str = r#"You are helping select git commits that represent significant architecture or structure changes in a codebase.

Given a list of commits (one per line: SHORT_SHA TAB ISO_DATE TAB SUBJECT), identify which commits are likely to represent:
- Refactors, new modules, splits or merges of components
- Migrations, new services, major dependency or layout changes
- Architectural decisions (not routine fixes, docs, style, or typo-only changes)

Reply with ONLY a JSON array of the short SHAs (7-char) in chronological order (oldest first), at most 25 commits. No explanation.
Example: ["a1b2c3d","e4f5g6h"]"#;

/// Run `git log` in the repo and return lines: "SHORT_SHA\tDATE\tSUBJECT".
fn git_log_commits(repo_path: &Path, max: usize) -> Result<Vec<String>, CliError> {
    let out = Command::new("git")
        .args(["-C", repo_path.as_os_str().to_str().unwrap_or("."), "log"])
        .arg(format!("-{}", max))
        .arg("--format=%h%x09%ci%x09%s")
        .output()
        .map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    if !out.status.success() {
        return Err(CliError::Validation(format!(
            "git log failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }

    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<String> = s
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();

    Ok(lines)
}

/// Suggest architecture-significant refs (short SHAs) from the repo's git log using an LLM.
/// Output is a JSON array of strings printed to stdout for capture_timeline.sh to consume.
pub async fn suggest_refs(repo_path: &str) -> Result<(), CliError> {
    let path = Path::new(repo_path);
    if !path.exists() {
        return Err(CliError::Validation(format!("Repository path does not exist: {}", repo_path)));
    }
    if !path.join(".git").exists() {
        return Err(CliError::Validation(format!("Not a git repository: {}", repo_path)));
    }

    let mut lines = git_log_commits(path, MAX_COMMITS)?;
    if lines.is_empty() {
        return Err(CliError::Validation("No commits in repository".to_string()));
    }
    // git log is newest-first; prompt expects chronological (oldest at top) for manifest order
    lines.reverse();
    let commit_list = lines.join("\n");
    let user_prompt = format!(
        "List of commits (SHORT_SHA TAB DATE TAB SUBJECT), chronological (oldest at top):\n\n{}\n\nReturn a JSON array of short SHAs (at most {}) that represent architecture-significant commits, in chronological order. Only the array, no other text.",
        commit_list,
        MAX_SUGGESTED_REFS
    );

    let text = call_llm(SUGGEST_REFS_SYSTEM, &user_prompt).await?;
    let text = text
        .trim()
        .strip_prefix("```json")
        .unwrap_or(text.trim())
        .strip_suffix("```")
        .unwrap_or(text.trim())
        .trim();

    let array_start = text.find('[').ok_or_else(|| {
        CliError::Validation("LLM response did not contain a JSON array".to_string())
    })?;
    let array_end = text.rfind(']').ok_or_else(|| {
        CliError::Validation("LLM response did not contain a JSON array".to_string())
    })? + 1;
    let array_str = &text[array_start..array_end];

    let shas: Vec<String> = serde_json::from_str(array_str).map_err(|e| {
        CliError::Validation(format!("Could not parse LLM response as JSON array: {}", e))
    })?;

    // Validate: all elements should be short hex strings; cap at MAX_SUGGESTED_REFS
    let shas: Vec<String> = shas
        .into_iter()
        .filter_map(|s| {
            let s = s.trim().trim_matches('"');
            if s.chars().all(|c| c.is_ascii_hexdigit()) && !s.is_empty() && s.len() <= 40 {
                Some(s.to_string())
            } else {
                None
            }
        })
        .take(MAX_SUGGESTED_REFS)
        .collect();

    // Output JSON array for shell to parse (e.g. jq -r '.[]')
    println!("{}", serde_json::to_string(&shas).map_err(CliError::Json)?);
    Ok(())
}

const TIMELINE_EXPLAIN_WINDOW: usize = 200;
const TIMELINE_EXPLAIN_TOP_K: usize = 30;

const TIMELINE_EXPLAIN_SYSTEM: &str = r#"You are summarizing architecture evolution from a list of git commits.
Given a chronological list (oldest first) of SHORT_SHA, DATE, SUBJECT, write 1-3 short paragraphs describing how the architecture or structure of the codebase evolved. Focus on refactors, new modules, migrations, and structural changes. Be concise."#;

/// Explain architecture evolution across a smart subset of commits (deterministic scoring).
pub async fn timeline_explain(
    repo_path: &str,
    max_commits: usize,
    format: &str,
) -> Result<(), CliError> {
    let path = Path::new(repo_path);
    if !path.exists() {
        return Err(CliError::Validation(format!(
            "Repository path does not exist: {}",
            repo_path
        )));
    }
    if !path.join(".git").exists() {
        return Err(CliError::Validation(format!(
            "Not a git repository: {}",
            repo_path
        )));
    }

    let candidates =
        crate::ai::score_commits(path, TIMELINE_EXPLAIN_WINDOW, TIMELINE_EXPLAIN_TOP_K, max_commits)?;
    if candidates.is_empty() {
        println!("No commits selected.");
        return Ok(());
    }

    let summary = build_timeline_summary(&candidates).await;

    if format == "json" {
        let out: Vec<serde_json::Value> = candidates
            .iter()
            .map(|c| {
                serde_json::json!({
                    "short_sha": c.short_sha,
                    "date": c.date,
                    "subject": c.subject,
                    "score": c.score,
                })
            })
            .collect();
        let json_out = serde_json::json!({
            "commits": out,
            "summary_markdown": summary.as_deref().unwrap_or(""),
        });
        println!("{}", serde_json::to_string_pretty(&json_out).map_err(CliError::Json)?);
    } else {
        println!("Architecture-significant commits (oldest → newest, max {})", max_commits);
        println!("{}", "─".repeat(60));
        for c in &candidates {
            println!("  {}  {}  (score: {:.1})  {}", c.date, c.short_sha, c.score, c.subject);
        }
        println!("{}", "─".repeat(60));
        if let Some(ref s) = summary {
            println!("\nEvolution summary:");
            println!("{}", "─".repeat(60));
            println!("{}", s.trim());
        }
    }
    Ok(())
}

/// Build a short LLM summary of architecture evolution for the given commits. Returns None on LLM failure.
async fn build_timeline_summary(
    candidates: &[crate::ai::CommitCandidate],
) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }
    let list: String = candidates
        .iter()
        .map(|c| format!("  {}  {}  {}", c.date, c.short_sha, c.subject))
        .collect::<Vec<_>>()
        .join("\n");
    let user = format!(
        "Commits (oldest → newest):\n\n{}\n\nWrite a brief architecture evolution summary (1-3 paragraphs).",
        list
    );
    match call_llm(TIMELINE_EXPLAIN_SYSTEM, &user).await {
        Ok(s) => Some(s.trim().to_string()),
        Err(_) => None,
    }
}
