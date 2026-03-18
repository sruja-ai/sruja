//! Sources command: list source bindings for architecture elements.

use std::io;
use std::path::Path;

use sruja_language::{collect_elements, Parser};

use super::CliError;
use crate::utils::architecture_path::resolve_architecture_path;

fn resolve_baseline(
    repo: &str,
    architecture: Option<&str>,
) -> Result<std::path::PathBuf, CliError> {
    let repo_path = Path::new(repo);
    if let Some(a) = architecture {
        let p = repo_path.join(a);
        if p.exists() {
            return Ok(p);
        }
        return Err(
            io::Error::new(io::ErrorKind::NotFound, format!("Architecture file not found: {}", p.display()))
                .into(),
        );
    }
    resolve_architecture_path(repo_path).ok_or_else(|| {
        CliError::Validation(
            "No architecture file found (repo.sruja, architecture.sruja, or docs/architecture.sruja). Specify -a <file>.".to_string(),
        )
    })
}

pub async fn sources(
    repo: &str,
    architecture: Option<&str>,
    element_id: Option<&str>,
    source_type: Option<&str>,
    validate: bool,
    format: &str,
) -> Result<(), CliError> {
    let baseline = resolve_baseline(repo, architecture)?;
    let content = std::fs::read_to_string(&baseline)?;
    let parser = Parser::new(baseline.display().to_string());
    let program = parser.parse(&content).map_err(|diags| CliError::Parse {
        file: baseline.display().to_string(),
        message: diags
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("; "),
    })?;

    let (elements, _) = collect_elements(&program);
    let repo_path = Path::new(repo)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(repo).to_path_buf());

    if let Some(elem_id) = element_id {
        let elem = elements.get(elem_id).or_else(|| {
            elements
                .iter()
                .find(|(fqn, _)| {
                    fqn.as_str() == elem_id || fqn.ends_with(&format!(".{}", elem_id))
                })
                .map(|(_, e)| e)
        });

        let elem = match elem {
            Some(e) => e,
            None => {
                return Err(CliError::Validation(format!(
                    "Element '{}' not found",
                    elem_id
                )));
            }
        };

        let body = elem.assignment.body.as_ref();
        let sources: Vec<_> = body
            .map(|b| {
                b.sources
                    .iter()
                    .filter(|s| {
                        source_type
                            .map(|t| s.kind.as_str() == t || s.kind.as_str().to_lowercase() == t.to_lowercase())
                            .unwrap_or(true)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if sources.is_empty() {
            println!("Element '{}' has no source bindings.", elem_id);
            return Ok(());
        }

        match format {
            "json" => {
                let output: Vec<serde_json::Value> = sources
                    .iter()
                    .map(|s| {
                        let full_path = repo_path.join(&s.path);
                        serde_json::json!({
                            "type": s.kind.as_str(),
                            "path": s.path,
                            "exists": full_path.exists()
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            _ => {
                for s in &sources {
                    let full_path = repo_path.join(&s.path);
                    let status = if validate {
                        if full_path.exists() {
                            "✓"
                        } else {
                            "✗"
                        }
                    } else {
                        ""
                    };
                    println!(
                        "  {}: {} {}",
                        s.kind.as_str(),
                        s.path,
                        status
                    );
                }
            }
        }
    } else {
        let mut all_sources: Vec<(String, &sruja_language::SourceBinding)> = Vec::new();

        for (fqn, elem) in &elements {
            if let Some(body) = elem.assignment.body.as_ref() {
                for s in &body.sources {
                    if source_type
                        .map(|t| s.kind.as_str() == t || s.kind.as_str().to_lowercase() == t.to_lowercase())
                        .unwrap_or(true)
                    {
                        all_sources.push((fqn.clone(), s));
                    }
                }
            }
        }

        if all_sources.is_empty() {
            println!("No source bindings found in architecture.");
            return Ok(());
        }

        match format {
            "json" => {
                let output: Vec<serde_json::Value> = all_sources
                    .iter()
                    .map(|(fqn, s)| {
                        let full_path = repo_path.join(&s.path);
                        serde_json::json!({
                            "element": fqn,
                            "type": s.kind.as_str(),
                            "path": s.path,
                            "exists": full_path.exists()
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            _ => {
                if validate {
                    let mut valid_count = 0;
                    let mut invalid_count = 0;

                    for (fqn, s) in &all_sources {
                        let full_path = repo_path.join(&s.path);
                        if full_path.exists() {
                            valid_count += 1;
                        } else {
                            invalid_count += 1;
                            println!("✗ {}: {} ({})", fqn, s.kind.as_str(), s.path);
                        }
                    }

                    if invalid_count == 0 {
                        println!("✓ All {} source paths valid", all_sources.len());
                    } else {
                        println!(
                            "\nSummary: {} valid, {} invalid",
                            valid_count, invalid_count
                        );
                    }
                } else {
                    println!("Source bindings ({}):", all_sources.len());
                    println!();
                    for (fqn, s) in &all_sources {
                        println!("  {}:", fqn);
                        println!("    {}: {}", s.kind.as_str(), s.path);
                    }
                }
            }
        }
    }

    Ok(())
}
