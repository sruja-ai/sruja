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
    hook: bool,
    ci: bool,
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

    if hook {
        install_pre_commit_hook(repo_path)?;
    }

    if ci {
        install_github_actions_workflow(repo_path)?;
    }

    Ok(())
}

fn install_github_actions_workflow(repo_path: &Path) -> Result<(), CliError> {
    let workflows_dir = repo_path.join(".github").join("workflows");
    if !workflows_dir.exists() {
        fs::create_dir_all(&workflows_dir).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create workflows directory at {}: {}", workflows_dir.display(), e),
            ))
        })?;
    }

    let workflow_path = workflows_dir.join("sruja-check.yml");
    
    let workflow_content = r#"name: Sruja Architecture Check

on:
  pull_request:
    branches: [ "main", "master" ]
  push:
    branches: [ "main", "master" ]

jobs:
  sruja-check:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Sruja CLI
        run: |
          # Download the latest Sruja release
          curl -sL https://github.com/sruja-ai/sruja/releases/latest/download/sruja-linux-amd64.tar.gz | tar xz
          sudo mv sruja /usr/local/bin/

      - name: Run Sruja Check
        run: sruja check --format github-actions
"#;

    fs::write(&workflow_path, workflow_content).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write GitHub Actions workflow to {}: {}", workflow_path.display(), e),
        ))
    })?;

    eprintln!("✅ Installed Sruja GitHub Actions workflow at {}", workflow_path.display());
    Ok(())
}

fn install_pre_commit_hook(repo_path: &Path) -> Result<(), CliError> {
    let hooks_dir = repo_path.join(".git").join("hooks");
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create hooks directory at {}: {}", hooks_dir.display(), e),
            ))
        })?;
    }

    let pre_commit_path = hooks_dir.join("pre-commit");
    
    let hook_script = r#"#!/bin/bash
# Sruja Pre-commit Hook
set -e

# Find staged .sruja files
STAGED_SRUJA=$(git diff --cached --name-only --diff-filter=ACMR | grep '\.sruja$' || true)

if [ -n "$STAGED_SRUJA" ]; then
    echo "🔍 Linting Sruja files..."
    for file in $STAGED_SRUJA; do
        if command -v sruja &> /dev/null; then
            sruja lint "$file"
        else
            echo "⚠️  'sruja' command not found in PATH. Skipping DSL linting."
        fi
    done
fi
"#;

    fs::write(&pre_commit_path, hook_script).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write pre-commit hook to {}: {}", pre_commit_path.display(), e),
        ))
    })?;

    // Make the hook executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&pre_commit_path)
            .map_err(|e| CliError::Io(e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_commit_path, perms).map_err(|e| CliError::Io(e))?;
    }

    eprintln!("✅ Installed Sruja pre-commit hook at {}", pre_commit_path.display());
    Ok(())
}
