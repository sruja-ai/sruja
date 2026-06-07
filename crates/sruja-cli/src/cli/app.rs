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

#[derive(Parser)]
#[command(name = "sruja")]
#[command(
    about = "Architecture-aware AI coding: define architecture, keep code aligned, guide AI editors",
    long_about = "Sruja gives AI editors and CI pipelines architecture context.\n\n\
        Define your architecture once (repo.sruja or inferred from code),\n\
        keep code aligned (drift detection), and guide AI coding (MCP/focus).\n\n\
        Deterministic scan-first workflow: check without .sruja, then focus/MCP\n\
        for editors, verify-task for gates. Optional repo.sruja is for viz + strict CI.",
    after_help = r#"Quick start:
  sruja init -r .               Set up .sruja/ and scan
  sruja check -r .              Detect drift (structural or vs repo.sruja)
  sruja status -r .             Unified dashboard (health, density, AI readiness)
  sruja focus -r . --file <f>   Architecture briefing for AI coding

Daily workflow:
  sruja sync -r .               Refresh evidence + sync IDE rules
  sruja check -r . --pr         Check only new violations in a PR
  sruja watch -r .              Live feedback while coding

Editor integration:
  sruja mcp -r .                MCP server for Cursor, Claude Code, etc.
  sruja focus -r . --format for-ai  Paste-ready AI brief

CI / gates:
  sruja check -r . --ci         GitHub Actions format
  sruja check baseline -r .     Snapshot violations for suppression
  sruja verify-task -r .        Run verification steps

DSL authoring:
  sruja lint <file>             Validate .sruja file
  sruja dsl list|tree|diff|explain|import|compile|generate|fmt|export

Knowledge & decisions:
  sruja decision new|list|show|trace|link|accept|supersede
  sruja memory search|timeline|reindex
  sruja agent history|record|curate|plan|apply
  sruja event append|list

Workflow & review:
  sruja workflow init|status|approve|advance|summary|next-steps
  sruja propose create|list|approve
  sruja intent check|propose|evaluate|history

Multi-repo:
  sruja federation publish|compose

Docs: https://sruja.dev | Repo: https://github.com/sruja/sruja"#
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
