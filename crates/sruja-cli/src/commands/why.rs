use std::path::Path;

use sruja_graph::QueryError;

use super::CliError;

pub async fn why(repo_root: &str, question: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = crate::graph_store::load_or_build_graph(repo_path)?;
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
