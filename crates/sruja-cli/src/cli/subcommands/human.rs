use clap::Subcommand;

#[derive(Subcommand)]
pub enum HumanCommand {
    /// "What happens when..." — trace a flow across all repos
    Trace {
        /// Natural language query (e.g. "user clicks checkout")
        query: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Max traversal depth
        #[arg(long, default_value_t = 3)]
        depth: usize,
        /// Filter by team/owner
        #[arg(long)]
        team: Option<String>,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// "What is this thing?" — human-centric element explanation
    Explain {
        /// Element name, ID, or alias
        target: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text, json, md)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Save explanation to docs/architecture/<element>.md
        #[arg(long)]
        persist: bool,
    },
    /// "How does the system work?" — compressed cognitive map
    Map {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Filter by team/owner
        #[arg(long)]
        team: Option<String>,
        /// Focus on a specific element neighborhood
        #[arg(long)]
        focus: Option<String>,
        /// Show all nodes (including modules, docs, assets)
        #[arg(long)]
        all: bool,
    },
    /// Pre-change impact check: "what will I break?"
    Before {
        /// File path to check impact for
        file: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// CI mode: exit non-zero if downstream count exceeds threshold
        #[arg(long)]
        ci: bool,
        /// Downstream threshold for CI mode (default: 10)
        #[arg(long, default_value_t = 10)]
        threshold: usize,
    },
    /// Morning intelligence briefing: what changed, what to care about
    Daily {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Cognitive debt score: how well does the team understand the system?
    CognitiveDebt {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// CI mode: exit non-zero if score below threshold
        #[arg(long)]
        ci: bool,
    },
    /// "What if I change X?" — safe change modeling
    WhatIf {
        /// Description of the change to model
        query: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// CI mode: exit non-zero if blast radius exceeds threshold
        #[arg(long)]
        ci: bool,
        /// Direct effects threshold for CI mode (default: 5)
        #[arg(long, default_value_t = 5)]
        threshold: usize,
    },
}
