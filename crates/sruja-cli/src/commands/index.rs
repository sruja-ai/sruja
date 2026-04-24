use crate::commands::CliError;
use sruja_export::vector::SemanticSearcher;
use sruja_extract::ExtractionEngine;
use std::fs;
use std::path::Path;

/// Rename of the original index command for semantic search
pub async fn semantic_index(
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

/// New command for Architecture Index (Registry)
pub async fn registry_index(
    repo_path: &str,
    architecture_file: Option<&str>,
    fix: bool,
    format: &str,
) -> Result<(), CliError> {
    let repo_root = Path::new(repo_path);
    
    // 1. Parse architecture file if provided
    let arch_file = architecture_file.unwrap_or("repo.sruja");
    let program = if Path::new(arch_file).exists() {
        let (_, p) = crate::commands::parse_sruja_file(arch_file)?;
        Some(p)
    } else {
        None
    };

    println!("🔍 Discovering architectural artifacts in {}...", repo_path);
    let engine = ExtractionEngine::new();
    let discovered = engine.discover_all(repo_root);

    if discovered.is_empty() {
        println!("∅ No artifacts discovered.");
        return Ok(());
    }

    if format == "json" {
        let json = serde_json::to_string_pretty(&discovered)?;
        println!("{}", json);
    } else {
        println!("✅ Discovered {} artifacts:", discovered.len());
        for d in &discovered {
            println!("  [{}] {} (suggested: {})", 
                d.binding.kind.as_str(), 
                d.binding.path, 
                d.suggested_element.as_deref().unwrap_or("unknown")
            );
        }
    }

    if fix {
        if let Some(mut program) = program {
            println!("🛠️  Updating {} with discovered sources...", arch_file);
            let updated = apply_discovered_sources(&mut program, discovered);
            if updated > 0 {
                let printer = sruja_export::dsl::DslPrinter::new();
                let content = printer.print(&program);
                fs::write(arch_file, content)?;
                println!("✅ Updated {} (added {} sources).", arch_file, updated);
            } else {
                println!("No updates needed.");
            }
        } else {
            return Err(CliError::Discovery(format!("Architecture file {} not found. Cannot --fix.", arch_file)));
        }
    }

    Ok(())
}

fn apply_discovered_sources(program: &mut sruja_language::ast::Program, discovered: Vec<sruja_extract::DiscoveredSource>) -> usize {
    let mut updated_count = 0;

    // Build a map of elements for easy lookup
    // For now, we only look at top-level elements or direct children of systems
    for item in &mut program.items {
        if let sruja_language::ast::TopLevelItem::ElementDef(elem) = item {
            updated_count += update_element(elem, &discovered);
        }
    }

    updated_count
}

fn update_element(elem: &mut sruja_language::ast::ElementDef, discovered: &[sruja_extract::DiscoveredSource]) -> usize {
    let mut added = 0;
    let elem_name = &elem.assignment.name;
    
    for d in discovered {
        if let Some(suggested) = &d.suggested_element {
            // Fuzzy match suggested name with element name or title
            let title = elem.assignment.title.as_ref().unwrap_or(elem_name);
            if suggested.to_lowercase() == elem_name.to_lowercase() || suggested.to_lowercase() == title.to_lowercase() {
                // Check if already has this source
                let has_source = elem.assignment.body.as_ref().map(|b| {
                    b.sources.iter().any(|s| s.path == d.binding.path)
                }).unwrap_or(false);

                if !has_source {
                    if elem.assignment.body.is_none() {
                        elem.assignment.body = Some(sruja_language::ast::ElementDefBody::default());
                    }
                    if let Some(body) = &mut elem.assignment.body {
                        body.sources.push(d.binding.clone());
                        added += 1;
                    }
                }
            }
        }
    }

    // Recurse into nested elements
    if let Some(body) = &mut elem.assignment.body {
        for item in &mut body.items {
            if let sruja_language::ast::ElementDefBodyItem::ElementDef(nested) = item {
                added += update_element(nested, discovered);
            }
        }
    }

    added
}

/// Query the architectural registry for elements matching a criteria
pub async fn query_registry(
    repo_path: &str,
    architecture_file: Option<&str>,
    query: &str,
    format: &str,
) -> Result<(), CliError> {
    let root = Path::new(repo_path);
    let arch_file = architecture_file.unwrap_or("repo.sruja");
    let query_lower = query.to_lowercase();
    
    // 1. Search Local Registry (DSL)
    let mut local_found = Vec::new();
    if Path::new(arch_file).exists() {
        let (_, program) = crate::commands::parse_sruja_file(arch_file)?;
        for item in &program.items {
            if let sruja_language::ast::TopLevelItem::ElementDef(elem) = item {
                search_elements(elem, &query_lower, &mut local_found);
            }
        }
    }

    // 2. Search Federated Index (Global)
    let mut global_found = Vec::new();
    if let Some(index_path) = crate::commands::federation::find_system_index(root) {
        if let Ok(index) = crate::commands::federation::load_system_index(&index_path) {
            for node in &index.nodes {
                if node.label.to_lowercase().contains(&query_lower) || 
                   node.local_id.to_lowercase().contains(&query_lower) ||
                   node.repo_id.to_lowercase().contains(&query_lower) 
                {
                    global_found.push(node.clone());
                }
            }
        }
    }

    if local_found.is_empty() && global_found.is_empty() {
        println!("No elements found matching '{}'.", query);
        return Ok(());
    }

    if format == "json" {
        let result = serde_json::json!({
            "local": local_found,
            "global": global_found,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if !local_found.is_empty() {
            println!("🔍 Found {} local elements matching '{}':", local_found.len(), query);
            for name in &local_found {
                println!("  • {}", name);
            }
        }
        
        if !global_found.is_empty() {
            if !local_found.is_empty() { println!(); }
            println!("🌐 Found {} global elements in federated index:", global_found.len());
            for node in &global_found {
                println!("  • {} ({}) - Repo: {}", node.label, node.kind, node.repo_id);
                if let Some(owner) = &node.owner {
                    println!("    Owner: {}", owner);
                }
            }
        }
    }

    Ok(())
}

fn search_elements(elem: &sruja_language::ast::ElementDef, query: &str, found: &mut Vec<String>) {
    let name = &elem.assignment.name;
    let title = elem.assignment.title.as_ref().unwrap_or(name);

    if name.to_lowercase().contains(query) || title.to_lowercase().contains(query) {
        found.push(name.clone());
    }

    if let Some(body) = &elem.assignment.body {
        for item in &body.items {
            if let sruja_language::ast::ElementDefBodyItem::ElementDef(nested) = item {
                search_elements(nested, query, found);
            }
        }
    }
}

/// Generate a visual dashboard for the architectural registry
pub async fn registry_dashboard(
    repo_path: &str,
    output_path: &str,
) -> Result<(), CliError> {
    let root = Path::new(repo_path);
    let mut md = String::new();
    
    md.push_str("# Sruja Architecture Registry Dashboard\n\n");
    
    // 1. Try to find federated index
    if let Some(index_path) = crate::commands::federation::find_system_index(root) {
        if let Ok(index) = crate::commands::federation::load_system_index(&index_path) {
            md.push_str("## 🌐 Federated Landscape\n\n");
            md.push_str(&format!("- **Total Repositories**: {}\n", index.repos.len()));
            md.push_str(&format!("- **Total Elements**: {}\n", index.nodes.len()));
            md.push_str(&format!("- **Total Relationships**: {}\n\n", index.edges.len()));
            
            md.push_str("### Repositories\n\n");
            md.push_str("| Repo ID | Status | Last Commit |\n|---|---|---|\n");
            for repo in &index.repos {
                md.push_str(&format!("| `{}` | {} | {} |\n", 
                    repo.repo_id, 
                    repo.truth_status, 
                    repo.git_commit.as_deref().unwrap_or("-")
                ));
            }
            md.push_str("\n");

            if !index.conflicts.is_empty() {
                md.push_str("### ⚠ Conflicts\n\n");
                for c in &index.conflicts {
                    md.push_str(&format!("- **{}**: {} (involved: {})\n", 
                        c.key, c.message, c.repos.join(", ")
                    ));
                }
                md.push_str("\n");
            }

            md.push_str("### High-Level Elements\n\n");
            md.push_str("| Element | Kind | Repo | Owner |\n|---|---|---|---|\n");
            for node in &index.nodes {
                if node.kind != "module" {
                    md.push_str(&format!("| {} | {} | `{}` | {} |\n", 
                        node.label, 
                        node.kind, 
                        node.repo_id,
                        node.owner.as_deref().unwrap_or("-")
                    ));
                }
            }
        }
    } else {
        md.push_str("## 📍 Local Registry\n\n");
        let arch_file = "repo.sruja";
        if Path::new(arch_file).exists() {
            let (_, program) = crate::commands::parse_sruja_file(arch_file)?;
            let (elements, _) = sruja_language::collect_elements(&program);
            md.push_str(&format!("- **Total Elements**: {}\n\n", elements.len()));
            
            md.push_str("| Element | Kind | Title |\n|---|---|---|\n");
            for (fqn, elem) in elements {
                md.push_str(&format!("| `{}` | {} | {} |\n", 
                    fqn, 
                    elem.assignment.kind, 
                    elem.assignment.title.as_deref().unwrap_or("-")
                ));
            }
        } else {
            md.push_str("No registry found. Run `sruja index registry` to get started.\n");
        }
    }

    fs::write(output_path, md)?;
    println!("✅ Dashboard generated: {}", output_path);

    Ok(())
}
