use std::path::Path;

use sruja_graph::{QueryError, ReasonedWhyResult};

use super::CliError;

pub async fn why(
    repo_root: &str,
    question: &str,
    format: &str,
    reasoned: bool,
    llmguided: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = crate::graph_store::load_or_build_graph(repo_path)?;

    if reasoned {
        let result = graph.query_why_reasoned(question, 3).map_err(|e| match e {
            QueryError::NoResults => CliError::validation("No results found".to_string()),
            _ => CliError::validation(e.to_string()),
        })?;
        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&result)?);
                Ok(())
            }
            "text" => {
                print_reasoned_text(&result);
                Ok(())
            }
            _ => Err(CliError::validation(format!(
                "Unknown format: {}. Use: text, json",
                format
            ))),
        }
    } else if llmguided {
        let result = graph
            .query_why_llmguided(question, 4)
            .map_err(|e| match e {
                QueryError::NoResults => CliError::validation("No results found".to_string()),
                _ => CliError::validation(e.to_string()),
            })?;
        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&result)?);
                Ok(())
            }
            "text" => {
                print_llmguided_text(&result);
                Ok(())
            }
            _ => Err(CliError::validation(format!(
                "Unknown format: {}. Use: text, json",
                format
            ))),
        }
    } else {
        let result = graph.query(question).map_err(|e| match e {
            QueryError::NoResults => CliError::validation("No results found".to_string()),
            _ => CliError::validation(e.to_string()),
        })?;
        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&result)?);
                Ok(())
            }
            "text" => {
                println!("{}", result.answer);
                println!();
                println!("Confidence: {:.0}%", result.confidence * 100.0);
                if !result.evidence.is_empty() {
                    println!();
                    println!("Evidence:");
                    for ev in &result.evidence {
                        println!("  - {:?}: {} | {}", ev.kind, ev.reference, ev.excerpt);
                    }
                }
                Ok(())
            }
            _ => Err(CliError::validation(format!(
                "Unknown format: {}. Use: text, json",
                format
            ))),
        }
    }
}

fn print_reasoned_text(result: &ReasonedWhyResult) {
    println!("=== Traceable Why Explanation ===");
    println!();
    println!("Question: {}", result.question);
    println!("Target: {} ({})", result.target_label, result.target_id);
    println!();

    if result.steps.is_empty() {
        println!(
            "No traversal possible: {} is isolated in the graph.",
            result.target_label
        );
    } else {
        println!("Reasoning walk ({} step(s)):", result.steps.len());
        println!();
        for step in &result.steps {
            let dir_initial = step
                .direction
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect::<String>())
                .unwrap_or_else(|| "?".to_string());
            println!(
                "  [{}{}] {} ({})",
                dir_initial,
                step.direction.len(),
                step.node_label,
                step.node_id
            );
            println!(
                "    via '{}' relationship → {}",
                step.relationship, step.direction
            );
            println!("    Reasoning: {}", step.reasoning);
            if let Some(ref decision) = step.decision_ref {
                println!("    Decision: {}", decision);
            }
            println!();
        }
    }

    println!("---");
    println!("{}", result.final_answer);
    println!("Confidence: {:.0}%", result.confidence * 100.0);
}

use sruja_graph::LlmGuidedWhyResult;

fn print_llmguided_text(result: &LlmGuidedWhyResult) {
    println!("=== LLM-Guided Tree-Search Why ===");
    println!();
    println!("Question: {}", result.question);
    println!("Target: {} ({})", result.target_label, result.target_id);
    println!();

    if result.steps.is_empty() {
        println!(
            "No traversal possible: {} is isolated in the graph.",
            result.target_label
        );
    } else {
        println!("LLM-guided walk ({} step(s)):", result.steps.len());
        println!();
        for step in &result.steps {
            let decision_note = step
                .decision_ref
                .as_ref()
                .map(|d| format!(" [Decision: {}]", d))
                .unwrap_or_default();
            println!(
                "  [{}] {} ({})",
                step.direction.to_uppercase(),
                step.node_label,
                step.node_id
            );
            println!("    via '{}'", step.relationship);
            println!(
                "    Relevance: {} (confidence: {:.0}%)",
                step.relevance_score,
                step.confidence * 100.0
            );
            println!("    LLM reasoning: {}", step.llm_reasoning);
            println!("{}", decision_note);
            println!();
        }
    }

    println!("---");
    println!("{}", result.summary);
    println!(
        "Total LLM-guided steps: {} | Confidence: {:.0}%",
        result.steps.len(),
        result.confidence * 100.0
    );
}
