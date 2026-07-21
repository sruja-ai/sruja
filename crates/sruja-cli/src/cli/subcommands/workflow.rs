use clap::Subcommand;

#[derive(Subcommand)]
pub enum WorkflowCommand {
    /// Create workflow manifest + phase directories under .sruja/workflows/<id>/
    Init {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow title
        #[arg(long)]
        title: String,
        /// Optional workflow id (defaults to random short id)
        #[arg(long)]
        id: Option<String>,
        /// Target architecture element ids (optional; used for record-impact and context)
        #[arg(long = "element", short = 'e')]
        target_elements: Vec<String>,
        /// Enforce strict gates (default: true)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        strict_gates: bool,
        /// Enable AI-DLC artifact dirs and manifest.aidlc
        #[arg(long)]
        with_aidlc: bool,
        /// AIDLC gate profile when --with-aidlc (minimal|full)
        #[arg(long, default_value = "minimal")]
        aidlc_profile: String,
        /// Run workflow install-rules during init
        #[arg(long)]
        install_aidlc_rules: bool,
        /// Workflow profile (minimal|full|e2e)
        #[arg(long, default_value = "minimal")]
        profile: String,
        /// Workflow scaffold template (e2e|feature|bugfix|minimal)
        #[arg(long)]
        template: Option<String>,
    },
    /// List workflows under .sruja/workflows/
    List {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
    /// Show workflow phase and gate readiness
    Status {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
        /// Exit non-zero if the current phase gate fails
        #[arg(long)]
        check: bool,
    },
    /// Record impact.json for the workflow's target_elements
    RecordImpact {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id
        #[arg(long)]
        id: String,
        /// Impact traversal depth (default: 3)
        #[arg(long, default_value_t = 3)]
        depth: usize,
    },
    /// Approve a phase after verifying required artifacts are present
    Approve {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id
        #[arg(long)]
        id: String,
        /// Phase to approve (inception|construction|operations)
        #[arg(long)]
        phase: String,
        /// Actor name (defaults to "human")
        #[arg(long)]
        by: Option<String>,
    },
    /// Advance to the next phase if the current phase is approved (strict) or always (non-strict)
    Advance {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id
        #[arg(long)]
        id: String,
    },
    /// Copy vendored AIDLC rules into .aidlc/ for the editor host
    InstallRules {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
    /// Validate workflow + optional AIDLC artifact checklist (same checks as status --check)
    Validate {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: Option<String>,
    },
    /// Append an audit event to workflow audit.jsonl
    Audit {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        event: String,
        #[arg(long)]
        by: Option<String>,
    },
    /// Generate traceability matrix from workflow aidlc-docs (requires aidlc-traceability Python package)
    Trace {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: String,
        #[arg(long, default_value = "markdown")]
        format: String,
        #[arg(long)]
        check: bool,
    },
    /// Optional headless AIDLC run via aidlc-evaluator (requires Python + AWS when not --dry-run)
    Run {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        vision: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Grounded design review for workflow inception (writes design-review.md)
    DesignReview {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: String,
        #[arg(long, short = 'o')]
        output: Option<String>,
        #[arg(long)]
        enrich_cmd: Option<String>,
    },
    /// Scaffold or capture requirements under .sruja/workflows/<id>/inception/requirements.md
    CaptureRequirements {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
        /// Optional issue URL or identifier to ingest from
        #[arg(long)]
        from_issue: Option<String>,
        /// Optional external enrichment command to run (reads JSON from stdin; writes markdown to stdout)
        #[arg(long)]
        enrich_cmd: Option<String>,
    },
    /// Record test verification results under .sruja/workflows/<id>/construction/test-results.json
    RecordTestResults {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
        /// Task verification profile (coding|bugfix|review|arch)
        #[arg(long)]
        profile: Option<String>,
        /// Path to a pre-recorded test output JSON file to copy
        #[arg(long)]
        from_file: Option<String>,
    },
    /// Record operations readiness checklist under .sruja/workflows/<id>/operations/readiness.json
    RecordReadiness {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
    },
    /// Show a beautiful end-to-end workflow summary and health dashboard
    Summary {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Show actionable next steps for the current workflow phase
    NextSteps {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
    },
}
