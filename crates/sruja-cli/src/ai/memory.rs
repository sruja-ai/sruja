//! Repo-local memory store: `.sruja/memory/` with facts, interactions, feedback, state.
//!
//! All paths are resolved relative to the repository root. No global state.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::facts::Fact;
use super::schemas::{FeedbackRecord, InteractionRecord};
use crate::commands::CliError;

/// Directory name under repo root for memory files.
pub const MEMORY_DIR: &str = ".sruja/memory";

/// Filenames (under MEMORY_DIR).
pub const FACTS_FILE: &str = "facts.jsonl";
pub const INTERACTIONS_FILE: &str = "interactions.jsonl";
pub const FEEDBACK_FILE: &str = "feedback.jsonl";
pub const STATE_FILE: &str = "state.json";

/// Resolve absolute path to repo memory dir. Does not create it.
pub fn memory_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(MEMORY_DIR)
}

fn facts_path(repo_root: &Path) -> PathBuf {
    memory_dir(repo_root).join(FACTS_FILE)
}

fn interactions_path(repo_root: &Path) -> PathBuf {
    memory_dir(repo_root).join(INTERACTIONS_FILE)
}

fn feedback_path(repo_root: &Path) -> PathBuf {
    memory_dir(repo_root).join(FEEDBACK_FILE)
}

fn state_path(repo_root: &Path) -> PathBuf {
    memory_dir(repo_root).join(STATE_FILE)
}

/// Ensure `.sruja/memory` exists.
pub fn ensure_memory_dir(repo_root: &Path) -> Result<(), CliError> {
    let dir = memory_dir(repo_root);
    fs::create_dir_all(&dir).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create {}: {}", dir.display(), e),
        ))
    })?;
    Ok(())
}

/// Generate a unique id with prefix (e.g. fact_01H..., ans_01H...).
/// Uses timestamp in hex + nanos remainder for uniqueness.
fn new_id(prefix: &str) -> String {
    let now = chrono::Utc::now();
    let ms = now.timestamp_millis() as u64;
    let ns = now.timestamp_nanos_opt().unwrap_or(0) as u64;
    format!("{}_{:012x}{:04x}", prefix, ms, ns % 65536)
}

/// Append one JSON line to a file. Creates file and parent dir if needed.
fn append_jsonl(repo_root: &Path, filename: &str, line: &str) -> Result<(), CliError> {
    ensure_memory_dir(repo_root)?;
    let path = memory_dir(repo_root).join(filename);
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| CliError::Io(e))?;
    writeln!(f, "{}", line).map_err(|e| CliError::Io(e))?;
    Ok(())
}

/// Read all non-empty lines from a JSONL file, parsed as T. Missing file => empty vec.
fn read_jsonl<T: for<'de> Deserialize<'de>>(
    path: &Path,
) -> Result<Vec<T>, CliError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).map_err(CliError::Io)?;
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let t: T = serde_json::from_str(line).map_err(CliError::Json)?;
        out.push(t);
    }
    Ok(out)
}

// --- Facts

/// Load all facts for the repo. Missing file => empty.
pub fn load_facts(repo_root: &Path) -> Result<Vec<Fact>, CliError> {
    read_jsonl(&facts_path(repo_root))
}

/// Append a fact and return its assigned fact_id.
pub fn append_fact(repo_root: &Path, mut fact: Fact) -> Result<String, CliError> {
    if fact.fact_id.is_empty() {
        fact.fact_id = new_id("fact");
    }
    let line = serde_json::to_string(&fact).map_err(CliError::Json)?;
    append_jsonl(repo_root, FACTS_FILE, &line)?;
    Ok(fact.fact_id.clone())
}

/// Overwrite facts file with updated list (e.g. after feedback). Use sparingly.
pub fn write_facts(repo_root: &Path, facts: &[Fact]) -> Result<(), CliError> {
    ensure_memory_dir(repo_root)?;
    let path = facts_path(repo_root);
    let mut f = fs::File::create(&path).map_err(CliError::Io)?;
    for fact in facts {
        let line = serde_json::to_string(fact).map_err(CliError::Json)?;
        writeln!(f, "{}", line).map_err(CliError::Io)?;
    }
    Ok(())
}

// --- Interactions

/// Load all interactions. Missing file => empty.
pub fn load_interactions(repo_root: &Path) -> Result<Vec<InteractionRecord>, CliError> {
    read_jsonl(&interactions_path(repo_root))
}

/// Append an interaction and return answer_id.
pub fn append_interaction(
    repo_root: &Path,
    mut rec: InteractionRecord,
) -> Result<String, CliError> {
    if rec.answer_id.is_empty() {
        rec.answer_id = new_id("ans");
    }
    let line = serde_json::to_string(&rec).map_err(CliError::Json)?;
    append_jsonl(repo_root, INTERACTIONS_FILE, &line)?;
    Ok(rec.answer_id.clone())
}

// --- Feedback

/// Load all feedback. Missing file => empty.
pub fn load_feedback(repo_root: &Path) -> Result<Vec<FeedbackRecord>, CliError> {
    read_jsonl(&feedback_path(repo_root))
}

/// Append a feedback record.
pub fn append_feedback(repo_root: &Path, mut rec: FeedbackRecord) -> Result<String, CliError> {
    if rec.feedback_id.is_empty() {
        rec.feedback_id = new_id("fb");
    }
    let line = serde_json::to_string(&rec).map_err(CliError::Json)?;
    append_jsonl(repo_root, FEEDBACK_FILE, &line)?;
    Ok(rec.feedback_id.clone())
}

// --- State (optional: last validated sha, etc.)

/// In-memory state for the repo. Persisted as state.json.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MemoryState {
    pub last_validated_sha: Option<String>,
    pub last_commit_sha: Option<String>,
}

/// Load state. Missing => default.
pub fn load_state(repo_root: &Path) -> Result<MemoryState, CliError> {
    let path = state_path(repo_root);
    if !path.exists() {
        return Ok(MemoryState::default());
    }
    let content = fs::read_to_string(&path).map_err(CliError::Io)?;
    serde_json::from_str(&content).map_err(CliError::Json)
}

/// Save state.
pub fn save_state(repo_root: &Path, state: &MemoryState) -> Result<(), CliError> {
    ensure_memory_dir(repo_root)?;
    let path = state_path(repo_root);
    let json = serde_json::to_string_pretty(state).map_err(CliError::Json)?;
    fs::write(&path, json).map_err(CliError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::facts::Fact;

    #[test]
    fn fact_roundtrip_append_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let fact = Fact {
            fact_id: "fact_test1".to_string(),
            statement: "Test statement".to_string(),
            kind: "flow".to_string(),
            status: "candidate".to_string(),
            confidence: 0.7,
            source: "llm".to_string(),
            repo: "/tmp/repo".to_string(),
            commit_sha: Some("abc1234".to_string()),
            evidence: vec![],
            tags: vec!["t1".to_string()],
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            last_validated_sha: Some("abc1234".to_string()),
        };
        let id = append_fact(root, fact).unwrap();
        assert_eq!(id, "fact_test1");
        let loaded = load_facts(root).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].statement, "Test statement");
        assert_eq!(loaded[0].confidence, 0.7);
    }
}
