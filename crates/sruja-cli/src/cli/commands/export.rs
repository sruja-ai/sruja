use clap::Args;

use crate::cli::subcommands::DiscoverCommand;

#[derive(Args)]
pub struct ExportArgs {
    pub format: String,
    pub file: String,
    #[arg(long)]
    pub from_scan: bool,
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long = "output-dir")]
    pub output_dir: Option<String>,
}

#[derive(Args)]
pub struct ListArgs {
    pub file: String,
}

#[derive(Args)]
pub struct TreeArgs {
    pub file: String,
}

#[derive(Args)]
pub struct DiffArgs {
    pub file1: String,
    pub file2: String,
    #[arg(long, default_value = "text")]
    pub format: String,
}

#[derive(Args)]
pub struct ExplainArgs {
    pub element_id: String,
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct DiscoverArgs {
    #[command(subcommand)]
    pub cmd: Option<DiscoverCommand>,
    #[arg(long, short = 'r', alias = "path", default_value = ".")]
    pub repo: String,
    #[arg(long, hide = true)]
    pub context: bool,
    #[arg(long, hide = true)]
    pub explain: bool,
    #[arg(long, hide = true)]
    pub repomap: bool,
    #[arg(long, default_value = "text")]
    pub format: String,
    #[arg(long, default_value_t = 100)]
    pub max_files: usize,
    #[arg(long, default_value_t = 5000)]
    pub max_tokens: usize,
    #[arg(long)]
    pub export_report: Option<String>,
    #[command(flatten)]
    pub enrich: crate::enrichment::EnrichmentArgs,
    #[arg(long, short = 'u', alias = "incremental")]
    pub update: bool,
}
