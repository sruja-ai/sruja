//! Initialize Sruja in a repository: create .sruja/, run quickstart, optionally generate prompt.

use std::fs;
use std::path::Path;

use colored::Colorize;
use super::generate::generate_prompt;
use super::scan::quickstart;
use super::CliError;
use sruja_export::mermaid::exporter::{MermaidConfig, MermaidExporter};

/// Initialize Sruja in the given repo: ensure `.sruja/`, run quickstart, optionally generate prompt or auto-onboard.
#[allow(clippy::too_many_arguments)]
pub async fn init(
    repo_root: &str,
    generate_prompt_file: bool,
    auto: bool,
    scan: bool,
    force: bool,
    hook: bool,
    ci: bool,
    dry_run: bool,
    schema: &str,
    sync_rules: bool,
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
    let mut should_scan = scan;
    let mut should_ci = ci;
    let mut should_hook = hook;
    let mut should_sync_rules = sync_rules;

    if schema != "architecture" {
        should_auto = false;
        should_scan = false;
    }

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
            .item_checked("Sync IDE rules (.cursorrules, copilot-instructions.md)", sync_rules)
            .interact()
            .unwrap_or_default();

        for i in extras {
            if i == 0 {
                should_ci = true;
            }
            if i == 1 {
                should_hook = true;
            }
            if i == 2 {
                should_sync_rules = true;
            }
        }
    }

    if should_auto {
        let pb = progress::spinner("📦 Building structural draft from workspace manifests...");
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
            Some(super::scan::draft_summary::draft_baseline_path(repo_path))
        };
        pb.finish_and_clear();

        if let Some(path) = baseline {
            if is_interactive || dry_run {
                println!(
                    "  {} Structural draft (evidence): {}",
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
                            .filter(|n| n.kind == sruja_scan::NodeKind::SERVICE)
                            .count()
                            .to_string()
                    )
                    .bold()
                );

                println!();
                println!("{}", colors::style("Next steps:").bold());
                println!(
                    "  1. {} Use the sruja-architecture skill to author repo.sruja from this draft; lint when promoted.",
                    colors::info("Review:"),
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
            eprintln!("⚠️ Draft or reviewed baseline already exists. Use --force to refresh the structural draft.");
        }
    }

    if should_scan {
        let pb = progress::spinner("🔍 Scanning repository architecture...");
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

        let repo_sruja_path = repo_path.join("repo.sruja");
        if repo_sruja_path.exists() && !force {
            pb.finish_and_clear();
            println!(
                "  {} repo.sruja already exists. Use --force to overwrite.",
                colors::warning("⚠️")
            );
            return Ok(());
        }

        // Generate repo.sruja from scan
        let program = super::scan::draft_summary::build_summary_draft_program(
            &graph,
            "repo.sruja",
        );
        let printer = sruja_export::DslPrinter::new();
        let dsl = printer.print(&program);

        let header = format!(
            r#"// Sruja architecture — auto-generated from code scan
// This is a starting point. Review and refine to match your team's understanding.
// Run `sruja lint repo.sruja` after making changes.
//
// Scan stats: {} components, {} relationships
// Generated: {}

"#,
            graph.nodes.len(),
            graph.edges.len(),
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        );

        if !dry_run {
            fs::write(&repo_sruja_path, format!("{}{}", header, dsl))?;
        }
        pb.finish_and_clear();

        println!(
            "  {} Generated {}",
            colors::success("✅"),
            colors::info("repo.sruja")
        );

        // Run lint on the generated file
        if !dry_run {
            let lint_pb = progress::spinner("🔧 Validating architecture...");
            let lint_result = crate::commands::lint(
                &repo_sruja_path.to_string_lossy(),
                "text",
                None,
                None,
            )
            .await;
            lint_pb.finish_and_clear();

            match lint_result {
                Ok(()) => {
                    println!(
                        "  {} Lint passed — no errors",
                        colors::success("✅")
                    );
                }
                Err(e) => {
                    println!(
                        "  {} Lint warnings: {}",
                        colors::warning("⚠️"),
                        e
                    );
                }
            }
        }

        // Generate classification.json
        if !dry_run {
            let classify_pb = progress::spinner("🏷️  Generating classification...");
            let classify_result = super::classify::classify(super::classify::ClassifyOptions {
                repo: repo_root,
                force: true,  // Always regenerate during init --scan
            });
            classify_pb.finish_and_clear();

            match classify_result {
                Ok(()) => {
                    println!(
                        "  {} Generated {}",
                        colors::success("✅"),
                        colors::dim(".sruja/classification.json")
                    );
                }
                Err(e) => {
                    println!(
                        "  {} Could not generate classification: {}",
                        colors::warning("⚠️"),
                        e
                    );
                }
            }
        }

        // Show architecture visualization
        println!();
        println!("{}", colors::style("Architecture Visualization:").bold());
        println!("{}", "─".repeat(60).truecolor(100, 100, 100));

        // Generate Mermaid diagram
        let mermaid_exporter = MermaidExporter::new(MermaidConfig {
            direction: "LR".to_string(),
            view_level: 0,
            target_id: None,
        });
        let mermaid = mermaid_exporter.export(&program);
        println!("{}", mermaid);

        println!("{}", "─".repeat(60).truecolor(100, 100, 100));

        // Show health score
        let drift_report = sruja_diff::detect_architectural_drift(&graph);

        let score = drift_report.health_score;
        let score_str = format!("{}/100", score);
        let colored_score = match score {
            80..=100 => score_str.green().bold(),
            60..=79 => score_str.yellow().bold(),
            _ => score_str.red().bold(),
        };
        println!();
        println!(
            "  {} Architecture Health Score: {}",
            colors::style("💚").bold(),
            colored_score
        );

        // Show violations summary
        let errors = drift_report.violations.iter().filter(|v| matches!(v.severity, sruja_diff::Severity::Error)).count();
        let warnings = drift_report.violations.iter().filter(|v| matches!(v.severity, sruja_diff::Severity::Warning)).count();
        if errors > 0 || warnings > 0 {
            println!(
                "    {} errors, {} warnings",
                errors.to_string().red(),
                warnings.to_string().yellow()
            );
        }

        // Show summary card
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
                    .filter(|n| n.kind == sruja_scan::NodeKind::SERVICE)
                    .count()
                    .to_string()
            )
            .bold()
        );

        // Sync IDE rules if requested
        if should_sync_rules && !dry_run {
            println!();
            let sync_pb = progress::spinner("📝 Syncing IDE rules...");
            let sync_result = super::sync_ide_rules::sync_ide_rules(
                super::sync_ide_rules::SyncIdeRulesOptions {
                    repo: repo_root,
                    max_tokens: 10000,
                    check: false,
                },
            )
            .await;
            sync_pb.finish_and_clear();

            match sync_result {
                Ok(()) => {
                    println!(
                        "  {} Synced IDE rules (.cursorrules, copilot-instructions.md, llms-architecture.txt)",
                        colors::success("✅")
                    );
                }
                Err(e) => {
                    println!(
                        "  {} Could not sync IDE rules: {}",
                        colors::warning("⚠️"),
                        e
                    );
                }
            }
        }

        // Print next steps
        println!();
        println!("{}", colors::style("Next steps:").bold());
        println!(
            "  1. {} Review repo.sruja and rename components to match your team's language.",
            colors::info("Review:"),
        );
        println!(
            "  2. {} Run 'sruja drift -r . -a repo.sruja' to check for drift.",
            colors::info("Check:")
        );
        println!(
            "  3. {} Run 'sruja watch' while you code for live feedback.",
            colors::info("Monitor:")
        );
        println!(
            "  4. {} Add repo.sruja to version control.",
            colors::info("Commit:")
        );
    }

    if !should_auto && !should_scan {
        let dest_filename = if schema == "architecture" {
            "repo.sruja".to_string()
        } else {
            format!("{}.sruja", schema)
        };
        let dest_path = repo_path.join(&dest_filename);
        let mut scaffolded = false;

        if dest_path.exists() && !force {
            if is_interactive || dry_run {
                println!(
                    "  {} Existing {} file: {}",
                    colors::info("i"),
                    schema,
                    dest_path.display()
                );
            }
        } else {
            if schema == "architecture" && (is_interactive || dry_run) {
                println!(
                    "  {} No architecture file found. Creating manual skeleton recommended.",
                    colors::warning("!")
                );
            }

            let should_scaffold = if schema != "architecture" {
                true // Auto-scaffold custom schemas immediately
            } else if is_interactive {
                Confirm::new()
                    .with_prompt(
                        "Would you like to scaffold the multi-agent team evolutionary template?",
                    )
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            } else {
                false
            };

            if should_scaffold {
                if !dry_run {
                    let content = match schema {
                        "compliance" => include_str!("../../../../templates/blueprints/compliance.sruja"),
                        "business_process" | "business-process" => include_str!("../../../../templates/blueprints/business-process.sruja"),
                        "knowledge" => "// Sruja Context Graph: Knowledge Graph Domain\n// Tracks concepts, citations, and facts.\n",
                        _ => include_str!("../../../../templates/blueprints/agent-team.sruja"),
                    };
                    fs::write(&dest_path, content)?;

                    if schema == "architecture" {
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
                }
                println!(
                    "  {} Scaffolded {} template to {}",
                    colors::success("✓"),
                    schema,
                    colors::info(&dest_filename)
                );
                scaffolded = true;
            }
        }

        if !dry_run && !scaffolded {
            quickstart(repo_root, "text", false, None, true).await?;
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
            "{} Sruja is ready! Run {} for a structural scan (no repo.sruja required).",
            colors::success("🎉"),
            colors::info("sruja drift -r . --structural-only --advisory")
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
