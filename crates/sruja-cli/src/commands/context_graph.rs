use crate::commands::CliError;
use crate::graph_store;
use sruja_export::HtmlExporter;
use std::path::Path;

pub async fn context_graph(repo_root: &str, output_path: &str, open: bool) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    // Load the knowledge graph (prefer architecture baseline if it exists)
    let baseline_path = crate::utils::architecture_path::resolve_architecture_path(repo_path);
    let kg = if let Some(ref path) = baseline_path {
        let content = std::fs::read_to_string(path)?;
        let parser = sruja_language::Parser::new(path.to_string_lossy().to_string());
        let program = parser.parse(&content).map_err(|diags| {
            CliError::parse_with_diagnostics(path.to_string_lossy().to_string(), diags)
        })?;

        let scan_graph = sruja_diff::program_to_graph(&program);
        let mut kg = sruja_graph::KnowledgeGraph::new();
        sruja_graph::scan_merge::merge_scan_into_graph(
            &mut kg,
            &scan_graph,
            &repo_path.display().to_string(),
        );
        sruja_graph::scan_merge::merge_program_into_graph(
            &mut kg,
            &program,
            &repo_path.display().to_string(),
        );
        kg
    } else {
        graph_store::load_or_build_graph(repo_path)?
    };

    println!(
        "🎨 Generating interactive context graph for {}...",
        kg.metadata.name
    );

    let exporter = HtmlExporter::new();
    let html = exporter.export(&kg).map_err(|e| {
        CliError::Io(std::io::Error::other(format!(
            "Failed to generate HTML: {}",
            e
        )))
    })?;

    std::fs::write(output_path, html)?;
    println!("✅ Visualization saved to: {}", output_path);

    if open {
        let abs_path = std::fs::canonicalize(output_path)?;
        let url = format!("file://{}", abs_path.display());
        if let Err(e) = opener::open(url) {
            eprintln!("⚠️ Failed to open browser: {}", e);
        }
    }

    Ok(())
}
