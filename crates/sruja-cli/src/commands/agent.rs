use crate::commands::CliError;
use crate::utils::colors;
use sruja_agent::{AgenticMemory, ExperimentOutcome, LearningEntry};
use std::path::Path;

/// Displays the history of architectural learnings and guardrails recorded in agentic memory.
///
/// If `element_id` is provided, filters the history to only show learnings relevant to that element.
/// Supports `json` format via the `format` parameter.
pub async fn agent_history(
    repo: &str,
    element_id: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let memory = AgenticMemory::load(Path::new(repo))
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    if format == "json" {
        let entries = if let Some(id) = element_id {
            memory.find_relevant(id)
        } else {
            memory.learnings.iter().collect()
        };
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    println!(
        "{}",
        colors::style("Architectural Learning History & Guardrails").bold()
    );
    println!(
        "{}",
        colors::dim("------------------------------------------")
    );

    let entries = if let Some(id) = element_id {
        println!("Filtering by element: {}", colors::info(id));
        memory.find_relevant(id)
    } else {
        memory.learnings.iter().collect()
    };

    if entries.is_empty() {
        println!("No learnings recorded yet.");
        return Ok(());
    }

    for (i, entry) in entries.iter().enumerate() {
        let outcome_color = match entry.outcome {
            ExperimentOutcome::Success => colors::success("SUCCESS"),
            ExperimentOutcome::Failed => colors::error("FAILED"),
        };

        println!(
            "\n[{}] {} - {}",
            i + 1,
            entry.timestamp.format("%Y-%m-%d %H:%M"),
            outcome_color
        );
        println!("  Context:    {}", colors::style(&entry.context).bold());
        println!("  Hypothesis: {}", entry.hypothesis);
        if let Some(reason) = &entry.reason {
            println!("  Reason:     {}", colors::dim(reason));
        }
        println!("  Guardrail:  {}", colors::warning(&entry.guardrail_advice));
        if !entry.affected_elements.is_empty() {
            println!("  Elements:   {}", entry.affected_elements.join(", "));
        }
    }

    Ok(())
}

/// Records a new architectural learning entry in agentic memory.
///
/// This is typically used by AI agents to record the results of their experiments,
/// helping future agents avoid repeating mistakes or to replicate successful patterns.
pub async fn agent_record(
    repo: &str,
    context: &str,
    hypothesis: &str,
    outcome_str: &str,
    guardrail: &str,
    reason: Option<&str>,
    elements: Option<&str>,
) -> Result<(), CliError> {
    let outcome = match outcome_str.to_lowercase().as_str() {
        "success" | "succeeded" | "pass" => ExperimentOutcome::Success,
        _ => ExperimentOutcome::Failed,
    };

    let affected_elements = elements
        .map(|e| {
            e.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let mut memory = AgenticMemory::load(Path::new(repo))
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    memory.add_learning(LearningEntry {
        timestamp: chrono::Utc::now(),
        context: context.to_string(),
        hypothesis: hypothesis.to_string(),
        outcome,
        reason: reason.map(|s| s.to_string()),
        guardrail_advice: guardrail.to_string(),
        affected_elements,
    });

    memory
        .save(Path::new(repo))
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    println!("✅ Learning recorded in Agentic Memory.");
    Ok(())
}

/// Clears all agentic memory for the specified repository.
///
/// Requires `force` to be true to confirm deletion.
pub async fn agent_clear(repo: &str, force: bool) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !AgenticMemory::exists(repo_path) {
        println!("Memory is already empty.");
        return Ok(());
    }

    if !force {
        println!("⚠️  Are you sure you want to clear all agentic memory for this repository?");
        println!("   This will delete all prior learnings and guardrails.");
        // In a real CLI we might use dialoguer here, but for now we'll require --force
        return Err(CliError::validation(
            "Action requires --force to confirm deletion.",
        ));
    }

    AgenticMemory::clear(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
    println!("🗑️  Agentic memory cleared.");
    Ok(())
}
