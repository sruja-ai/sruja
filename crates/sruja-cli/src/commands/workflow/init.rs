use std::path::Path;

use crate::commands::workflow_aidlc::{self, AidlcConfig};
use crate::commands::CliError;

use super::manifest::save_manifest;
use super::operations::workflow_install_rules;
use super::types::*;

pub fn workflow_init(
    repo_root: &str,
    title: &str,
    id: Option<&str>,
    target_elements: Vec<String>,
    strict_gates: bool,
    options: WorkflowInitOptions,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = id
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()[..8].to_string());
    let dir = workflow_dir(repo, &wf_id);
    let inc_dir = inception_dir(repo, &wf_id);
    let con_dir = construction_dir(repo, &wf_id);
    let op_dir = operations_dir(repo, &wf_id);

    std::fs::create_dir_all(&inc_dir)?;
    std::fs::create_dir_all(&con_dir)?;
    std::fs::create_dir_all(&op_dir)?;

    let template = options
        .template
        .clone()
        .unwrap_or_else(|| "minimal".to_string());
    let profile = if options.profile != "minimal" {
        options.profile.clone()
    } else {
        match template.as_str() {
            "e2e" => "e2e".to_string(),
            _ => "minimal".to_string(),
        }
    };

    // Scaffolding based on template
    match template.as_str() {
        "e2e" => {
            std::fs::write(inc_dir.join("requirements.md"), requirements_template())?;
            std::fs::write(inc_dir.join("scope.md"), scope_template())?;
            std::fs::write(
                inc_dir.join("impact.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_impact/v1",
                    "workflow_id": wf_id,
                    "targets": []
                }))?,
            )?;
            std::fs::write(inc_dir.join("design-review.md"), design_review_template())?;

            std::fs::write(con_dir.join("task-plan.md"), task_plan_template())?;
            std::fs::write(con_dir.join("linked_proposal_ids.json"), "[]")?;
            std::fs::write(con_dir.join("test-plan.md"), test_plan_template())?;
            std::fs::write(
                con_dir.join("test-results.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_test_results/v1",
                    "workflow_id": wf_id,
                    "recorded_at": now_iso(),
                    "profile": "coding",
                    "all_passed": false,
                    "steps": []
                }))?,
            )?;

            std::fs::write(
                op_dir.join("deploy-scope.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_deploy_scope/v1",
                    "resources": []
                }))?,
            )?;
            std::fs::write(
                op_dir.join("readiness.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_readiness/v1",
                    "workflow_id": wf_id,
                    "recorded_at": now_iso(),
                    "checks": {
                        "drift_clean": false,
                        "lint_clean": false,
                        "intent_aligned": false,
                        "tests_passed": false,
                        "rollback_plan": false,
                        "deploy_scope_defined": false
                    },
                    "all_ready": false
                }))?,
            )?;
            std::fs::write(op_dir.join("rollback-plan.md"), rollback_plan_template())?;
        }
        "feature" => {
            std::fs::write(inc_dir.join("requirements.md"), requirements_template())?;
            std::fs::write(inc_dir.join("scope.md"), scope_template())?;
            std::fs::write(
                inc_dir.join("impact.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_impact/v1",
                    "workflow_id": wf_id,
                    "targets": []
                }))?,
            )?;

            std::fs::write(con_dir.join("task-plan.md"), task_plan_template())?;
            std::fs::write(con_dir.join("linked_proposal_ids.json"), "[]")?;
            std::fs::write(
                con_dir.join("test-results.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_test_results/v1",
                    "workflow_id": wf_id,
                    "recorded_at": now_iso(),
                    "profile": "coding",
                    "all_passed": false,
                    "steps": []
                }))?,
            )?;

            std::fs::write(
                op_dir.join("deploy-scope.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_deploy_scope/v1",
                    "resources": []
                }))?,
            )?;
        }
        "bugfix" => {
            std::fs::write(inc_dir.join("scope.md"), scope_template())?;
            std::fs::write(
                inc_dir.join("impact.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_impact/v1",
                    "workflow_id": wf_id,
                    "targets": []
                }))?,
            )?;

            std::fs::write(con_dir.join("task-plan.md"), task_plan_template())?;
            std::fs::write(con_dir.join("linked_proposal_ids.json"), "[]")?;
            std::fs::write(
                con_dir.join("test-results.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_test_results/v1",
                    "workflow_id": wf_id,
                    "recorded_at": now_iso(),
                    "profile": "coding",
                    "all_passed": false,
                    "steps": []
                }))?,
            )?;

            std::fs::write(
                op_dir.join("deploy-scope.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_deploy_scope/v1",
                    "resources": []
                }))?,
            )?;
        }
        _ => {
            // "minimal"
            std::fs::write(inc_dir.join("scope.md"), scope_template())?;
            std::fs::write(
                inc_dir.join("impact.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_impact/v1",
                    "workflow_id": wf_id,
                    "targets": []
                }))?,
            )?;

            std::fs::write(con_dir.join("task-plan.md"), task_plan_template())?;
            std::fs::write(con_dir.join("linked_proposal_ids.json"), "[]")?;

            std::fs::write(
                op_dir.join("deploy-scope.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "workflow_deploy_scope/v1",
                    "resources": []
                }))?,
            )?;
        }
    }

    if options.with_aidlc {
        let aidlc = AidlcConfig {
            enabled: true,
            profile: options.aidlc_profile.clone(),
            ..Default::default()
        };
        std::fs::create_dir_all(workflow_aidlc::aidlc_docs_dir(&inc_dir, &aidlc))?;
        std::fs::create_dir_all(workflow_aidlc::aidlc_docs_dir(&con_dir, &aidlc))?;
        if options.install_rules {
            workflow_install_rules(repo_root)?;
        }
    }

    let mut manifest = WorkflowManifest::new(wf_id.clone(), title.to_string());
    manifest.target_elements = target_elements;
    manifest.strict_gates = strict_gates;
    manifest.repo_root = repo_root.to_string();
    manifest.profile = profile;
    if options.with_aidlc {
        manifest.aidlc = Some(AidlcConfig {
            enabled: true,
            profile: options.aidlc_profile,
            docs_root: workflow_aidlc::default_docs_root_for_manifest(),
            ..Default::default()
        });
    }
    save_manifest(repo, &manifest)?;

    println!("{}", serde_json::to_string_pretty(&manifest)?);
    println!(
        "Workflow initialized at {}",
        dir.join("manifest.json").display()
    );
    Ok(())
}
