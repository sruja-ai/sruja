//! Review command: refresh evidence, detect drift, propose updates or open questions.

use std::fs;
use std::path::Path;
use std::time::Instant;

use super::violation_shared::*;
use super::{scan_repo_cached, CliError};
use crate::utils::architecture_path::{
    resolve_architecture_path, resolve_architecture_path_or_default,
};
use crate::utils::colors;
use sruja_diff::Violation;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ReviewOutput {
    pub truth_status: String,
    pub baseline: Option<String>,
    pub has_drift: bool,
    pub violations_count: usize,
    pub health_score: Option<u8>,
    #[serde(default, skip_serializing_if = "is_usize_zero")]
    pub suppressed_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub violations: Vec<ViolationSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suppressed_violations: Vec<ViolationSummary>,
    pub new_components: Vec<String>,
    pub missing_components: Vec<String>,
    pub drifted_dependencies: Vec<String>,
    pub open_questions: Vec<String>,
    pub suggestions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_score: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u128>,
}

fn is_usize_zero(v: &usize) -> bool {
    *v == 0
}

pub async fn review(
    repo_root: &str,
    format: &str,
    verbose: bool,
    include_critique: bool,
) -> Result<(), CliError> {
    let start_time = Instant::now();
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    if include_critique {
        // Run critique for modified (but unstaged) + staged changes
        let mut files = Vec::new();
        let output = std::process::Command::new("git")
            .args(["diff", "HEAD", "--name-only"])
            .current_dir(repo_path)
            .output()
            .map_err(CliError::Io)?;

        let git_files = String::from_utf8_lossy(&output.stdout);
        for f in git_files.lines() {
            if !f.is_empty() {
                files.push(f.to_string());
            }
        }

        if !files.is_empty() {
            // We just print it to stdout for now as part of the dashboard
            super::critique::critique(
                repo_root, files, None, None, None, None, false, format, false, None, None, None,
                None, 15_000, 20_000, None,
            )
            .await?;
            println!();
        }
    }

    let baseline_path = resolve_architecture_path(repo_path);

    // Review is the day-to-day workflow, so refresh cached evidence first.
    super::sync_cmd::sync(repo_root, "quiet").await?;
    let graph = scan_repo_cached(repo_path)?;

    let (truth_status, violations, health_score) = if let Some(ref baseline) = baseline_path {
        let content = fs::read_to_string(baseline)?;
        let parser = sruja_language::Parser::new(baseline.to_string_lossy().as_ref());
        let program = parser.parse(&content).map_err(|diags| {
            CliError::parse_with_diagnostics(baseline.to_string_lossy().to_string(), diags)
        })?;
        let proposed_graph = sruja_diff::program_to_graph(&program);
        let diff = sruja_diff::compare_graphs(&graph, &proposed_graph);

        let truth = match diff.truth_status {
            sruja_diff::TruthStatus::Reviewed => "reviewed",
            sruja_diff::TruthStatus::Drifted => "drifted",
            sruja_diff::TruthStatus::Unknown => "unknown",
        };

        (
            truth.to_string(),
            diff.violations,
            Some(diff.summary.health_score),
        )
    } else {
        let drift = sruja_diff::detect_architectural_drift(&graph);
        let truth = match drift.truth_status {
            sruja_diff::TruthStatus::Reviewed => "reviewed",
            sruja_diff::TruthStatus::Drifted => "drifted",
            sruja_diff::TruthStatus::Unknown => "unknown",
        };

        (
            truth.to_string(),
            drift.violations,
            Some(drift.health_score),
        )
    };

    let mut filtered_violations: Vec<_> = violations
        .into_iter()
        .filter(is_production_relevant)
        .collect();

    // Sort by severity (error first)
    filtered_violations.sort_by(|a, b| {
        use sruja_diff::Severity;
        let a_sev = match a.severity {
            Severity::Error => 0,
            Severity::Warning => 1,
            Severity::Info => 2,
        };
        let b_sev = match b.severity {
            Severity::Error => 0,
            Severity::Warning => 1,
            Severity::Info => 2,
        };
        a_sev.cmp(&b_sev)
    });

    for v in &mut filtered_violations {
        v.production_relevant = Some(true);
        if v.evidence_count.is_none() {
            v.evidence_count = Some(v.sources.len());
        }
    }

    let violations_baseline_path = repo_path.join(".sruja").join("violations.baseline.json");
    let baseline_set: Option<std::collections::HashSet<String>> =
        if violations_baseline_path.exists() {
            let content = fs::read_to_string(&violations_baseline_path)?;
            let baseline: super::check::ViolationBaseline =
                serde_json::from_str(&content).map_err(|e| CliError::validation(e.to_string()))?;
            Some(baseline.fingerprints.into_iter().collect())
        } else {
            None
        };

    let (active_violations, suppressed_violations): (Vec<Violation>, Vec<Violation>) =
        if let Some(ref set) = baseline_set {
            filtered_violations
                .into_iter()
                .map(|mut v| {
                    let suppressed = set.contains(&fingerprint_violation(&v));
                    v.suppressed = Some(suppressed);
                    v.baseline_delta =
                        Some(if suppressed { "baseline" } else { "new" }.to_string());
                    v
                })
                .partition(|v| v.suppressed != Some(true))
        } else {
            (filtered_violations, Vec::new())
        };

    let has_drift =
        truth_status == "drifted" || (baseline_path.is_none() && !active_violations.is_empty());
    let (new_components, missing_components, drifted_dependencies) =
        categorize_violations(&active_violations);
    let open_questions = generate_open_questions(&active_violations);
    let suggestions = generate_suggestions(
        repo_root,
        baseline_path.as_deref(),
        &truth_status,
        &active_violations,
    );

    let context_score = (|| {
        let kg = crate::graph_store::load_or_build_graph(repo_path).ok()?;
        let age_hours = crate::utils::context::context_age_hours(repo_path);
        Some(sruja_graph::compute_context_score(&kg, graph.nodes.len(), repo_path, age_hours).score)
    })();

    let elapsed = start_time.elapsed();
    let output = ReviewOutput {
        truth_status: truth_status.clone(),
        baseline: baseline_path.and_then(|p| p.to_str().map(String::from)),
        has_drift,
        violations_count: active_violations.len(),
        health_score,
        suppressed_count: suppressed_violations.len(),
        violations: active_violations.iter().map(summarize_violation).collect(),
        suppressed_violations: suppressed_violations
            .iter()
            .map(summarize_violation)
            .collect(),
        new_components,
        missing_components,
        drifted_dependencies,
        open_questions,
        suggestions,
        context_score,
        elapsed_ms: Some(elapsed.as_millis()),
    };

    match format {
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .map_err(|e| CliError::validation(e.to_string()))?
            );
        }
        _ => {
            use crate::utils::table_formatter::TableFormatter;
            let formatter = TableFormatter::auto();
            let mut blocks = Vec::new();

            // 1. Health Summary
            let mut health_info = String::new();
            if let Some(score) = output.health_score {
                health_info.push_str(&format!("Health:  {}\n", colors::health_bar(score, 20)));
            }
            if let Some(score) = output.context_score {
                health_info.push_str(&format!("Context: {}\n", colors::health_bar(score, 20)));
            }
            let status_color = match output.truth_status.as_str() {
                "reviewed" => colors::success(&output.truth_status),
                "drifted" => colors::error(&output.truth_status),
                _ => colors::warning(&output.truth_status),
            };
            health_info.push_str(&format!("Status: {}\n", status_color));
            health_info.push_str(&format!(
                "Issues: {} active, {} suppressed\n",
                colors::style(output.violations_count).bold(),
                colors::dim(output.suppressed_count)
            ));
            blocks.push(("Architecture Review".to_string(), health_info));

            // 2. Priority Fix (DX highlight)
            if !output.violations.is_empty() {
                let priority = &output.violations[0];
                let mut fix_info = String::new();
                fix_info.push_str(&format!(
                    "{} {}\n",
                    colors::severity_icon(&priority.severity),
                    colors::style(&priority.message).bold()
                ));
                if let Some(ref loc) = priority.location {
                    fix_info.push_str(&format!("{} {}\n", colors::dim("Loc:"), loc));
                }
                blocks.push((colors::error("Priority Fix").to_string(), fix_info));
            }

            // 3. Structural Changes
            let mut changes_info = String::new();
            if !output.new_components.is_empty() {
                changes_info.push_str(&format!(
                    "{} new components detected\n",
                    colors::success(output.new_components.len())
                ));
            }
            if !output.missing_components.is_empty() {
                changes_info.push_str(&format!(
                    "{} components missing from code\n",
                    colors::error(output.missing_components.len())
                ));
            }
            if !output.drifted_dependencies.is_empty() {
                changes_info.push_str(&format!(
                    "{} drifted dependencies\n",
                    colors::warning(output.drifted_dependencies.len())
                ));
            }
            if changes_info.is_empty() {
                changes_info.push_str("No structural changes detected.\n");
            }
            blocks.push(("Structural Read".to_string(), changes_info));

            println!(
                "{}",
                formatter.format_dashboard("DAILY ARCHITECTURE REVIEW", blocks)
            );

            // Detailed Violations
            if !output.violations.is_empty() {
                println!("{}", colors::style("Detailed Findings:").bold());
                let limit = if verbose { output.violations.len() } else { 5 };
                for v in output.violations.iter().take(limit) {
                    println!(
                        "  {} {}: {} {}",
                        colors::severity_icon(&v.severity),
                        colors::style(&v.kind).bold(),
                        v.message,
                        colors::dim(v.location.as_deref().unwrap_or(""))
                    );
                }

                if output.violations.len() > limit {
                    println!(
                        "  {} ... and {} more issues. Run with {} to see all.",
                        colors::dim("•"),
                        output.violations.len() - limit,
                        colors::info("--verbose")
                    );
                }
                println!();
            }

            // Suggestions
            if !output.suggestions.is_empty() {
                println!("{}", colors::style("Top Actions:").bold());
                for (i, s) in output.suggestions.iter().take(3).enumerate() {
                    println!("  {}. {}", i + 1, s);
                }
            }

            println!();
            println!(
                "{}",
                colors::dim(format!("Done in {}", colors::elapsed_display(elapsed)))
            );
        }
    }

    Ok(())
}

/// Structured output for the design review (machine-consumable by AI agents).
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DesignReviewOutput {
    pub schema_version: String,
    pub workflow_id: String,
    pub workflow_title: String,
    pub phase: String,
    pub profile: String,
    /// Deterministic file-existence checks.
    pub artifact_checklist: Vec<DesignReviewCheck>,
    /// Architecture evidence summary (from author_evidence.json).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture_summary: Option<DesignReviewArchSummary>,
    /// Impact / blast radius for target elements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub impact_analysis: Vec<DesignReviewImpact>,
    /// Existing proposals and their statuses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proposals: Vec<DesignReviewProposal>,
    /// Requirements cross-reference (if requirements.md exists).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements_status: Option<DesignReviewRequirements>,
    /// Gate check result.
    pub gate: DesignReviewGate,
    /// Warnings and blocking issues that must be resolved.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<String>,
    /// Non-blocking suggestions for the agent / human.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
    /// Overall pass/fail verdict (true = all blockers resolved).
    pub ready: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DesignReviewCheck {
    pub artifact: String,
    pub present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DesignReviewArchSummary {
    pub truth_status: String,
    pub primary_language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    pub architecture_style: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub community_count: usize,
    pub entrypoint_count: usize,
    pub data_store_count: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DesignReviewImpact {
    pub element_id: String,
    pub upstream_count: usize,
    pub downstream_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub downstream_ids: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DesignReviewProposal {
    pub id: String,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
    pub change_count: usize,
    pub has_validation: bool,
    pub is_valid: Option<bool>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DesignReviewRequirements {
    pub present: bool,
    /// Approximate line count (signals effort).
    pub line_count: usize,
    /// Whether a YAML frontmatter section was detected.
    pub has_frontmatter: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DesignReviewGate {
    pub allowed: bool,
    pub phase: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
}

/// Grounded design review for a workflow inception phase.
///
/// Runs deterministic checks, loads architecture evidence, computes blast radius
/// for target elements, reads proposals, and cross-references requirements.
/// Produces structured JSON output alongside the markdown, making it
/// machine-consumable by any AI coding agent via MCP or CLI.
pub async fn review_design(
    repo_root: &str,
    workflow_id: &str,
    output: Option<&Path>,
    enrich_cmd: Option<&str>,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let manifest = crate::commands::workflow_get(repo_root, workflow_id)?;
    let wf_dir = repo.join(".sruja").join("workflows").join(workflow_id);
    let inception = wf_dir.join("inception");

    // ── 1. Artifact checklist (deterministic file-existence checks) ──────
    let mut artifact_checklist = Vec::new();
    let mut blockers: Vec<String> = Vec::new();

    let check_artifact = |name: &str, dir: &Path| -> DesignReviewCheck {
        let present = dir.join(name).is_file();
        DesignReviewCheck {
            artifact: name.to_string(),
            present,
            detail: if present {
                None
            } else {
                Some(format!(
                    "MISSING: {}/{}",
                    dir.file_name().unwrap_or_default().to_string_lossy(),
                    name
                ))
            },
        }
    };

    artifact_checklist.push(check_artifact("scope.md", &inception));
    artifact_checklist.push(check_artifact("impact.json", &inception));

    // e2e profile has additional requirements
    if manifest.profile == "e2e" {
        artifact_checklist.push(check_artifact("requirements.md", &inception));
    }

    for check in &artifact_checklist {
        if !check.present {
            blockers.push(format!("Missing artifact: {}", check.artifact));
        }
    }

    // Check architecture baseline
    let sruja_file = resolve_architecture_path_or_default(repo, None);
    artifact_checklist.push(DesignReviewCheck {
        artifact: "architecture baseline".to_string(),
        present: sruja_file.exists(),
        detail: if sruja_file.exists() {
            Some(format!("{}", sruja_file.display()))
        } else {
            Some("No .sruja baseline found; scan-only mode".to_string())
        },
    });

    // ── 2. Architecture evidence (from author_evidence.json) ────────────
    let architecture_summary =
        match crate::commands::author::load_or_build_author_evidence(repo_root) {
            Ok(evidence) => Some(DesignReviewArchSummary {
                truth_status: evidence.truth_status.clone(),
                primary_language: evidence.summary.primary_language.clone(),
                framework: evidence.summary.framework.clone(),
                architecture_style: evidence.summary.architecture_style.clone(),
                node_count: evidence.summary.node_count,
                edge_count: evidence.summary.edge_count,
                community_count: evidence.communities.len(),
                entrypoint_count: evidence.entrypoints.len(),
                data_store_count: evidence.data_stores.len(),
            }),
            Err(_) => None,
        };

    if let Some(ref summary) = architecture_summary {
        if summary.truth_status == "drifted" {
            blockers.push(
                "Architecture baseline is drifted — run `sruja drift -r .` to investigate"
                    .to_string(),
            );
        }
    }

    // ── 3. Impact analysis for target elements ──────────────────────────
    let mut impact_analysis = Vec::new();
    let mut suggestions: Vec<String> = Vec::new();

    for target_id in &manifest.target_elements {
        match super::impact::impact_compute_output(repo, target_id, 2) {
            Ok(impact) => {
                let downstream_ids: Vec<String> = impact
                    .downstream
                    .iter()
                    .map(|h| h.node.id.clone())
                    .collect();
                let downstream_count = downstream_ids.len();
                if downstream_count > 5 {
                    suggestions.push(format!(
                        "High blast radius for '{}': {} downstream dependents — consider splitting the change",
                        target_id, downstream_count
                    ));
                }
                impact_analysis.push(DesignReviewImpact {
                    element_id: target_id.clone(),
                    upstream_count: impact.upstream.len(),
                    downstream_count,
                    downstream_ids,
                });
            }
            Err(_) => {
                // Target element not found in scan graph — not a blocker
                suggestions.push(format!(
                    "Target element '{}' not found in scan graph — impact analysis skipped",
                    target_id
                ));
            }
        }
    }

    if manifest.target_elements.is_empty() {
        suggestions.push("No target_elements declared in workflow — impact analysis skipped. Consider adding target elements for blast radius checks.".to_string());
    }

    // ── 4. Proposals ────────────────────────────────────────────────────
    let proposals: Vec<DesignReviewProposal> = sruja_diff::Proposal::load_all(repo)
        .unwrap_or_default()
        .into_iter()
        .filter(|p| {
            // Include proposals linked to this workflow, or all if no workflow link
            p.workflow_id.as_deref() == Some(workflow_id) || p.workflow_id.is_none()
        })
        .map(|p| {
            let is_valid = p.validation.as_ref().map(|v| v.is_valid);
            DesignReviewProposal {
                id: p.id.clone(),
                title: p.title.clone(),
                status: format!("{:?}", p.status).to_lowercase(),
                workflow_id: p.workflow_id.clone(),
                change_count: p.changes.len(),
                has_validation: p.validation.is_some(),
                is_valid,
            }
        })
        .collect();

    // Check for invalid proposals
    for p in &proposals {
        if p.is_valid == Some(false) {
            blockers.push(format!(
                "Proposal '{}' has validation errors — review before proceeding",
                p.id
            ));
        }
    }

    // ── 5. Requirements cross-reference ─────────────────────────────────
    let requirements_path = inception.join("requirements.md");
    let requirements_status = if requirements_path.is_file() {
        let content = fs::read_to_string(&requirements_path).unwrap_or_default();
        let line_count = content.lines().count();
        let has_frontmatter = content.starts_with("---");
        if line_count < 5 {
            suggestions.push("Requirements file is very short — consider adding more detail (user stories, acceptance criteria)".to_string());
        }
        Some(DesignReviewRequirements {
            present: true,
            line_count,
            has_frontmatter,
        })
    } else {
        if manifest.profile == "e2e" {
            // Already covered in artifact_checklist blockers
        } else {
            suggestions.push(
                "No requirements.md found — consider capturing requirements for traceability"
                    .to_string(),
            );
        }
        Some(DesignReviewRequirements {
            present: false,
            line_count: 0,
            has_frontmatter: false,
        })
    };

    // ── 6. Gate check ───────────────────────────────────────────────────
    let gate_result = crate::commands::workflow_gate_check(repo_root, workflow_id)?;
    let gate = DesignReviewGate {
        allowed: gate_result.allowed,
        phase: gate_result.phase.clone(),
        missing: gate_result.missing.clone(),
    };

    if !gate.allowed {
        blockers.push(format!(
            "Workflow gate not passed — missing: {}",
            gate.missing.join(", ")
        ));
    }

    // ── 7. Build structured output ──────────────────────────────────────
    let ready = blockers.is_empty();
    let review_output = DesignReviewOutput {
        schema_version: "design_review/v2".to_string(),
        workflow_id: workflow_id.to_string(),
        workflow_title: manifest.title.clone(),
        phase: manifest.phase.clone(),
        profile: manifest.profile.clone(),
        artifact_checklist,
        architecture_summary,
        impact_analysis,
        proposals,
        requirements_status,
        gate,
        blockers,
        suggestions,
        ready,
    };

    // ── 8. Write JSON output alongside markdown ─────────────────────────
    let json_output = serde_json::to_string_pretty(&review_output)?;
    let json_path = wf_dir.join("design-review.json");
    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&json_path, &json_output)?;

    // ── 9. Generate markdown body ───────────────────────────────────────
    let mut body = String::from("# Design Review (Grounded)\n\n");
    body.push_str(&format!(
        "Workflow: **{}** — {}\n",
        manifest.id, manifest.title
    ));
    body.push_str(&format!(
        "Phase: {} | Profile: {} | Ready: {}\n\n",
        manifest.phase,
        manifest.profile,
        if review_output.ready { "✅" } else { "❌" }
    ));

    // Artifact checklist
    body.push_str("## Artifact Checklist\n\n");
    for check in &review_output.artifact_checklist {
        let icon = if check.present { "✅" } else { "❌" };
        body.push_str(&format!("- {} {}", icon, check.artifact));
        if let Some(ref detail) = check.detail {
            body.push_str(&format!(" — {}", detail));
        }
        body.push('\n');
    }
    body.push('\n');

    // Architecture summary
    if let Some(ref summary) = review_output.architecture_summary {
        body.push_str("## Architecture Context\n\n");
        body.push_str(&format!("- Truth status: **{}**\n", summary.truth_status));
        body.push_str(&format!(
            "- Language: {} | Style: {}\n",
            summary.primary_language, summary.architecture_style
        ));
        if let Some(ref fw) = summary.framework {
            body.push_str(&format!("- Framework: {}\n", fw));
        }
        body.push_str(&format!(
            "- Graph: {} nodes, {} edges | {} communities | {} entrypoints | {} data stores\n",
            summary.node_count,
            summary.edge_count,
            summary.community_count,
            summary.entrypoint_count,
            summary.data_store_count
        ));
        body.push('\n');
    }

    // Impact analysis
    if !review_output.impact_analysis.is_empty() {
        body.push_str("## Impact Analysis\n\n");
        for impact in &review_output.impact_analysis {
            body.push_str(&format!(
                "- **{}**: {} upstream, {} downstream\n",
                impact.element_id, impact.upstream_count, impact.downstream_count
            ));
            if !impact.downstream_ids.is_empty() {
                let preview: Vec<&str> = impact
                    .downstream_ids
                    .iter()
                    .take(5)
                    .map(|s| s.as_str())
                    .collect();
                body.push_str(&format!("  Downstream: {}\n", preview.join(", ")));
                if impact.downstream_ids.len() > 5 {
                    body.push_str(&format!(
                        "  ... and {} more\n",
                        impact.downstream_ids.len() - 5
                    ));
                }
            }
        }
        body.push('\n');
    }

    // Proposals
    if !review_output.proposals.is_empty() {
        body.push_str("## Proposals\n\n");
        for p in &review_output.proposals {
            let valid_icon = match p.is_valid {
                Some(true) => " ✅",
                Some(false) => " ❌",
                None => "",
            };
            body.push_str(&format!(
                "- **{}** — {} ({} changes, status: {}{})\n",
                p.id, p.title, p.change_count, p.status, valid_icon
            ));
        }
        body.push('\n');
    }

    // Requirements
    if let Some(ref req) = review_output.requirements_status {
        body.push_str("## Requirements\n\n");
        if req.present {
            body.push_str(&format!(
                "- requirements.md: {} lines{}\n",
                req.line_count,
                if req.has_frontmatter {
                    " (has YAML frontmatter)"
                } else {
                    ""
                }
            ));
        } else {
            body.push_str("- ❌ No requirements.md found\n");
        }
        body.push('\n');
    }

    // Gate
    body.push_str("## Gate Check\n\n");
    body.push_str(&format!(
        "- Allowed: {} | Phase: {}\n",
        gate_result.allowed, gate_result.phase
    ));
    if !gate_result.missing.is_empty() {
        body.push_str(&format!("- Missing: {}\n", gate_result.missing.join(", ")));
    }
    body.push('\n');

    // Blockers and suggestions
    if !review_output.blockers.is_empty() {
        body.push_str("## ❌ Blockers\n\n");
        for b in &review_output.blockers {
            body.push_str(&format!("- {}\n", b));
        }
        body.push('\n');
    }

    if !review_output.suggestions.is_empty() {
        body.push_str("## 💡 Suggestions\n\n");
        for s in &review_output.suggestions {
            body.push_str(&format!("- {}\n", s));
        }
        body.push('\n');
    }

    // Optional enrichment
    if let Some(cmd) = enrich_cmd {
        let input = serde_json::json!({
            "schema_version": "design_review_input/v2",
            "workflow_id": workflow_id,
            "manifest": manifest,
            "review": review_output,
        });
        let stdin_payload = serde_json::to_string(&input)?;
        let limits = crate::integrations::EnrichmentLimits::with_defaults(15_000, 20_000);
        let narrative =
            crate::integrations::run_cmd_enrichment(cmd, stdin_payload.as_bytes(), limits)
                .map_err(CliError::validation)?;
        body.push_str("## Narrative Review (Enrichment)\n\n");
        body.push_str(&narrative);
        body.push('\n');
    }

    // ── 10. Write markdown output ───────────────────────────────────────
    let out_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| inception.join("design-review.md"));
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out_path, &body)?;

    // Print both paths for discoverability
    println!("{}", out_path.display());
    eprintln!("JSON: {}", json_path.display());
    Ok(())
}
