use clap::Parser;

use super::commands::Commands;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ContextIntent {
    AddFeature,
    Refactor,
    FixBug,
    AddTest,
}

impl ContextIntent {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContextIntent::AddFeature => "add-feature",
            ContextIntent::Refactor => "refactor",
            ContextIntent::FixBug => "fix-bug",
            ContextIntent::AddTest => "add-test",
        }
    }
}

pub(crate) fn was_invoked_as(alias: &str) -> bool {
    std::env::args().nth(1).is_some_and(|arg| arg == alias)
}

#[derive(Parser)]
#[command(name = "sruja")]
#[command(
    about = "Structural drift detection and agent context for your codebase",
    long_about = "Deterministic scan-first workflow: drift without .sruja, then focus/MCP for editors, verify-task for gates. Optional repo.sruja is for viz + strict CI only.",
    after_help = r#"OSS start (no .sruja required):
  sruja start -r .              Create .sruja/ and .srujaignore
  sruja drift -r . --structural-only --advisory   Structural scan + findings

Agent context (any source file):
  sruja focus -r . --file <path>
  sruja ai -r . --task "…"
  sruja mcp -r .                MCP server (profile: coding, ≤18 tools)

Optional reviewed intent:
  sruja sync -r .
  sruja lint repo.sruja
  sruja drift -r . -a repo.sruja

Team / CI (advanced):
  sruja verify-task --profile coding -r .
  sruja drift --ci -r .         github-actions format (replaces hidden `check`)"#
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Increase logging verbosity (-v for info, -vv for debug, -vvv for trace)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Path to custom classification rules YAML file
    #[arg(long, global = true)]
    pub classification_rules: Option<String>,
}
