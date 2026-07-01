use crate::commands::{agent_run_to_string, AgentRunOptions};

/// Understand scope and produce a reviewable plan.
///
/// By default outputs human-readable text. Use `json=true` for machine-readable output.
pub async fn plan_run(
    repo: &str,
    goal: &str,
    file: Option<&str>,
    element_id: Option<&str>,
    query: Option<&str>,
    _pipeline: bool,
    output: Option<&str>,
    json: bool,
    _compact: bool,
) -> Result<(), crate::commands::CliError> {
    let enrich_ref = crate::enrichment::EnrichmentRef {
        enrich: false,
        provider: None,
        cmd: None,
        model: None,
        base_url: None,
        timeout_ms: 15000,
        max_bytes: 20000,
    };

    let result = agent_run_to_string(AgentRunOptions {
        repo,
        goal,
        file,
        element_id,
        query,
        mode: "plan",
        ai_mode: "standard",
        format: "json",
        run_id: None,
        max_steps: None,
        max_runtime_ms_per_step: None,
        enrich: &enrich_ref,
        continue_on_error: false,
        force_sync: false,
    })
    .await?;

    // Save raw JSON if output path given (always, regardless of display format)
    if let Some(out_path) = output {
        tokio::fs::write(out_path, &result).await?;
        eprintln!("Plan JSON saved to {}", out_path);
    }

    if json {
        println!("{}", result);
        return Ok(());
    }

    // Human-readable output: parse the JSON plan artifact (flat structure).
    // The plan output has keys at the top level: goal, target, steps, risks, verification, etc.
    let raw: serde_json::Value = match serde_json::from_str(&result) {
        Ok(v) => v,
        Err(_) => {
            println!("{}", result);
            return Ok(());
        }
    };

    let goal_text = raw.get("goal").and_then(|g| g.as_str()).unwrap_or(goal);

    let scope_file = raw
        .get("target")
        .and_then(|t| t.get("selector"))
        .and_then(|s| s.as_str());

    let steps: Vec<&serde_json::Value> = raw
        .get("steps")
        .and_then(|s| s.as_array())
        .map(|a| a.iter().collect())
        .unwrap_or_default();

    let risks: Vec<&serde_json::Value> = raw
        .get("risks")
        .and_then(|r| r.as_array())
        .map(|a| a.iter().collect())
        .unwrap_or_default();

    let verification: Vec<&serde_json::Value> = raw
        .get("verification")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().collect())
        .unwrap_or_default();

    // Build concise output
    println!("  ── Plan ──\n");
    println!("  Goal: {}", goal_text);
    if let Some(s) = scope_file {
        if s != goal {
            println!("  Scope: {}", s);
        }
    }
    println!();

    if !steps.is_empty() {
        println!("  Steps");
        for (i, step) in steps.iter().enumerate() {
            let kind = step.get("kind").and_then(|k| k.as_str()).unwrap_or("cmd");
            let desc = step
                .get("expected")
                .and_then(|e| e.as_str())
                .unwrap_or("Execute step");
            println!("    {}. [{}] {}", i + 1, kind, desc);
        }
        println!();
    }

    if !risks.is_empty() {
        println!("  Risks");
        for risk in &risks {
            if let Some(r) = risk.as_str() {
                println!("    · {}", r);
            }
        }
        println!();
    }

    if !verification.is_empty() {
        println!("  Recommended verification");
        for v in &verification {
            let argv: Vec<&str> = v
                .get("argv")
                .and_then(|a| a.as_array())
                .map(|a| a.iter().filter_map(|s| s.as_str()).collect())
                .unwrap_or_default();
            if !argv.is_empty() {
                println!("    · {}", argv.join(" "));
            }
        }
        println!();
    }

    Ok(())
}
