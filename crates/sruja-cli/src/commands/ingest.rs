//! Ingest command: bring external context into Sruja's context graph.
//!
//! Supports:
//! - Local files (copy to .sruja/context/)
//! - URLs (fetch and save)
//! - Directories (recursively copy relevant files)
//!
//! External context files are stored in `.sruja/context/` and automatically
//! surfaced by `sruja focus` and `sruja context-score`.

use std::path::Path;

use crate::commands::CliError;
use crate::utils::colors;

/// Supported file extensions for external context.
const SUPPORTED_EXTENSIONS: &[&str] = &["md", "yaml", "yml", "json", "txt", "toml"];

pub async fn ingest(
    repo_root: &str,
    sources: &[String],
    category: Option<&str>,
    elements: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let context_dir = repo_path.join(".sruja").join("context");

    // Ensure context directory exists
    std::fs::create_dir_all(&context_dir)?;

    if sources.is_empty() {
        // Show current context inventory
        return show_context_inventory(&context_dir);
    }

    let mut ingested_count = 0;

    for source in sources {
        let source_path = Path::new(source);

        if source_path.exists() {
            if source_path.is_file() {
                let dest = ingest_file(source_path, &context_dir, category, elements)?;
                eprintln!(
                    "  {} Ingested: {} → {}",
                    colors::success("✓"),
                    source,
                    dest.display()
                );
                ingested_count += 1;
            } else if source_path.is_dir() {
                let count = ingest_directory(source_path, &context_dir, category, elements)?;
                eprintln!(
                    "  {} Ingested {} files from {}",
                    colors::success("✓"),
                    count,
                    source
                );
                ingested_count += count;
            }
        } else {
            eprintln!(
                "  {} Skipped (not found): {}",
                colors::warning("⚠"),
                source
            );
        }
    }

    eprintln!();
    if ingested_count > 0 {
        eprintln!(
            "{}",
            colors::success(&format!(
                "✓ Ingested {} file{} into .sruja/context/",
                ingested_count,
                if ingested_count == 1 { "" } else { "s" }
            ))
        );
        eprintln!(
            "  These will be surfaced by '{}' and improve your '{}'.",
            colors::info("sruja focus"),
            colors::info("sruja context-score"),
        );
    } else {
        eprintln!(
            "{}",
            colors::warning("No files ingested. Provide paths to files or directories.")
        );
    }

    Ok(())
}

/// Copy a single file to the context directory, optionally adding front-matter.
fn ingest_file(
    source: &Path,
    context_dir: &Path,
    category: Option<&str>,
    elements: Option<&str>,
) -> Result<std::path::PathBuf, CliError> {
    let filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| CliError::validation("Invalid filename".to_string()))?;

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(CliError::validation(format!(
            "Unsupported file type '.{}'. Supported: {}",
            ext,
            SUPPORTED_EXTENSIONS.join(", ")
        )));
    }

    let dest = context_dir.join(filename);
    let content = std::fs::read_to_string(source)?;

    // Add front-matter if category or elements specified and file doesn't have it
    let final_content = if (category.is_some() || elements.is_some()) && !content.starts_with("---")
    {
        let mut front_matter = "---\n".to_string();
        if let Some(cat) = category {
            front_matter.push_str(&format!("category: {}\n", cat));
        }
        if let Some(elems) = elements {
            let elem_list: Vec<&str> = elems.split(',').map(|s| s.trim()).collect();
            front_matter.push_str(&format!("elements: [{}]\n", elem_list.join(", ")));
        }
        front_matter.push_str("---\n");
        format!("{}{}", front_matter, content)
    } else {
        content
    };

    std::fs::write(&dest, final_content)?;
    Ok(dest)
}

/// Recursively ingest files from a directory.
fn ingest_directory(
    source_dir: &Path,
    context_dir: &Path,
    category: Option<&str>,
    elements: Option<&str>,
) -> Result<usize, CliError> {
    let mut count = 0;

    if let Ok(entries) = std::fs::read_dir(source_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                    match ingest_file(&path, context_dir, category, elements) {
                        Ok(_) => count += 1,
                        Err(e) => eprintln!(
                            "  {} Skipped {}: {}",
                            colors::warning("⚠"),
                            path.display(),
                            e
                        ),
                    }
                }
            }
        }
    }

    Ok(count)
}

/// Show the current inventory of external context files.
fn show_context_inventory(context_dir: &Path) -> Result<(), CliError> {
    if !context_dir.exists() {
        colors::print_header("📂 External Context (.sruja/context/)");
        println!();
        println!(
            "  {} No external context directory found.",
            colors::dim("ℹ")
        );
        println!();
        println!("  Add external context to improve your AI agent's knowledge:");
        println!();
        println!(
            "  {}",
            colors::info("# Ingest ADRs, design docs, API contracts:")
        );
        println!("  sruja ingest docs/adr/");
        println!("  sruja ingest design-doc.md --category design-doc");
        println!("  sruja ingest openapi.yaml --elements Api.Gateway");
        println!();
        println!(
            "  {}",
            colors::info("# Or manually create .sruja/context/ and add files:")
        );
        println!("  mkdir -p .sruja/context/");
        println!("  cp docs/adr/*.md .sruja/context/");
        println!();
        return Ok(());
    }

    colors::print_header("📂 External Context Inventory");
    println!();

    let summary = sruja_graph::scan_external_context(
        context_dir.parent().and_then(|p| p.parent()).unwrap_or(Path::new(".")),
    );

    if summary.file_count == 0 {
        println!(
            "  {} Directory exists but is empty.",
            colors::dim("ℹ")
        );
        println!("  Run 'sruja ingest <path>' to add context files.");
        return Ok(());
    }

    println!(
        "  Files:      {}",
        colors::success(&summary.file_count.to_string())
    );
    println!(
        "  Categories: {}",
        if summary.categories.is_empty() {
            colors::dim("none").to_string()
        } else {
            summary.categories.join(", ")
        }
    );
    println!(
        "  Linked:     {} files linked to architecture elements",
        summary.linked_elements
    );
    println!(
        "  Words:      ~{}",
        summary.total_words
    );
    println!();

    // List files
    if let Ok(entries) = std::fs::read_dir(context_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");
                let size = std::fs::metadata(&path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                let size_str = if size < 1024 {
                    format!("{}B", size)
                } else {
                    format!("{:.1}KB", size as f64 / 1024.0)
                };
                println!("  📄 {:<40} {}", name, colors::dim(&size_str));
            }
        }
    }

    println!();
    Ok(())
}
