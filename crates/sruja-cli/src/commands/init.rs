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
    dry_run: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    use crate::utils::{colors, progress};
    use dialoguer::{Confirm, MultiSelect};

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let is_interactive = !auto && !generate_prompt_file && !ci && !hook && !force && !dry_run;

    if is_interactive {
        colors::print_header("🚀 Sruja - Repository Initialization");
        println!("This will set up Sruja in your repository for architecture-as-code.");
        println!();
    }

    if dry_run {
        println!(
            "{}",
            colors::warning("DRY RUN MODE: No files will be written.")
        );
        println!();
    }

    let dot_sruja = repo_path.join(".sruja");
    if !dot_sruja.exists() {
        if !dry_run {
            fs::create_dir_all(&dot_sruja).map_err(|e| {
                CliError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to create {}: {}", dot_sruja.display(), e),
                ))
            })?;
        }
        if is_interactive || dry_run {
            println!(
                "  {} [1/4] Created {}",
                colors::success("✓"),
                colors::dim(".sruja/")
            );
        }
    }

    // Project detection
    let project_type = detect_project_type(repo_path);
    if is_interactive || dry_run {
        println!(
            "  {} [2/4] Detected project: {}",
            colors::success("✓"),
            colors::info(&project_type)
        );
    }

    // .srujaignore generation
    let srujaignore_path = repo_path.join(".srujaignore");
    if !srujaignore_path.exists() || force {
        let ignore_content = generate_srujaignore(&project_type);
        if !dry_run {
            fs::write(&srujaignore_path, ignore_content)?;
        }
        if is_interactive || dry_run {
            println!(
                "  {} [3/4] Generated {}",
                colors::success("✓"),
                colors::dim(".srujaignore")
            );
        }
    }

    if generate_prompt_file {
        let prompt_path = dot_sruja.join("init_prompt.txt");
        if !prompt_path.exists() || force {
            if !dry_run {
                let repos = vec![repo_root.to_string()];
                let out = prompt_path.to_string_lossy().to_string();
                generate_prompt(&repos, None, Some(&out))?;
            }
            if is_interactive || dry_run {
                println!(
                    "  {} [4/4] Generated {}",
                    colors::success("✓"),
                    colors::dim(".sruja/init_prompt.txt")
                );
            }
        } else if is_interactive || dry_run {
            println!(
                "  {} Skipped {} (already exists; use --force to overwrite)",
                colors::info("i"),
                colors::dim(".sruja/init_prompt.txt")
            );
        }
    }

    let mut should_auto = auto;
    let mut should_ci = ci;
    let mut should_hook = hook;

    if is_interactive {
        println!();
        should_auto = Confirm::new()
            .with_prompt("Generate initial architecture baseline from code?")
            .default(true)
            .interact()
            .unwrap_or(auto);

        let extras = MultiSelect::new()
            .with_prompt("Select additional setup components:")
            .item_checked("GitHub Actions workflow", ci)
            .item_checked("Git pre-commit hook", hook)
            .interact()
            .unwrap_or_default();

        for i in extras {
            if i == 0 {
                should_ci = true;
            }
            if i == 1 {
                should_hook = true;
            }
        }
    }

    if should_auto {
        let pb = progress::spinner("✨ Running AI discovery to generate baseline...");
        let graph_result = sruja_scan::scan_repo(repo_path).map_err(|e| CliError::Scan {
            message: e.to_string(),
            help: Some("Ensure your repo has source files and proper permissions.".into()),
        });

        let graph = match graph_result {
            Ok(g) => g,
            Err(e) => {
                pb.abandon();
                return Err(e);
            }
        };

        let baseline = if !dry_run {
            super::scan::output::write_draft_baseline(repo_path, &graph, force)?
        } else {
            Some(repo_path.join("repo.sruja"))
        };
        pb.finish_and_clear();

        if let Some(path) = baseline {
            if is_interactive || dry_run {
                println!(
                    "  {} Generated baseline: {}",
                    colors::success("✅"),
                    colors::info(path.display().to_string())
                );

                // Show Summary Card
                println!();
                println!("  {}", colors::style("Architecture Summary:").bold());
                println!(
                    "    • Components:   {}",
                    colors::style(graph.nodes.len().to_string()).bold()
                );
                println!(
                    "    • Relations:    {}",
                    colors::style(graph.edges.len().to_string()).bold()
                );
                println!(
                    "    • Entrypoints:  {}",
                    colors::style(
                        graph
                            .nodes
                            .iter()
                            .filter(|n| n.kind == sruja_scan::NodeKind::Service)
                            .count()
                            .to_string()
                    )
                    .bold()
                );

                println!();
                println!("{}", colors::style("Next steps:").bold());
                println!(
                    "  1. {} Use 'sruja lint {}' to check the architecture.",
                    colors::info("Review:"),
                    path.display()
                );
                println!(
                    "  2. {} Use 'sruja export mermaid {} --all-views' to visualize.",
                    colors::info("View:"),
                    path.display()
                );
                println!(
                    "  3. {} Run 'sruja watch' while you code.",
                    colors::info("Monitor:")
                );
            } else {
                eprintln!("✅ Generated baseline: {}", path.display());
            }
            if !is_interactive && !dry_run {
                return Ok(());
            }
        } else if !is_interactive && !dry_run {
            eprintln!("⚠️ Baseline already exists. Use --force to overwrite.");
        }
    }

    if !should_auto {
        let baseline_path = architecture_path::resolve_architecture_path(repo_path);
        let mut scaffolded = false;

        if let Some(ref path) = baseline_path {
            if is_interactive || dry_run {
                println!(
                    "  {} Existing architecture file: {}",
                    colors::info("i"),
                    path.display()
                );
            }
        } else {
            if is_interactive || dry_run {
                println!(
                    "  {} No architecture file found. Creating manual skeleton recommended.",
                    colors::warning("!")
                );
            }

            if is_interactive {
                let scaffold_template = Confirm::new()
                    .with_prompt(
                        "Would you like to scaffold the multi-agent team evolutionary template?",
                    )
                    .default(true)
                    .interact()
                    .unwrap_or(false);

                if scaffold_template {
                    let dest = repo_path.join("repo.sruja");
                    if !dry_run {
                        let blueprint_content =
                            include_str!("../../../../templates/blueprints/agent-team.sruja");
                        fs::write(&dest, blueprint_content)?;

                        // Create mock scripts so evaluate works immediately!
                        let scripts_dir = repo_path.join("scripts");
                        fs::create_dir_all(&scripts_dir).ok();
                        let script_accuracy = scripts_dir.join("evaluate_accuracy.sh");
                        let script_cost = scripts_dir.join("calculate_token_costs.sh");
                        fs::write(
                            &script_accuracy,
                            "#!/bin/sh\necho \"success_rate: 99.5%\"\nexit 0\n",
                        )
                        .ok();
                        fs::write(
                            &script_cost,
                            "#!/bin/sh\necho \"cost_per_job: $0.12\"\nexit 0\n",
                        )
                        .ok();
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(m) = fs::metadata(&script_accuracy) {
                                let mut perms = m.permissions();
                                perms.set_mode(0o755);
                                let _ = fs::set_permissions(&script_accuracy, perms);
                            }
                            if let Ok(m) = fs::metadata(&script_cost) {
                                let mut perms = m.permissions();
                                perms.set_mode(0o755);
                                let _ = fs::set_permissions(&script_cost, perms);
                            }
                        }
                    }
                    println!(
                        "  {} Scaffolded multi-agent team template to {}",
                        colors::success("✓"),
                        colors::info("repo.sruja")
                    );
                    scaffolded = true;
                }
            }
        }

        if !dry_run && !scaffolded {
            quickstart(repo_root, "text", false, None).await?;
        }
    }

    if should_hook && !dry_run {
        install_pre_commit_hook(repo_path)?;
    }

    if should_ci && !dry_run {
        install_github_actions_workflow(repo_path)?;
    }

    if is_interactive || dry_run {
        println!();
        println!(
            "{} Sruja is ready! Try running {} to start monitoring your project.",
            colors::success("🎉"),
            colors::info("sruja watch")
        );
    }

    Ok(())
}

fn detect_project_type(path: &Path) -> String {
    if path.join("Cargo.toml").exists() {
        "Rust".to_string()
    } else if path.join("package.json").exists() {
        "Node.js".to_string()
    } else if path.join("go.mod").exists() {
        "Go".to_string()
    } else if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
        "Python".to_string()
    } else if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
        "Java".to_string()
    } else {
        "Generic".to_string()
    }
}

fn generate_srujaignore(project_type: &str) -> String {
    let mut content = String::from(
        "# Sruja ignore patterns\n# Exclude non-production code from architecture scans\n\n",
    );
    content.push_str("node_modules/\ntarget/\ndist/\nbuild/\n.git/\n.next/\nout/\n\n");
    content.push_str("# Sruja config\n.sruja/\nrepo.sruja.draft\n\n");

    match project_type {
        "Rust" => content.push_str("# Rust specific\ntests/\nbenches/\nexamples/\n"),
        "Node.js" => content.push_str("# Node specific\ncoverage/\n.npm/\nlogs/\n"),
        "Python" => content.push_str("# Python specific\n__pycache__/\n*.pyc\nvenv/\n.venv/\n"),
        "Java" => content.push_str("# Java specific\nbin/\n*.class\n.gradle/\n.metadata/\n"),
        _ => {}
    }

    content
}

fn install_github_actions_workflow(repo_path: &Path) -> Result<(), CliError> {
    let workflows_dir = repo_path.join(".github").join("workflows");
    if !workflows_dir.exists() {
        fs::create_dir_all(&workflows_dir).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to create workflows directory at {}: {}",
                    workflows_dir.display(),
                    e
                ),
            ))
        })?;
    }

    let check_path = workflows_dir.join("sruja-check.yml");
    let onboard_path = workflows_dir.join("sruja-onboard.yml");

    let check_workflow = r#"name: Sruja Check

on:
  pull_request:
    branches: [main, master]
  push:
    branches: [main, master]

permissions:
  contents: read

jobs:
  check:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Sruja CLI
        run: |
          curl -fsSL https://sruja.ai/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH

      - name: Run Sruja check (annotations)
        run: sruja check -r . --format github-actions
"#;

    let onboard_workflow = r#"name: Sruja Onboarding Brief

on:
  pull_request:
    branches: [main, master]
  push:
    branches: [main, master]

permissions:
  contents: read

jobs:
  onboard:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Sruja CLI
        run: |
          curl -fsSL https://sruja.ai/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH

      - name: Generate onboarding brief (job summary)
        run: |
          sruja onboard -r . -o sruja-onboard.md
          cat sruja-onboard.md >> $GITHUB_STEP_SUMMARY

      - name: Emit onboarding annotations (GitHub Actions)
        run: |
          sruja onboard -r . -f github-actions
"#;

    fs::write(&check_path, check_workflow).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!(
                "Failed to write GitHub Actions workflow to {}: {}",
                check_path.display(),
                e
            ),
        ))
    })?;
    fs::write(&onboard_path, onboard_workflow).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!(
                "Failed to write GitHub Actions workflow to {}: {}",
                onboard_path.display(),
                e
            ),
        ))
    })?;

    eprintln!("✅ Installed Sruja GitHub Actions workflows:");
    eprintln!("  - {}", check_path.display());
    eprintln!("  - {}", onboard_path.display());
    Ok(())
}

fn install_pre_commit_hook(repo_path: &Path) -> Result<(), CliError> {
    let hooks_dir = repo_path.join(".git").join("hooks");
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to create hooks directory at {}: {}",
                    hooks_dir.display(),
                    e
                ),
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
            format!(
                "Failed to write pre-commit hook to {}: {}",
                pre_commit_path.display(),
                e
            ),
        ))
    })?;

    // Make the hook executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&pre_commit_path)
            .map_err(CliError::Io)?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_commit_path, perms).map_err(CliError::Io)?;
    }

    eprintln!(
        "✅ Installed Sruja pre-commit hook at {}",
        pre_commit_path.display()
    );
    Ok(())
}
