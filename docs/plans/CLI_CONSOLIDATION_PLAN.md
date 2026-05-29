# CLI Consolidation Plan for AI-DLC

## Goal

Consolidate the Sruja CLI surface to reduce duplication, make the AI-DLC path discoverable, and eliminate maintenance debt from copy-pasted enrichment args.

## Changes (3 phases, independent PRs)

---

### Phase 1: Extract `EnrichmentArgs` shared struct

**Problem**: The same 7 enrichment fields (`enrich`, `enrich_provider`, `enrich_cmd`, `enrich_model`, `enrich_base_url`, `enrich_timeout_ms`, `enrich_max_bytes`) are copy-pasted across 9 CLI command/subcommand definitions and 4 handler function signatures.

**Approach**: Create a shared `EnrichmentArgs` struct with `#[command(flatten)]` for CLI definitions and `&EnrichmentArgs` for handler functions. This is a pure refactor — no user-facing change.

#### Step 1: Create `crates/sruja-cli/src/cli/enrichment.rs`

```rust
use clap::Args;

/// Shared enrichment arguments for LLM-powered narrative output.
///
/// Used by commands that support `--enrich` to add an optional LLM-generated
/// narrative section grounded in deterministic Sruja output.
#[derive(Args, Clone, Debug)]
pub struct EnrichmentArgs {
    /// Add an LLM-enriched narrative section to the output
    #[arg(long)]
    pub enrich: bool,

    /// Enrichment provider: cmd (default) or openai. Also via SRUJA_ENRICH_PROVIDER.
    #[arg(long, alias = "llm-provider")]
    pub enrich_provider: Option<String>,

    /// External enrichment command (reads JSON from stdin; writes markdown to stdout)
    #[arg(long)]
    pub enrich_cmd: Option<String>,

    /// Model name (used for provider=openai). Also via SRUJA_ENRICH_MODEL.
    #[arg(long, alias = "llm-model")]
    pub enrich_model: Option<String>,

    /// Base URL (used for provider=openai). Also via SRUJA_ENRICH_BASE_URL.
    #[arg(long, alias = "llm-base-url")]
    pub enrich_base_url: Option<String>,

    /// Timeout for enrichment in milliseconds (default: 15000)
    #[arg(long, default_value_t = 15000)]
    pub enrich_timeout_ms: u64,

    /// Max bytes to read from enrichment stdout (default: 20000)
    #[arg(long, default_value_t = 20000)]
    pub enrich_max_bytes: usize,
}
```

Also add a borrowing view for handler functions:

```rust
/// Borrowed view of enrichment args, for passing to handler functions.
pub struct EnrichmentRef<'a> {
    pub enrich: bool,
    pub provider: Option<&'a str>,
    pub cmd: Option<&'a str>,
    pub model: Option<&'a str>,
    pub base_url: Option<&'a str>,
    pub timeout_ms: u64,
    pub max_bytes: usize,
}

impl EnrichmentArgs {
    pub fn as_ref(&self) -> EnrichmentRef<'_> {
        EnrichmentRef {
            enrich: self.enrich,
            provider: self.enrich_provider.as_deref(),
            cmd: self.enrich_cmd.as_deref(),
            model: self.enrich_model.as_deref(),
            base_url: self.enrich_base_url.as_deref(),
            timeout_ms: self.enrich_timeout_ms,
            max_bytes: self.enrich_max_bytes,
        }
    }
}
```

#### Step 2: Replace enrichment fields in `commands.rs` (5 commands)

Replace the 7 duplicated fields with `#[command(flatten)] pub enrich: EnrichmentArgs` in:
- `Commands::Critique`
- `Commands::Ai`
- `Commands::Onboard`
- `Commands::Discover`
- `Commands::Focus`

#### Step 3: Replace enrichment fields in `subcommands.rs` (4 subcommands)

Replace in:
- `AgentCommand::Run`
- `AgentCommand::Plan`
- `InspectCommand::Onboard`
- `GuardCommand::Critique`

#### Step 4: Update handler functions to take `EnrichmentRef`

Update these 4 functions to accept `&EnrichmentRef` instead of 7 individual params:
- `commands/critique.rs` → `critique(..., enrich: &EnrichmentRef, ...)`
- `commands/focus.rs` → `focus(..., enrich: &EnrichmentRef, ...)`
- `commands/onboard.rs` → `onboard(..., enrich: &EnrichmentRef, ...)` (also remove the separate `LlmConfig` param — fold `provider`/`model`/`base_url` into `EnrichmentRef`)
- `commands/discover.rs` → `discover_explain(..., enrich: &EnrichmentRef, ...)`

For `ai.rs` and `agent_run.rs`, extract the enrichment fields from their options structs into a nested `EnrichmentRef`:
- `AiBriefOptions` gets `pub enrich: EnrichmentRef<'a>` replacing 7 fields
- `AgentRunOptions` gets `pub enrich: EnrichmentRef<'a>` replacing 7 fields

#### Step 5: Update `run.rs` dispatch

Replace the manual destructuring of 7 enrichment fields with:
```rust
Commands::Ai { ref enrich, .. } => {
    commands::ai_brief(AiBriefOptions {
        enrich: enrich.as_ref(),
        // ... other fields
    })
}
```

#### Files changed
- NEW: `crates/sruja-cli/src/cli/enrichment.rs`
- `crates/sruja-cli/src/cli/mod.rs` (add `pub mod enrichment;`)
- `crates/sruja-cli/src/cli/commands.rs` (5 commands)
- `crates/sruja-cli/src/cli/subcommands.rs` (4 subcommands)
- `crates/sruja-cli/src/cli/run.rs` (9 dispatch arms)
- `crates/sruja-cli/src/commands/critique.rs` (signature)
- `crates/sruja-cli/src/commands/focus.rs` (signature)
- `crates/sruja-cli/src/commands/onboard.rs` (signature + remove LlmConfig)
- `crates/sruja-cli/src/commands/discover.rs` (signature)
- `crates/sruja-cli/src/commands/ai.rs` (AiBriefOptions struct)
- `crates/sruja-cli/src/commands/agent_run.rs` (AgentRunOptions struct)
- `crates/sruja-cli/src/commands/mod.rs` (remove LlmConfig, re-export EnrichmentRef)

#### Risk
Low. Pure refactor. Every existing test should pass unchanged. The CLI interface is identical.

---

### Phase 2: Un-hide grouped namespaces, hide duplicate top-levels

**Problem**: The 4 grouped namespaces (`dsl`, `inspect`, `guard`, `federation`) are hidden, and their constituent top-level commands are also hidden. Users see neither in `--help`. The grouped namespaces are the better-organized surface.

**Approach**: Un-hide the 4 grouped namespaces. Keep top-level duplicates hidden (they already are). Update `after_help` in `app.rs` to show the grouped command surface.

#### Step 1: Un-hide grouped namespaces in `commands.rs`

Remove `#[command(hide = true)]` from:
- `Commands::Dsl`
- `Commands::Inspect`
- `Commands::Guard`
- `Commands::Federation`

Also un-hide `Commands::Workflow` and `Commands::Agent` (both are grouped namespaces that are currently hidden but contain important subcommands).

#### Step 2: Update `after_help` in `app.rs`

Add the grouped commands to the help text:

```
Grouped commands:
  sruja dsl list|tree|diff|explain|import|compile|validate|generate|fmt|export|lsp
  sruja inspect health|impact|why|query|context-score|onboard|quickstart|watch|learn|ingest
  sruja guard critique|compliance|baseline|drift-pr
  sruja workflow init|status|approve|advance|summary
  sruja agent history|record|curate|plan|apply
  sruja federation publish|compose
```

#### Step 3: Ensure top-level duplicates stay hidden

Verify that these remain `#[command(hide = true)]`:
- `Commands::Health`, `Commands::Impact`, `Commands::Why`, `Commands::Query`
- `Commands::ContextScore`, `Commands::ContextGraph`, `Commands::Onboard`
- `Commands::Quickstart`, `Commands::Watch`, `Commands::Learn`, `Commands::Ingest`
- `Commands::Critique`, `Commands::Compliance`, `Commands::Baseline`, `Commands::DriftPr`
- `Commands::Publish`, `Commands::Compose`
- `Commands::List`, `Commands::Tree`, `Commands::Diff`, `Commands::Explain`
- `Commands::Import`, `Commands::Compile`, `Commands::Validate`, `Commands::Generate`

#### Step 4: Add tests for grouped namespace parsing

Add to `crates/sruja-cli/src/cli/tests.rs`:
- Test `sruja dsl list <file>` parses correctly
- Test `sruja inspect health -r .` parses correctly
- Test `sruja guard critique --staged` parses correctly
- Test `sruja federation publish -r .` parses correctly

#### Files changed
- `crates/sruja-cli/src/cli/commands.rs` (remove hide from 6 grouped namespaces)
- `crates/sruja-cli/src/cli/app.rs` (update after_help)
- `crates/sruja-cli/src/cli/tests.rs` (add grouped namespace tests)

#### Risk
Low. No behavioral change — commands that worked before still work. The only change is what appears in `--help`.

---

### Phase 3: Add `sruja aidlc` top-level entry point

**Problem**: The AI-DLC workflow requires knowing `sruja workflow init --with-aidlc --aidlc-profile minimal --install-aidlc-rules --title "..."`. That's 5 flags. There's no discoverable entry point for AI-DLC users.

**Approach**: Add a top-level `sruja aidlc` command with 6 subcommands that are syntactic sugar over `sruja workflow` with AI-DLC defaults pre-filled.

#### Step 1: Create `AidlcCommand` enum in `subcommands.rs`

```rust
#[derive(Subcommand)]
pub enum AidlcCommand {
    /// Create an AI-DLC workflow with defaults pre-filled
    Init {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow title (e.g., "Add payment service")
        #[arg(long, short = 't')]
        title: String,
        /// Optional workflow id (defaults to random short id)
        #[arg(long)]
        id: Option<String>,
        /// AI-DLC profile: minimal (default) or full
        #[arg(long, default_value = "minimal")]
        profile: String,
        /// Scaffold template: minimal, feature, bugfix, e2e
        #[arg(long)]
        template: Option<String>,
        /// Target architecture element ids
        #[arg(long = "element", short = 'e')]
        target_elements: Vec<String>,
    },
    /// Show gate readiness + AI-DLC status
    Status {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
        /// Exit non-zero if the current phase gate fails
        #[arg(long)]
        check: bool,
    },
    /// Validate workflow + AI-DLC artifact checklist
    Validate {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: Option<String>,
    },
    /// Show actionable next steps for current phase
    NextSteps {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: Option<String>,
    },
    /// Copy vendored AIDLC rules into .aidlc/ for the editor host
    InstallRules {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
    /// Show a beautiful end-to-end workflow summary
    Summary {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: Option<String>,
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
}
```

#### Step 2: Add `Commands::Aidlc` variant in `commands.rs`

```rust
/// AI-DLC workflow: inception → construction → operations with phase gates
Aidlc {
    #[command(subcommand)]
    cmd: AidlcCommand,
},
```

NOT hidden — this is a visible, discoverable entry point.

#### Step 3: Add dispatch in `run.rs`

```rust
Commands::Aidlc { cmd } => match cmd {
    AidlcCommand::Init { repo, title, id, profile, template, target_elements } => {
        commands::workflow_init(
            &repo, &title, id.as_deref(), target_elements, true,
            commands::WorkflowInitOptions {
                with_aidlc: true,
                aidlc_profile: profile,
                install_rules: true,
                profile: "minimal".to_string(),
                template,
            },
        )
    }
    AidlcCommand::Status { repo, id, check } => {
        commands::workflow_status(&repo, id.as_deref(), check)
    }
    AidlcCommand::Validate { repo, id } => {
        commands::workflow_validate(&repo, id.as_deref())
    }
    AidlcCommand::NextSteps { repo, id } => {
        // Call workflow_summary and extract next_steps
        commands::workflow_next_steps(&repo, id.as_deref())
    }
    AidlcCommand::InstallRules { repo } => {
        commands::workflow_install_rules(&repo)
    }
    AidlcCommand::Summary { repo, id, format } => {
        commands::workflow_summary(&repo, id.as_deref(), &format)
    }
},
```

#### Step 4: Add `workflow_next_steps` to `workflow.rs`

The `workflow_next_steps_json_value` function already exists internally. Expose a public wrapper:

```rust
pub fn workflow_next_steps(repo_root: &str, id: Option<&str>) -> Result<(), CliError> {
    let value = workflow_next_steps_json_value(repo_root, id)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
```

#### Step 5: Update `after_help` and docs

Add to `app.rs` after_help:
```
AI-DLC workflow:
  sruja aidlc init --title "..."          Start AI-DLC workflow (inception → construction → operations)
  sruja aidlc status                      Check phase gate readiness
  sruja aidlc next-steps                  What to do next
```

#### Step 6: Add tests

Test that `sruja aidlc init --title "test"` parses with `with_aidlc: true` and `install_rules: true`.

#### Files changed
- `crates/sruja-cli/src/cli/subcommands.rs` (add AidlcCommand)
- `crates/sruja-cli/src/cli/commands.rs` (add Commands::Aidlc)
- `crates/sruja-cli/src/cli/run.rs` (add dispatch)
- `crates/sruja-cli/src/cli/app.rs` (update after_help)
- `crates/sruja-cli/src/commands/workflow.rs` (expose workflow_next_steps)
- `crates/sruja-cli/src/commands/mod.rs` (re-export workflow_next_steps)
- `crates/sruja-cli/src/cli/tests.rs` (add aidlc parsing tests)

#### Risk
Low. Additive only — no existing commands change. The `sruja workflow` command continues to work as before.

---

## Implementation Order

1. **Phase 1** (EnrichmentArgs) — standalone, no dependencies on other phases
2. **Phase 2** (un-hide grouped namespaces) — standalone, no dependencies
3. **Phase 3** (aidlc entry point) — standalone, no dependencies

All three can be separate PRs. Phase 1 and 2 can be done in parallel. Phase 3 is independent.

## Verification

After each phase:
- `cargo build --release` compiles
- `cargo test --workspace` passes
- `cargo clippy -- -D warnings` clean
- Manual: `sruja --help` shows expected commands
- Manual: `sruja aidlc --help` (phase 3) shows 6 subcommands
