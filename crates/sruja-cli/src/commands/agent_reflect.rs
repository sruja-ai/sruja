//! Post-apply reflect: suggest LearningEntry rows from facts_bundle (no auto-write unless --write).

use std::path::Path;

use crate::commands::CliError;
use serde_json::Value;

#[derive(Debug, serde::Serialize)]
pub struct ReflectSuggestion {
    pub context: String,
    pub hypothesis: String,
    pub outcome: String,
    pub guardrail_advice: String,
    pub hitl_kind: String,
}

pub async fn agent_reflect(
    repo_root: &str,
    run_id: Option<&str>,
    write: bool,
    format: &str,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let runs_dir = repo.join(".sruja").join("agent").join("runs");
    let bundle_path = if let Some(id) = run_id {
        runs_dir.join(id).join("facts_bundle.json")
    } else {
        latest_facts_bundle(&runs_dir)?
    };

    let text = std::fs::read_to_string(&bundle_path).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("{}: {}", bundle_path.display(), e),
        ))
    })?;
    let bundle: Value =
        serde_json::from_str(&text).map_err(|e| CliError::validation(e.to_string()))?;

    let suggestions = suggest_from_bundle(&bundle);
    let out = serde_json::json!({
        "schema_version": "agent_reflect/v1",
        "run_id": bundle.get("run_id").and_then(|v| v.as_str()),
        "suggestions": suggestions,
        "write": write,
    });

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        for s in &suggestions {
            println!("- [{}] {}", s.outcome, s.context);
            println!("  guardrail: {}", s.guardrail_advice);
        }
    }

    if write {
        for s in suggestions {
            crate::commands::agent_record(
                repo_root,
                &s.context,
                &s.hypothesis,
                &s.outcome,
                &s.guardrail_advice,
                None,
                None,
                Some(&s.hitl_kind),
            )
            .await?;
        }
    }

    Ok(())
}

fn latest_facts_bundle(runs_dir: &Path) -> Result<std::path::PathBuf, CliError> {
    if !runs_dir.is_dir() {
        return Err(CliError::validation(
            "No agent runs under .sruja/agent/runs".to_string(),
        ));
    }
    let mut dirs: Vec<_> = std::fs::read_dir(runs_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    dirs.sort_by_key(|e| e.path());
    let latest = dirs
        .last()
        .ok_or_else(|| CliError::validation("No agent runs found".to_string()))?;
    Ok(latest.path().join("facts_bundle.json"))
}

fn suggest_from_bundle(bundle: &Value) -> Vec<ReflectSuggestion> {
    let mut out = Vec::new();
    let verification = bundle
        .get("verification")
        .or_else(|| bundle.get("verification_results"))
        .and_then(|v| v.as_array());

    if let Some(steps) = verification {
        let any_failed = steps.iter().any(|s| {
            s.get("status")
                .and_then(|v| v.as_str())
                .is_some_and(|st| st == "failed" || st == "error")
        });
        let outcome = if any_failed { "failed" } else { "success" };
        out.push(ReflectSuggestion {
            context: "Agent apply verification".into(),
            hypothesis: "Run plan verification steps after architecture changes".into(),
            outcome: outcome.into(),
            guardrail_advice: if any_failed {
                "Re-run failed verification commands before promoting repo.sruja".into()
            } else {
                "Repeat verification pattern for similar workflow changes".into()
            },
            hitl_kind: "guardrail".into(),
        });
    }

    if out.is_empty() {
        out.push(ReflectSuggestion {
            context: "Agent run completed".into(),
            hypothesis: "Review facts_bundle for manual learnings".into(),
            outcome: "success".into(),
            guardrail_advice: "Use sruja agent record for stable guardrails".into(),
            hitl_kind: "precedent".into(),
        });
    }
    out
}
