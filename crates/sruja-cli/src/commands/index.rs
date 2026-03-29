use crate::commands::CliError;
use sruja_export::vector::SemanticSearcher;
use std::fs;
use std::path::Path;

pub async fn index(
    repo_path: &str,
    architecture_file: Option<&str>,
    output_path: &str,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_path);

    // 1. Get the graph (either from .sruja/graph.json or fresh scan)
    let graph = if let Some(arch) = architecture_file {
        let _ = crate::commands::parse_sruja_file(arch)?;
        sruja_scan::scan_repo(repo_path)?
    } else {
        crate::commands::scan_repo_cached(repo_path)?
    };

    println!("🚀 Initializing semantic indexer with BGE-small-en-v1.5...");
    let mut searcher = SemanticSearcher::new().map_err(|e| {
        CliError::Io(std::io::Error::other(format!(
            "Failed to init searcher: {}",
            e
        )))
    })?;

    println!(
        "🧠 Generating embeddings for {} nodes...",
        graph.nodes.len()
    );
    let nodes_to_index: Vec<(String, String, String)> = graph
        .nodes
        .iter()
        .map(|n| {
            let desc = n
                .metadata
                .get("description")
                .map(|s| s.as_str())
                .unwrap_or(n.label.as_str());
            (
                n.id.clone(),
                n.label.clone(),
                format!("node: {} - {}", n.label, desc),
            )
        })
        .collect();

    let index = searcher
        .index_nodes(nodes_to_index)
        .map_err(|e| CliError::Io(std::io::Error::other(format!("Embedding failed: {}", e))))?;

    // 2. Save the index
    let output = Path::new(output_path);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&index)?;
    fs::write(output, json)?;

    println!("✅ Semantic index saved to {}", output_path);

    Ok(())
}
