use clap::Args;

use crate::cli::subcommands::{
    AgentCommand, AidlcCommand, AuthorCommand, DecisionCommand, DslCommand, EvalCommand,
    EventCommand, FederationCommand, GraphCommand, GuardCommand, HumanCommand, IndexCommand,
    InspectCommand, IntentCommand, MemoryCommand, ProposeCommand, RunCommand, WorkflowCommand,
};
use crate::enrichment::EnrichmentArgs;

#[derive(Args)]
pub struct WorkflowArgs {
    #[command(subcommand)]
    pub cmd: WorkflowCommand,
}

#[derive(Args)]
pub struct AidlcArgs {
    #[command(subcommand)]
    pub cmd: AidlcCommand,
}

#[derive(Args)]
pub struct ProposeArgs {
    #[command(subcommand)]
    pub cmd: ProposeCommand,
}

#[derive(Args)]
pub struct AuthorArgs {
    #[command(subcommand)]
    pub cmd: AuthorCommand,
}

#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub cmd: AgentCommand,
}

#[derive(Args)]
pub struct AutoArgs {
    pub goal: String,
    #[arg(long, short = 'r', default_value = ".")]
    pub repo: String,
    #[arg(long)]
    pub max_iterations: Option<usize>,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub yes: bool,
    #[arg(long)]
    pub pipeline: Option<String>,
    #[arg(long)]
    pub resume: bool,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
    #[arg(long = "show-details")]
    pub show_details: bool,
}

#[derive(Args)]
pub struct PlanArgs {
    pub goal: String,
    #[arg(long, short = 'r', default_value = ".")]
    pub repo: String,
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long)]
    pub element_id: Option<String>,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long)]
    pub pipeline: bool,
    #[arg(long)]
    pub output: Option<String>,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub compact: bool,
}

#[derive(Args)]
pub struct VerifyArgs {
    #[arg(long, short = 'r', default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'p', default_value = "full")]
    pub profile: String,
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long)]
    pub confidence: bool,
    #[arg(long)]
    pub plan: Option<String>,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub continue_on_error: bool,
}

#[derive(Args)]
pub struct VerifyTaskArgs {
    #[arg(long, short = 'r', default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'p', default_value = "coding")]
    pub profile: String,
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long)]
    pub max_runtime_ms: Option<u64>,
    #[arg(long)]
    pub evidence_pack: bool,
    #[arg(long)]
    pub evidence_pack_dir: Option<String>,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
}

#[derive(Args)]
pub struct ConfidenceArgs {
    #[arg(long, short = 'r', default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'p', default_value = "review")]
    pub profile: String,
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long)]
    pub max_runtime_ms: Option<u64>,
    #[arg(long)]
    pub evidence_pack: bool,
    #[arg(long)]
    pub evidence_pack_dir: Option<String>,
    #[arg(long, short = 'f', default_value = "md")]
    pub format: String,
}

#[derive(Args)]
pub struct RunArgs {
    #[command(subcommand)]
    pub cmd: RunCommand,
}

#[derive(Args)]
pub struct DslArgs {
    #[command(subcommand)]
    pub cmd: DslCommand,
}

#[derive(Args)]
pub struct InspectArgs {
    #[command(subcommand)]
    pub cmd: InspectCommand,
}

#[derive(Args)]
pub struct WatchArgs {
    #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
    pub path: String,
    #[arg(long)]
    pub clear: bool,
    #[arg(long)]
    pub focus: Option<String>,
}

#[derive(Args)]
pub struct LearnArgs {
    #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
    pub path: String,
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long)]
    pub since: Option<String>,
    #[arg(long)]
    pub skip_proposals: bool,
    #[arg(long = "apply-proposals", default_value_t = true, action = clap::ArgAction::Set)]
    pub apply_proposals: bool,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
}

#[derive(Args)]
pub struct GuardArgs {
    #[command(subcommand)]
    pub cmd: GuardCommand,
}

#[derive(Args)]
pub struct CritiqueArgs {
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'f')]
    pub files: Vec<String>,
    #[arg(long, short = 'd')]
    pub description: Option<String>,
    #[arg(long, short = 'p')]
    pub proposal: Option<String>,
    #[arg(long)]
    pub base: Option<String>,
    #[arg(long)]
    pub head: Option<String>,
    #[arg(long)]
    pub staged: bool,
    #[arg(long, default_value = "text")]
    pub format: String,
    #[command(flatten)]
    pub enrich: EnrichmentArgs,
    #[arg(long)]
    pub fail_on: Option<String>,
}

#[derive(Args)]
pub struct FederationArgs {
    #[command(subcommand)]
    pub cmd: FederationCommand,
}

#[derive(Args)]
pub struct PublishArgs {
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long)]
    pub repo_id: Option<String>,
    #[arg(long, short = 'o', default_value = "repo.bundle.json")]
    pub output: String,
}

#[derive(Args)]
pub struct ComposeArgs {
    #[arg(long, short = 'i', action = clap::ArgAction::Append)]
    pub input: Vec<String>,
    #[arg(long)]
    pub recursive: bool,
    #[arg(long, short = 'o', default_value = "system.index.json")]
    pub output: String,
}

#[derive(Args)]
pub struct HumanArgs {
    #[command(subcommand)]
    pub cmd: HumanCommand,
}

#[derive(Args)]
pub struct EvalArgs {
    #[command(subcommand)]
    pub cmd: EvalCommand,
}

#[derive(Args)]
pub struct FocusArgs {
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long)]
    pub element_id: Option<String>,
    #[arg(long, short = 't')]
    pub task: Option<String>,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
    #[command(flatten)]
    pub enrich: EnrichmentArgs,
    #[arg(long)]
    pub base_ref: Option<String>,
    #[arg(long)]
    pub head_ref: Option<String>,
    #[arg(long)]
    pub compact: bool,
    #[arg(long)]
    pub staged: bool,
    #[arg(long, default_value_t = 8000)]
    pub max_tokens: usize,
    #[arg(long, short = 'o')]
    pub output: Option<String>,
    #[arg(long)]
    pub cache_friendly: bool,
}

#[derive(Args)]
pub struct AiArgs {
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 't')]
    pub task: Option<String>,
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long)]
    pub element_id: Option<String>,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long)]
    pub base_ref: Option<String>,
    #[arg(long)]
    pub head_ref: Option<String>,
    #[arg(long)]
    pub staged: bool,
    #[arg(long, default_value_t = 8000)]
    pub max_tokens: usize,
    #[arg(long, short = 'o')]
    pub output: Option<String>,
    #[command(flatten)]
    pub enrich: EnrichmentArgs,
}

#[derive(Args)]
pub struct IngestArgs {
    pub sources: Vec<String>,
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'c')]
    pub category: Option<String>,
    #[arg(long, short = 'e')]
    pub elements: Option<String>,
}

#[derive(Args)]
pub struct ImpactArgs {
    pub target: String,
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, default_value_t = 3)]
    pub depth: usize,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
}

#[derive(Args)]
pub struct WhyArgs {
    pub question: String,
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
    #[arg(long)]
    pub reasoned: bool,
    #[arg(long)]
    pub llmguided: bool,
}

#[derive(Args)]
pub struct QueryArgs {
    pub query: String,
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'a')]
    pub architecture: Option<String>,
    #[arg(long, default_value = "text")]
    pub format: String,
}

#[derive(Args)]
pub struct GenerateArgs {
    #[arg(long, short = 'r', action = clap::ArgAction::Append)]
    pub repo: Vec<String>,
    #[arg(long)]
    pub skill_path: Option<String>,
    #[arg(long, required = true)]
    pub prompt_only: bool,
    #[arg(short = 'o', long)]
    pub output: Option<String>,
}

#[derive(Args)]
pub struct IndexArgs {
    #[command(subcommand)]
    pub cmd: IndexCommand,
}

#[derive(Args)]
pub struct EventArgs {
    #[command(subcommand)]
    pub cmd: EventCommand,
}

#[derive(Args)]
pub struct MemoryArgs {
    #[command(subcommand)]
    pub cmd: MemoryCommand,
}

#[derive(Args)]
pub struct DecisionArgs {
    #[command(subcommand)]
    pub cmd: DecisionCommand,
}

#[derive(Args)]
pub struct GraphArgs {
    #[command(subcommand)]
    pub cmd: GraphCommand,
}

#[derive(Args)]
pub struct IntentArgs {
    #[command(subcommand)]
    pub cmd: IntentCommand,
}
