use std::path::Path;

use sruja_export::json::Exporter as JsonExporter;
use sruja_export::markdown::{MarkdownExporter, MarkdownOptions};
use sruja_export::mermaid::exporter::{MermaidConfig, MermaidExporter};
use sruja_export::{GraphMLExporter, Neo4jExporter, ObsidianExporter};

use crate::commands::CliError;

pub async fn export(
    format: &str,
    file: &str,
    from_scan: bool,
    repo: Option<&str>,
    output_dir: Option<&str>,
) -> Result<(), CliError> {
    if from_scan {
        let repo_root = repo.unwrap_or(".");
        let graph = sruja_scan::scan_repo(Path::new(repo_root))
            .map_err(|e| CliError::scan(e.to_string()))?;
        match format {
            "graphml" => {
                println!("{}", GraphMLExporter::export(&graph));
                Ok(())
            }
            "neo4j" => {
                println!("{}", Neo4jExporter::export(&graph));
                Ok(())
            }
            "obsidian" => {
                let out_dir = output_dir.ok_or_else(|| {
                    CliError::validation(
                        "Missing --output-dir for obsidian export (use with --from-scan)."
                            .to_string(),
                    )
                })?;
                ObsidianExporter::export(&graph, Path::new(out_dir)).map_err(CliError::Io)?;
                Ok(())
            }
            other => Err(CliError::validation(format!(
                "Unsupported scan export format: {}. Supported: graphml, neo4j, obsidian",
                other
            ))),
        }
    } else {
        let path = Path::new(file);
        if !path.exists() {
            return Err(CliError::validation(format!(
                "Architecture file '{}' does not exist.",
                file
            )));
        }
        let (_content, program) = crate::commands::parse_sruja_file(file)?;
        match format {
            "json" => {
                let out = JsonExporter::new().export(&program)?;
                println!("{}", out);
                Ok(())
            }
            "markdown" => {
                let out = MarkdownExporter::new(MarkdownOptions::default()).export(&program);
                println!("{}", out);
                Ok(())
            }
            "mermaid" => {
                let out = MermaidExporter::new(MermaidConfig {
                    direction: "LR".to_string(),
                    view_level: 1,
                    target_id: None,
                })
                .export(&program);
                println!("{}", out);
                Ok(())
            }
            other => Err(CliError::validation(format!(
                "Unsupported export format: {}. Supported: json, markdown, mermaid (or use --from-scan for graphml, neo4j, obsidian)",
                other
            ))),
        }
    }
}
