//! Initialize Sruja in a repository: create .sruja/, run quickstart, optionally generate prompt.

use std::fs;
use std::path::Path;

use super::generate::generate_prompt;
use super::scan::quickstart;
use super::CliError;
use crate::utils::architecture_path;

/// Initialize Sruja in the given repo: ensure `.sruja/`, run quickstart, optionally generate prompt or auto-onboard.
pub async fn init(
    repo_root: &str,
    generate_prompt_file: bool,
    auto: bool,
    force: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let dot_sruja = repo_path.join(".sruja");
    if !dot_sruja.exists() {
        fs::create_dir_all(&dot_sruja).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create {}: {}", dot_sruja.display(), e),
            ))
        })?;
        eprintln!("Created {}", dot_sruja.display());
    }

    if auto {
        eprintln!("✨ Running AI discovery to generate baseline...");
        let graph = sruja_scan::scan_repo(repo_path)?;
        let baseline = super::scan::write_draft_baseline(repo_path, &graph, force)?;

        if let Some(path) = baseline {
            eprintln!("✅ Generated baseline: {}", path.display());
            eprintln!();
            eprintln!("Next steps:");
            eprintln!(
                "  1. Review: Use 'sruja lint {}' to check the architecture.",
                path.display()
            );
            eprintln!(
                "  2. View: Use 'sruja export mermaid {} --all-views' to visualize.",
                path.display()
            );
            eprintln!("  3. Monitor: Run 'sruja watch' while you code.");
            return Ok(());
        } else {
            eprintln!("⚠️ Baseline already exists. Use --force to overwrite.");
        }
    }

    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    if let Some(ref p) = baseline_path {
        eprintln!("Existing architecture file: {}", p.display());
    } else {
        eprintln!(
            "No architecture file yet (looked for repo.sruja, architecture.sruja, docs/architecture.sruja)."
        );
    }

    eprintln!();
    quickstart(repo_root, "text", false, None).await?;

    if generate_prompt_file {
        let prompt_path = dot_sruja.join("init_prompt.txt");
        let repo_roots = vec![repo_root.to_string()];
        generate_prompt(&repo_roots, None, Some(&prompt_path.to_string_lossy()))?;
        eprintln!();
        eprintln!("Next steps:");
        eprintln!(
            "  1. Use the sruja-architecture skill with the prompt in {}",
            prompt_path.display()
        );
        eprintln!(
            "  2. Save the model output as repo.sruja (or architecture.sruja) in the repo root."
        );
        eprintln!("  3. Run: sruja lint repo.sruja");
        eprintln!("  4. Day-to-day, run: sruja daily -r {}", repo_root);
    } else if baseline_path.is_none() {
        eprintln!();
        eprintln!("Next steps:");
        eprintln!("  Run: sruja init -a --force to automatically generate a baseline from code.",);
        eprintln!("  Or create repo.sruja manually and run: sruja lint repo.sruja");
    } else {
        eprintln!();
        eprintln!("Daily loop:");
        eprintln!("  sruja daily -r {}", repo_root);
        eprintln!("  sruja watch -r {}", repo_root);
    }

    Ok(())
}
