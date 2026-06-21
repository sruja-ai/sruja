//! Critique command: adversarial architectural review of changes.

use crate::commands::{scan_repo_cached, CliError};
use crate::integrations::EnrichmentResult;
use crate::utils::architecture_path;
use sruja_intent::{
    format_critique_json, format_critique_text, CritiqueEngine, CritiqueRequest, CritiqueSeverity,
};
use std::path::Path;

#[derive(Debug, serde::Serialize)]
struct CritiqueForAiOutput {
    artifact_kind: String,
    schema_version: String,
    repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proposal_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    head_ref: Option<String>,
    changed_files: Vec<String>,
    report: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    enrichment: Option<EnrichmentResult>,
}

fn build_critique_enrichment(
    repo_path: &Path,
    payload: &serde_json::Value,
    enrich: &crate::enrichment::EnrichmentRef<'_>,
) -> Option<EnrichmentResult> {
    crate::integrations::build_enrichment(
        repo_path,
        payload,
        enrich,
        "You are a careful architecture reviewer. Never fabricate.",
        crate::integrations::CRITIQUE_ENRICHMENT_PROMPT_TEMPLATE,
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn critique(
    repo_root: &str,
    mut files: Vec<String>,
    description: Option<String>,
    proposal_id: Option<String>,
    base: Option<String>,
    head: Option<String>,
    staged: bool,
    format: &str,
    enrich: &crate::enrichment::EnrichmentRef<'_>,
    fail_on: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    // 1. Resolve files from git if requested
    if staged {
        let output = std::process::Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(repo_path)
            .output()
            .map_err(CliError::Io)?;

        let git_files = String::from_utf8_lossy(&output.stdout);
        for f in git_files.lines() {
            if !f.is_empty() {
                files.push(f.to_string());
            }
        }
    }

    if files.is_empty() && base.is_none() {
        println!("No files provided for critique. Use --files, --staged, or --base/--head.");
        return Ok(());
    }

    // 2. Load context
    let graph = scan_repo_cached(repo_path)?;
    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let program = if let Some(ref path) = baseline_path {
        let content = std::fs::read_to_string(path)?;
        let parser = sruja_language::Parser::new(repo_root);
        parser.parse(&content).ok()
    } else {
        None
    };
    let intent = if let Some(ref path) = baseline_path {
        sruja_intent::IntentModel::from_sruja_file(path).ok()
    } else {
        None
    };

    // 3. Run critique
    let engine = CritiqueEngine::new(graph, program).with_intent(intent);
    let request = CritiqueRequest {
        changed_files: files,
        description: description.clone(),
        proposal_id: proposal_id.clone(),
        base_ref: base.clone(),
        head_ref: head.clone(),
    };
    let report = engine.critique(&request);
    let report_json = format_critique_json(&report);
    let report_value: serde_json::Value = serde_json::from_str(&report_json)
        .unwrap_or_else(|_| serde_json::Value::String(report_json.clone()));

    // 4. Output
    match format {
        "for-ai" => {
            let payload = serde_json::json!({
                "schema_version": "critique_enrichment_input/v1",
                "repo": repo_root,
                "description": description,
                "proposal_id": proposal_id,
                "base_ref": base,
                "head_ref": head,
                "changed_files": request.changed_files,
                "report": report_value,
            });
            let enrichment = build_critique_enrichment(repo_path, &payload, enrich);

            let out = CritiqueForAiOutput {
                artifact_kind: "deterministic_review".to_string(),
                schema_version: "critique_for_ai_output/v1".to_string(),
                repo: repo_root.to_string(),
                description: payload
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                proposal_id: payload
                    .get("proposal_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                base_ref: payload
                    .get("base_ref")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                head_ref: payload
                    .get("head_ref")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                changed_files: request.changed_files.clone(),
                report: report_value,
                enrichment,
            };

            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        "json" => println!("{}", format_critique_json(&report)),
        _ => {
            println!("{}", format_critique_text(&report));
            let payload = serde_json::json!({
                "schema_version": "critique_enrichment_input/v1",
                "repo": repo_root,
                "description": description,
                "proposal_id": proposal_id,
                "base_ref": base,
                "head_ref": head,
                "changed_files": request.changed_files,
                "report": report_value,
            });
            let enrichment = build_critique_enrichment(repo_path, &payload, enrich);
            if let Some(enrichment) = enrichment {
                if let Some(md) = enrichment.narrative_markdown.as_deref() {
                    println!();
                    println!("{}", md);
                }
            }
        }
    }

    // 5. Exit logic
    if let Some(level) = fail_on {
        let threshold = match level.to_lowercase().as_str() {
            "critical" => CritiqueSeverity::Critical,
            "high" => CritiqueSeverity::High,
            "medium" => CritiqueSeverity::Medium,
            "low" => CritiqueSeverity::Low,
            _ => {
                return Err(CliError::validation(format!(
                    "Invalid fail-on level: {}",
                    level
                )))
            }
        };

        // --fail-on considers violations only, not context
        let has_violation = report.violations.iter().any(|f| f.severity >= threshold);
        if has_violation {
            std::process::exit(1);
        }
    }

    Ok(())
}
