use crate::commands::scan_domain::scan::DriftRequest;
use crate::commands::{agent_apply, confidence, drift, intent_check, CliError, ConfidenceOptions};

/// Check architecture health: drift + lint + intent + confidence.
pub async fn verify_run(
    repo: &str,
    profile: &str,
    file: Option<&str>,
    with_confidence: bool,
    plan: Option<&str>,
    json: bool,
    continue_on_error: bool,
) -> Result<(), CliError> {
    let format = if json { "json" } else { "text" };

    // Plan-driven verification
    if let Some(plan_path) = plan {
        eprintln!("Running verification from plan: {}", plan_path);
        return agent_apply(std::path::Path::new(plan_path), repo, format).await;
    }

    // Confidence report
    if with_confidence {
        let report = confidence(ConfidenceOptions {
            repo,
            profile: if profile == "full" { "review" } else { profile },
            file,
            max_runtime_ms: None,
            evidence_pack: false,
            evidence_pack_dir: None,
        })
        .await?;
        let display_format = if json { "json" } else { "md" };
        println!(
            "{}",
            crate::commands::format_confidence(&report, display_format)
        );
        return Ok(());
    }

    // Full profile: run drift + intent check in parallel, capture output
    if profile == "full" {
        if !json {
            println!("  ── Verify ──\n");
            println!("  Checking architecture health...\n");
        }

        // Run drift and intent concurrently — each prints its own output internally
        let (drift_result, intent_result) = tokio::join!(
            drift(DriftRequest {
                repo_root: repo,
                architecture_path: None,
                format,
                violations_only: false,
                fail_on: None,
                violations_baseline: None,
                baseline_mode: None,
                structural_only: false,
                advisory: true,
                exclude_barrel_files: true,
            }),
            intent_check(repo, None, format, false),
        );

        if json {
            // JSON mode: suppress per-check output, emit a summary
            let summary = serde_json::json!({
                "profile": "full",
                "drift_ok": drift_result.is_ok(),
                "intent_ok": intent_result.is_ok(),
                "status": if drift_result.is_ok() && intent_result.is_ok() { "passed" } else { "has_findings" }
            });
            println!("{}", serde_json::to_string_pretty(&summary).unwrap());
        }

        let all_ok = drift_result.is_ok() && intent_result.is_ok();
        if !all_ok && !continue_on_error {
            return Err(CliError::validation(
                "Verification found issues (run `sruja verify` for details)",
            ));
        }
        return Ok(());
    }

    // Profile-based verification
    let output = crate::commands::verify_task(crate::commands::VerifyTaskOptions {
        repo,
        profile,
        file,
        max_runtime_ms: None,
        evidence_pack: false,
        evidence_pack_dir: None,
    })
    .await?;

    println!("{}", crate::commands::format_verify_task(&output, format));
    if !output.all_passed && !continue_on_error {
        return Err(CliError::validation("Verification failed"));
    }
    Ok(())
}
