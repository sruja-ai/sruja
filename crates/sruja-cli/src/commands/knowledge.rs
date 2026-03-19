//! Component knowledge commands: list, show, gaps.

use std::io;
use std::path::Path;

use sruja_language::{collect_elements, Parser};

use crate::commands::error::CliError;
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
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Architecture file not found: {}", p.display()),
        )
        .into());
    }
    resolve_architecture_path(repo_path).ok_or_else(|| {
        CliError::Validation(
            "No architecture file found (repo.sruja, architecture.sruja, or docs/architecture.sruja). Specify -a <file>.".to_string(),
        )
    })
}

pub async fn knowledge(cmd: crate::KnowledgeCommand) -> Result<(), CliError> {
    use crate::KnowledgeCommand as KC;
    match cmd {
        KC::List { repo, architecture } => {
            let baseline = resolve_baseline(&repo, architecture.as_deref())?;
            let content = std::fs::read_to_string(&baseline)?;
            let parser = Parser::new(baseline.display().to_string());
            let program = parser.parse(&content).map_err(|diags| CliError::Parse {
                file: baseline.display().to_string(),
                message: diags
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; "),
                diagnostics: diags,
            })?;
            let (elements, _) = collect_elements(&program);
            let repo_path = Path::new(&repo)
                .canonicalize()
                .unwrap_or_else(|_| Path::new(&repo).to_path_buf());
            println!("Elements with doc link:");
            for (fqn, elem) in &elements {
                if let Some(ref body) = elem.assignment.body {
                    if let Some(ref doc_path) = body.doc {
                        let full = repo_path.join(doc_path);
                        let exists = full.exists();
                        println!(
                            "  {}  {}  {}",
                            fqn,
                            doc_path,
                            if exists { "(exists)" } else { "(missing)" }
                        );
                    }
                }
            }
        }
        KC::Show {
            element_id,
            repo,
            architecture,
        } => {
            let baseline = resolve_baseline(&repo, architecture.as_deref())?;
            let content = std::fs::read_to_string(&baseline)?;
            let parser = Parser::new(baseline.display().to_string());
            let program = parser.parse(&content).map_err(|diags| CliError::Parse {
                file: baseline.display().to_string(),
                message: diags
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; "),
                diagnostics: diags,
            })?;
            let (elements, _) = collect_elements(&program);
            let elem = elements.get(&element_id).or_else(|| {
                elements
                    .iter()
                    .find(|(fqn, _)| {
                        fqn.as_str() == element_id || fqn.ends_with(&format!(".{}", element_id))
                    })
                    .map(|(_, e)| e)
            });
            let doc_path = match elem
                .and_then(|e| e.assignment.body.as_ref().and_then(|b| b.doc.clone()))
            {
                Some(p) => p,
                None => {
                    eprintln!("Element '{}' has no doc link. Add doc \".sruja/knowledge/...\" to the element.", element_id);
                    std::process::exit(1);
                }
            };
            let repo_path = Path::new(&repo)
                .canonicalize()
                .unwrap_or_else(|_| Path::new(&repo).to_path_buf());
            let full = repo_path.join(&doc_path);
            if !full.exists() {
                eprintln!("Knowledge file not found: {}", full.display());
                std::process::exit(1);
            }
            let text = std::fs::read_to_string(&full)?;
            print!("{}", text);
        }
        KC::Gaps { repo, architecture } => {
            let baseline = resolve_baseline(&repo, architecture.as_deref())?;
            let content = std::fs::read_to_string(&baseline)?;
            let parser = Parser::new(baseline.display().to_string());
            let program = parser.parse(&content).map_err(|diags| CliError::Parse {
                file: baseline.display().to_string(),
                message: diags
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; "),
                diagnostics: diags,
            })?;
            let (elements, _) = collect_elements(&program);
            let gaps: Vec<_> = elements
                .iter()
                .filter(|(_, elem)| {
                    elem.assignment
                        .body
                        .as_ref()
                        .and_then(|b| b.doc.as_ref())
                        .is_none()
                })
                .map(|(fqn, _)| fqn.as_str())
                .collect();
            println!("Elements without doc link ({}):", gaps.len());
            for fqn in &gaps {
                println!("  {}", fqn);
            }
        }
    }
    Ok(())
}
