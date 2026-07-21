use std::path::Path;

use crate::commands::CliError;

use super::manifest::*;
use super::types::*;

pub fn workflow_summary(
    repo_root: &str,
    id: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = if let Some(i) = id {
        i.to_string()
    } else {
        resolve_single_workflow_id(repo)?
    };

    let manifest = load_manifest(repo, &wf_id)?;
    let gate = compute_gate_check(repo, &manifest)?;

    // Calculate completeness & health score
    let mut total_checks = 0;
    let mut passed_checks = 0;

    let inc_reqs = inception_required_files(repo, &wf_id, &manifest);
    total_checks += inc_reqs.len();
    passed_checks += inc_reqs.len() - list_missing(&inc_reqs).len();

    let con_reqs = construction_required_files(repo, &wf_id, &manifest);
    total_checks += con_reqs.len();
    passed_checks += con_reqs.len() - list_missing(&con_reqs).len();

    let op_reqs = operations_required_files(repo, &wf_id, &manifest);
    total_checks += op_reqs.len();
    passed_checks += op_reqs.len() - list_missing(&op_reqs).len();

    let test_results_path = construction_dir(repo, &wf_id).join("test-results.json");
    let mut tests_pass = false;
    if test_results_path.exists() {
        total_checks += 1;
        if let Ok(text) = std::fs::read_to_string(&test_results_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                if val["all_passed"].as_bool() == Some(true) {
                    tests_pass = true;
                    passed_checks += 1;
                }
            }
        }
    }

    let readiness_path = operations_dir(repo, &wf_id).join("readiness.json");
    let mut ready_pass = false;
    if readiness_path.exists() {
        total_checks += 1;
        if let Ok(text) = std::fs::read_to_string(&readiness_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                if val["all_ready"].as_bool() == Some(true) {
                    ready_pass = true;
                    passed_checks += 1;
                }
            }
        }
    }

    let health_score = if total_checks > 0 {
        ((passed_checks as f32 / total_checks as f32) * 100.0) as u8
    } else {
        100
    };

    if format == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "workflow_summary/v1",
                "workflow_id": wf_id,
                "title": manifest.title,
                "profile": manifest.profile,
                "phase": manifest.phase,
                "health_score": health_score,
                "gate_allowed": gate.allowed,
                "missing": gate.missing,
                "inception_approved": manifest.phase_approvals.inception.is_some(),
                "construction_approved": manifest.phase_approvals.construction.is_some(),
                "operations_approved": manifest.phase_approvals.operations.is_some(),
            }))?
        );
        return Ok(());
    }

    // Text format - beautiful terminal dashboard
    println!("============================================================");
    println!("                    SRUJA WORKFLOW DASHBOARD                ");
    println!("============================================================");
    println!("Workflow ID   : {}", wf_id);
    println!("Title         : {}", manifest.title);
    println!("Profile       : {}", manifest.profile.to_uppercase());
    println!("Health Score  : {}/100", health_score);
    println!(
        "Gate Allowed  : {}",
        if gate.allowed { "✔ YES" } else { "✘ NO" }
    );
    println!("------------------------------------------------------------");

    // Phase progression timeline
    print!("Timeline      : ");
    let p = manifest.phase.as_str();
    if p == "inception" {
        print!("[INCEPTION] ===> construction ===> operations");
    } else if p == "construction" {
        print!("inception ===> [CONSTRUCTION] ===> operations");
    } else {
        print!("inception ===> construction ===> [OPERATIONS]");
    }
    println!("\n------------------------------------------------------------");

    // Artifacts check
    println!("Phase Artifacts Status:");
    println!("  [Inception]");
    for path in inc_reqs {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        println!("    {} {}", if path.exists() { "✔" } else { "✘" }, name);
    }
    println!("  [Construction]");
    for path in con_reqs {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        println!("    {} {}", if path.exists() { "✔" } else { "✘" }, name);
    }
    println!("  [Operations]");
    for path in op_reqs {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        println!("    {} {}", if path.exists() { "✔" } else { "✘" }, name);
    }
    println!("------------------------------------------------------------");

    // Linked proposals
    let linked_path = construction_dir(repo, &wf_id).join("linked_proposal_ids.json");
    if linked_path.exists() {
        if let Ok(text) = std::fs::read_to_string(&linked_path) {
            if let Ok(ids) = serde_json::from_str::<Vec<String>>(&text) {
                if !ids.is_empty() {
                    println!("Linked Proposals:");
                    for pid in ids {
                        println!("  - {}", pid);
                    }
                    println!("------------------------------------------------------------");
                }
            }
        }
    }

    // Test Results Summary
    if test_results_path.exists() {
        println!(
            "Test Results  : {}",
            if tests_pass {
                "✔ PASSED"
            } else {
                "✘ FAILED / UNRECORDED"
            }
        );
        println!("------------------------------------------------------------");
    }

    // Readiness Summary
    if readiness_path.exists() {
        println!(
            "Readiness     : {}",
            if ready_pass {
                "✔ READY"
            } else {
                "✘ NOT READY"
            }
        );
        println!("------------------------------------------------------------");
    }

    // Last 3 Audit Events
    let audit_path = workflow_dir(repo, &wf_id).join("audit.jsonl");
    if audit_path.exists() {
        if let Ok(text) = std::fs::read_to_string(&audit_path) {
            let lines: Vec<&str> = text.lines().rev().take(3).collect();
            if !lines.is_empty() {
                println!("Audit Trail (Last 3):");
                for line in lines {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                        let at_str = val["at"].as_str().unwrap_or("");
                        let at = at_str.get(..19).unwrap_or(at_str);
                        let actor = val["actor"].as_str().unwrap_or("");
                        let event = val["event"].as_str().unwrap_or("");
                        println!("  [{}] {} - {}", at, actor, event);
                    }
                }
                println!("------------------------------------------------------------");
            }
        }
    }

    Ok(())
}

pub fn workflow_summary_json_value(
    repo_root: &str,
    id: &str,
) -> Result<serde_json::Value, CliError> {
    let repo = Path::new(repo_root);
    let manifest = load_manifest(repo, id)?;
    let gate = compute_gate_check(repo, &manifest)?;

    let mut total_checks = 0;
    let mut passed_checks = 0;

    let inc_reqs = inception_required_files(repo, id, &manifest);
    total_checks += inc_reqs.len();
    passed_checks += inc_reqs.len() - list_missing(&inc_reqs).len();

    let con_reqs = construction_required_files(repo, id, &manifest);
    total_checks += con_reqs.len();
    passed_checks += con_reqs.len() - list_missing(&con_reqs).len();

    let op_reqs = operations_required_files(repo, id, &manifest);
    total_checks += op_reqs.len();
    passed_checks += op_reqs.len() - list_missing(&op_reqs).len();

    let test_results_path = construction_dir(repo, id).join("test-results.json");
    let mut tests_pass = false;
    if test_results_path.exists() {
        total_checks += 1;
        if let Ok(text) = std::fs::read_to_string(&test_results_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                if val["all_passed"].as_bool() == Some(true) {
                    tests_pass = true;
                    passed_checks += 1;
                }
            }
        }
    }

    let readiness_path = operations_dir(repo, id).join("readiness.json");
    let mut ready_pass = false;
    if readiness_path.exists() {
        total_checks += 1;
        if let Ok(text) = std::fs::read_to_string(&readiness_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                if val["all_ready"].as_bool() == Some(true) {
                    ready_pass = true;
                    passed_checks += 1;
                }
            }
        }
    }

    let health_score = if total_checks > 0 {
        ((passed_checks as f32 / total_checks as f32) * 100.0) as u8
    } else {
        100
    };

    Ok(serde_json::json!({
        "schema_version": "workflow_summary/v1",
        "workflow_id": id,
        "title": manifest.title,
        "profile": manifest.profile,
        "phase": manifest.phase,
        "health_score": health_score,
        "gate_allowed": gate.allowed,
        "missing": gate.missing,
        "inception_approved": manifest.phase_approvals.inception.is_some(),
        "construction_approved": manifest.phase_approvals.construction.is_some(),
        "operations_approved": manifest.phase_approvals.operations.is_some(),
        "tests_passed": tests_pass,
        "readiness_passed": ready_pass,
    }))
}

pub fn workflow_next_steps_json_value(
    repo_root: &str,
    wf_id: &str,
) -> Result<serde_json::Value, CliError> {
    let repo = Path::new(repo_root);
    let manifest = load_manifest(repo, wf_id)?;
    let gate = compute_gate_check(repo, &manifest)?;

    let mut steps = Vec::new();
    let mut recommendations = Vec::new();

    let phase = manifest.phase.as_str();
    let profile = manifest.profile.as_str();

    match phase {
        "inception" => {
            if profile == "e2e" {
                let req_path = inception_dir(repo, wf_id).join("requirements.md");
                if !req_path.exists() {
                    steps.push("Create requirements.md by running `sruja workflow capture-requirements` or creating `.sruja/workflows/<id>/inception/requirements.md`".to_string());
                    recommendations.push(
                        "Traceable requirements are mandatory for the e2e profile.".to_string(),
                    );
                }
            }

            let scope_path = inception_dir(repo, wf_id).join("scope.md");
            if !scope_path.exists() {
                steps.push(
                    "Scaffold scope.md by creating `.sruja/workflows/<id>/inception/scope.md`"
                        .to_string(),
                );
            }

            let impact_path = inception_dir(repo, wf_id).join("impact.json");
            if !impact_path.exists() {
                steps.push("Generate impact.json for your target elements under `.sruja/workflows/<id>/inception/impact.json`".to_string());
            }

            let dr_path = inception_dir(repo, wf_id).join("design-review.md");
            if !dr_path.exists() {
                steps.push(
                    "Perform design review by running `sruja workflow design-review`".to_string(),
                );
            } else {
                let dr_json = workflow_dir(repo, wf_id).join("design-review.json");
                if dr_json.exists() {
                    if let Ok(text) = std::fs::read_to_string(&dr_json) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(blockers) = val["blockers"].as_array() {
                                for b in blockers {
                                    if let Some(b_str) = b.as_str() {
                                        steps.push(format!(
                                            "Resolve design review blocker: {}",
                                            b_str
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if steps.is_empty() && manifest.phase_approvals.inception.is_none() {
                steps.push("Approve the inception phase by running `sruja workflow approve` or updating phase_approvals in manifest.".to_string());
                recommendations.push(
                    "Once approved, transition the phase in manifest.json to 'construction'."
                        .to_string(),
                );
            }
        }
        "construction" => {
            let tp_path = construction_dir(repo, wf_id).join("task-plan.md");
            if !tp_path.exists() {
                steps.push("Create task-plan.md under `.sruja/workflows/<id>/construction/task-plan.md` to detail implementation steps.".to_string());
            }

            let linked_path = construction_dir(repo, wf_id).join("linked_proposal_ids.json");
            let mut has_linked = false;
            if linked_path.exists() {
                if let Ok(text) = std::fs::read_to_string(&linked_path) {
                    if let Ok(ids) = serde_json::from_str::<Vec<String>>(&text) {
                        has_linked = !ids.is_empty();
                    }
                }
            }
            if !has_linked {
                steps.push("Propose architecture topology changes or link an existing proposal by listing it in `.sruja/workflows/<id>/construction/linked_proposal_ids.json`".to_string());
            }

            let test_results_path = construction_dir(repo, wf_id).join("test-results.json");
            if !test_results_path.exists() {
                steps.push("Run and record test verification results using `sruja workflow record-test-results`".to_string());
            } else if let Ok(text) = std::fs::read_to_string(&test_results_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                    if val["all_passed"].as_bool() != Some(true) {
                        steps.push(
                            "Fix test failures in verification suite and re-record test results"
                                .to_string(),
                        );
                    }
                }
            }

            if steps.is_empty() && manifest.phase_approvals.construction.is_none() {
                steps.push("Approve the construction phase by running `sruja workflow approve` or updating phase_approvals in manifest.".to_string());
                recommendations.push(
                    "Once approved, transition the phase in manifest.json to 'operations'."
                        .to_string(),
                );
            }
        }
        "operations" => {
            let ds_path = operations_dir(repo, wf_id).join("deploy-scope.json");
            if !ds_path.exists() {
                steps.push("Define deployment scope under `.sruja/workflows/<id>/operations/deploy-scope.json`".to_string());
            }

            let readiness_path = operations_dir(repo, wf_id).join("readiness.json");
            if !readiness_path.exists() {
                steps.push("Perform operations readiness checks and record results using `sruja workflow record-readiness`".to_string());
            } else if let Ok(text) = std::fs::read_to_string(&readiness_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                    if val["all_ready"].as_bool() != Some(true) {
                        steps.push("Resolve readiness blockers (such as architectural drift, lint failures, or failing tests) and re-record readiness".to_string());
                    }
                }
            }

            if steps.is_empty() && manifest.phase_approvals.operations.is_none() {
                steps.push("Approve operations to complete the workflow by running `sruja workflow approve`.".to_string());
            }
        }
        _ => {
            steps.push(format!("Unknown phase: {}", phase));
        }
    }

    if !gate.allowed {
        recommendations.push(format!(
            "Currently gated! Missing requirements to proceed: {}",
            gate.missing.join(", ")
        ));
    }

    Ok(serde_json::json!({
        "schema_version": "workflow_next_steps/v1",
        "workflow_id": wf_id,
        "current_phase": phase,
        "profile": profile,
        "gate_allowed": gate.allowed,
        "next_steps": steps,
        "recommendations": recommendations,
    }))
}

/// Public wrapper: show actionable next steps for the current workflow phase.
pub fn workflow_next_steps(repo_root: &str, id: Option<&str>) -> Result<(), CliError> {
    let repo = Path::new(repo_root);
    let wf_id = if let Some(id) = id {
        id.to_string()
    } else {
        resolve_single_workflow_id(repo)?
    };
    let value = workflow_next_steps_json_value(repo_root, &wf_id)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
