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
    about = "Capture knowledge, retrieve context, verify changes",
    long_about = "Sruja is a context engineering tool for software changes.\n\n\
        Core loop:\n\
        1) capture knowledge and decisions\n\
        2) retrieve task-scoped context before editing\n\
        3) verify the result after editing\n\n\
        Reviewed intent in Git (repo.sruja) is optional and used only when you want strict enforcement.",
    after_help = r#"Quick start:
  sruja auto "add health check"               Autonomous: plan → execute → verify → learn
  sruja plan "what does auth affect"          Understand scope + blast radius
  sruja verify                                Check architecture health (drift + lint + intent)

Workflow:
  sruja focus -r . --file <path>              Retrieve task context before editing
  sruja ingest docs/adr/ --category adr       Bring external context into the repo
  sruja decision new -t "..." --typ product   Record a decision

Editor integration:
  sruja mcp -r .                              MCP server for Cursor, Claude Code, etc.

Optional reviewed intent:
  sruja lint repo.sruja
  sruja sync -r .
  sruja drift -r . -a repo.sruja

Extensions (available, but hidden from the default help):
  sruja dsl --help | sruja inspect --help | sruja workflow --help

Docs: https://sruja.ai | Repo: https://github.com/sruja-ai/sruja"#
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
