//! AI commands: explain, ask, feedback, memory.
//!
//! Architecture Explainer + Memory loop: grounded answers, persistence, feedback.

use std::path::Path;
use std::process::Command;

use crate::ai::{
    build_context, parse_envelope, explain_user_prompt, EXPLAIN_SYSTEM,
    append_fact, append_feedback, append_interaction, load_facts, load_feedback, load_interactions,
    load_state, save_state,
    write_facts, apply_verdict, should_deprecate, Verdict,
    Fact, FeedbackRecord, InteractionRecord,
    EvidenceEntry,
};
use std::collections::HashSet;
use crate::commands::llm::call_llm;
use super::CliError;

/// When LLM is unavailable, print evidence and hint instead of failing.
fn fallback_no_llm(
    topic: &str,
    ctx_text: &str,
    format: &str,
    what_changed: Option<&str>,
) -> Result<(), CliError> {
    if format == "json" {
        let out = serde_json::json!({
            "answer_markdown": null,
            "confidence": 0.0,
            "fallback": true,
            "message": "LLM unavailable. Set SRUJA_LLM_PROVIDER and API key (e.g. OPENAI_API_KEY) for full explanation.",
            "evidence_preview": ctx_text.lines().take(20).collect::<Vec<_>>().join("\n"),
            "what_changed_since_validated": what_changed,
        });
        println!("{}", serde_json::to_string_pretty(&out).map_err(CliError::Json)?);
    } else {
        println!("LLM unavailable. Set SRUJA_LLM_PROVIDER and an API key (e.g. OPENAI_API_KEY, or SRUJA_LLM_PROVIDER=ollama) for a full explanation.");
        println!("\nTopic: {}", topic);
        println!("\nEvidence from scan (preview):");
        println!("{}", "─".repeat(50));
        for line in ctx_text.lines().take(25) {
            println!("{}", line);
        }
        if ctx_text.lines().count() > 25 {
            println!("... (truncated)");
        }
        println!("{}", "─".repeat(50));
        if let Some(msg) = what_changed {
            println!("Note: {}", msg);
        }
    }
    Ok(())
}

const MAX_EVIDENCE_ITEMS: usize = 30;

/// Get current commit short SHA in repo, or None if not a git repo.
fn current_commit_sha(repo_root: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["-C", repo_root.as_os_str().to_str().unwrap_or("."), "rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

/// Run `sruja ai explain -r <repo> --topic <topic>`.
pub async fn ai_explain(
    repo: &str,
    topic: &str,
    format: &str,
    graph_file: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Validation(format!("Repository path does not exist: {}", repo)));
    }

    let state = load_state(repo_path).unwrap_or_default();
    let current_sha = current_commit_sha(repo_path);
    let what_changed = build_what_changed_since_validated(repo_path, state.last_validated_sha.as_deref(), current_sha.as_deref());

    let ctx = build_context(
        repo_path,
        topic,
        graph_file.map(Path::new),
        MAX_EVIDENCE_ITEMS,
    )?;
    let allowed_paths: HashSet<String> = ctx.evidence_paths.into_iter().collect();
    let user_prompt = explain_user_prompt(topic, &ctx.text);

    let raw = match call_llm(EXPLAIN_SYSTEM, &user_prompt).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("LLM error: {}. Using fallback.", e);
            return fallback_no_llm(topic, &ctx.text, format, what_changed.as_deref());
        }
    };
    let envelope = match parse_envelope(&raw) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("LLM response could not be parsed as JSON envelope: {}. Using fallback.", e);
            return fallback_no_llm(topic, &ctx.text, format, what_changed.as_deref());
        }
    };

    let commit_sha = current_commit_sha(repo_path);
    let repo_abs = repo_path.canonicalize().unwrap_or_else(|_| repo_path.to_path_buf());
    let repo_str = repo_abs.to_string_lossy().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let mut new_fact_ids = Vec::new();
    let mut evidence_paths_out: HashSet<String> = HashSet::new();

    for ef in &envelope.facts {
        // Reject facts without evidence path match (plan: do not cite paths not in evidence)
        let evidence_list: Vec<EvidenceEntry> = ef
            .evidence_paths
            .iter()
            .filter(|p| allowed_paths.contains(*p))
            .cloned()
            .map(|path| {
                evidence_paths_out.insert(path.clone());
                EvidenceEntry {
                    kind: "file".to_string(),
                    path,
                    line_hint: None,
                    why_relevant: None,
                }
            })
            .collect();
        if evidence_list.is_empty() && !ef.evidence_paths.is_empty() {
            continue; // skip facts that cite paths not in evidence
        }
        let fact = Fact {
            fact_id: String::new(),
            statement: ef.statement.clone(),
            kind: ef.fact_type.clone(),
            status: "candidate".to_string(),
            confidence: ef.confidence.clamp(0.0, 1.0),
            source: "llm".to_string(),
            repo: repo_str.clone(),
            commit_sha: commit_sha.clone(),
            evidence: evidence_list,
            tags: vec![],
            created_at: now.clone(),
            updated_at: now.clone(),
            last_validated_sha: commit_sha.clone(),
        };
        let id = append_fact(repo_path, fact)?;
        new_fact_ids.push(id);
    }

    let used_fact_ids: Vec<String> = load_facts(repo_path)?
        .iter()
        .filter(|f| f.status != "deprecated" && f.confidence >= 0.25)
        .map(|f| f.fact_id.clone())
        .take(10)
        .collect();

    let interaction = InteractionRecord {
        answer_id: String::new(),
        question: topic.to_string(),
        response_markdown: envelope.answer_markdown.clone(),
        used_fact_ids,
        new_fact_ids: new_fact_ids.clone(),
        confidence: envelope.confidence,
        commit_sha: commit_sha.clone(),
        created_at: now,
    };
    let answer_id = append_interaction(repo_path, interaction)?;

    // Update state: this answer is validated at current commit (plan §5.2).
    if let Some(ref sha) = commit_sha {
        let mut new_state = state;
        new_state.last_validated_sha = Some(sha.clone());
        new_state.last_commit_sha = Some(sha.clone());
        let _ = save_state(repo_path, &new_state);
    }

    let evidence_paths_vec: Vec<String> = evidence_paths_out.into_iter().collect();
    if format == "json" {
        let out = serde_json::json!({
            "answer_markdown": envelope.answer_markdown,
            "confidence": envelope.confidence,
            "answer_id": answer_id,
            "new_fact_ids": new_fact_ids,
            "evidence_paths": evidence_paths_vec,
            "assumptions": envelope.assumptions,
            "gaps": envelope.gaps,
            "what_changed_since_validated": what_changed,
        });
        println!("{}", serde_json::to_string_pretty(&out).map_err(CliError::Json)?);
    } else {
        println!("{}", envelope.answer_markdown);
        println!("\n---");
        println!("Confidence: {:.2}", envelope.confidence);
        println!("Answer ID: {}", answer_id);
        println!("New fact IDs: {:?}", new_fact_ids);
        if let Some(ref msg) = what_changed {
            println!("Note: {}", msg);
        }
        if !evidence_paths_vec.is_empty() {
            println!("Evidence (files):");
            let mut ep = evidence_paths_vec;
            ep.sort();
            for p in ep.into_iter().take(15) {
                println!("  - {}", p);
            }
        }
        if !envelope.assumptions.is_empty() {
            println!("Assumptions: {:?}", envelope.assumptions);
        }
    }
    Ok(())
}

/// Build a short message when repo has moved since last validated commit. None if not applicable.
fn build_what_changed_since_validated(
    repo_path: &Path,
    last_validated: Option<&str>,
    current: Option<&str>,
) -> Option<String> {
    let (last, cur) = match (last_validated, current) {
        (Some(a), Some(b)) if a != b => (a, b),
        _ => return None,
    };
    let count = Command::new("git")
        .args([
            "-C",
            repo_path.as_os_str().to_str().unwrap_or("."),
            "rev-list",
            "--count",
            &format!("{}..HEAD", last),
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<u32>().ok());
    let msg = match count {
        Some(n) if n > 0 => format!(
            "Repository has {} new commit(s) since last validated (was {}, now {}).",
            n, last, cur
        ),
        _ => format!(
            "Repository ref has changed since last validated (was {}, now {}).",
            last, cur
        ),
    };
    Some(msg)
}

/// Run `sruja ai ask -r <repo> "<question>"`. Same as explain but with free-form question.
pub async fn ai_ask(
    repo: &str,
    question: &str,
    format: &str,
    graph_file: Option<&str>,
) -> Result<(), CliError> {
    ai_explain(repo, question, format, graph_file).await
}

/// Run `sruja ai feedback -r <repo> --answer-id <id> --fact-id <id> --verdict correct|wrong|partial`.
pub async fn ai_feedback(
    repo: &str,
    answer_id: &str,
    fact_id: &str,
    verdict_str: &str,
    comment: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Validation(format!("Repository path does not exist: {}", repo)));
    }

    let verdict = match verdict_str.to_lowercase().as_str() {
        "correct" => Verdict::Correct,
        "wrong" => Verdict::Wrong,
        "partial" => Verdict::Partial,
        _ => {
            return Err(CliError::Validation(format!(
                "Invalid verdict '{}'. Use: correct, wrong, partial",
                verdict_str
            )));
        }
    };

    let mut facts = load_facts(repo_path)?;
    let pos = facts
        .iter()
        .position(|f| f.fact_id == fact_id)
        .ok_or_else(|| CliError::Validation(format!("Fact not found: {}", fact_id)))?;

    let (new_conf, new_status) =
        apply_verdict(facts[pos].confidence, &facts[pos].status, verdict);
    facts[pos].confidence = new_conf;
    facts[pos].status = new_status.clone();
    facts[pos].updated_at = chrono::Utc::now().to_rfc3339();

    // Deprecate if two consecutive wrong verdicts and confidence < 0.25 (plan §10).
    let feedback_list = load_feedback(repo_path)?;
    let consecutive_wrong = count_consecutive_wrong_for_fact(&feedback_list, fact_id);
    let wrong_after_this = consecutive_wrong + if matches!(verdict, Verdict::Wrong) { 1 } else { 0 };
    if should_deprecate(facts[pos].confidence, wrong_after_this) {
        facts[pos].status = "deprecated".to_string();
    }

    write_facts(repo_path, &facts)?;

    let fb = FeedbackRecord {
        feedback_id: String::new(),
        answer_id: answer_id.to_string(),
        fact_id: fact_id.to_string(),
        verdict: verdict_str.to_string(),
        comment: comment.map(String::from),
        actor: "user".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    append_feedback(repo_path, fb)?;

    println!(
        "Feedback recorded. Fact {} updated: confidence={:.2}, status={}",
        fact_id, facts[pos].confidence, facts[pos].status
    );
    Ok(())
}

/// Count consecutive "wrong" verdicts at the end of feedback history for this fact.
fn count_consecutive_wrong_for_fact(feedback: &[FeedbackRecord], fact_id: &str) -> u32 {
    let mut for_fact: Vec<&FeedbackRecord> = feedback
        .iter()
        .filter(|fb| fb.fact_id == fact_id)
        .collect();
    for_fact.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    let mut count = 0u32;
    for fb in for_fact.iter().rev() {
        if fb.verdict.to_lowercase() == "wrong" {
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Run `sruja ai memory -r <repo>`. Print summary of facts, interactions, feedback.
pub async fn ai_memory(repo: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Validation(format!("Repository path does not exist: {}", repo)));
    }

    let facts = load_facts(repo_path)?;
    let interactions = load_interactions(repo_path)?;
    let feedback = load_feedback(repo_path)?;

    let confirmed = facts.iter().filter(|f| f.status == "confirmed").count();
    let disputed = facts.iter().filter(|f| f.status == "disputed").count();
    let candidate = facts.iter().filter(|f| f.status == "candidate").count();
    let deprecated = facts.iter().filter(|f| f.status == "deprecated").count();

    if format == "json" {
        let out = serde_json::json!({
            "facts_count": facts.len(),
            "quality": { "confirmed": confirmed, "disputed": disputed, "candidate": candidate, "deprecated": deprecated },
            "interactions_count": interactions.len(),
            "feedback_count": feedback.len(),
            "facts": facts.iter().map(|f| serde_json::json!({ "fact_id": f.fact_id, "statement": f.statement, "status": f.status, "confidence": f.confidence })).collect::<Vec<_>>(),
            "recent_interactions": interactions.iter().rev().take(5).map(|i| serde_json::json!({ "answer_id": i.answer_id, "question": i.question, "confidence": i.confidence })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&out).map_err(CliError::Json)?);
    } else {
        println!("Memory summary (repo: {})", repo);
        println!("  Facts: {} total", facts.len());
        println!(
            "  Quality: {} confirmed, {} disputed, {} candidate, {} deprecated",
            confirmed, disputed, candidate, deprecated
        );
        println!("  Interactions: {}", interactions.len());
        println!("  Feedback: {}", feedback.len());
        if !facts.is_empty() {
            println!("\nRecent facts:");
            for f in facts.iter().rev().take(5) {
                println!("  - [{}] {} ({}), conf={:.2}", f.fact_id, f.statement, f.status, f.confidence);
            }
        }
    }
    Ok(())
}
