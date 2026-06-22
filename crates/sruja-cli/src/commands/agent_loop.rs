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

use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::sync::Arc;

use sruja_agent::calibration::{self, AskInput, Thresholds};
use sruja_agent::cognition::{Hook, LoopIteration};
use sruja_agent::llm::{OpenAiClient, TieredClient};
use sruja_agent::tool::ToolRegistry;
use sruja_agent::verify::VerifyOptions;
use sruja_agent::{
    AgentConfig, AgentError, GoalSpec, LoopConfig, LoopManifest, ModelMapping, VerifierConfig,
};

use super::CliError;
use crate::config;
use crate::utils::run_id::generate_run_id;
use crate::utils::run_snapshots::write_json_snapshot;

/// Default shell commands the agent is allowed to execute when the user hasn't
/// configured an explicit `shell_allowlist` in `.sruja/loop.toml`. These are
/// the most common safe, non-destructive tools for a coding agent.
const DEFAULT_SHELL_ALLOWLIST: &[&str] = &["cargo", "git"];

/// A hook that prints per-iteration progress to stderr during `sruja agent loop`.
/// Only active when stdin is a TTY (interactive mode).
struct ProgressHook;

#[async_trait::async_trait]
impl Hook for ProgressHook {
    async fn before_iteration(&self, iteration: usize, max_iterations: usize) {
        eprintln!("  [{iteration}/{max_iterations}] planning...");
    }

    async fn after_iteration(
        &self,
        iteration: usize,
        max_iterations: usize,
        result: &LoopIteration,
    ) {
        let mark = if result.critique_approved {
            "PASS"
        } else {
            "FAIL"
        };
        let cost = result.usage.estimated_cost_usd();
        eprintln!(
            "  [{iteration}/{max_iterations}] {mark} | {} subtasks, {} ok, {} failed | score {:.1} | ~${cost:.4}",
            result.subtask_count, result.succeeded, result.failed, result.critique_score,
        );
        for issue in &result.critique_issues {
            eprintln!("         issue: {issue}");
        }
    }
}

/// Outcome of the pre-flight calibration gate.
#[derive(Debug)]
pub(crate) enum GateOutcome {
    /// Calibration says halt — human approval required.
    Halt { reason: String },
    /// Calibration says proceed — optional DR already constructed.
    Proceed {
        plan: Box<sruja_agent::AskPlan>,
        record: Option<Box<sruja_agent::cognition::DecisionRecord>>,
    },
}

/// Pure calibration gate: decides Halt vs Proceed from goal scope + thresholds.
///
/// No async, no LLM, no I/O — fully unit-testable.
pub(crate) fn calibration_gate(
    goal: &str,
    target_elements: &[String],
    target_files: &[String],
    has_precedent: bool,
    thresholds: &Thresholds,
    force_proceed: bool,
) -> GateOutcome {
    // Heuristic blast radius: target elements + target files, saturated at u16::MAX.
    let blast_radius = (target_elements.len() + target_files.len()).min(u16::MAX as usize) as u16;

    // Infer reversibility from the goal text (conservative: keywords).
    let reversibility = calibration::infer_reversibility(calibration::TargetHints {
        kind: "Goal",
        label: goal,
    });

    let input = AskInput {
        reversibility,
        blast_radius,
        confidence: None,
        trust_level: None,
        has_precedent,
        policy_says_ask: false,
    };

    let plan = calibration::decide(&input, thresholds);

    if plan.verdict.should_ask() {
        if force_proceed {
            // Forced bypass — proceed but write no calibration DR.
            GateOutcome::Proceed {
                plan: Box::new(plan),
                record: None,
            }
        } else {
            GateOutcome::Halt {
                reason: plan.reason.clone(),
            }
        }
    } else {
        let record = sruja_agent::proceed_decision_record(&plan, goal).map(Box::new);
        GateOutcome::Proceed {
            plan: Box::new(plan),
            record,
        }
    }
}

/// Check whether agentic memory contains a precedent learning *relevant to
/// this goal*. Scoped to avoid a single global precedent from unlocking every
/// one-way-door goal. Relevance is a simple text-contains match on the goal
/// or target element IDs — consistent with `Memory::search` semantics.
fn has_goal_precedent(repo_path: &Path, goal: &str, target_elements: &[String]) -> bool {
    let mem = match sruja_agent::AgenticMemory::load(repo_path) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let goal_lower = goal.to_lowercase();
    mem.learnings.iter().any(|l| {
        if l.hitl_kind.as_deref() != Some("precedent") {
            return false;
        }
        // Precedent is relevant if any target element matches, or if the goal
        // text overlaps with the learning's context/hypothesis.
        let ctx_lower = l.context.to_lowercase();
        let hyp_lower = l.hypothesis.to_lowercase();
        target_elements
            .iter()
            .any(|e| ctx_lower.contains(&e.to_lowercase()) || hyp_lower.contains(&e.to_lowercase()))
            || ctx_lower.contains(&goal_lower)
            || hyp_lower.contains(&goal_lower)
            || goal_lower.contains(&ctx_lower)
    })
}

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
}

/// Entry point for `sruja agent loop`.
pub async fn agent_loop(options: &AgentLoopOptions<'_>) -> Result<(), CliError> {
    let repo_path = Path::new(&options.repo);

    // ── Load .sruja/loop.toml for defaults ────────────────────────────────
    let manifest = LoopManifest::load_from_path(repo_path);

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
            let client = OpenAiClient::new(&tier_cfg.api_key, &tier_cfg.base_url, &tier_cfg.model)
                .map_err(|e| CliError::validation(format!("Failed to create LLM client: {e}")))?;
            tiered = tiered.with_route(&tier_cfg.model, Arc::new(client));
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

    let goal_prompt = goal_spec.to_prompt();

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

    let config = AgentConfig {
        models,
        tdd,
        review_every_change: manifest.review_every_change,
        dry_run,
        system_hints,
        ..Default::default()
    };

    let agent = {
        let mut builder = sruja_agent::Agent::builder()
            .llm(Arc::new(tiered))
            .tools(tools)
            .config(config)
            .memory(repo_path);

        // Connect to declared MCP servers (graceful degradation if none/failed)
        if !manifest.mcp.servers.is_empty() {
            builder = builder
                .with_mcp(&manifest, repo_path.to_path_buf())
                .await
                .map_err(|e| CliError::validation(format!("MCP initialization error: {e}")))?;
        }

        if io::stdin().is_terminal() {
            builder = builder.hook(Box::new(ProgressHook));
        }
        builder.build().map_err(agent_err_to_cli)?
    };

    // ── Run the loop ──────────────────────────────────────────────────────
    let run_id = generate_run_id();

    if io::stdin().is_terminal() {
        eprintln!();
        eprintln!("Goal: {}", options.goal);
        eprintln!("Model: {model}");
        eprintln!("Max iterations: {max_iterations}");
        eprintln!();
    }

    let spend_cap_usd = options.spend_cap_usd.or(manifest.spend_cap_usd);
    let detect_oscillation = if options.no_oscillation_detection {
        false
    } else {
        manifest.detect_oscillation
    };

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

    let loop_config = LoopConfig {
        max_iterations,
        spend_cap_usd,
        detect_oscillation,
        verifier,
        ..Default::default()
    };

    // ── Calibration gate (pre-flight) ────────────────────────────────────
    // The grader (deterministic calibration) decides whether the actor
    // (agent loop) should proceed autonomously or halt for human approval.
    // This is the loop-engineering thesis: the actor asks the grader
    // "should I proceed?" before it starts.
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
            eprintln!("⛔  Calibration gate: HALT");
            eprintln!("   {reason}");

            // Interactive prompt when stdin is a TTY (not piped/redirected).
            if io::stdin().is_terminal() {
                eprintln!("   Proceed anyway? [y/N]: ");
                eprint!("   ");
                io::stderr().flush().ok();
                let mut input = String::new();
                io::stdin().read_line(&mut input).ok();
                let choice = input.trim().to_lowercase();
                if choice != "y" && choice != "yes" {
                    eprintln!("   Aborted. Use --yes to force (no calibration DR).");
                    return Ok(());
                }
                eprintln!("   Proceeding despite calibration Ask (forced by user).");
            } else {
                eprintln!("   Use --yes to override (no calibration DR will be written).");
                return Ok(());
            }
        }
        GateOutcome::Proceed { plan, record } => {
            println!("✓  Calibration gate: PROCEED ({:?})", plan.verdict);
            // Write the calibration DR to .sruja/decisions/ if present.
            if let Some(dr) = record {
                let decisions_dir = repo_path.join(".sruja").join("decisions");
                if let Err(e) = std::fs::create_dir_all(&decisions_dir) {
                    eprintln!("   Warning: could not create decisions dir: {e}");
                } else {
                    let path = decisions_dir.join(dr.filename());
                    if let Err(e) = std::fs::write(&path, dr.to_markdown()) {
                        eprintln!("   Warning: could not write calibration DR: {e}");
                    } else {
                        println!("   Calibration DR: {}", path.display());
                    }
                }
            }
        }
    }

    let mut result = agent
        .run_loop(&goal_prompt, &loop_config)
        .await
        .map_err(agent_err_to_cli)?;

    result.grader_source = grader_source;

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
    match trajectory_path {
        Ok(path) => println!("  Trajectory: {}", path.display()),
        Err(e) => eprintln!("  Warning: could not write trajectory: {e}"),
    }

    // ── Deterministic verification verdict ───────────────────────────────
    // The verifier already ran in-loop on every iteration (when configured),
    // so we derive the final verdict from the loop result rather than
    // re-running expensive steps like `cargo test` a second time.
    let verify_passed = if manifest.verify_steps.is_empty() {
        None
    } else {
        // The verifier vetoed convergence iff the final iteration recorded
        // verify failures.
        let last = result.iterations.last();
        let passed = last.map(|i| i.verify_failed.is_empty()).unwrap_or(true);
        if passed {
            println!("✓  Verification passed (in-loop grader)");
        } else {
            eprintln!("⚠  Verification FAILED (in-loop grader vetoed convergence):");
            if let Some(i) = last {
                for f in &i.verify_failed {
                    eprintln!("  ✗ {f}");
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
            print_loop_result_human(&result);
            if let Some(passed) = verify_passed {
                println!();
                if passed {
                    println!("Verification: PASSED");
                } else {
                    println!("Verification: FAILED (loop result may be unreliable)");
                }
            }
        }
    }

    Ok(())
}

fn print_loop_result_human(result: &sruja_agent::LoopResult) {
    let status = if result.converged {
        "CONVERGED"
    } else {
        "NOT CONVERGED"
    };
    println!("═══════════════════════════════════════════");
    println!("  Agent Loop: {status}");
    println!("═══════════════════════════════════════════");
    println!();
    println!("Goal: {}", result.goal);
    println!(
        "Iterations: {} | Termination: {:?}",
        result.iteration_count(),
        result.termination
    );
    println!(
        "Tokens: {} prompt + {} completion = {} total (~${:.4})",
        result.total_usage.prompt_tokens,
        result.total_usage.completion_tokens,
        result.total_usage.total_tokens,
        result.total_usage.estimated_cost_usd()
    );
    println!();

    for iter in &result.iterations {
        let mark = if iter.critique_approved {
            "PASS"
        } else {
            "FAIL"
        };
        println!(
            "  [{}/{}] {} | plan:{} succeed:{} failed:{} score:{:.1} {}",
            iter.iteration,
            result.iteration_count(),
            mark,
            iter.subtask_count,
            iter.succeeded,
            iter.failed,
            iter.critique_score,
            if iter.replanned { "(replanned)" } else { "" }
        );
        for issue in &iter.critique_issues {
            println!("         issue: {issue}");
        }
    }

    println!();
    if let Some(critique) = result.final_result.critique.as_ref() {
        println!(
            "Final critique: score={:.1} approved={}",
            critique.score, critique.approved
        );
        if !critique.issues.is_empty() {
            println!("Issues:");
            for issue in &critique.issues {
                println!("  - {issue}");
            }
        }
    }
}

fn agent_err_to_cli(e: AgentError) -> CliError {
    CliError::validation(format!("Agent error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_agent::Verdict;

    fn t() -> Thresholds {
        Thresholds::default()
    }

    #[test]
    fn one_way_door_no_precedent_no_force_halts() {
        let goal = "migrate the database schema";
        let outcome = calibration_gate(goal, &[], &[], false, &t(), false);
        match outcome {
            GateOutcome::Halt { reason } => assert!(reason.contains("One-way door")),
            GateOutcome::Proceed { .. } => expected_halt(),
        }
    }

    #[test]
    fn one_way_door_no_precedent_force_proceeds_without_dr() {
        let goal = "migrate the database schema";
        let outcome = calibration_gate(goal, &[], &[], false, &t(), true);
        match outcome {
            GateOutcome::Proceed { plan, record } => {
                assert_eq!(plan.verdict, Verdict::Ask);
                assert!(record.is_none(), "forced bypass should write no DR");
            }
            GateOutcome::Halt { .. } => expected_proceed(),
        }
    }

    #[test]
    fn two_way_door_bounded_blast_proceeds_silent_no_dr() {
        let goal = "rename a variable in the handler";
        let outcome = calibration_gate(goal, &[], &[], false, &t(), false);
        match outcome {
            GateOutcome::Proceed { plan, record } => {
                assert_eq!(plan.verdict, Verdict::ProceedSilent);
                assert!(record.is_none(), "ProceedSilent should write no DR");
            }
            GateOutcome::Halt { .. } => expected_proceed(),
        }
    }

    #[test]
    fn mid_confidence_proceeds_with_flag_and_dr() {
        // Mid-confidence requires a confidence signal; with None (unmeasured)
        // on a two-way door we get ProceedSilent. With precedent we get
        // ProceedCitingPrecedent and a DR. Test the precedent path.
        let goal = "refactor API handler";
        let outcome = calibration_gate(goal, &[], &[], true, &t(), false);
        match outcome {
            GateOutcome::Proceed { plan, record } => {
                assert_eq!(plan.verdict, Verdict::ProceedCitingPrecedent);
                assert!(record.is_some(), "ProceedCitingPrecedent should write a DR");
            }
            GateOutcome::Halt { .. } => expected_proceed(),
        }
    }

    #[test]
    fn precedent_proceeds_with_dr() {
        let goal = "delete old migration files";
        let outcome = calibration_gate(goal, &[], &[], true, &t(), false);
        match outcome {
            GateOutcome::Proceed { plan, record } => {
                assert_eq!(plan.verdict, Verdict::ProceedCitingPrecedent);
                assert!(record.is_some(), "precedent path should write a DR");
            }
            GateOutcome::Halt { .. } => expected_proceed(),
        }
    }

    fn expected_halt() {
        panic!("expected Halt but got Proceed");
    }

    fn expected_proceed() {
        panic!("expected Proceed but got Halt");
    }

    #[test]
    fn default_shell_allowlist_has_cargo_and_git() {
        assert!(DEFAULT_SHELL_ALLOWLIST.contains(&"cargo"));
        assert!(DEFAULT_SHELL_ALLOWLIST.contains(&"git"));
    }
}
