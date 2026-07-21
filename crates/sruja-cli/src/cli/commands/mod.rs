use clap::Subcommand;

pub mod agent_workflow;
pub mod analysis;
pub mod export;

pub use agent_workflow::*;
pub use analysis::*;
pub use export::*;

#[derive(Subcommand)]
pub enum Commands {
    /// Print version information
    Version,
    /// Format a Sruja file in-place
    #[command(hide = true, name = "fmt")]
    Fmt {
        #[arg(long)]
        check: bool,
        file: String,
    },
    /// Export a Sruja file to another format
    #[command(hide = true, name = "export")]
    Export(ExportArgs),
    /// List elements from a Sruja file
    #[command(hide = true, name = "list")]
    List(ListArgs),
    /// Print an architecture tree from a Sruja file
    #[command(hide = true, name = "tree")]
    Tree(TreeArgs),
    /// Show differences between two Sruja files
    #[command(hide = true, name = "diff")]
    Diff(DiffArgs),
    /// Explain an element from a Sruja file
    #[command(hide = true, name = "explain")]
    Explain(ExplainArgs),
    /// Workflow manifest + phase gates (Inception → Construction → Operations)
    #[command(hide = true)]
    Workflow(WorkflowArgs),
    /// AI-DLC workflow: inception → construction → operations with phase gates
    #[command(hide = true)]
    Aidlc(AidlcArgs),
    /// Propose architectural changes for review
    #[command(hide = true)]
    Propose(ProposeArgs),
    /// Grounded architecture authoring helpers (evidence bundle + proposal synthesis)
    #[command(hide = true)]
    Author(AuthorArgs),
    /// Scan a repository and infer an architecture graph
    #[command(hide = true)]
    Scan(ScanArgs),
    /// Impact analysis: blast radius (upstream dependents + downstream dependencies)
    #[command(hide = true)]
    Impact(ImpactArgs),
    /// Investigate a question with deterministic evidence
    #[command(hide = true)]
    Why(WhyArgs),
    /// Lint a Sruja file
    #[command(alias = "validate")]
    Lint(LintArgs),
    /// Start LSP server (stdio)
    #[command(hide = true)]
    Lsp {
        #[arg(long)]
        stdio: bool,
    },
    /// Start MCP server (stdio)
    Mcp(McpArgs),
    /// Drift and structural checks (from code, optional baseline)
    #[command(name = "drift", alias = "check")]
    Check(CheckArgs),
    /// First look: structural overview and optional repo.sruja.draft
    #[command(visible_alias = "overview", hide = true)]
    Quickstart(QuickstartArgs),
    /// Set up Sruja in a repo (.sruja/ and initial evidence)
    #[command(name = "start", alias = "init")]
    Init(InitArgs),
    /// Show current density tier and progression hints
    #[command(hide = true)]
    Density(DensityArgs),
    /// Unified repo status: truth freshness, structural health, AI readiness, density, agent memory
    #[command(visible_alias = "doctor", hide = true)]
    Status(StatusArgs),
    /// Refresh evidence files for context retrieval and reviewed intent workflows
    #[command(hide = true)]
    Sync(SyncArgs),
    /// Write editor rule files from validated architecture
    #[command(name = "sync-ide-rules")]
    #[command(hide = true)]
    SyncIdeRules(SyncIdeRulesArgs),
    /// Generate .sruja/classification.json for a repository
    #[command(name = "classify")]
    #[command(hide = true)]
    Classify(ClassifyArgs),
    /// Generate a prompt for AI to extract procedural knowledge and create a project skill
    #[command(name = "generate-skill")]
    #[command(hide = true)]
    GenerateSkill(GenerateSkillArgs),
    /// Daily action list: refresh evidence, detect drift, suggest next steps
    #[command(visible_alias = "daily", hide = true)]
    Review(ReviewArgs),
    /// Baseline: snapshot current violations to ignore them in CI
    #[command(hide = true)]
    Baseline(BaselineArgs),
    /// Compare declared intent (decisions, ADRs) vs actual implementation
    Intent(IntentArgs),
    /// Structural drift + intent + policy gate
    #[command(hide = true)]
    Compliance(ComplianceArgs),
    /// Complete architecture brief for human or AI reader
    #[command(hide = true)]
    Onboard(OnboardArgs),
    /// Structured architecture context for AI editor integration
    #[command(name = "ai-context", hide = true)]
    AiContext(AiContextArgs),
    /// Scanner introspection for AI/debug: explain scan, repomap, discovery questions
    #[command(hide = true)]
    Discover(DiscoverArgs),
    /// Generate a prompt for LLM-based architecture generation
    #[command(hide = true)]
    Generate(GenerateArgs),
    /// Generate indices for architectural nodes
    #[command(hide = true)]
    Index(IndexArgs),
    /// Query the architectural registry for elements and relationships
    #[command(hide = true)]
    Query(QueryArgs),
    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Architecture health score from structural violations (0-100)
    #[command(hide = true)]
    Health(HealthArgs),
    /// AI-readiness score (0-100)
    #[command(name = "context-score", hide = true)]
    ContextScore(ContextScoreArgs),
    /// Generate the Architecture Explorer model (JSON) for the VS Code webview
    #[command(name = "explore")]
    #[command(hide = true)]
    Explore(ExploreArgs),
    /// Generate an interactive HTML/D3.js visualization
    #[command(name = "context-graph", hide = true)]
    ContextGraph(ContextGraphArgs),
    /// Fetch a compact concept card for one architecture element
    Lookup(LookupArgs),
    /// Retrieve task context before editing
    Focus(FocusArgs),
    /// Paste-ready AI coding brief (includes task context section)
    #[command(name = "ai")]
    Ai(AiArgs),
    /// Ingest external context into `.sruja/context/`
    Ingest(IngestArgs),
    /// Append-only context lineage (intent, drift, proposals, decision traces)
    #[command(hide = true)]
    Event(EventArgs),
    /// Indexed cross-session memory (SQLite + FTS5 under `.sruja/memory.sqlite`)
    #[command(hide = true)]
    Memory(MemoryArgs),
    /// Decision Records (generalized ADRs) under `.sruja/decisions/`
    Decision(DecisionArgs),
    /// Graph temporal queries (history, velocity, etc.)
    #[command(hide = true)]
    Graph(GraphArgs),
    /// List and filter requirements from .sruja files
    #[command(hide = true)]
    Requirements(RequirementsArgs),
    /// Agentic memory: learnings, guardrails, failed hypotheses (bounded to architecture work)
    #[command(hide = true)]
    Agent(AgentArgs),
    /// Autonomous execution loop: comprehend → plan → execute → verify → learn.
    Auto(AutoArgs),
    /// Understand scope and produce a reviewable plan.
    Plan(PlanArgs),
    /// Check architecture health: drift + lint + intent + confidence.
    Verify(VerifyArgs),
    /// Run verification steps for a task profile
    #[command(hide = true)]
    VerifyTask(VerifyTaskArgs),
    /// Post-AI-edit confidence report
    #[command(hide = true)]
    Confidence(ConfidenceArgs),
    /// Inspect and replay saved run snapshots under `.sruja/runs/`
    #[command(hide = true)]
    Run(RunArgs),
    /// DSL authoring tools
    #[command(hide = true)]
    Dsl(DslArgs),
    /// Analysis & scores
    #[command(hide = true)]
    Inspect(InspectArgs),
    /// Keep architecture feedback live while you code
    #[command(hide = true, name = "watch")]
    Watch(WatchArgs),
    /// Build scan evidence and learned-fact hypotheses
    #[command(hide = true, name = "learn")]
    Learn(LearnArgs),
    /// Review & compliance gates
    #[command(hide = true)]
    Guard(GuardArgs),
    /// Adversarial architectural critique of proposed changes
    #[command(hide = true, name = "critique")]
    Critique(CritiqueArgs),
    /// Multi-repo federation: publish, compose
    #[command(hide = true)]
    Federation(FederationArgs),
    /// Publish repo truth + evidence to repo.bundle.json
    #[command(hide = true, name = "publish")]
    Publish(PublishArgs),
    /// Compose one or more repo bundles into system.index.json
    #[command(hide = true, name = "compose")]
    Compose(ComposeArgs),
    /// Human-centric system intelligence: trace, explain, map, before, daily, what-if
    #[command(hide = true)]
    Human(HumanArgs),
    /// Agent capability eval harness
    #[command(hide = true)]
    Eval(EvalArgs),
}
