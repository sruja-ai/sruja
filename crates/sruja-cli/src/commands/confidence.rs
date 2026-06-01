//! Confidence Report: post-AI-edit review artifact for human trust.
//!
//! Composes existing Sruja verification steps into a single deterministic report
//! showing what changed, what evidence was checked, what risks remain, and what
//! to inspect at 3AM.
//!
//! Advisory by default: exits successfully when it can generate a report, even if
//! the report contains blockers. Only exits non-zero for fatal execution/input errors.
//!
//! Usage:
//! ```bash
//! sruja confidence -r .
//! sruja confidence -r . -f json
//! sruja confidence --profile bugfix --file crates/sruja-cli/src/commands/foo.rs -r .
//! sruja confidence --profile coding --evidence-pack -r .
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::intent_domain::verify_task::{verify_task, VerifyTaskOptions, VerifyTaskOutput};
use super::CliError;

/// Schema version for confidence report output.
pub const CONFIDENCE_REPORT_SCHEMA: &str = "confidence_report/v1";

/// Confidence level derived from verification results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfidenceLevel::High => write!(f, "high"),
            ConfidenceLevel::Medium => write!(f, "medium"),
            ConfidenceLevel::Low => write!(f, "low"),
        }
    }
}

/// Top-level confidence report DTO.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfidenceReport {
    pub schema_version: String,
    pub repo: String,
    pub generated_at: String,
    pub profile: String,
    pub advisory: bool,
    pub verdict: Verdict,
    pub change_surface: ChangeSurface,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent_alignment: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture_alignment: Option<serde_json::Value>,
    pub verification: VerificationSummary,
    pub risks: RiskSummary,
    pub evidence: EvidenceSummary,
    pub three_am_notes: ThreeAmNotes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_memory: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Verdict {
    pub confidence_level: ConfidenceLevel,
    pub summary: String,
    pub all_verification_passed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeSurface {
    pub changed_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub steps: Vec<StepSummary>,
    pub elapsed_ms: u128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StepSummary {
    pub step_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u128>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskSummary {
    pub blockers: Vec<String>,
    pub watch_items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvidenceSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_pack: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreeAmNotes {
    pub likely_first_places_to_check: Vec<String>,
    pub commands_to_run: Vec<String>,
}

/// Options for running the confidence command.
pub struct ConfidenceOptions<'a> {
    pub repo: &'a str,
    pub profile: &'a str,
    pub file: Option<&'a str>,
    pub max_runtime_ms: Option<u64>,
    pub evidence_pack: bool,
    pub evidence_pack_dir: Option<&'a str>,
}

/// Collect changed files from git diff (tracked + untracked).
fn collect_changed_files(repo_path: &Path) -> Vec<String> {
    let mut files = Vec::new();

    // Tracked changes (staged + unstaged)
    if let Ok(output) = std::process::Command::new("git")
        .args(["diff", "HEAD", "--name-only"])
        .current_dir(repo_path)
        .output()
    {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                if !line.is_empty() {
                    files.push(line.to_string());
                }
            }
        }
    }

    // Untracked files
    if let Ok(output) = std::process::Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(repo_path)
        .output()
    {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                if !line.is_empty() && !files.contains(&line.to_string()) {
                    files.push(line.to_string());
                }
            }
        }
    }

    files
}

/// Compute confidence level from verification output.
fn compute_confidence(output: &VerifyTaskOutput) -> ConfidenceLevel {
    let any_failed = output.steps.iter().any(|s| s.status == "error");

    if any_failed {
        return ConfidenceLevel::Low;
    }

    // Check for medium-confidence signals
    let has_review_step = output.steps.iter().any(|s| s.step_id == "review");
    let has_intent_step = output.steps.iter().any(|s| s.step_id == "intent_check");

    // Check if review has open questions or warnings
    let review_has_issues = output.steps.iter().any(|s| {
        if s.step_id != "review" || s.status != "ok" {
            return false;
        }
        parse_review_json(&s.stdout)
            .map(|r| !r.open_questions.is_empty() || r.has_drift)
            .unwrap_or(false)
    });

    // Check if drift detected issues
    let drift_has_violations = output.steps.iter().any(|s| {
        if s.step_id != "drift_check" || s.status != "ok" {
            return false;
        }
        parse_drift_json(&s.stdout)
            .map(|d| d.has_drift)
            .unwrap_or(false)
    });

    // Check if intent check found issues
    let intent_has_errors = output.steps.iter().any(|s| {
        if s.step_id != "intent_check" || s.status != "ok" {
            return false;
        }
        parse_intent_json(&s.stdout)
            .map(|i| i.has_errors)
            .unwrap_or(false)
    });

    // Check if intent output is missing or unparseable (treat as unknown)
    let intent_unparseable = has_intent_step
        && output.steps.iter().any(|s| {
            if s.step_id != "intent_check" || s.status != "ok" {
                return false;
            }
            let trimmed = s.stdout.trim();
            trimmed.is_empty() || parse_intent_json(trimmed).is_none()
        });

    // Check agent memory staleness
    let memory_stale = output
        .agent_memory
        .as_ref()
        .map(|m| m.is_stale)
        .unwrap_or(false);

    if review_has_issues || drift_has_violations || intent_has_errors {
        return ConfidenceLevel::Low;
    }

    // Medium if intent is unknown, baseline missing, or memory stale
    let intent_unknown = has_intent_step
        && output.steps.iter().any(|s| {
            if s.step_id != "intent_check" || s.status != "ok" {
                return false;
            }
            parse_intent_json(&s.stdout)
                .map(|i| i.truth_status == "unknown")
                .unwrap_or(false)
        });

    let review_truth_unknown = has_review_step
        && output.steps.iter().any(|s| {
            if s.step_id != "review" || s.status != "ok" {
                return false;
            }
            parse_review_json(&s.stdout)
                .map(|r| r.truth_status == "unknown")
                .unwrap_or(false)
        });

    if intent_unknown
        || review_truth_unknown
        || memory_stale
        || !has_intent_step
        || intent_unparseable
    {
        return ConfidenceLevel::Medium;
    }

    ConfidenceLevel::High
}

/// Helper struct for parsing review JSON output.
#[derive(Deserialize)]
struct ReviewJson {
    truth_status: String,
    has_drift: bool,
    open_questions: Vec<String>,
}

/// Helper struct for parsing drift JSON output.
#[derive(Deserialize)]
struct DriftJson {
    has_drift: bool,
}

/// Helper struct for parsing intent check JSON output.
#[derive(Deserialize)]
struct IntentJson {
    truth_status: String,
    has_errors: bool,
}

fn parse_review_json(stdout: &str) -> Option<ReviewJson> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str(trimmed).ok()
}

fn parse_drift_json(stdout: &str) -> Option<DriftJson> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str(trimmed).ok()
}

fn parse_intent_json(stdout: &str) -> Option<IntentJson> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str(trimmed).ok()
}

/// Build risk queues from verification output.
fn build_risks(output: &VerifyTaskOutput) -> RiskSummary {
    let mut blockers = Vec::new();
    let mut watch_items = Vec::new();

    for step in &output.steps {
        match step.status.as_str() {
            "error" => {
                blockers.push(format!(
                    "Step '{}' failed: {}",
                    step.step_id,
                    step.stderr.lines().next().unwrap_or("unknown error")
                ));
            }
            "ok" => {
                // Check for medium-risk signals in successful steps
                if step.step_id == "review" {
                    if let Some(review) = parse_review_json(&step.stdout) {
                        if review.has_drift {
                            blockers.push("Review detected architectural drift".to_string());
                        }
                        for q in &review.open_questions {
                            watch_items.push(format!("Open question: {}", q));
                        }
                    }
                }
                if step.step_id == "drift_check" {
                    if let Some(drift) = parse_drift_json(&step.stdout) {
                        if drift.has_drift {
                            blockers.push(
                                "Architectural drift detected — run `sruja drift -r .` to investigate"
                                    .to_string(),
                            );
                        }
                    }
                }
                if step.step_id == "intent_check" {
                    if let Some(intent) = parse_intent_json(&step.stdout) {
                        if intent.has_errors {
                            blockers.push("Intent check found errors".to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Agent memory staleness
    if let Some(ref memory) = output.agent_memory {
        if memory.is_stale {
            watch_items.push(format!(
                "Agent memory adoption low ({} learnings). Record guardrails when Sruja catches a miss.",
                memory.learnings_count
            ));
        }
    }

    RiskSummary {
        blockers,
        watch_items,
    }
}

/// Build 3AM notes from verification output and changed files.
fn build_three_am_notes(
    output: &VerifyTaskOutput,
    changed_files: &[String],
    focus_file: Option<&str>,
    _repo_path: &Path,
) -> ThreeAmNotes {
    let mut places = Vec::new();

    // Changed files are the first places to check
    for f in changed_files.iter().take(10) {
        places.push(f.clone());
    }

    // Focus file is a priority check
    if let Some(ff) = focus_file {
        if !places.contains(&ff.to_string()) {
            places.insert(0, ff.to_string());
        }
    }

    let mut commands = Vec::new();

    // Focus command if we have a focus file
    if let Some(ff) = focus_file {
        commands.push(format!("sruja focus --file {} -r . -f for-ai", ff));
    }

    // Standard follow-up commands
    commands.push("sruja drift -r . -f json".to_string());
    commands.push("sruja intent check -r . -f json".to_string());

    // Evidence pack location if available
    if let Some(ref ep) = output.evidence_pack {
        commands.push(format!("ls {}", ep.output_dir));
    }

    ThreeAmNotes {
        likely_first_places_to_check: places,
        commands_to_run: commands,
    }
}

/// Generate a confidence summary string.
fn generate_summary(
    level: ConfidenceLevel,
    output: &VerifyTaskOutput,
    changed_files: &[String],
) -> String {
    let file_count = changed_files.len();
    let step_count = output.steps.len();
    let passed = output
        .steps
        .iter()
        .filter(|s| s.status == "ok" || s.status == "skipped")
        .count();

    match level {
        ConfidenceLevel::High => format!(
            "{} files changed. All {} verification steps passed. No blockers found.",
            file_count, passed
        ),
        ConfidenceLevel::Medium => format!(
            "{} files changed. {}/{} steps passed, but some signals are missing or unclear.",
            file_count, passed, step_count
        ),
        ConfidenceLevel::Low => format!(
            "{} files changed. {}/{} steps passed. Blockers found — review before merging.",
            file_count, passed, step_count
        ),
    }
}

/// Run the confidence command.
pub async fn confidence(options: ConfidenceOptions<'_>) -> Result<ConfidenceReport, CliError> {
    let repo_path = Path::new(options.repo);
    if !repo_path.exists() {
        return Err(CliError::validation(format!(
            "Repository not found: {}",
            options.repo
        )));
    }

    // 1. Collect changed files
    let changed_files = collect_changed_files(repo_path);

    // Warn if bugfix profile is used without --file
    if options.profile == "bugfix" && options.file.is_none() {
        eprintln!("warning: bugfix profile without --file skips the focus step. Use --file for best results.");
    }

    // 2. Run verification
    let verify_output = verify_task(VerifyTaskOptions {
        repo: options.repo,
        profile: options.profile,
        file: options.file,
        max_runtime_ms: options.max_runtime_ms,
        evidence_pack: options.evidence_pack,
        evidence_pack_dir: options.evidence_pack_dir,
    })
    .await?;

    // 3. Compute confidence
    let confidence_level = compute_confidence(&verify_output);
    let all_passed = verify_output.all_passed;
    let risks = build_risks(&verify_output);

    // 4. Parse step outputs for the report
    let mut intent_alignment = None;
    let mut architecture_alignment = None;

    for step in &verify_output.steps {
        match step.step_id.as_str() {
            "intent_check" if step.status == "ok" => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(step.stdout.trim()) {
                    intent_alignment = Some(val);
                }
            }
            "drift_check" if step.status == "ok" => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(step.stdout.trim()) {
                    architecture_alignment = Some(val);
                }
            }
            _ => {}
        }
    }

    // 5. Build verification summary
    let steps: Vec<StepSummary> = verify_output
        .steps
        .iter()
        .map(|s| {
            let message = if s.status == "error" {
                Some(
                    s.stderr
                        .lines()
                        .next()
                        .unwrap_or("unknown error")
                        .to_string(),
                )
            } else {
                None
            };
            StepSummary {
                step_id: s.step_id.clone(),
                status: s.status.clone(),
                message,
                elapsed_ms: Some(s.elapsed_ms),
            }
        })
        .collect();

    let verification = VerificationSummary {
        steps,
        elapsed_ms: verify_output.elapsed_ms,
    };

    // 6. Build 3AM notes
    let three_am_notes =
        build_three_am_notes(&verify_output, &changed_files, options.file, repo_path);

    // 7. Build evidence summary
    let provenance = serde_json::to_value(&verify_output.provenance).ok();
    let evidence_pack = verify_output
        .evidence_pack
        .as_ref()
        .map(|ep| ep.output_dir.clone());

    let evidence = EvidenceSummary {
        provenance,
        evidence_pack,
    };

    // 8. Build agent memory
    let agent_memory = verify_output
        .agent_memory
        .as_ref()
        .and_then(|m| serde_json::to_value(m).ok());

    // 9. Generate summary
    let summary = generate_summary(confidence_level, &verify_output, &changed_files);

    let report = ConfidenceReport {
        schema_version: CONFIDENCE_REPORT_SCHEMA.to_string(),
        repo: options.repo.to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        profile: options.profile.to_string(),
        advisory: true,
        verdict: Verdict {
            confidence_level,
            summary,
            all_verification_passed: all_passed,
        },
        change_surface: ChangeSurface {
            changed_files,
            focus_file: options.file.map(|s| s.to_string()),
        },
        intent_alignment,
        architecture_alignment,
        verification,
        risks,
        evidence,
        three_am_notes,
        agent_memory,
    };

    Ok(report)
}

/// Format confidence report as markdown.
#[allow(clippy::vec_init_then_push)]
fn format_markdown(report: &ConfidenceReport) -> String {
    let mut lines = Vec::new();

    // Header
    lines.push("# Sruja Confidence Report".to_string());
    lines.push(String::new());

    // Verdict
    lines.push("## Verdict".to_string());
    lines.push(String::new());
    let level_icon = match report.verdict.confidence_level {
        ConfidenceLevel::High => "[HIGH]",
        ConfidenceLevel::Medium => "[MEDIUM]",
        ConfidenceLevel::Low => "[LOW]",
    };
    lines.push(format!(
        "**Confidence: {} {}**",
        level_icon, report.verdict.confidence_level
    ));
    lines.push(String::new());
    lines.push(report.verdict.summary.clone());
    lines.push(String::new());

    // What Changed
    lines.push("## What Changed".to_string());
    lines.push(String::new());
    if report.change_surface.changed_files.is_empty() {
        lines.push("No changed files detected (git may be unavailable).".to_string());
    } else {
        for f in &report.change_surface.changed_files {
            lines.push(format!("- `{}`", f));
        }
    }
    if let Some(ref ff) = report.change_surface.focus_file {
        lines.push(String::new());
        lines.push(format!("**Focus file:** `{}`", ff));
    }
    lines.push(String::new());

    // Intent Alignment
    lines.push("## Intent Alignment".to_string());
    lines.push(String::new());
    if let Some(ref intent) = report.intent_alignment {
        if let Some(status) = intent.get("truth_status").and_then(|v| v.as_str()) {
            lines.push(format!("Status: **{}**", status));
        }
        if let Some(drifts) = intent.get("drifts").and_then(|v| v.as_array()) {
            if !drifts.is_empty() {
                lines.push(format!("{} intent drift(s) detected.", drifts.len()));
            }
        }
    } else {
        lines.push("Intent check not run or unavailable.".to_string());
    }
    lines.push(String::new());

    // Architecture Alignment
    lines.push("## Architecture Alignment".to_string());
    lines.push(String::new());
    if let Some(ref arch) = report.architecture_alignment {
        if let Some(has_drift) = arch.get("has_drift").and_then(|v| v.as_bool()) {
            if has_drift {
                lines.push("**Drift detected.** Run `sruja drift -r .` for details.".to_string());
            } else {
                lines.push("No architectural drift detected.".to_string());
            }
        }
        if let Some(health) = arch.get("health_score").and_then(|v| v.as_u64()) {
            lines.push(format!("Health score: {}", health));
        }
    } else {
        lines.push("Drift check not run or unavailable.".to_string());
    }
    lines.push(String::new());

    // Evidence Checked
    lines.push("## Evidence Checked".to_string());
    lines.push(String::new());
    for step in &report.verification.steps {
        let icon = match step.status.as_str() {
            "ok" => "[OK]",
            "skipped" => "[SKIP]",
            _ => "[FAIL]",
        };
        let elapsed = step
            .elapsed_ms
            .map(|ms| format!(" ({}ms)", ms))
            .unwrap_or_default();
        lines.push(format!("{} {}{}", icon, step.step_id, elapsed));
        if let Some(ref msg) = step.message {
            lines.push(format!("  > {}", msg));
        }
    }
    lines.push(String::new());
    lines.push(format!(
        "Total verification time: {}ms",
        report.verification.elapsed_ms
    ));
    lines.push(String::new());

    // Human Review Queue
    lines.push("## Human Review Queue".to_string());
    lines.push(String::new());
    if report.risks.blockers.is_empty() && report.risks.watch_items.is_empty() {
        lines.push("No blockers or watch items. Review is green.".to_string());
    } else {
        if !report.risks.blockers.is_empty() {
            lines.push("**Blockers:**".to_string());
            for b in &report.risks.blockers {
                lines.push(format!("- {}", b));
            }
            lines.push(String::new());
        }
        if !report.risks.watch_items.is_empty() {
            lines.push("**Watch items:**".to_string());
            for w in &report.risks.watch_items {
                lines.push(format!("- {}", w));
            }
        }
    }
    lines.push(String::new());

    // 3AM Notes
    lines.push("## 3AM Notes".to_string());
    lines.push(String::new());
    if !report
        .three_am_notes
        .likely_first_places_to_check
        .is_empty()
    {
        lines.push("**First places to check:**".to_string());
        for f in &report.three_am_notes.likely_first_places_to_check {
            lines.push(format!("- `{}`", f));
        }
        lines.push(String::new());
    }

    // Follow-Up Commands
    lines.push("## Follow-Up Commands".to_string());
    lines.push(String::new());
    for cmd in &report.three_am_notes.commands_to_run {
        lines.push(format!("```bash\n{}\n```", cmd));
    }

    // Evidence pack location
    if let Some(ref ep) = report.evidence.evidence_pack {
        lines.push(String::new());
        lines.push(format!("Evidence pack: `{}`", ep));
    }

    lines.push(String::new());
    lines.join("\n")
}

/// Format confidence report for display.
pub fn format_confidence(report: &ConfidenceReport, format: &str) -> String {
    match format {
        "json" => serde_json::to_string_pretty(report).unwrap_or_default(),
        "text" | "md" => format_markdown(report),
        _ => format_markdown(report),
    }
}

#[cfg(test)]
mod tests {
    use super::super::intent_domain::agent_run::StepObservation;
    use super::*;

    fn make_step(id: &str, status: &str, stdout: &str) -> StepObservation {
        StepObservation {
            step_id: id.to_string(),
            status: status.to_string(),
            exit_code: if status == "error" { Some(1) } else { Some(0) },
            stdout: stdout.to_string(),
            stderr: if status == "error" {
                "test error".to_string()
            } else {
                String::new()
            },
            elapsed_ms: 100,
            content_hash: None,
        }
    }

    fn make_verify_output(steps: Vec<StepObservation>, all_passed: bool) -> VerifyTaskOutput {
        VerifyTaskOutput {
            schema_version: "verify_task/v2".to_string(),
            profile: "review".to_string(),
            repo: ".".to_string(),
            all_passed,
            steps,
            elapsed_ms: 500,
            provenance: super::super::intent_domain::verify_task::VerifyProvenance {
                sruja_version: "0.0.0".to_string(),
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                config_hash: None,
                repo_commit: None,
                generated_at: "2025-01-01T00:00:00Z".to_string(),
            },
            evidence_pack: None,
            agent_memory: None,
        }
    }

    #[test]
    fn confidence_high_when_all_ok() {
        let steps = vec![
            make_step(
                "review",
                "ok",
                r#"{"truth_status":"reviewed","has_drift":false,"open_questions":[]}"#,
            ),
            make_step(
                "intent_check",
                "ok",
                r#"{"truth_status":"reviewed","has_errors":false}"#,
            ),
            make_step("drift_check", "ok", r#"{"has_drift":false}"#),
        ];
        let output = make_verify_output(steps, true);
        assert_eq!(compute_confidence(&output), ConfidenceLevel::High);
    }

    #[test]
    fn confidence_low_when_step_failed() {
        let steps = vec![
            make_step(
                "review",
                "ok",
                r#"{"truth_status":"reviewed","has_drift":false,"open_questions":[]}"#,
            ),
            make_step("drift_check", "error", ""),
        ];
        let output = make_verify_output(steps, false);
        assert_eq!(compute_confidence(&output), ConfidenceLevel::Low);
    }

    #[test]
    fn confidence_low_when_drift_detected() {
        let steps = vec![
            make_step(
                "review",
                "ok",
                r#"{"truth_status":"reviewed","has_drift":false,"open_questions":[]}"#,
            ),
            make_step("drift_check", "ok", r#"{"has_drift":true}"#),
        ];
        let output = make_verify_output(steps, true);
        assert_eq!(compute_confidence(&output), ConfidenceLevel::Low);
    }

    #[test]
    fn confidence_medium_when_intent_unknown() {
        let steps = vec![
            make_step(
                "review",
                "ok",
                r#"{"truth_status":"reviewed","has_drift":false,"open_questions":[]}"#,
            ),
            make_step(
                "intent_check",
                "ok",
                r#"{"truth_status":"unknown","has_errors":false}"#,
            ),
            make_step("drift_check", "ok", r#"{"has_drift":false}"#),
        ];
        let output = make_verify_output(steps, true);
        assert_eq!(compute_confidence(&output), ConfidenceLevel::Medium);
    }

    #[test]
    fn confidence_medium_when_no_intent_step() {
        let steps = vec![
            make_step(
                "review",
                "ok",
                r#"{"truth_status":"reviewed","has_drift":false,"open_questions":[]}"#,
            ),
            make_step("drift_check", "ok", r#"{"has_drift":false}"#),
        ];
        let output = make_verify_output(steps, true);
        assert_eq!(compute_confidence(&output), ConfidenceLevel::Medium);
    }

    #[test]
    fn risks_include_blockers_on_failure() {
        let steps = vec![make_step("review", "error", "")];
        let output = make_verify_output(steps, false);
        let risks = build_risks(&output);
        assert!(!risks.blockers.is_empty());
    }

    #[test]
    fn risks_include_watch_items_on_open_questions() {
        let steps = vec![make_step(
            "review",
            "ok",
            r#"{"truth_status":"reviewed","has_drift":false,"open_questions":["Is X needed?"]}"#,
        )];
        let output = make_verify_output(steps, true);
        let risks = build_risks(&output);
        assert!(!risks.watch_items.is_empty());
    }

    #[test]
    fn markdown_has_required_sections() {
        let report = ConfidenceReport {
            schema_version: CONFIDENCE_REPORT_SCHEMA.to_string(),
            repo: ".".to_string(),
            generated_at: "2025-01-01T00:00:00Z".to_string(),
            profile: "review".to_string(),
            advisory: true,
            verdict: Verdict {
                confidence_level: ConfidenceLevel::High,
                summary: "All good".to_string(),
                all_verification_passed: true,
            },
            change_surface: ChangeSurface {
                changed_files: vec!["src/main.rs".to_string()],
                focus_file: None,
            },
            intent_alignment: None,
            architecture_alignment: None,
            verification: VerificationSummary {
                steps: vec![],
                elapsed_ms: 100,
            },
            risks: RiskSummary {
                blockers: vec![],
                watch_items: vec![],
            },
            evidence: EvidenceSummary {
                provenance: None,
                evidence_pack: None,
            },
            three_am_notes: ThreeAmNotes {
                likely_first_places_to_check: vec!["src/main.rs".to_string()],
                commands_to_run: vec!["sruja drift -r . -f json".to_string()],
            },
            agent_memory: None,
        };
        let md = format_markdown(&report);
        assert!(md.contains("# Sruja Confidence Report"));
        assert!(md.contains("## Verdict"));
        assert!(md.contains("## What Changed"));
        assert!(md.contains("## Intent Alignment"));
        assert!(md.contains("## Architecture Alignment"));
        assert!(md.contains("## Evidence Checked"));
        assert!(md.contains("## Human Review Queue"));
        assert!(md.contains("## 3AM Notes"));
        assert!(md.contains("## Follow-Up Commands"));
    }

    #[test]
    fn advisory_always_true() {
        let steps = vec![make_step("review", "error", "")];
        let output = make_verify_output(steps, false);
        let confidence_level = compute_confidence(&output);
        let risks = build_risks(&output);
        let report = ConfidenceReport {
            schema_version: CONFIDENCE_REPORT_SCHEMA.to_string(),
            repo: ".".to_string(),
            generated_at: "2025-01-01T00:00:00Z".to_string(),
            profile: "review".to_string(),
            advisory: true,
            verdict: Verdict {
                confidence_level,
                summary: "test".to_string(),
                all_verification_passed: false,
            },
            change_surface: ChangeSurface {
                changed_files: vec![],
                focus_file: None,
            },
            intent_alignment: None,
            architecture_alignment: None,
            verification: VerificationSummary {
                steps: vec![],
                elapsed_ms: 0,
            },
            risks,
            evidence: EvidenceSummary {
                provenance: None,
                evidence_pack: None,
            },
            three_am_notes: ThreeAmNotes {
                likely_first_places_to_check: vec![],
                commands_to_run: vec![],
            },
            agent_memory: None,
        };
        // Advisory is always true regardless of confidence level
        assert!(report.advisory);
        assert_eq!(report.verdict.confidence_level, ConfidenceLevel::Low);
    }

    #[test]
    fn malformed_step_json_does_not_panic() {
        let steps = vec![
            make_step("review", "ok", "not valid json"),
            make_step("drift_check", "ok", "also not json"),
            make_step("intent_check", "ok", ""),
        ];
        let output = make_verify_output(steps, true);
        // Should not panic - malformed JSON is handled gracefully
        let level = compute_confidence(&output);
        // Empty intent stdout means intent is unknown -> Medium
        assert_eq!(level, ConfidenceLevel::Medium);
    }
}
