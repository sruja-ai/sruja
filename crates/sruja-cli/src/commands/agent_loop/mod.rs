//! `sruja agent loop` — the closed-loop autonomous coding agent.
//!
//! Drives the full cognition loop (comprehend -> plan -> execute via tools ->
//! critique -> replan until approved) against a real workspace using the
//! `sruja-agent` crate's `Agent::run_loop`.
//!
//! This is the CLI-first path that turns Sruja from a passive harness into an
//! autonomous actor graded by its own deterministic tools.
//!
//! # Configuration
//!
//! Uses the industry-standard resolution chain:
//! 1. CLI flags (highest priority)
//! 2. Environment variables (provider-specific)
//! 3. `.sruja/config.toml` (non-secrets only)
//! 4. Built-in defaults (lowest priority)
//!
//! ## Multi-Provider Support
//!
//! Configure different providers for different tasks in `.sruja/config.toml`:
//!
//! ```toml
//! [integrations]
//! default_provider = "zai"
//!
//! [integrations.providers.zai]
//! base_url = "https://api.z.ai/api/coding/paas/v4"
//! key_env = "ZAI_API_KEY"
//!
//! [agent.models]
//! cheap = { provider = "zai", model = "GLM-4-Flash" }
//! mid = { provider = "zai", model = "GLM-4.7" }
//! premium = { provider = "openrouter", model = "anthropic/claude-sonnet-4" }
//! review = { provider = "openrouter", model = "google/gemini-2.5-flash" }
//! ```
//!
//! See `config::resolve_multi_provider_config` for details.

mod calibration;
mod output;
mod utils;

pub(crate) use calibration::{calibration_gate, model_family, GateOutcome};
pub(crate) use output::print_loop_result_human;
pub(crate) use utils::{agent_err_to_cli, consolidate_memory, preloaded_architecture_context};

use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::sync::Arc;

use tokio::sync::mpsc;

use sruja_agent::calibration as agent_calibration;
use sruja_agent::cognition::loop_event::LoopEvent;
use sruja_agent::llm::{OpenAiClient, TieredClient};
use sruja_agent::tool::ToolRegistry;
use sruja_agent::verify::VerifyOptions;
use sruja_agent::{
    AgentChangelog, AgentConfig, GoalSpec, LoopConfig, LoopManifest, ModelMapping, VerifierConfig,
};

use super::loop_checkpoint::{self, GitCheckpoint};
use super::loop_events::{self, StatusBar};
use super::loop_report::LiveReportHook;

use super::CliError;
use crate::config;
use crate::utils::colors;
use crate::utils::run_id::generate_run_id;
use crate::utils::run_snapshots::write_json_snapshot;

use calibration::has_goal_precedent;

/// Default shell commands the agent is allowed to execute when the user hasn't
/// configured an explicit `shell_allowlist` in `.sruja/loop.toml`. These are
/// the most common safe, non-destructive tools for a coding agent.
const DEFAULT_SHELL_ALLOWLIST: &[&str] = &["cargo", "git"];

/// Maximum file size (bytes) for pre-loading into the comprehension prompt.
/// Files larger than this are skipped to avoid blowing up the context window.
const PRELOAD_MAX_BYTES: usize = 50 * 1024; // 50 KB

/// Maximum tokens for architecture context injection.
/// Keeps the context compact to avoid blowing up the prompt.
const ARCH_CONTEXT_MAX_TOKENS: usize = 2000;

/// Options received from the CLI.
#[derive(Debug)]
pub struct AgentLoopOptions<'a> {
    pub repo: &'a str,
    pub goal: &'a str,
    pub max_iterations: Option<usize>,
    pub no_tdd: bool,
    pub dry_run: bool,
    pub model: Option<&'a str>,
    pub base_url: Option<&'a str>,
    pub spend_cap_usd: Option<f64>,
    pub no_oscillation_detection: bool,
    pub format: &'a str,
    pub force_proceed: bool,
    pub no_default_grader: bool,
    pub steer: bool,
    pub resume: bool,
    pub show_plan: bool,
    pub plan_only: bool,
    pub show_pipeline: bool,
    /// Path to a pipeline TOML file that overrides the manifest's pipeline.
    pub pipeline_override: Option<std::path::PathBuf>,
    pub checkpoint: bool,
    pub no_checkpoint: bool,
    pub changelog: bool,
    pub verbose: bool,
}

/// Entry point for `sruja agent loop`.
pub async fn agent_loop(options: &AgentLoopOptions<'_>) -> Result<(), CliError> {
    let repo_path = Path::new(&options.repo);

    // ── Load .sruja/loop.toml for defaults ────────────────────────────────
    let mut manifest = LoopManifest::load_from_path(repo_path);

    // ── Pipeline override ────────────────────────────────────────────────
    // When `--pipeline <path>` is provided, load that file and use it as
    // the pipeline configuration instead of the one from .sruja/loop.toml.
    if let Some(ref pipeline_path) = options.pipeline_override {
        let content = std::fs::read_to_string(pipeline_path)
            .map_err(|e| CliError::validation(format!("cannot read pipeline file: {e}")))?;
        let pipeline: sruja_agent::PipelineConfig = toml::from_str(&content)
            .map_err(|e| CliError::validation(format!("cannot parse pipeline file: {e}")))?;
        manifest.pipeline = pipeline;
    }

    // ── Show pipeline (early return, no execution) ───────────────────────
    if options.show_pipeline {
        let pipeline = &manifest.pipeline;
        println!(
            "{}",
            serde_json::to_string_pretty(pipeline)
                .map_err(|e| CliError::validation(format!("pipeline serialization: {e}")))?
        );
        return Ok(());
    }

    // ── Plan-only mode ───────────────────────────────────────────────────
    // Override the pipeline to skip verification and block file mutations.
    // The agent comprehends the goal and runs tools to produce a plan, but
    // cannot write files. Output is printed at the end.
    if options.plan_only {
        eprintln!(
            "{}",
            colors::detail_line("Plan-only mode: producing plan without code changes.")
        );
    }

    // ── Pre-flight: check for LLM config ─────────────────────────────────
    let config_path = repo_path.join(".sruja/config.toml");
    if !config_path.exists() {
        return Err(CliError::validation(
            "No LLM provider configured.\n\
             \n\
             The agent needs an LLM to work. Run the setup wizard once:\n\
               sruja agent setup\n\
             \n\
             This creates .sruja/config.toml with your provider settings.\n\
             API keys are stored in environment variables, not the config file."
                .to_string(),
        ));
    }

    // ── Resolve multi-provider configuration ──────────────────────────────
    // Supports different providers for different tasks (cheap/mid/premium/review)
    let multi_config = config::resolve_multi_provider_config(repo_path)?;

    // ── Resolve loop-specific configuration (CLI > manifest > defaults) ───
    let max_iterations = options
        .max_iterations
        .or(if manifest.max_iterations != 3 {
            Some(manifest.max_iterations)
        } else {
            None
        })
        .unwrap_or(3);

    let tdd = if options.no_tdd { false } else { manifest.tdd };

    let dry_run = options.dry_run || manifest.dry_run;

    // Build model mapping from multi-provider config.
    // Each tier can use a different provider/model.
    let models = ModelMapping {
        cheap: multi_config.cheap.model.clone(),
        mid: multi_config.mid.model.clone(),
        premium: multi_config.premium.model.clone(),
        review: multi_config.review.model.clone(),
    };

    // ── Create LLM client ─────────────────────────────────────────────────
    // Build per-tier clients so each tier can hit its own provider with the
    // correct API key and base URL. The TieredClient routes by model name.
    let mid_config = &multi_config.mid;
    let model = options.model.unwrap_or(&mid_config.model);
    let base_url = options.base_url.unwrap_or(&mid_config.base_url);

    let default_client = Arc::new(
        OpenAiClient::new(&mid_config.api_key, base_url, model)
            .map_err(|e| CliError::validation(format!("Failed to create LLM client: {e}")))?,
    );

    let mut tiered = TieredClient::new(default_client.clone());
    for tier_cfg in [
        &multi_config.cheap,
        &multi_config.mid,
        &multi_config.premium,
        &multi_config.review,
    ] {
        // Only register a separate client when this tier's credentials differ
        // from the default (mid-tier). Same key + URL = same client.
        let needs_own_client =
            tier_cfg.api_key != mid_config.api_key || tier_cfg.base_url != mid_config.base_url;

        if needs_own_client {
            let client = Arc::new(
                OpenAiClient::new(&tier_cfg.api_key, &tier_cfg.base_url, &tier_cfg.model).map_err(
                    |e| CliError::validation(format!("Failed to create LLM client: {e}")),
                )?,
            );
            // Exact route for the tier model name.
            tiered = tiered.with_route(&tier_cfg.model, client.clone());
            // Name-substring route derived from the model family name, so
            // persona model overrides (e.g. "mimo-v2.5" when the tier model
            // is "mimo-v2.5-pro") find the correct provider via containment.
            let family = model_family(&tier_cfg.model);
            if !family.is_empty() {
                tiered = tiered.with_provider_name_contains(family, client);
            }
        }
    }

    // ── Build goal spec (CLI statement + manifest criteria/constraints) ───
    // CLI --goal always wins for the statement; the manifest's [goal] section
    // provides structured acceptance criteria and constraints.
    let goal_spec = if manifest.goal.has_criteria() || !manifest.goal.constraints.is_empty() {
        GoalSpec {
            statement: options.goal.to_string(),
            acceptance_criteria: manifest.goal.acceptance_criteria.clone(),
            target_files: manifest.goal.target_files.clone(),
            target_elements: manifest.goal.target_elements.clone(),
            constraints: manifest.goal.constraints.clone(),
        }
    } else {
        GoalSpec::new(options.goal)
    };

    // ── Compression layer ────────────────────────────────────────────────
    // Wrap the tiered client in a CompressingClient so old tool messages are
    // compressed before each LLM call. The CCR store is shared with the
    // compress_restore tool so the agent can fetch verbatim originals.
    let compressing = sruja_agent::llm::CompressingClient::new(Arc::new(tiered));
    let ccr_store = compressing.ccr_store();

    tracing::info!("agent_loop: context compression enabled (TextCrusher + CCR)");

    // ── Build tools + agent ───────────────────────────────────────────────
    // Built-in filesystem/shell tools + the sruja-native "eyes" (focus, explain,
    // drift, compliance, query). The latter ground the agent in the architecture
    // knowledge graph so comprehension/critique cite element IDs, not guesses.
    let shell_allowlist = if manifest.shell_allowlist.is_empty() {
        DEFAULT_SHELL_ALLOWLIST
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        manifest.shell_allowlist.clone()
    };
    let tools = ToolRegistry::with_builtin(repo_path.to_path_buf(), shell_allowlist.clone())
        .with(Box::new(sruja_agent::tool::sruja::SrujaFocusTool::new(
            repo_path.to_path_buf(),
        )))
        .with(Box::new(sruja_agent::tool::sruja::SrujaExplainTool::new(
            repo_path.to_path_buf(),
        )))
        .with(Box::new(sruja_agent::tool::sruja::SrujaDriftTool::new(
            repo_path.to_path_buf(),
        )))
        .with(Box::new(
            sruja_agent::tool::sruja::SrujaComplianceTool::new(repo_path.to_path_buf()),
        ))
        .with(Box::new(sruja_agent::tool::sruja::SrujaQueryTool::new(
            repo_path.to_path_buf(),
        )))
        .with(Box::new(sruja_agent::tool::sruja::SrujaLookupTool::new(
            repo_path.to_path_buf(),
        )))
        .with(Box::new(sruja_agent::tool::CompressRestoreTool::new(
            ccr_store,
        )));

    let mut system_hints = Vec::new();
    if repo_path.join("repo.sruja").exists() {
        system_hints.push(
            "Before planning, call the sruja_focus tool to ground your understanding \
             in the architecture graph. This gives you element IDs, blast radius, \
             and dependencies — plan from evidence, not guesses."
                .to_string(),
        );
    }

    // ── Build critique persona ensemble ────────────────────────────────
    // Merge manifest persona model overrides with the default personas.
    // Each manifest entry overrides the `model` field of the matching
    // default persona (by id). Unlisted personas keep models.review.
    let critique_personas = if manifest.critique.personas.is_empty() {
        // No overrides — use defaults (all route to models.review).
        sruja_agent::cognition::CritiquePersona::default_personas()
    } else {
        let mut personas = sruja_agent::cognition::CritiquePersona::default_personas();
        let default_ids: Vec<String> = personas.iter().map(|p| p.id.clone()).collect();
        for override_cfg in &manifest.critique.personas {
            if let Some(persona) = personas.iter_mut().find(|p| p.id == override_cfg.id) {
                persona.model = Some(override_cfg.model.clone());
            } else {
                eprintln!(
                    "  Warning: critique persona override '{}' does not match any \
                     default persona ({}) — ignored.",
                    override_cfg.id,
                    default_ids.join(", ")
                );
            }
        }
        personas
    };

    let config = AgentConfig {
        models,
        tdd,
        review_every_change: manifest.review_every_change,
        dry_run,
        system_hints,
        enable_tool_call_tracing: true,
        max_tool_iterations: manifest.max_tool_iterations,
        critique_personas,
        // Stuck-detection thresholds (configurable via AgentConfig directly).
        max_consecutive_tool_only: 3,
        max_consecutive_same_call: 3,
        max_non_converged_fraction: 0.5,
        ..Default::default()
    };

    // ── Run the loop ──────────────────────────────────────────────────────
    let run_id = generate_run_id();

    // ── Pre-load target files for comprehension (Phase 2) ────────────────
    // When --file is specified, read the file once and inject it into the
    // comprehension prompt. Eliminates redundant file_read tool calls.
    // Skip files larger than PRELOAD_MAX_BYTES to avoid context window bloat.
    let preloaded_files: std::collections::HashMap<String, String> = goal_spec
        .target_files
        .iter()
        .filter_map(|path| {
            let full_path = repo_path.join(path);
            match std::fs::metadata(&full_path) {
                Ok(meta) if meta.len() as usize > PRELOAD_MAX_BYTES => {
                    eprintln!(
                        "  Warning: skipping pre-load of {path} ({}KB > {}KB limit)",
                        meta.len() / 1024,
                        PRELOAD_MAX_BYTES / 1024
                    );
                    None
                }
                Ok(_) => match std::fs::read_to_string(&full_path) {
                    Ok(content) => Some((path.clone(), content)),
                    Err(e) => {
                        eprintln!("  Warning: could not pre-load {path}: {e}");
                        None
                    }
                },
                Err(e) => {
                    eprintln!("  Warning: could not pre-load {path}: {e}");
                    None
                }
            }
        })
        .collect();

    // ── Pre-load architecture context for comprehension ────────────────
    // Injects repomap and topology into the comprehension prompt so the
    // agent doesn't need to call MCP tools for basic architecture context.
    // This saves tokens and makes the agent more efficient.
    let arch_context = preloaded_architecture_context(repo_path, ARCH_CONTEXT_MAX_TOKENS);

    let agent = {
        let mut builder = sruja_agent::Agent::builder()
            .llm(Arc::new(compressing))
            .tools(tools)
            .config(config)
            .memory(repo_path)
            .trace_context(&run_id, &run_id)
            .tool_call_tracer(Box::new(super::context_events::ContextEventsTracer))
            .preloaded_files(preloaded_files)
            .preloaded_arch_context(arch_context);

        // Connect to declared MCP servers (graceful degradation if none/failed)
        if !manifest.mcp.servers.is_empty() {
            builder = builder
                .with_mcp(&manifest, repo_path.to_path_buf())
                .await
                .map_err(|e| CliError::validation(format!("MCP initialization error: {e}")))?;
        }

        if io::stdin().is_terminal() {
            let report_dir = repo_path.join(".sruja").join("agent");
            builder = builder.hook(Box::new(LiveReportHook::new(
                options.goal,
                max_iterations,
                options.steer,
                report_dir,
            )));
        }
        builder.build().map_err(agent_err_to_cli)?
    };

    if io::stdin().is_terminal() {
        eprintln!();
        if options.verbose {
            eprintln!("{}", colors::section_header("Agent Loop"));
            eprintln!("{}", colors::summary_line("Goal", options.goal));
            eprintln!("{}", colors::summary_line("Model", model));
            eprintln!(
                "{}",
                colors::summary_line("Max iterations", &max_iterations.to_string())
            );
            eprintln!("{}", colors::section_footer());
        } else {
            eprintln!(
                "{}",
                colors::detail_line(&format!(
                    "Running {} (max {} iterations)",
                    options.goal, max_iterations
                ))
            );
        }
        eprintln!();
        eprintln!("{}", colors::detail_line("The agent will:"));
        eprintln!(
            "  {} {}",
            colors::detail_line("1."),
            colors::detail_line("Analyze your codebase to understand the structure")
        );
        eprintln!(
            "  {} {}",
            colors::detail_line("2."),
            colors::detail_line("Create a plan with concrete steps")
        );
        eprintln!(
            "  {} {}",
            colors::detail_line("3."),
            colors::detail_line("Execute changes and verify they work")
        );
        eprintln!(
            "  {} {}",
            colors::detail_line("4."),
            colors::detail_line("Review results and provide feedback")
        );
        eprintln!();
    }

    let spend_cap_usd = options.spend_cap_usd.or(manifest.spend_cap_usd);
    let detect_oscillation = if options.no_oscillation_detection {
        false
    } else {
        manifest.detect_oscillation
    };

    // ── Auto-baseline: snapshot current violations so the grader only
    // fails on NEW violations the agent introduces, not pre-existing noise
    // (god-module thresholds, orphan book JS, tree-sitter false positives).
    let baseline_path = repo_path.join(".sruja").join("violations.baseline.json");
    if !baseline_path.exists() && !options.no_default_grader && !dry_run {
        let bp = baseline_path.to_string_lossy().to_string();
        match std::process::Command::new(super::loop_grader::resolve_sruja_binary().as_str())
            .args(["baseline", "-r", ".", "-o", &bp])
            .current_dir(repo_path)
            .output()
        {
            Ok(output) if output.status.success() => {
                if options.verbose {
                    eprintln!(
                        "{}",
                        colors::detail_line(&format!("✓ Violations baseline created: {}", bp))
                    );
                }
            }
            Ok(output) => {
                if options.verbose {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!(
                        "  ⚠ Could not create violations baseline ({}): will run \
                         without pre-existing violation suppression.",
                        stderr.trim().lines().next().unwrap_or("unknown error")
                    );
                }
            }
            Err(e) => {
                if options.verbose {
                    eprintln!(
                        "  ⚠ Could not create violations baseline ({e}): will run \
                         without pre-existing violation suppression."
                    );
                }
            }
        }
    }

    // ── In-loop verifier configuration ───────────────────────────────────
    // Priority chain: user manifest steps > default grader > none.
    // The default grader makes the "never self-graded" thesis true without
    // configuration by running sruja's own deterministic architecture checks.
    let (verifier, grader_source) = if !manifest.verify_steps.is_empty() {
        let verifier = VerifierConfig {
            steps: manifest.verify_steps.clone(),
            options: VerifyOptions {
                allowed_executables: shell_allowlist.clone(),
                ..Default::default()
            },
            workdir: repo_path.to_path_buf(),
        };
        (Some(verifier), "manifest".to_string())
    } else if !options.no_default_grader {
        let sruja_bin = super::loop_grader::resolve_sruja_binary();
        let steps = super::loop_grader::default_grader_steps(
            repo_path,
            &sruja_bin,
            &manifest.default_grader_fail_on,
        );
        let verifier = if steps.is_empty() {
            None
        } else {
            Some(VerifierConfig {
                steps,
                options: VerifyOptions {
                    allowed_executables: vec![sruja_bin.clone()],
                    continue_on_error: true,
                    ..Default::default()
                },
                workdir: repo_path.to_path_buf(),
            })
        };
        (verifier, "default".to_string())
    } else {
        (None, "none".to_string())
    };

    // ── Default grader health check ───────────────────────────────────────
    // Verifies the sruja binary and the default grader toolchain work before
    // the agent loop starts. Warnings only — the loop still runs even if the
    // grader is misconfigured (e.g. under --no-default-grader).
    if let ("default", Some(ref vc)) = (grader_source.as_str(), &verifier) {
        let sruja_bin = vc
            .options
            .allowed_executables
            .first()
            .map(|s| s.as_str())
            .unwrap_or("sruja");
        if let Err(problems) = super::loop_grader::verify_grader_health(repo_path, sruja_bin) {
            if options.verbose {
                eprintln!("⚠️  Default grader health check:");
                for p in &problems {
                    eprintln!("   • {p}");
                }
                eprintln!(
                    "   The agent loop will still run, but verification results may be unreliable."
                );
            }
        }
    }

    // Checkpoint directory for crash-resume support.
    let checkpoint_dir = crate::utils::run_snapshots::run_dir(repo_path, &run_id);

    // Plan-only mode: remove verify stage, force dry-run, disable grader.
    let pipeline = if options.plan_only {
        // Keep only comprehend + implement stages (skip verify).
        // The agent can still run the tool loop to produce output, but
        // dry-run blocks all file mutations.
        let stages: Vec<sruja_agent::StageKind> = manifest
            .pipeline
            .stages
            .iter()
            .filter(|s| !matches!(s, sruja_agent::StageKind::Verify))
            .copied()
            .collect();
        sruja_agent::PipelineConfig {
            stages,
            ..manifest.pipeline.clone()
        }
    } else {
        manifest.pipeline.clone()
    };
    let dry_run = options.plan_only || dry_run;

    let loop_config = LoopConfig {
        max_iterations,
        spend_cap_usd,
        detect_oscillation,
        pipeline,
        verifier,
        checkpoint_dir: Some(checkpoint_dir.clone()),
        ..Default::default()
    };

    // ── Safety check (pre-flight) ──────────────────────────────────────
    // Determines whether the agent can proceed autonomously or
    // should ask for human approval before touching files.
    let thresholds = crate::commands::intent_domain::focus::load_ask_thresholds(repo_path);
    let has_precedent = has_goal_precedent(repo_path, options.goal, &goal_spec.target_elements);

    let gate = calibration_gate(
        options.goal,
        &goal_spec.target_elements,
        &goal_spec.target_files,
        has_precedent,
        &thresholds,
        options.force_proceed,
    );

    match &gate {
        GateOutcome::Halt { reason } => {
            eprintln!("{}", colors::section_header("Safety Check"));
            eprintln!("{}", colors::verdict_badge("REVIEW NEEDED", "halt"));
            eprintln!("{}", colors::detail_line(reason));
            eprintln!();

            // Interactive prompt when stdin is a TTY (not piped/redirected).
            if io::stdin().is_terminal() {
                eprintln!("{}", colors::detail_line("Proceed anyway? [y/N]: "));
                eprint!("  ");
                io::stderr().flush().ok();
                let mut input = String::new();
                io::stdin().read_line(&mut input).ok();
                let choice = input.trim().to_lowercase();
                if choice != "y" && choice != "yes" {
                    eprintln!("{}", colors::detail_line("Aborted. Use --yes to force."));
                    return Ok(());
                }
                eprintln!(
                    "{}",
                    colors::detail_line("Proceeding despite safety check (forced by user).")
                );
            } else {
                eprintln!("{}", colors::detail_line("Use --yes to override."));
                return Ok(());
            }
            eprintln!("{}", colors::section_footer());
        }
        GateOutcome::Proceed { plan, record } => {
            let verdict_human = match plan.verdict {
                sruja_agent::calibration::Verdict::ProceedSilent => {
                    "no concerns — running without confirmation"
                }
                sruja_agent::calibration::Verdict::ProceedAndFlag => "flagged for review",
                sruja_agent::calibration::Verdict::ProceedCitingPrecedent => {
                    "learned from past — running without confirmation"
                }
                sruja_agent::calibration::Verdict::Ask => "needs review",
            };
            if io::stdin().is_terminal() {
                eprintln!(
                    "{}",
                    colors::detail_line(&format!("Safety check: {verdict_human}"))
                );
            }
            // Write the calibration DR to .sruja/decisions/ if present.
            if let Some(dr) = record {
                let decisions_dir = repo_path.join(".sruja").join("decisions");
                if let Err(e) = std::fs::create_dir_all(&decisions_dir) {
                    if options.verbose {
                        eprintln!("   Warning: could not create decisions dir: {e}");
                    }
                } else {
                    let path = decisions_dir.join(dr.filename());
                    if let Err(e) = std::fs::write(&path, dr.to_markdown()) {
                        if options.verbose {
                            eprintln!("   Warning: could not write calibration DR: {e}");
                        }
                    } else if options.verbose {
                        eprintln!("   Calibration DR: {}", path.display());
                    }
                }
            }
        }
    }

    // ── Capture calibration verdict for checkpoint + changelog ──────────
    let calibration_ask_plan: Option<sruja_agent::AskPlan> = match &gate {
        GateOutcome::Proceed { plan, .. } => Some((**plan).clone()),
        GateOutcome::Halt { .. } => None,
    };

    // ── Git checkpoint (U5) ─────────────────────────────────────────────
    // Auto-enable for one-way-door goals; explicit flags override.
    let reversibility = calibration_ask_plan
        .as_ref()
        .map(|p| p.reversibility)
        .unwrap_or(agent_calibration::Reversibility::TwoWay);

    let cp_enabled = loop_checkpoint::should_checkpoint(
        reversibility,
        options.checkpoint,
        options.no_checkpoint,
    );

    let checkpoint = if cp_enabled {
        match GitCheckpoint::create(repo_path) {
            Ok(Some(cp)) => {
                if options.verbose {
                    eprintln!(
                        "{}",
                        colors::detail_line(&format!(
                            "✓ Git checkpoint created: {}",
                            cp.ref_name()
                        ))
                    );
                } else if io::stdin().is_terminal() {
                    eprintln!("{}", colors::detail_line("✓ Git checkpoint created"));
                }
                Some(cp)
            }
            Ok(None) => {
                eprintln!(
                    "{}",
                    colors::detail_line("⚠ --checkpoint: not a git repo, skipping checkpoint")
                );
                None
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    colors::detail_line(&format!("⚠ Failed to create git checkpoint: {e}"))
                );
                None
            }
        }
    } else {
        None
    };

    // ── Event channel for plan preview (U2) + status bar (U3) ──────────
    let (event_tx, mut event_rx) = mpsc::channel::<LoopEvent>(128);
    let show_plan = options.show_plan;

    // The render task handles panics gracefully via tokio's JoinHandle.
    // A panic here doesn't crash the process — the loop continues.
    let render_task = tokio::spawn(async move {
        let mut status_bar = StatusBar::new();
        while let Some(event) = event_rx.recv().await {
            match &event {
                LoopEvent::PlanReady {
                    plan_brief,
                    ask_plan,
                } => {
                    if show_plan || ask_plan.verdict.should_ask() {
                        loop_events::render_plan_preview(plan_brief, ask_plan, show_plan);
                    }
                }
                LoopEvent::Done { .. } => {
                    status_bar.finish_phase();
                }
                _ => {
                    status_bar.render(&event);
                }
            }
        }
        // Clear stale status line when channel closes (error path)
        status_bar.finish_phase();
    });

    // ── Resume from checkpoint or start fresh ─────────────────────────────
    // When --resume is set, look for an existing checkpoint and continue
    // from where the previous run left off.
    let loop_result = if options.resume {
        let cp_dir = crate::utils::run_snapshots::run_dir(repo_path, &run_id);
        if sruja_agent::cognition::RunCheckpoint::exists(&cp_dir) {
            eprintln!(
                "{}",
                colors::detail_line(&format!("Resuming from checkpoint in {}", cp_dir.display()))
            );
            agent
                .resume_loop(&goal_spec, &loop_config)
                .await
                .map_err(agent_err_to_cli)
        } else {
            // No checkpoint in current run — search for most recent checkpoint
            // across all runs in this repo.
            let runs_dir = repo_path.join(".sruja").join("runs");
            let mut found_checkpoint: Option<std::path::PathBuf> = None;
            if let Ok(entries) = std::fs::read_dir(&runs_dir) {
                let mut checkpoints: Vec<(std::time::SystemTime, std::path::PathBuf)> = entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let cp = e.path().join("checkpoint.json");
                        if cp.exists() {
                            let modified = std::fs::metadata(&cp)
                                .ok()
                                .and_then(|m| m.modified().ok())
                                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                            Some((modified, e.path()))
                        } else {
                            None
                        }
                    })
                    .collect();
                checkpoints.sort_by_key(|b| std::cmp::Reverse(b.0)); // most recent first
                if let Some((_, path)) = checkpoints.first() {
                    found_checkpoint = Some(path.clone());
                }
            }
            if let Some(ref cp_dir) = found_checkpoint {
                eprintln!("  Resuming from checkpoint: {}", cp_dir.display());
                // Temporarily override the checkpoint_dir in loop_config.
                let mut resume_config = loop_config.clone();
                resume_config.checkpoint_dir = Some(cp_dir.clone());
                agent
                    .resume_loop(&goal_spec, &resume_config)
                    .await
                    .map_err(agent_err_to_cli)
            } else {
                eprintln!("  No checkpoint found — starting fresh run");
                agent
                    .run_loop(
                        &goal_spec,
                        &loop_config,
                        Some(&event_tx),
                        calibration_ask_plan.as_ref(),
                    )
                    .await
                    .map_err(agent_err_to_cli)
            }
        }
    } else {
        agent
            .run_loop(
                &goal_spec,
                &loop_config,
                Some(&event_tx),
                calibration_ask_plan.as_ref(),
            )
            .await
            .map_err(agent_err_to_cli)
    };

    // ── Drain event channel (always, even on error) ─────────────────────
    drop(event_tx);
    let _ = render_task.await;

    // ── Print checkpoint restore hint (U5) ──────────────────────────────
    if options.verbose {
        if let Some(ref cp) = checkpoint {
            cp.print_restore_hint();
        }
    }

    let mut result = loop_result?;
    result.grader_source = grader_source;

    // ── Plan-only mode: print the plan text ─────────────────────────────
    if options.plan_only {
        for step in &result.final_result.step_results {
            if step.status == sruja_agent::cognition::StepStatus::Ok && !step.output.is_empty() {
                println!("\n{}", step.output);
            }
        }
    }

    // ── Persist loop trajectory to disk ─────────────────────────────────
    // Write the full LoopResult to .sruja/runs/<run_id>/loop.json so
    // `sruja run show` and `sruja run export` can inspect it post-hoc.
    let trajectory_json = match serde_json::to_value(&result) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("  Warning: could not serialize trajectory: {e}");
            serde_json::Value::Null
        }
    };
    let trajectory_path = write_json_snapshot(repo_path, &run_id, "loop.json", &trajectory_json);
    if options.verbose {
        match trajectory_path {
            Ok(path) => println!("  Trajectory: {}", path.display()),
            Err(e) => eprintln!("  Warning: could not write trajectory: {e}"),
        }
    }

    // ── Post-loop auto-consolidation (U2) ──────────────────────────────
    // Archive stale entries and prune low-utility ones. Runs after
    // trajectory persistence so a crash during consolidation never
    // loses the run. Gated by `auto_consolidate` (default on) and
    // skipped in `--dry-run`.
    if manifest.auto_consolidate && !dry_run {
        match consolidate_memory(repo_path) {
            Ok(msg) => {
                if options.verbose {
                    println!("  {msg}");
                }
            }
            Err(e) => {
                if options.verbose {
                    eprintln!("  Warning: memory consolidation failed: {e}");
                }
            }
        }
    }

    // ── Post-loop changelog (U4) ───────────────────────────────────────
    // Complexity-aware: skip for trivial/direct-execution runs to avoid
    // clutter, unless --changelog forces it.
    let complexity = &result.final_result.comprehension.complexity;
    let should_write_changelog =
        options.changelog || complexity.generate_artifacts() || result.iteration_count() > 1;

    if should_write_changelog {
        let cl = AgentChangelog::from_loop(&result, calibration_ask_plan.as_ref(), dry_run);
        let cl_dir = repo_path.join(".sruja").join("changelogs");
        if let Err(e) = std::fs::create_dir_all(&cl_dir) {
            if options.verbose {
                eprintln!("  Warning: could not create changelogs dir: {e}");
            }
        } else {
            let path = cl_dir.join(cl.filename());
            match std::fs::write(&path, cl.to_markdown()) {
                Ok(()) => {
                    if options.verbose {
                        eprintln!("  Changelog: {}", path.display());
                    }
                }
                Err(e) => {
                    if options.verbose {
                        eprintln!("  Warning: could not write changelog: {e}");
                    }
                }
            }
        }
    }

    // ── Deterministic verification verdict ───────────────────────────────
    let verify_passed = if manifest.verify_steps.is_empty() {
        None
    } else {
        let last = result.iterations.last();
        let passed = last.map(|i| i.verify_failed.is_empty()).unwrap_or(true);
        if !passed {
            eprintln!();
            if let Some(i) = last {
                for f in &i.verify_failed {
                    eprintln!("{} {}", colors::detail_line("✗"), colors::detail_line(f));
                }
            }
        }
        Some(passed)
    };

    // ── Report ────────────────────────────────────────────────────────────
    match options.format {
        "json" => {
            let mut value = serde_json::to_value(&result)?;
            if let Some(vp) = verify_passed {
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("verification_passed".into(), vp.into());
                }
            }
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        _ => {
            print_loop_result_human(&result, options.verbose);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── U2: consolidate_memory ────────────────────────────────────────────

    fn make_learning_entry(
        context: &str,
        hypothesis: &str,
        retrieval_count: u32,
        success: u32,
        total: u32,
        age_days: i64,
    ) -> sruja_agent::LearningEntry {
        use chrono::{Duration, Utc};
        sruja_agent::LearningEntry {
            id: sruja_agent::generate_entry_id(),
            kind: None,
            timestamp: Utc::now() - Duration::days(age_days),
            run_id: None,
            repo: None,
            selector: None,
            context: context.to_string(),
            hypothesis: hypothesis.to_string(),
            outcome: sruja_agent::ExperimentOutcome::Failed,
            reason: None,
            guardrail_advice: String::new(),
            affected_elements: vec![],
            evidence_refs: vec![],
            confidence: None,
            tags: vec![],
            hitl_kind: None,
            related_ids: vec![],
            retrieval_count,
            task_success_after: success,
            task_total_after: total,
            category: None,
            signals_match: vec![],
            constraints: None,
            validation: vec![],
            blast_radius: None,
        }
    }

    fn make_invariant_entry(context: &str) -> sruja_agent::LearningEntry {
        let mut e = make_learning_entry(context, "invariant hyp", 10, 0, 10, 60);
        e.kind = Some(sruja_agent::LearningKind::Invariant);
        e
    }

    #[test]
    fn consolidate_archives_stale_entries() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        let mut mem = sruja_agent::AgenticMemory::default();

        // Stale: old age + low decay score (90-day half-life, need age >> 90 days).
        mem.add_learning(make_learning_entry(
            "stale context",
            "stale hypothesis",
            1,
            1,
            1,
            365, // 365 days old → decay ≈ 0.06 (well below 0.15)
        ));
        // Fresh: recent entry.
        mem.add_learning(make_learning_entry(
            "fresh context",
            "fresh hypothesis",
            1,
            1,
            1,
            1, // 1 day old → decay ≈ 0.99
        ));
        mem.save(repo).unwrap();

        let summary = consolidate_memory(repo).unwrap();
        assert!(summary.contains("archived 1 stale"), "Summary: {summary}");
        assert!(
            summary.contains("pruned 0 low-utility"),
            "Summary: {summary}"
        );

        let loaded = sruja_agent::AgenticMemory::load(repo).unwrap();
        assert_eq!(loaded.learnings.len(), 1, "Only fresh entry should remain");
        assert_eq!(loaded.learnings[0].context, "fresh context");
    }

    #[test]
    fn consolidate_prunes_low_utility_entries() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        let mut mem = sruja_agent::AgenticMemory::default();

        // Low utility: retrieved 3+ times, success < 25%.
        mem.add_learning(make_learning_entry(
            "low utility ctx",
            "low utility hyp",
            5,  // retrieval_count >= 3
            1,  // 1 success / 4 total = 25% — need < 25%, so 1/5 = 20%
            5,  // total = 5
            10, // recent enough to not be stale
        ));
        // High utility: never retrieved.
        mem.add_learning(make_learning_entry(
            "high utility ctx",
            "high utility hyp",
            0,
            0,
            0,
            10,
        ));
        mem.save(repo).unwrap();

        let summary = consolidate_memory(repo).unwrap();
        assert!(
            summary.contains("pruned 1 low-utility"),
            "Summary: {summary}"
        );

        let loaded = sruja_agent::AgenticMemory::load(repo).unwrap();
        assert_eq!(loaded.learnings.len(), 1);
        assert_eq!(loaded.learnings[0].context, "high utility ctx");
    }

    #[test]
    fn consolidate_preserves_invariant_entries() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        let mut mem = sruja_agent::AgenticMemory::default();

        // Invariant that would be low utility — should be preserved.
        mem.add_learning(make_invariant_entry("invariant must not be pruned"));
        mem.save(repo).unwrap();

        let summary = consolidate_memory(repo).unwrap();
        assert!(
            summary.contains("pruned 0"),
            "Invariants must not be pruned: {summary}"
        );

        let loaded = sruja_agent::AgenticMemory::load(repo).unwrap();
        assert_eq!(loaded.learnings.len(), 1);
    }

    #[test]
    fn consolidate_no_op_when_empty() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();

        // No memory file exists.
        let summary = consolidate_memory(repo).unwrap();
        assert!(summary.contains("archived 0 stale"));
        assert!(summary.contains("pruned 0 low-utility"));
        assert!(summary.contains("0 entries remain"));
    }

    // ── U3: print_loop_result_human observability ─────────────────────────

    #[test]
    fn print_loop_result_shows_applied_learnings() {
        use sruja_agent::llm::Usage;
        use sruja_agent::{Comprehension, Critique, LoopResult, LoopTermination};

        let result = LoopResult {
            goal: "test goal".to_string(),
            converged: true,
            termination: LoopTermination::Approved,
            iterations: vec![],
            final_result: sruja_agent::AgentRunResult {
                goal: "test goal".to_string(),
                comprehension: Comprehension {
                    goal: "test goal".to_string(),
                    summary: "test".to_string(),
                    cited_elements: vec![],
                    key_findings: vec![],
                    risks: vec![],
                    usage: Usage::default(),
                    retrieved_learning_ids: vec!["lrn_abc".to_string(), "lrn_def".to_string()],
                    complexity: sruja_agent::TaskComplexity::default(),
                    pre_conditions: vec![],
                },
                plan: sruja_agent::Plan {
                    goal: "test goal".to_string(),
                    goal_statement: "test goal".to_string(),
                    criteria: vec![],
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: sruja_agent::TaskComplexity::default(),
                },
                step_results: vec![],
                critique: Some(Critique {
                    approved: true,
                    score: 0.9,
                    issues: vec![],
                    suggestions: vec![],
                    usage: Usage::default(),
                    persona_breakdown: vec![],
                    injected_learning_ids: vec!["lrn_abc".to_string()],
                    criteria: vec![],
                    source: String::new(),
                }),
                decision: None,
                runbook: None,
                total_usage: Usage::default(),
            },
            total_usage: Usage::default(),
            grader_source: "default".to_string(),
        };

        // Should not panic.
        print_loop_result_human(&result, false);
    }

    #[test]
    fn print_loop_result_shows_hint_when_no_learnings_on_failure() {
        use sruja_agent::llm::Usage;
        use sruja_agent::{Comprehension, Critique, LoopResult, LoopTermination};

        let result = LoopResult {
            goal: "test goal".to_string(),
            converged: false,
            termination: LoopTermination::MaxIterations,
            iterations: vec![],
            final_result: sruja_agent::AgentRunResult {
                goal: "test goal".to_string(),
                comprehension: Comprehension {
                    goal: "test goal".to_string(),
                    summary: "test".to_string(),
                    cited_elements: vec![],
                    key_findings: vec![],
                    risks: vec![],
                    usage: Usage::default(),
                    retrieved_learning_ids: vec![],
                    complexity: sruja_agent::TaskComplexity::default(),
                    pre_conditions: vec![],
                },
                plan: sruja_agent::Plan {
                    goal: "test goal".to_string(),
                    goal_statement: "test goal".to_string(),
                    criteria: vec![],
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                    schema_version: String::new(),
                    complexity: sruja_agent::TaskComplexity::default(),
                },
                step_results: vec![],
                critique: Some(Critique {
                    approved: false,
                    score: 0.3,
                    issues: vec!["bad".to_string()],
                    suggestions: vec![],
                    usage: Usage::default(),
                    persona_breakdown: vec![],
                    injected_learning_ids: vec![],
                    criteria: vec![],
                    source: String::new(),
                }),
                decision: None,
                runbook: None,
                total_usage: Usage::default(),
            },
            total_usage: Usage::default(),
            grader_source: "default".to_string(),
        };

        // Should print the hint, not panic.
        print_loop_result_human(&result, false);
    }
}
