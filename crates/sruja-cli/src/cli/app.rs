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
    about = "Architecture-as-code CLI for keeping repo context, drift checks, and AI guidance in sync",
    long_about = "Retrieval ladder (deterministic first): focus (before a task) → ai (paste-ready brief) → MCP in your editor. LLM enrichment (--enrich) is optional interpretation, never reviewed truth.",
    after_help = r#"Product loop (define truth → context → drift → review):
  Use the sruja-architecture skill + repo.sruja for reviewed intent; lint after edits;
  sync/review/drift for freshness; focus or ai before coding; MCP inside AI tools.

Start here:
  sruja start -r . --prompt   Set up .sruja/ and emit a skill-ready prompt
  sruja scan .                Scan repo and emit an inferred graph (sruja.graph.json)
  sruja focus -r . --file <path>   Task-scoped blast radius before you edit

Daily loop:
  sruja review -r .           Evidence refresh + drift + next actions (alias: daily)
  sruja status -r .           Truth freshness + baseline (alias: doctor)

Docs & CI:
  sruja lint repo.sruja
  sruja export markdown repo.sruja
  sruja drift --ci -r .       CI drift (github-actions format; replaces hidden `check`)"#
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
