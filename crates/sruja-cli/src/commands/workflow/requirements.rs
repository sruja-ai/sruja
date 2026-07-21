use std::path::Path;

use crate::commands::workflow_aidlc;
use crate::commands::CliError;

use super::manifest::{load_manifest, resolve_single_workflow_id, drift_clean};
use super::types::*;

pub fn workflow_capture_requirements(
    repo_root: &str,
    id: Option<&str>,
    from_issue: Option<&str>,
    enrich_cmd: Option<&str>,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = if let Some(i) = id {
        i.to_string()
    } else {
        resolve_single_workflow_id(repo)?
    };

    let manifest = load_manifest(repo, &wf_id)?;
    let requirements_path = inception_dir(repo, &wf_id).join("requirements.md");

    let mut current_content = if requirements_path.exists() {
        std::fs::read_to_string(&requirements_path)?
    } else {
        requirements_template().to_string()
    };

    if let Some(issue_url) = from_issue {
        current_content.push_str(&format!(
            "\n\n## Ingested from Issue\nCaptured from: {}\n\n- Ingested on: {}\n",
            issue_url,
            now_iso()
        ));
    }

    if let Some(cmd) = enrich_cmd {
        let input_json = serde_json::json!({
            "workflow_id": wf_id,
            "title": manifest.title,
            "profile": manifest.profile,
            "current_requirements": current_content,
        });
        let payload =
            serde_json::to_vec(&input_json).map_err(|e| CliError::validation(e.to_string()))?;
        let limits = crate::integrations::EnrichmentLimits {
            timeout_ms: 15000,
            max_bytes: 20000,
        };
        match crate::integrations::run_cmd_enrichment(cmd, &payload, limits) {
            Ok(narrative) => {
                current_content = narrative;
            }
            Err(e) => {
                return Err(CliError::validation(format!(
                    "Enrichment command failed: {}",
                    e
                )));
            }
        }
    }

    std::fs::write(&requirements_path, &current_content)?;

    let event_msg = if from_issue.is_some() && enrich_cmd.is_some() {
        "Captured and enriched requirements"
    } else if from_issue.is_some() {
        "Captured requirements from issue"
    } else if enrich_cmd.is_some() {
        "Enriched requirements"
    } else {
        "Scaffolded requirements"
    };

    let audit_path =
        workflow_aidlc::append_workflow_audit(repo, &wf_id, event_msg, "human")?;

    println!(
        "Requirements captured/updated at {}",
        requirements_path.display()
    );
    println!("Appended audit log at {}", audit_path.display());

    Ok(())
}

pub async fn workflow_record_test_results(
    repo_root: &str,
    id: Option<&str>,
    profile: Option<&str>,
    from_file: Option<&str>,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = if let Some(i) = id {
        i.to_string()
    } else {
        resolve_single_workflow_id(repo)?
    };

    let _manifest = load_manifest(repo, &wf_id)?;
    let test_results_path = construction_dir(repo, &wf_id).join("test-results.json");

    let payload = if let Some(file_path) = from_file {
        let text = std::fs::read_to_string(file_path)?;
        let val: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| CliError::validation(format!("Invalid JSON in test-results: {}", e)))?;
        val
    } else {
        let prof = profile.unwrap_or("coding");
        let verify_opts = crate::commands::VerifyTaskOptions {
            repo: repo_root,
            profile: prof,
            file: None,
            max_runtime_ms: None,
            evidence_pack: false,
            evidence_pack_dir: None,
        };
        let out = crate::commands::verify_task(verify_opts).await?;
        serde_json::to_value(&out)?
    };

    if let Some(parent) = test_results_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&test_results_path, serde_json::to_string_pretty(&payload)?)?;

    let all_passed = payload["all_passed"].as_bool().unwrap_or(false);

    let event_msg = format!(
        "Recorded test results (profile={:?}, all_passed={})",
        profile.unwrap_or("coding"),
        all_passed
    );
    let audit_path =
        workflow_aidlc::append_workflow_audit(repo, &wf_id, &event_msg, "human")?;

    println!("Test results recorded at {}", test_results_path.display());
    println!("Appended audit log at {}", audit_path.display());

    Ok(())
}

pub fn workflow_record_readiness(
    repo_root: &str,
    id: Option<&str>,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = if let Some(i) = id {
        i.to_string()
    } else {
        resolve_single_workflow_id(repo)?
    };

    let _manifest = load_manifest(repo, &wf_id)?;
    let readiness_path = operations_dir(repo, &wf_id).join("readiness.json");

    let drift_val = drift_clean(repo)?;
    let lint_val = if repo.join("repo.sruja").exists() {
        let content = std::fs::read_to_string(repo.join("repo.sruja"))?;
        let parser = sruja_language::Parser::new("repo.sruja".to_string());
        parser.parse(&content).is_ok()
    } else {
        true
    };

    let test_results_path = construction_dir(repo, &wf_id).join("test-results.json");
    let tests_val = if test_results_path.exists() {
        if let Ok(text) = std::fs::read_to_string(&test_results_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                val["all_passed"].as_bool().unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    let rollback_path = operations_dir(repo, &wf_id).join("rollback-plan.md");
    let rollback_val = rollback_path.exists()
        && std::fs::metadata(&rollback_path)
            .map(|m| m.len() > 10)
            .unwrap_or(false);

    let deploy_scope_path = operations_dir(repo, &wf_id).join("deploy-scope.json");
    let deploy_val = deploy_scope_path.exists()
        && std::fs::metadata(&deploy_scope_path)
            .map(|m| m.len() > 5)
            .unwrap_or(false);

    let all_ready = drift_val && lint_val && tests_val && rollback_val && deploy_val;

    let payload = serde_json::json!({
        "schema_version": "workflow_readiness/v1",
        "workflow_id": wf_id,
        "recorded_at": now_iso(),
        "checks": {
            "drift_clean": drift_val,
            "lint_clean": lint_val,
            "intent_aligned": true,
            "tests_passed": tests_val,
            "rollback_plan": rollback_val,
            "deploy_scope_defined": deploy_val
        },
        "all_ready": all_ready
    });

    if let Some(parent) = readiness_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&readiness_path, serde_json::to_string_pretty(&payload)?)?;

    let event_msg = format!("Recorded operations readiness (all_ready={})", all_ready);
    let audit_path =
        workflow_aidlc::append_workflow_audit(repo, &wf_id, &event_msg, "human")?;

    println!("Readiness recorded at {}", readiness_path.display());
    println!("Appended audit log at {}", audit_path.display());

    Ok(())
}
