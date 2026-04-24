//! Critique command: adversarial architectural review of changes.

use std::path::Path;
use sruja_intent::{CritiqueEngine, CritiqueRequest, format_critique_text, format_critique_json, CritiqueSeverity};
use crate::commands::{scan_repo_cached, CliError};
use crate::utils::architecture_path;

pub async fn critique(
    repo_root: &str,
    mut files: Vec<String>,
    description: Option<String>,
    proposal_id: Option<String>,
    base: Option<String>,
    head: Option<String>,
    staged: bool,
    format: &str,
    fail_on: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    
    // 1. Resolve files from git if requested
    if staged {
        let output = std::process::Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| CliError::Io(e))?;
        
        let git_files = String::from_utf8_lossy(&output.stdout);
        for f in git_files.lines() {
            if !f.is_empty() {
                files.push(f.to_string());
            }
        }
    }

    if files.is_empty() && base.is_none() {
        println!("No files provided for critique. Use --files, --staged, or --base/--head.");
        return Ok(());
    }

    // 2. Load context
    let graph = scan_repo_cached(repo_path)?;
    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let program = if let Some(path) = baseline_path {
        let content = std::fs::read_to_string(path)?;
        let parser = sruja_language::Parser::new(repo_root);
        parser.parse(&content).ok()
    } else {
        None
    };

    // 3. Run critique
    let engine = CritiqueEngine::new(graph, program);
    let request = CritiqueRequest {
        changed_files: files,
        description,
        proposal_id,
        base_ref: base,
        head_ref: head,
    };

    let report = engine.critique(&request);

    // 4. Output
    match format {
        "json" => println!("{}", format_critique_json(&report)),
        _ => println!("{}", format_critique_text(&report)),
    }

    // 5. Exit logic
    if let Some(level) = fail_on {
        let threshold = match level.to_lowercase().as_str() {
            "critical" => CritiqueSeverity::Critical,
            "high" => CritiqueSeverity::High,
            "medium" => CritiqueSeverity::Medium,
            "low" => CritiqueSeverity::Low,
            _ => return Err(CliError::validation(format!("Invalid fail-on level: {}", level))),
        };

        let has_violation = report.findings.iter().any(|f| f.severity >= threshold);
        if has_violation {
            std::process::exit(1);
        }
    }

    Ok(())
}
