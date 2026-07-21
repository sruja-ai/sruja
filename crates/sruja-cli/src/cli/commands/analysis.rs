use clap::Args;

use crate::cli::app::ContextIntent;

#[derive(Args)]
pub struct CheckArgs {
    #[arg(long, short = 'r', default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'a')]
    pub architecture: Option<String>,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
    #[arg(long)]
    pub violations_only: bool,
    #[arg(long)]
    pub fail_on: Option<String>,
    #[arg(long)]
    pub ci: bool,
    #[arg(long = "baseline", short = 'b')]
    pub violations_baseline: Option<String>,
    #[arg(long)]
    pub baseline_mode: Option<String>,
    #[arg(long)]
    pub structural_only: bool,
    #[arg(long)]
    pub advisory: bool,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub exclude_barrel_files: bool,
    #[arg(long)]
    pub pr: bool,
    #[arg(long, short = 'B')]
    pub base: Option<String>,
    #[arg(long, short = 'H')]
    pub head: Option<String>,
    #[arg(long)]
    pub compliance: bool,
    #[arg(long, short = 'i')]
    pub intent: Option<String>,
    #[arg(long)]
    pub strict: bool,
}

#[derive(Args)]
pub struct LintArgs {
    pub file: String,
    #[arg(long, default_value = "text")]
    pub format: String,
    #[arg(long)]
    pub baseline: Option<String>,
    #[arg(long)]
    pub write_baseline: Option<String>,
}

#[derive(Args)]
pub struct ScanArgs {
    #[arg(default_value = ".")]
    pub path: String,
    #[arg(long, default_value = "sruja.graph.json")]
    pub output: String,
}

#[derive(Args)]
pub struct McpArgs {
    #[arg(long, short = 'r', default_value = ".")]
    pub root: String,
    #[arg(long, hide = true)]
    pub v2: bool,
}

#[derive(Args)]
pub struct QuickstartArgs {
    #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
    pub path: String,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
    #[arg(long)]
    pub generate_baseline: bool,
    #[arg(long)]
    pub fail_on: Option<String>,
    #[arg(long)]
    pub advisory: bool,
}

#[derive(Args)]
pub struct InitArgs {
    #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
    pub path: String,
    #[arg(long, group = "init_mode")]
    pub prompt: bool,
    #[arg(long, short = 'a', group = "init_mode")]
    pub auto: bool,
    #[arg(long, short = 's', group = "init_mode")]
    pub scan: bool,
    #[arg(long, short = 'f')]
    pub force: bool,
    #[arg(long, group = "init_mode")]
    pub hook: bool,
    #[arg(long, group = "init_mode")]
    pub ci: bool,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long, default_value = "architecture")]
    pub schema: String,
    #[arg(long)]
    pub sync_rules: bool,
}

#[derive(Args)]
pub struct DensityArgs {
    #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
    pub path: String,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
}

#[derive(Args)]
pub struct StatusArgs {
    #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
    pub path: String,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
    #[arg(long = "evolution", short = 'e')]
    pub evolution: bool,
}

#[derive(Args)]
pub struct SyncArgs {
    #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
    pub path: String,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
}

#[derive(Args)]
pub struct SyncIdeRulesArgs {
    #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, default_value_t = 10000)]
    pub max_tokens: usize,
    #[arg(long)]
    pub check: bool,
}

#[derive(Args)]
pub struct ClassifyArgs {
    #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct GenerateSkillArgs {
    #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'o')]
    pub output: Option<String>,
}

#[derive(Args)]
pub struct ReviewArgs {
    #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
    pub path: String,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
    #[arg(long, short = 'a')]
    pub show_all: bool,
    #[arg(long)]
    pub critique: bool,
}

#[derive(Args)]
pub struct BaselineArgs {
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'o', default_value = ".sruja/violations.baseline.json")]
    pub output: String,
}

#[derive(Args)]
pub struct ComplianceArgs {
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'a')]
    pub architecture: Option<String>,
    #[arg(long, short = 'i')]
    pub intent: Option<String>,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
    #[arg(long)]
    pub strict: bool,
}

#[derive(Args)]
pub struct OnboardArgs {
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'f', default_value = "markdown")]
    pub format: String,
    #[arg(long, default_value_t = 8)]
    pub max_items: usize,
    #[command(flatten)]
    pub enrich: crate::enrichment::EnrichmentArgs,
    #[arg(long, short = 'o')]
    pub output: Option<String>,
}

#[derive(Args)]
pub struct AiContextArgs {
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long, short = 'r', action = clap::ArgAction::Append)]
    pub repo: Vec<String>,
    #[arg(long, short = 'f', default_value = "cursor-rules")]
    pub format: String,
    #[arg(long, short = 'o')]
    pub output: Option<String>,
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
    pub intent: Option<ContextIntent>,
    #[arg(long, default_value_t = 2)]
    pub depth: usize,
    #[arg(long, default_value_t = 10000)]
    pub max_tokens: usize,
    #[arg(long)]
    pub cache_friendly: bool,
}

#[derive(Args)]
pub struct HealthArgs {
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'a')]
    pub architecture: Option<String>,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
}

#[derive(Args)]
pub struct ContextScoreArgs {
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
    #[arg(long)]
    pub fail_under: Option<u8>,
}

#[derive(Args)]
pub struct ExploreArgs {
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
}

#[derive(Args)]
pub struct ContextGraphArgs {
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'o', default_value = "context_graph.html")]
    pub output: String,
    #[arg(long)]
    pub open: bool,
}

#[derive(Args)]
pub struct LookupArgs {
    pub name: String,
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'f', default_value = "json")]
    pub format: String,
}

#[derive(Args)]
pub struct RequirementsArgs {
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, short = 'f', default_value = "text")]
    pub format: String,
    #[arg(long)]
    pub priority: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
}
