use clap::Subcommand;

use crate::enrichment::EnrichmentArgs;

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum InspectCommand {
    /// Architecture health score from structural violations (0-100)
    Health {
        /// Repository root to scan
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Path to architecture file
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Impact analysis: blast radius
    Impact {
        /// Node selector
        target: String,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Max traversal depth
        #[arg(long, default_value_t = 3)]
        depth: usize,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Investigate architecture decisions with deterministic evidence
    Why {
        /// Question to ask
        question: String,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Use reasoning-tree traversal
        #[arg(long)]
        reasoned: bool,
        /// Use LLM-guided tree search
        #[arg(long)]
        llmguided: bool,
    },
    /// Query the architectural registry
    Query {
        /// Query string
        query: String,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Path to architecture file
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Output format
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// AI-readiness score (0-100)
    #[command(name = "context-score")]
    ContextScore {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Fail with exit code 1 if score is below this threshold
        #[arg(long)]
        fail_under: Option<u8>,
    },
    /// Generate interactive HTML/D3.js visualization
    #[command(name = "context-graph")]
    ContextGraph {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output path for the HTML file
        #[arg(long, short = 'o', default_value = "context_graph.html")]
        output: String,
        /// Open the browser after generation
        #[arg(long)]
        open: bool,
    },
    /// Complete architecture brief for onboarding
    Onboard {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output format
        #[arg(long, short = 'f', default_value = "markdown")]
        format: String,
        /// Max number of items per section
        #[arg(long, default_value_t = 8)]
        max_items: usize,
        /// Optional LLM enrichment to add a narrative section.
        #[command(flatten)]
        enrich: EnrichmentArgs,
        /// Output file
        #[arg(long, short = 'o')]
        output: Option<String>,
    },
    /// Structural overview (first look structural brief)
    #[command(visible_alias = "overview")]
    Quickstart {
        /// Repository root
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Write repo.sruja.draft (workspace map evidence; not reviewed architecture)
        #[arg(long)]
        generate_baseline: bool,
        /// Fail on specified violations
        #[arg(long)]
        fail_on: Option<String>,
        /// First-run friendly: omit orphan info findings
        #[arg(long)]
        advisory: bool,
    },
    /// Keep architecture feedback live while you code
    Watch {
        /// Repository root
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Clear screen between runs
        #[arg(long)]
        clear: bool,
        /// Only watch specific paths
        #[arg(long)]
        focus: Option<String>,
    },
    /// Build scan evidence and learned-fact hypotheses
    Learn {
        /// Repository root
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Only include facts referencing this file
        #[arg(long)]
        file: Option<String>,
        /// Only include facts since git ref
        #[arg(long)]
        since: Option<String>,
        /// Do not write proposals
        #[arg(long)]
        skip_proposals: bool,
        /// Apply proposals
        #[arg(long = "apply-proposals", default_value_t = true, action = clap::ArgAction::Set)]
        apply_proposals: bool,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Ingest external context into .sruja/context/
    Ingest {
        /// Files or directories to ingest
        sources: Vec<String>,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Category tag
        #[arg(long, short = 'c')]
        category: Option<String>,
        /// Comma-separated architecture element IDs
        #[arg(long, short = 'e')]
        elements: Option<String>,
    },
}
