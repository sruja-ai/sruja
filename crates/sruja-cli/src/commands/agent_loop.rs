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
use sruja_agent::cognition::{Hook, HookAction};
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

/// Maximum file size (bytes) for pre-loading into the comprehension prompt.
/// Files larger than this are skipped to avoid blowing up the context window.
const PRELOAD_MAX_BYTES: usize = 50 * 1024; // 50 KB

/// Extract the alphabetic family name from a model identifier.
/// Used for provider prefix routing in the TieredClient.
///   "GLM-5.2" → "glm", "mimo-v2.5-pro" → "mimo",
///   "anthropic/claude-sonnet-4" → "claude"
fn model_family(model: &str) -> String {
    let base = model.rsplit('/').next().unwrap_or(model);
    base.chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>()
        .to_lowercase()
}

// ---------------------------------------------------------------------------
// Progress tracking + steering hook
// ---------------------------------------------------------------------------

/// Phase names for the live report.
fn phase_name(step: &sruja_agent::cognition::Subtask) -> &'static str {
    match step.kind {
        sruja_agent::cognition::SubtaskKind::Comprehend => "comprehend",
        sruja_agent::cognition::SubtaskKind::TestAuthor => "test-author",
        sruja_agent::cognition::SubtaskKind::Implement => "implement",
        sruja_agent::cognition::SubtaskKind::Verify => "verify",
        sruja_agent::cognition::SubtaskKind::AdversarialTest => "adversarial-test",
        sruja_agent::cognition::SubtaskKind::Review => "review",
    }
}

/// Mutable state accumulated across hook calls.
struct ReportState {
    goal: String,
    started_at: std::time::Instant,
    iteration: usize,
    max_iterations: usize,
    current_phase: String,
    subtasks: Vec<SubtaskInfo>,
    critique_score: Option<f64>,
    critique_approved: Option<bool>,
    persona_results: Vec<PersonaInfo>,
    issues: Vec<String>,
    verify_failures: Vec<String>,
    cost_usd: f64,
    steer: bool,
    report_dir: std::path::PathBuf,
    should_stop: bool,
    dirty: bool,
}

#[derive(Clone)]
struct SubtaskInfo {
    id: String,
    description: String,
    kind: String,
    tier: String,
    status: String,
}

#[derive(Clone)]
struct PersonaInfo {
    id: String,
    approved: bool,
    score: f64,
    issue_count: usize,
}

use std::sync::Mutex;

/// A hook that writes a live markdown dashboard and optionally prompts for
/// steering between iterations.
struct LiveReportHook {
    state: Mutex<ReportState>,
}

impl LiveReportHook {
    fn new(goal: &str, max_iterations: usize, steer: bool, report_dir: std::path::PathBuf) -> Self {
        Self {
            state: Mutex::new(ReportState {
                goal: goal.to_string(),
                started_at: std::time::Instant::now(),
                iteration: 0,
                max_iterations,
                current_phase: "starting".into(),
                subtasks: Vec::new(),
                critique_score: None,
                critique_approved: None,
                persona_results: Vec::new(),
                issues: Vec::new(),
                verify_failures: Vec::new(),
                cost_usd: 0.0,
                steer,
                report_dir,
                should_stop: false,
                dirty: false,
            }),
        }
    }

    fn write_report(&self) {
        let mut s = self.state.lock().unwrap();
        if !s.dirty {
            return;
        }
        // Clear dirty flag — next write will only happen when dirty is set
        // again by a state-modifying hook, debouncing consecutive callbacks
        // with no meaningful change (e.g. before_step + after_step).
        s.dirty = false;
        let elapsed = s.started_at.elapsed();
        let mins = elapsed.as_secs() / 60;
        let secs = elapsed.as_secs() % 60;

        let status_icon = match s.critique_approved {
            Some(true) => "PASS",
            Some(false) => "FAIL",
            None => "RUN",
        };

        let mut md = String::new();
        md.push_str(&format!(
            "# Agent Loop — Live Dashboard\n\n\
             Goal: {}\n\n\
             Started: {}m {:02}s ago · Iteration {}/{} · Phase: **{}** · Status: **{}** · Cost: ~${:.4}\n\n",
            s.goal,
            mins,
            secs,
            s.iteration,
            s.max_iterations,
            s.current_phase,
            status_icon,
            s.cost_usd,
        ));

        // Subtask table
        if !s.subtasks.is_empty() {
            md.push_str("## Subtasks\n\n");
            md.push_str("| # | Kind | Tier | Status | Description |\n");
            md.push_str("|---|------|------|--------|-------------|\n");
            for st in &s.subtasks {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    st.id, st.kind, st.tier, st.status, st.description
                ));
            }
            md.push('\n');
        }

        // Critique persona breakdown
        if !s.persona_results.is_empty() {
            md.push_str("## Critique Personas\n\n");
            md.push_str("| Persona | Approved | Score | Issues |\n");
            md.push_str("|---------|----------|-------|--------|\n");
            for p in &s.persona_results {
                let icon = if p.approved { "yes" } else { "NO" };
                md.push_str(&format!(
                    "| {} | {} | {:.1} | {} |\n",
                    p.id, icon, p.score, p.issue_count
                ));
            }
            md.push('\n');
        }

        // Issues
        if !s.issues.is_empty() {
            md.push_str("## Open Issues\n\n");
            for issue in &s.issues {
                md.push_str(&format!("- {issue}\n"));
            }
            md.push('\n');
        }

        // Verify failures
        if !s.verify_failures.is_empty() {
            md.push_str("## Verify Failures (independent grader)\n\n");
            for f in &s.verify_failures {
                md.push_str(&format!("- {f}\n"));
            }
            md.push('\n');
        }

        // Write atomically
        let _ = std::fs::create_dir_all(&s.report_dir);
        let path = s.report_dir.join("LIVE.md");
        let tmp = s.report_dir.join("LIVE.md.tmp");
        if let Err(e) = std::fs::write(&tmp, &md) {
            eprintln!("  Warning: could not write live report: {e}");
            return;
        }
        if let Err(e) = std::fs::rename(&tmp, &path) {
            eprintln!("  Warning: could not rename live report: {e}");
        }
    }

    fn print_summary(&self) {
        let s = self.state.lock().unwrap();
        let mark = match s.critique_approved {
            Some(true) => "PASS",
            Some(false) => "FAIL",
            None => "---",
        };
        let score_str = s
            .critique_score
            .map(|sc| format!("{sc:.1}"))
            .unwrap_or_else(|| "-".into());

        eprintln!(
            "  [{}/{}] {} | {} subtasks | score {} | ~${:.4}",
            s.iteration, s.max_iterations, mark, s.subtasks.len(), score_str, s.cost_usd,
        );

        // Print persona breakdown
        if !s.persona_results.is_empty() {
            for p in &s.persona_results {
                let icon = if p.approved { "+" } else { "X" };
                eprintln!(
                    "    [{icon}] {} score={:.1} issues={}",
                    p.id, p.score, p.issue_count
                );
            }
        }

        // Print issues
        for issue in &s.issues {
            eprintln!("    issue: {issue}");
        }
        for f in &s.verify_failures {
            eprintln!("    verify FAIL: {f}");
        }
    }

    /// Prompt the user for steering input. Returns false if the user wants to stop.
    fn prompt_steer(&self) -> bool {
        let s = self.state.lock().unwrap();
        if !s.steer {
            return true;
        }
        drop(s); // Release lock before stdin

        eprintln!();
        eprintln!("  ── Steering ──");
        eprintln!("  [Enter] continue  ·  [s] stop  ·  [r] show report");
        eprint!("  > ");
        use std::io::Write;
        let _ = std::io::stderr().flush();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            return true;
        }
        match input.trim().to_lowercase().as_str() {
            "s" | "stop" => false,
            "r" | "report" => {
                let s = self.state.lock().unwrap();
                let path = s.report_dir.join("LIVE.md");
                drop(s);
                if let Ok(content) = std::fs::read_to_string(&path) {
                    eprintln!();
                    eprintln!("{content}");
                }
                true
            }
            _ => true,
        }
    }
}

#[async_trait::async_trait]
impl Hook for LiveReportHook {
    async fn before_comprehend(&self, _goal: &str) -> HookAction {
        let mut s = self.state.lock().unwrap();
        s.current_phase = "comprehend".into();
        s.iteration = s.iteration.max(1);
        HookAction::Continue
    }

    async fn after_comprehend(&self, result: &sruja_agent::cognition::Comprehension) -> HookAction {
        let mut s = self.state.lock().unwrap();
        s.current_phase = "plan".into();
        eprintln!(
            "  Comprehend: {} elements cited, {} findings",
            result.cited_elements.len(),
            result.key_findings.len()
        );
        s.dirty = true;
        drop(s);
        self.write_report();
        HookAction::Continue
    }

    async fn before_plan(&self, _goal: &str) -> HookAction {
        // Check if the user requested to stop during steering.
        // before_plan fires at the start of every iteration (plan or replan),
        // so this catches the stop flag set by the prior after_iteration.
        let s = self.state.lock().unwrap();
        if s.should_stop {
            eprintln!("  Stopped by user.");
            return HookAction::Abort("Stopped by user.".into());
        }
        HookAction::Continue
    }

    async fn after_plan(&self, plan: &mut sruja_agent::cognition::Plan) -> HookAction {
        let mut s = self.state.lock().unwrap();
        s.current_phase = "execute".into();
        s.subtasks = plan
            .subtasks
            .iter()
            .map(|st| SubtaskInfo {
                id: st.id.clone(),
                description: st.description.chars().take(60).collect(),
                kind: format!("{:?}", st.kind).to_lowercase(),
                tier: format!("{:?}", st.tier).to_lowercase(),
                status: "pending".into(),
            })
            .collect();
        eprintln!("  Plan: {} subtasks, {} risks", plan.subtasks.len(), plan.risks.len());
        s.dirty = true;
        drop(s);
        self.write_report();
        HookAction::Continue
    }

    async fn before_step(&self, step: &sruja_agent::cognition::Subtask) -> HookAction {
        let mut s = self.state.lock().unwrap();
        s.current_phase = phase_name(step).into();
        if let Some(st) = s.subtasks.iter_mut().find(|st| st.id == step.id) {
            st.status = "running".into();
        }
        eprintln!("  Step {}: {} ({})", step.id, phase_name(step), format!("{:?}", step.tier).to_lowercase());
        s.dirty = true;
        drop(s);
        self.write_report();
        HookAction::Continue
    }

    async fn after_step(
        &self,
        step: &sruja_agent::cognition::Subtask,
        result: &sruja_agent::cognition::StepResult,
    ) {
        let mut s = self.state.lock().unwrap();
        let status = match result.status {
            sruja_agent::cognition::StepStatus::Ok => "done",
            sruja_agent::cognition::StepStatus::Failed => "FAILED",
            sruja_agent::cognition::StepStatus::Skipped => "skipped",
        };
        if let Some(st) = s.subtasks.iter_mut().find(|st| st.id == step.id) {
            st.status = status.into();
        }
        s.cost_usd += result.usage.estimated_cost_usd();
        s.dirty = true;
        drop(s);
        self.write_report();
    }

    async fn before_review(&self) -> HookAction {
        let mut s = self.state.lock().unwrap();
        s.current_phase = "critique".into();
        eprintln!("  Critique: running persona ensemble...");
        s.dirty = true;
        drop(s);
        self.write_report();
        HookAction::Continue
    }

    async fn after_review(&self, critique: &sruja_agent::cognition::Critique) -> HookAction {
        let mut s = self.state.lock().unwrap();
        s.current_phase = "done".into();
        s.critique_score = Some(critique.score);
        s.critique_approved = Some(critique.approved);
        s.issues = critique.issues.clone();
        s.cost_usd += critique.usage.estimated_cost_usd();
        s.persona_results = critique
            .persona_breakdown
            .iter()
            .map(|p| PersonaInfo {
                id: p.id.clone(),
                approved: p.approved,
                score: p.score,
                issue_count: p.issues.len(),
            })
            .collect();
        s.dirty = true;
        drop(s);

        self.print_summary();
        self.write_report();

        HookAction::Continue
    }

    async fn before_iteration(&self, iteration: usize, max_iterations: usize) {
        let mut s = self.state.lock().unwrap();
        s.iteration = iteration;
        s.max_iterations = max_iterations;
        s.current_phase = if iteration == 1 { "comprehend" } else { "replan" }.into();
        s.dirty = true;
        drop(s);
        self.write_report();
    }

    async fn after_iteration(
        &self,
        iteration: usize,
        max_iterations: usize,
        result: &sruja_agent::cognition::LoopIteration,
    ) {
        {
            let mut s = self.state.lock().unwrap();
            s.iteration = iteration;
            s.max_iterations = max_iterations;
            s.critique_score = Some(result.critique_score);
            s.critique_approved = Some(result.critique_approved);
            s.issues = result.critique_issues.clone();
            s.verify_failures = result.verify_failed.clone();
            s.cost_usd = result.usage.estimated_cost_usd();
            s.dirty = true;
        }
        self.write_report();

        // Steering prompt — if the user chose to stop, set the flag.
        // before_plan (called at the start of the next iteration) will
        // check this flag and return HookAction::Abort.
        if !self.prompt_steer() {
            let mut s = self.state.lock().unwrap();
            s.should_stop = true;
        }
    }

    async fn on_error(&self, error: &sruja_agent::AgentError) {
        eprintln!("  ERROR: {error}");
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
    pub steer: bool,
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
            let client = Arc::new(
                OpenAiClient::new(&tier_cfg.api_key, &tier_cfg.base_url, &tier_cfg.model)
                    .map_err(|e| CliError::validation(format!("Failed to create LLM client: {e}")))?,
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

    let agent = {
        let mut builder = sruja_agent::Agent::builder()
            .llm(Arc::new(tiered))
            .tools(tools)
            .config(config)
            .memory(repo_path)
            .trace_context(&run_id, &run_id)
            .tool_call_tracer(Box::new(super::context_events::ContextEventsTracer))
            .preloaded_files(preloaded_files);

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

    // ── Default grader health check ───────────────────────────────────────
    // Verifies the sruja binary and the default grader toolchain work before
    // the agent loop starts. Warnings only — the loop still runs even if the
    // grader is misconfigured (e.g. under --no-default-grader).
    if let ("default", Some(ref vc)) = (grader_source.as_str(), &verifier) {
        let sruja_bin = vc.options.allowed_executables.first().map(|s| s.as_str()).unwrap_or("sruja");
        if let Err(problems) = super::loop_grader::verify_grader_health(repo_path, sruja_bin) {
            eprintln!("⚠️  Default grader health check:");
            for p in &problems {
                eprintln!("   • {p}");
            }
            eprintln!(
                "   The agent loop will still run, but verification results may be unreliable."
            );
        }
    }

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
        .run_loop(&goal_spec, &loop_config)
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

    // ── Post-loop auto-consolidation (U2) ──────────────────────────────
    // Archive stale entries and prune low-utility ones. Runs after
    // trajectory persistence so a crash during consolidation never
    // loses the run. Gated by `auto_consolidate` (default on) and
    // skipped in `--dry-run`.
    if manifest.auto_consolidate && !dry_run {
        match consolidate_memory(repo_path) {
            Ok(msg) => println!("  {msg}"),
            Err(e) => eprintln!("  Warning: memory consolidation failed: {e}"),
        }
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

    // U3: Memory retrieval observability.
    let comp_ids = &result.final_result.comprehension.retrieved_learning_ids;
    let crit_ids: Vec<&str> = result
        .final_result
        .critique
        .as_ref()
        .map(|c| c.injected_learning_ids.iter().map(String::as_str).collect())
        .unwrap_or_default();
    let total = comp_ids.len() + crit_ids.len();
    if total > 0 {
        let mut all_ids: Vec<&str> = comp_ids.iter().map(String::as_str).collect();
        all_ids.extend(&crit_ids);
        all_ids.sort_unstable();
        all_ids.dedup();
        println!(
            "\nMemory: applied {} past learning{} ({})",
            all_ids.len(),
            if all_ids.len() == 1 { "" } else { "s" },
            all_ids.join(", ")
        );
    } else if !result.converged {
        println!("\nMemory: no relevant learnings found. Record one: sruja agent record ...");
    }
}

fn agent_err_to_cli(e: AgentError) -> CliError {
    CliError::validation(format!("Agent error: {e}"))
}

/// Post-loop memory consolidation (U2).
///
/// Archives stale entries (decay < 0.15, age > 30 days) and prunes
/// low-utility entries (retrieved ≥ 3×, success < 25%). Invariant
/// entries are never touched. Returns a human-readable summary.
fn consolidate_memory(repo_path: &Path) -> Result<String, CliError> {
    use sruja_agent::AgenticMemory;

    let mut memory = AgenticMemory::load(repo_path).unwrap_or_default();

    // 1. Archive stale entries.
    let archived = memory.auto_archive_stale(0.15, 30);
    let archived_count = archived.len();

    // 2. Prune low-utility entries (skip invariants).
    let low_utility: Vec<String> = memory
        .low_utility_entries(3, 0.25)
        .into_iter()
        .filter(|e| e.kind != Some(sruja_agent::LearningKind::Invariant))
        .map(|e| e.id.clone())
        .collect();
    let pruned_count = low_utility.len();
    for id in &low_utility {
        let _ = memory.delete_learning(id);
    }

    // 3. Save if anything changed.
    if archived_count > 0 || pruned_count > 0 {
        memory.save(repo_path).map_err(|e| {
            CliError::validation(format!("Failed to save consolidated memory: {e}"))
        })?;
    }

    let remaining = memory.learnings.len();
    Ok(format!(
        "Memory: archived {archived_count} stale, pruned {pruned_count} low-utility ({remaining} entries remain)"
    ))
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
                }),
                decision: None,
                runbook: None,
                total_usage: Usage::default(),
            },
            total_usage: Usage::default(),
            grader_source: "default".to_string(),
        };

        // Should not panic.
        print_loop_result_human(&result);
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
                }),
                decision: None,
                runbook: None,
                total_usage: Usage::default(),
            },
            total_usage: Usage::default(),
            grader_source: "default".to_string(),
        };

        // Should print the hint, not panic.
        print_loop_result_human(&result);
    }
}
