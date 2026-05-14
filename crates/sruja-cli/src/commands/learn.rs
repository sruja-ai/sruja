//! Learned architecture facts: deterministic scan evidence plus drift-derived hypotheses.
//!
//! Writes `.sruja/evidence_graph.json`, `.sruja/learned_facts.jsonl`, optional proposal JSON
//! under `.sruja/proposals/`, and appends `.sruja/context_events.jsonl` / learn feedback log.
//! Never mutates `repo.sruja`.

use super::{parse_sruja_file, CliError};
use serde::{Deserialize, Serialize};
use sruja_diff::{
    compare_graphs_with_options, program_to_graph, BaselineMode, DiffResult, Proposal,
    ProposalChange, ProposalStatus,
};
use sruja_scan::{Edge, Graph, NodeKind};
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const EVIDENCE_SCHEMA: &str = "sruja/evidence_graph/v1";
const LEARNED_FACT_SCHEMA: &str = "learned_fact/v1";

/// Lifecycle for a learned claim (separate from reviewed DSL truth).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactStatus {
    Observed,
    Inferred,
    Proposed,
    Reviewed,
    Rejected,
    Stale,
}

impl FactStatus {
    fn as_str(self) -> &'static str {
        match self {
            FactStatus::Observed => "observed",
            FactStatus::Inferred => "inferred",
            FactStatus::Proposed => "proposed",
            FactStatus::Reviewed => "reviewed",
            FactStatus::Rejected => "rejected",
            FactStatus::Stale => "stale",
        }
    }
}

/// One line in `.sruja/learned_facts.jsonl` (MVP contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedFact {
    pub schema_version: String,
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub claim: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    pub confidence: f64,
    pub status: FactStatus,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGraphFile {
    pub schema_version: String,
    pub generated_at: String,
    pub repo_root: String,
    pub graph: Graph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnFeedbackRecord {
    pub timestamp: String,
    pub fact_id: String,
    pub decision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

fn sruja_dir(repo: &Path) -> PathBuf {
    repo.join(".sruja")
}

fn learned_facts_path(repo: &Path) -> PathBuf {
    sruja_dir(repo).join("learned_facts.jsonl")
}

fn evidence_graph_path(repo: &Path) -> PathBuf {
    sruja_dir(repo).join("evidence_graph.json")
}

fn learn_feedback_path(repo: &Path) -> PathBuf {
    sruja_dir(repo).join("learn_feedback.jsonl")
}

fn stable_fact_id(subject: &str, predicate: &str, object: &str) -> String {
    let raw = format!("{subject}|{predicate}|{object}");
    let h = blake3::hash(raw.as_bytes()).to_hex();
    format!("fact_{}", &h.as_str()[..16])
}

fn normalize_repo_rel_path(repo: &Path, p: &str) -> String {
    let path = Path::new(p);
    if path.is_absolute() {
        path.strip_prefix(repo)
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or_else(|_| p.to_string())
    } else {
        p.trim_start_matches("./").to_string()
    }
}

/// Paths changed between `since_ref` and `HEAD`.
///
/// Returns `Err` when `git` cannot be run or the ref range is invalid (so callers do not
/// silently treat a bad `--since` as “no files changed”).
pub fn git_changed_paths(repo: &Path, since_ref: &str) -> Result<HashSet<String>, String> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(["diff", "--name-only", &format!("{since_ref}...HEAD")])
        .output()
        .map_err(|e| format!("failed to execute git: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let hint = stderr.trim();
        let detail = if hint.is_empty() {
            String::new()
        } else {
            format!(" ({hint})")
        };
        return Err(format!(
            "git diff --name-only {since_ref:?}...HEAD exited non-zero{detail}"
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// Fact IDs the user rejected via feedback log (do not re-emit matching proposals).
pub fn rejected_fact_ids(repo: &Path) -> HashSet<String> {
    let path = learn_feedback_path(repo);
    let Ok(file) = File::open(&path) else {
        return HashSet::new();
    };
    let mut out = HashSet::new();
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(rec) = serde_json::from_str::<LearnFeedbackRecord>(line) {
            if rec.decision.eq_ignore_ascii_case("reject")
                || rec.decision.eq_ignore_ascii_case("rejected")
            {
                out.insert(rec.fact_id);
            }
        }
    }
    out
}

/// Append a human decision on a learned fact (approve/reject) for future runs.
pub fn append_learn_feedback(
    repo: &Path,
    fact_id: &str,
    decision: &str,
    reason: Option<&str>,
) -> Result<(), CliError> {
    let dir = sruja_dir(repo);
    std::fs::create_dir_all(&dir)?;
    let path = learn_feedback_path(repo);
    let rec = LearnFeedbackRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        fact_id: fact_id.to_string(),
        decision: decision.to_string(),
        reason: reason.map(String::from),
    };
    let mut f = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(f, "{}", serde_json::to_string(&rec)?)?;
    Ok(())
}

/// Read up to `limit` facts from `.sruja/learned_facts.jsonl` (newest last; returns last `limit` lines that parse).
pub fn read_learned_facts(
    repo: &Path,
    limit: usize,
    status: Option<&str>,
) -> Result<Vec<LearnedFact>, CliError> {
    let path = learned_facts_path(repo);
    if !path.exists() || limit == 0 {
        return Ok(Vec::new());
    }
    let file = File::open(&path).map_err(CliError::Io)?;
    let mut parsed: Vec<LearnedFact> = Vec::new();
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(f) = serde_json::from_str::<LearnedFact>(line) {
            if let Some(s) = status {
                if f.status.as_str() != s {
                    continue;
                }
            }
            parsed.push(f);
        }
    }
    if parsed.len() > limit {
        let start = parsed.len() - limit;
        parsed = parsed.split_off(start);
    }
    Ok(parsed)
}

/// Return one fact by id, or `None` if missing (single pass over the JSONL file).
pub fn get_learned_fact_by_id(repo: &Path, fact_id: &str) -> Result<Option<LearnedFact>, CliError> {
    let path = learned_facts_path(repo);
    let Ok(file) = File::open(&path) else {
        return Ok(None);
    };
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(f) = serde_json::from_str::<LearnedFact>(line) {
            if f.id == fact_id {
                return Ok(Some(f));
            }
        }
    }
    Ok(None)
}

fn normalize_rel_path(p: &str) -> String {
    p.trim().trim_start_matches("./").replace('\\', "/")
}

/// Path or prefix match for `--file`: exact path, directory prefix, or final path segment.
/// Avoids naive `contains` false positives (e.g. `a.rs` matching `foo_a.rs`).
fn path_matches_focus(focus_raw: &str, path_raw: &str) -> bool {
    let focus = normalize_rel_path(focus_raw);
    let path = normalize_rel_path(path_raw);
    if focus.is_empty() {
        return true;
    }
    if path == focus {
        return true;
    }
    if path.starts_with(&focus) && path.as_bytes().get(focus.len()) == Some(&b'/') {
        return true;
    }
    if !focus.contains('/') {
        return path.ends_with(&format!("/{focus}"));
    }
    path.ends_with(&format!("/{focus}"))
}

fn subject_may_be_path(subject: &str) -> bool {
    let s = normalize_rel_path(subject);
    s.contains('/') && !s.contains("::")
}

fn evidence_refs_for_edge(edge: &Edge) -> Vec<String> {
    let mut v: Vec<String> = edge
        .evidence
        .iter()
        .filter_map(|e| e.file.as_ref().cloned())
        .collect();
    v.sort();
    v.dedup();
    v
}

fn passes_path_filters(
    evidence_refs: &[String],
    subject: &str,
    file_filter: Option<&str>,
    changed: Option<&HashSet<String>>,
) -> bool {
    if let Some(f) = file_filter {
        let hit_evidence = evidence_refs.iter().any(|p| path_matches_focus(f, p));
        let hit_subject = subject_may_be_path(subject) && path_matches_focus(f, subject);
        if !hit_evidence && !hit_subject {
            return false;
        }
    }
    if let Some(set) = changed {
        if set.is_empty() {
            return true;
        }
        let norm: Vec<String> = set.iter().map(|p| normalize_rel_path(p)).collect();
        let touched = evidence_refs
            .iter()
            .any(|p| norm.iter().any(|q| q == p || path_matches_focus(q, p)))
            || (subject_may_be_path(subject)
                && norm
                    .iter()
                    .any(|q| q == subject || path_matches_focus(q, subject)));
        if !touched {
            return false;
        }
    }
    true
}

/// Run the learn pipeline: scan → evidence file → learned facts → optional proposals.
pub async fn learn(
    repo_root: &str,
    file: Option<&str>,
    since: Option<&str>,
    skip_proposals: bool,
    format: &str,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    if !repo.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {repo_root}"),
        )));
    }

    std::fs::create_dir_all(sruja_dir(repo))?;

    let changed = match since {
        None => None,
        Some(s) => Some(git_changed_paths(repo, s).map_err(|e| {
            CliError::validation(format!(
                "{e}. Use a valid git ref (e.g. main) or omit --since."
            ))
        })?),
    };

    let graph = sruja_scan::scan_repo(repo).map_err(|e| CliError::scan(e.to_string()))?;

    let evidence = EvidenceGraphFile {
        schema_version: EVIDENCE_SCHEMA.to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        repo_root: repo.to_string_lossy().to_string(),
        graph: graph.clone(),
    };
    let evidence_json = serde_json::to_string_pretty(&evidence)?;
    std::fs::write(evidence_graph_path(repo), &evidence_json)?;

    let mut facts: Vec<LearnedFact> = Vec::new();

    for node in &graph.nodes {
        if let Some(ref p) = node.path {
            let rel = normalize_repo_rel_path(repo, p);
            let refs = vec![rel.clone()];
            if !passes_path_filters(&refs, &rel, file, changed.as_ref()) {
                continue;
            }
            let id = stable_fact_id(&rel, "implemented_as", &node.id);
            facts.push(LearnedFact {
                schema_version: LEARNED_FACT_SCHEMA.to_string(),
                id,
                subject: rel,
                predicate: "implemented_as".to_string(),
                object: node.id.clone(),
                claim: format!(
                    "Source tree module `{}` is anchored at file path `{}`.",
                    node.id,
                    node.path.as_deref().unwrap_or("")
                ),
                evidence_refs: refs,
                confidence: 0.92,
                status: FactStatus::Observed,
                source: "deterministic_scan".to_string(),
            });
        }
    }

    let rejected = rejected_fact_ids(repo);

    let arch_path = crate::utils::architecture_path::resolve_architecture_path(repo);
    let mut undocumented_edges: HashSet<(String, String, String)> = HashSet::new();
    let mut diff_opt: Option<DiffResult> = None;
    let mut declared_graph: Option<Graph> = None;

    if let Some(ref ap) = arch_path {
        if ap.exists() {
            let (_content, program) = parse_sruja_file(ap)?;
            let proposed_graph = program_to_graph(&program);
            let diff = compare_graphs_with_options(
                &graph,
                &proposed_graph,
                sruja_diff::CompareOptions {
                    baseline_mode: BaselineMode::Auto,
                },
            );
            for e in &diff.edge_diff.removed {
                let pred = e.kind.kind_str();
                undocumented_edges.insert((e.source.clone(), e.target.clone(), pred.to_string()));
            }
            diff_opt = Some(diff);
            declared_graph = Some(proposed_graph);
        }
    }

    for edge in &graph.edges {
        let refs = evidence_refs_for_edge(edge);
        if !passes_path_filters(&refs, &edge.source, file, changed.as_ref()) {
            continue;
        }
        let pred = edge.kind.kind_str();
        let id = stable_fact_id(&edge.source, pred, &edge.target);
        let key = (edge.source.clone(), edge.target.clone(), pred.to_string());
        let (status, source, confidence, claim) = if undocumented_edges.contains(&key) {
            if rejected.contains(&id) {
                continue;
            }
            (
                FactStatus::Proposed,
                "drift_compare",
                if refs.is_empty() { 0.62 } else { 0.78 },
                format!(
                    "Scan found `{}` {} `{}`, but the reviewed architecture does not declare this edge.",
                    edge.source,
                    pred.replace('_', " "),
                    edge.target
                ),
            )
        } else {
            (
                FactStatus::Observed,
                "deterministic_scan",
                if refs.is_empty() { 0.55 } else { 0.88 },
                format!(
                    "`{}` {} `{}` (from static analysis).",
                    edge.source,
                    pred.replace('_', " "),
                    edge.target
                ),
            )
        };
        facts.push(LearnedFact {
            schema_version: LEARNED_FACT_SCHEMA.to_string(),
            id,
            subject: edge.source.clone(),
            predicate: pred.to_string(),
            object: edge.target.clone(),
            claim,
            evidence_refs: refs,
            confidence,
            status,
            source: source.to_string(),
        });
    }

    if let (Some(ref ap), Some(diff)) = (&arch_path, &diff_opt) {
        if ap.exists() {
            for node in &diff.node_diff.removed {
                if node.kind != NodeKind::Module {
                    continue;
                }
                let scan_node = graph.nodes.iter().find(|n| n.id == node.id);
                let refs: Vec<String> = scan_node
                    .and_then(|n| n.path.as_ref())
                    .map(|p| vec![normalize_repo_rel_path(repo, p)])
                    .unwrap_or_default();
                if !passes_path_filters(&refs, &node.id, file, changed.as_ref()) {
                    continue;
                }
                let id = stable_fact_id(&node.id, "undocumented_module", "architecture");
                if rejected.contains(&id) {
                    continue;
                }
                facts.push(LearnedFact {
                    schema_version: LEARNED_FACT_SCHEMA.to_string(),
                    id,
                    subject: node.id.clone(),
                    predicate: "undocumented_in".to_string(),
                    object: ap.to_string_lossy().to_string(),
                    claim: format!(
                        "Module `{}` exists in the scan graph but is not represented in `{}`.",
                        node.id,
                        ap.file_name().unwrap_or_default().to_string_lossy()
                    ),
                    evidence_refs: refs,
                    confidence: 0.7,
                    status: FactStatus::Proposed,
                    source: "drift_compare".to_string(),
                });
            }
        }
    }

    let facts_path = learned_facts_path(repo);
    let mut ff = File::create(&facts_path).map_err(CliError::Io)?;
    for f in &facts {
        writeln!(ff, "{}", serde_json::to_string(f)?)?;
    }

    if !skip_proposals {
        let resolve = declared_graph.as_ref().zip(diff_opt.as_ref());
        emit_learn_proposals(repo, &facts, &rejected, resolve)?;
    }

    crate::commands::context_events::append_context_event(
        repo,
        crate::commands::context_events::ContextEventRecord {
            schema_version: crate::commands::context_events::CONTEXT_EVENTS_SCHEMA.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            kind: "learn_run".to_string(),
            outcome: "ok".to_string(),
            policy_fingerprint: crate::commands::context_events::policy_fingerprint(repo),
            strict: None,
            details: serde_json::json!({
                "facts_written": facts.len(),
                "skip_proposals": skip_proposals,
                "file_filter": file,
                "since": since,
            }),
        },
    );

    match format {
        "json" => {
            let summary = serde_json::json!({
                "evidence_graph": evidence_graph_path(repo).to_string_lossy(),
                "learned_facts": facts_path.to_string_lossy(),
                "fact_count": facts.len(),
                "skip_proposals": skip_proposals,
            });
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
        _ => {
            println!("Wrote {}", facts_path.display());
            println!("Wrote {}", evidence_graph_path(repo).display());
            println!("Facts: {}", facts.len());
            if skip_proposals {
                println!("Skipped writing learn proposals (--skip-proposals).");
            }
        }
    }

    Ok(())
}

const MAX_LEARN_PROPOSAL_CHANGES: usize = 120;

fn emit_learn_proposals(
    repo: &Path,
    facts: &[LearnedFact],
    rejected: &HashSet<String>,
    resolve: Option<(&Graph, &DiffResult)>,
) -> Result<(), CliError> {
    let scan_to_dsl: HashMap<String, String> = resolve
        .map(|(_, diff)| {
            diff.node_diff
                .matched
                .iter()
                .map(|m| (m.actual_id.clone(), m.proposal_id.clone()))
                .collect()
        })
        .unwrap_or_default();

    let rel_changes: Vec<ProposalChange> = facts
        .iter()
        .filter(|f| f.status == FactStatus::Proposed && f.source == "drift_compare")
        .filter(|f| !rejected.contains(&f.id))
        .filter(|f| f.predicate != "undocumented_in")
        .filter_map(|f| {
            let (source, target) = match resolve {
                Some((declared, _)) => {
                    let src = scan_to_dsl
                        .get(&f.subject)
                        .cloned()
                        .unwrap_or_else(|| f.subject.clone());
                    let tgt = scan_to_dsl
                        .get(&f.object)
                        .cloned()
                        .unwrap_or_else(|| f.object.clone());
                    let src_ok = declared.nodes.iter().any(|n| n.id == src);
                    let tgt_ok = declared.nodes.iter().any(|n| n.id == tgt);
                    if !src_ok || !tgt_ok {
                        return None;
                    }
                    (src, tgt)
                }
                None => (f.subject.clone(), f.object.clone()),
            };
            Some(ProposalChange::AddRelationship {
                source,
                target,
                label: Some(f.predicate.replace('_', " ")),
                kind: Some(f.predicate.clone()),
            })
        })
        .take(MAX_LEARN_PROPOSAL_CHANGES)
        .collect();

    if rel_changes.is_empty() {
        return Ok(());
    }

    let slug = uuid::Uuid::new_v4().to_string();
    let short = &slug[..8];
    let mut proposal = Proposal::new(
        format!("learn-{short}"),
        "Learned graph proposal".to_string(),
        "Auto-generated from `sruja learn` (scan vs reviewed architecture). Approve with `sruja propose approve` after human review.".to_string(),
    );
    proposal.author = Some("sruja_learn".to_string());
    proposal.status = ProposalStatus::Pending;
    proposal.changes = rel_changes;

    proposal
        .save(repo)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn stable_fact_id_is_deterministic() {
        let a = stable_fact_id("src/lib.rs", "uses", "foo::bar");
        let b = stable_fact_id("src/lib.rs", "uses", "foo::bar");
        assert_eq!(a, b);
        assert!(a.starts_with("fact_"));
    }

    #[test]
    fn path_matches_focus_avoids_substring_false_positives() {
        assert!(!path_matches_focus("a.rs", "foo_a.rs"));
        assert!(path_matches_focus(
            "learn.rs",
            "crates/sruja-cli/src/commands/learn.rs"
        ));
        assert!(path_matches_focus(
            "crates/sruja-cli",
            "crates/sruja-cli/src/commands/learn.rs"
        ));
    }

    #[test]
    fn git_changed_paths_errors_outside_git_repo() {
        let dir = tempdir().unwrap();
        let err = git_changed_paths(dir.path(), "main").expect_err("expected git error");
        assert!(!err.is_empty());
    }

    #[test]
    fn append_and_read_feedback_roundtrip() {
        let dir = tempdir().unwrap();
        append_learn_feedback(dir.path(), "fact_abc", "reject", Some("noise")).unwrap();
        let rejected = rejected_fact_ids(dir.path());
        assert!(rejected.contains("fact_abc"));
    }
}
