use crate::commands::CliError;
use crate::utils::colors;
use sruja_agent::{AgenticMemory, ExperimentOutcome, LearningEntry, LearningKind};
use std::collections::HashSet;
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
        id: String::new(),
        kind: Some(match outcome {
            ExperimentOutcome::Success => LearningKind::Playbook,
            ExperimentOutcome::Failed => LearningKind::Guardrail,
        }),
        timestamp: chrono::Utc::now(),
        run_id: None,
        repo: Some(repo.to_string()),
        selector: None,
        context: context.to_string(),
        hypothesis: hypothesis.to_string(),
        outcome,
        reason: reason.map(|s| s.to_string()),
        guardrail_advice: guardrail.to_string(),
        affected_elements,
        evidence_refs: Vec::new(),
        confidence: None,
        tags: Vec::new(),
        related_ids: Vec::new(),
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

/// Displays thematic clusters and tags from Zettelkasten-linked agentic memory.
pub async fn agent_clusters(
    repo: &str,
    entry_id: Option<&str>,
    tag: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let memory = AgenticMemory::load(Path::new(repo))
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    if let Some(eid) = entry_id {
        let cluster = memory.find_cluster(eid);
        if format == "json" {
            println!("{}", serde_json::to_string_pretty(&cluster)?);
        } else {
            println!(
                "{}",
                colors::style(&format!("Cluster for entry: {}", eid)).bold()
            );
            if cluster.is_empty() {
                println!("No cluster found for entry ID '{}'.", eid);
            } else {
                for entry in &cluster {
                    print_learning_summary(entry);
                }
            }
        }
        return Ok(());
    }

    if let Some(t) = tag {
        let entries = memory.find_by_tag(t);
        if format == "json" {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        } else {
            println!(
                "{}",
                colors::style(&format!("Entries tagged '{}'", t)).bold()
            );
            if entries.is_empty() {
                println!("No entries found for tag '{}'.", t);
            } else {
                for entry in &entries {
                    print_learning_summary(entry);
                }
            }
        }
        return Ok(());
    }

    // Default: show all tags and cluster overview
    let all_tags = memory.all_tags();

    if format == "json" {
        let mut clusters = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        for entry in &memory.learnings {
            if visited.contains(&entry.id) {
                continue;
            }
            let cluster = memory.find_cluster(&entry.id);
            let ids: Vec<String> = cluster.iter().map(|e| e.id.clone()).collect();
            for id in &ids {
                visited.insert(id.clone());
            }
            clusters.push(serde_json::json!({
                "root_id": entry.id,
                "size": cluster.len(),
                "entry_ids": ids,
                "tags": cluster.iter().flat_map(|e| e.tags.clone()).collect::<HashSet<_>>(),
            }));
        }
        let output = serde_json::json!({
            "total_entries": memory.learnings.len(),
            "total_tags": all_tags.len(),
            "tags": all_tags,
            "clusters": clusters,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!(
            "{}",
            colors::style("Agentic Memory: Clusters & Tags").bold()
        );
        println!("{}", colors::dim("────────────────────────────────"));
        println!(
            "  Entries: {}  Tags: {}",
            memory.learnings.len(),
            all_tags.len()
        );

        if !all_tags.is_empty() {
            println!("\n{}", colors::style("Tags:").bold());
            for t in &all_tags {
                let count = memory.find_by_tag(t).len();
                println!("  {} ({})", colors::info(t), count);
            }
        }

        // Show clusters
        let mut visited: HashSet<String> = HashSet::new();
        let mut cluster_idx = 0;
        for entry in &memory.learnings {
            if visited.contains(&entry.id) {
                continue;
            }
            let cluster = memory.find_cluster(&entry.id);
            let ids: Vec<String> = cluster.iter().map(|e| e.id.clone()).collect();
            for id in &ids {
                visited.insert(id.clone());
            }
            cluster_idx += 1;
            println!(
                "\n{} ({} entries)",
                colors::style(&format!("Cluster #{}", cluster_idx)).bold(),
                cluster.len()
            );
            for entry in &cluster {
                print_learning_summary(entry);
            }
        }

        if cluster_idx == 0 {
            println!("\nNo learnings recorded yet.");
        }
    }

    Ok(())
}

fn print_learning_summary(entry: &LearningEntry) {
    let outcome_str = match entry.outcome {
        ExperimentOutcome::Success => colors::success("SUCCESS"),
        ExperimentOutcome::Failed => colors::error("FAILED"),
    };
    let short_id: String = entry.id.chars().take(12).collect();
    println!(
        "  [{}] {} - {} | {}",
        short_id,
        entry.timestamp.format("%Y-%m-%d"),
        outcome_str,
        colors::dim(&entry.context),
    );
    if !entry.tags.is_empty() {
        println!("    tags: {}", entry.tags.join(", "));
    }
}
