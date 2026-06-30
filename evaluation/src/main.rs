//! Eval runner — orchestrates the self-improvement loop for the sruja agent.
//!
//! Commands:
//!   run           Run all eval tasks, capture results
//!   analyze       Cross-run pattern analysis
//!   retry-failed  Re-run only failed tasks from a previous run
//!   cycle         Full self-improvement cycle (run → analyze → recommendations)

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod analyze;
mod report;
mod runner;

pub(crate) const EVAL_RUNNER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "eval-runner", version = EVAL_RUNNER_VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all eval tasks, capture results
    Run {
        /// Run without memory (clear memory first, establish baseline)
        #[arg(long)]
        baseline: bool,

        /// Run with memory accumulation (learn from previous runs)
        #[arg(long)]
        with_memory: bool,

        /// Path to sruja binary (default: sruja from PATH or target/release/sruja)
        #[arg(long)]
        sruja_bin: Option<PathBuf>,

        /// Path to repository root (default: current directory)
        #[arg(long, default_value = ".")]
        repo: PathBuf,

        /// Tag for this run (used in results directory naming)
        #[arg(long)]
        tag: Option<String>,

        /// Max iterations per task (default: 3)
        #[arg(long, default_value_t = 3)]
        max_iterations: usize,

        /// Task filter (run only specific task IDs)
        #[arg(long)]
        task: Vec<String>,

        /// Dry run — only show what would run
        #[arg(long)]
        dry_run: bool,
    },
    /// Analyze results from a previous run
    Analyze {
        /// Run ID to analyze (default: latest)
        #[arg(long)]
        run_id: Option<String>,

        /// Optional baseline run ID for comparison
        #[arg(long)]
        compare: Option<String>,

        /// Path to evaluation directory
        #[arg(long, default_value = "evaluation")]
        eval_dir: PathBuf,
    },
    /// Re-run only failed tasks from a previous run
    RetryFailed {
        /// Run ID to retry (default: latest)
        #[arg(long)]
        run_id: Option<String>,

        /// Path to sruja binary
        #[arg(long)]
        sruja_bin: Option<PathBuf>,

        /// Path to repository root
        #[arg(long, default_value = ".")]
        repo: PathBuf,

        /// Tag for the retry run
        #[arg(long)]
        tag: Option<String>,

        /// Max iterations per task (default: 5)
        #[arg(long, default_value_t = 5)]
        max_iterations: usize,

        /// Dry run
        #[arg(long)]
        dry_run: bool,
    },
    /// Full self-improvement cycle
    Cycle {
        /// Path to sruja binary
        #[arg(long)]
        sruja_bin: Option<PathBuf>,

        /// Path to repository root
        #[arg(long, default_value = ".")]
        repo: PathBuf,

        /// Tag prefix for runs (default: cycle)
        #[arg(long, default_value = "cycle")]
        tag_prefix: String,

        /// Max iterations per task (default: 3)
        #[arg(long, default_value_t = 3)]
        max_iterations: usize,

        /// Dry run
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            baseline,
            with_memory,
            sruja_bin,
            repo,
            tag,
            max_iterations,
            task,
            dry_run,
        } => {
            let mode = if baseline {
                runner::RunMode::Baseline
            } else if with_memory {
                runner::RunMode::WithMemory
            } else {
                eprintln!("Specify --baseline or --with-memory");
                std::process::exit(1);
            };
            runner::run_tasks(&runner::RunOptions {
                mode,
                sruja_bin: sruja_bin.unwrap_or_else(|| PathBuf::from("sruja")),
                repo_root: repo,
                tag,
                max_iterations,
                task_filter: task,
                dry_run,
            })
            .await?;
        }
        Commands::Analyze {
            run_id,
            compare,
            eval_dir,
        } => {
            analyze::analyze(eval_dir, run_id, compare).await?;
        }
        Commands::RetryFailed {
            run_id,
            sruja_bin,
            repo,
            tag,
            max_iterations,
            dry_run,
        } => {
            runner::retry_failed(&runner::RetryOptions {
                run_id,
                sruja_bin: sruja_bin.unwrap_or_else(|| PathBuf::from("sruja")),
                repo_root: repo,
                tag,
                max_iterations,
                dry_run,
            })
            .await?;
        }
        Commands::Cycle {
            sruja_bin,
            repo,
            tag_prefix,
            max_iterations,
            dry_run,
        } => {
            let base_tag = format!("{}-baseline", tag_prefix);
            let mem_tag = format!("{}-with-memory", tag_prefix);

            // Phase 1: Baseline run (no memory)
            eprintln!("═══ Phase 1: Baseline run ═══");
            runner::run_tasks(&runner::RunOptions {
                mode: runner::RunMode::Baseline,
                sruja_bin: sruja_bin.clone().unwrap_or_else(|| PathBuf::from("sruja")),
                repo_root: repo.clone(),
                tag: Some(base_tag.clone()),
                max_iterations,
                task_filter: vec![],
                dry_run,
            })
            .await?;

            // Phase 2: With-memory run (accumulates learnings)
            eprintln!("\n═══ Phase 2: With-memory run ═══");
            runner::run_tasks(&runner::RunOptions {
                mode: runner::RunMode::WithMemory,
                sruja_bin: sruja_bin.clone().unwrap_or_else(|| PathBuf::from("sruja")),
                repo_root: repo.clone(),
                tag: Some(mem_tag.clone()),
                max_iterations,
                task_filter: vec![],
                dry_run,
            })
            .await?;

            // Phase 3: Analysis
            eprintln!("\n═══ Phase 3: Cross-run Analysis ═══");
            analyze::analyze_with_tags(&repo, &base_tag, &mem_tag).await?;

            eprintln!("\n═══ Self-improvement cycle complete ═══");
            eprintln!("Baseline: {base_tag}");
            eprintln!("With memory: {mem_tag}");
            eprintln!();
            eprintln!("Next steps:");
            eprintln!("  1. Review the analysis reports in evaluation/results/");
            eprintln!("  2. Apply recommended improvements to agent code");
            eprintln!("  3. Run: eval-runner retry-failed --run <run_id> to verify fixes");
        }
    }

    Ok(())
}
