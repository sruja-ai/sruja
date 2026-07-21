//! The main `agent_loop()` execution logic.

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

use super::super::loop_checkpoint::{self, GitCheckpoint};
use super::super::loop_events::{self, StatusBar};
use super::super::loop_report::LiveReportHook;
use super::super::CliError;
use crate::config;
use crate::utils::colors;
use crate::utils::run_id::generate_run_id;
use crate::utils::run_snapshots::write_json_snapshot;

use super::calibration::has_goal_precedent;
use super::calibration::{calibration_gate, model_family, GateOutcome};
use super::config::{
    AgentLoopOptions, ARCH_CONTEXT_MAX_TOKENS, DEFAULT_SHELL_ALLOWLIST, PRELOAD_MAX_BYTES,
};
use super::output::print_loop_result_human;
use super::utils::{agent_err_to_cli, consolidate_memory, preloaded_architecture_context};

/// Entry point for `sruja agent loop`.
pub async fn agent_loop(options: &AgentLoopOptions<'_>) -> Result<(), CliError> {
    let repo_path = Path::new(&options.repo);

    let mut manifest = LoopManifest::load_from_path(repo_path);

    if let Some(ref pipeline_path) = options.pipeline_override {
        let content = std::fs::read_to_string(pipeline_path)
            .map_err(|e| CliError::validation(format!("cannot read pipeline file: {e}")))?;
        let pipeline: sruja_agent::PipelineConfig = toml::from_str(&content)
            .map_err(|e| CliError::validation(format!("cannot parse pipeline file: {e}")))?;
        manifest.pipeline = pipeline;
    }

    if options.show_pipeline {
        let pipeline = &manifest.pipeline;
        println!(
            "{}",
            serde_json::to_string_pretty(pipeline)
                .map_err(|e| CliError::validation(format!("pipeline serialization: {e}")))?
        );
        return Ok(());
    }

    if options.plan_only {
        eprintln!(
            "{}",
            colors::detail_line("Plan-only mode: producing plan without code changes.")
        );
    }

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

    let multi_config = config::resolve_multi_provider_config(repo_path)?;

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

    let models = ModelMapping {
        cheap: multi_config.cheap.model.clone(),
        mid: multi_config.mid.model.clone(),
        premium: multi_config.premium.model.clone(),
        review: multi_config.review.model.clone(),
    };

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
        let needs_own_client =
            tier_cfg.api_key != mid_config.api_key || tier_cfg.base_url != mid_config.base_url;

        if needs_own_client {
            let client = Arc::new(
                OpenAiClient::new(&tier_cfg.api_key, &tier_cfg.base_url, &tier_cfg.model).map_err(
                    |e| CliError::validation(format!("Failed to create LLM client: {e}")),
                )?,
            );
            tiered = tiered.with_route(&tier_cfg.model, client.clone());
            let family = model_family(&tier_cfg.model);
            if !family.is_empty() {
                tiered = tiered.with_provider_name_contains(family, client);
            }
        }
    }

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

    let compressing = sruja_agent::llm::CompressingClient::new(Arc::new(tiered));
    let ccr_store = compressing.ccr_store();

    tracing::info!("agent_loop: context compression enabled (TextCrusher + CCR)");

    let retrying = sruja_agent::llm::RetryingClient::new(Arc::new(compressing));

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

    let critique_personas = if manifest.critique.personas.is_empty() {
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
        max_consecutive_tool_only: 3,
        max_consecutive_same_call: 3,
        max_non_converged_fraction: 0.5,
        ..Default::default()
    };

    let run_id = generate_run_id();

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

    let arch_context = preloaded_architecture_context(repo_path, ARCH_CONTEXT_MAX_TOKENS);

    let agent = {
        let mut builder = sruja_agent::Agent::builder()
            .llm(Arc::new(retrying))
            .tools(tools)
            .config(config)
            .memory(repo_path)
            .trace_context(&run_id, &run_id)
            .tool_call_tracer(Box::new(super::super::context_events::ContextEventsTracer))
            .preloaded_files(preloaded_files)
            .preloaded_arch_context(arch_context);

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

    let baseline_path = repo_path.join(".sruja").join("violations.baseline.json");
    if !baseline_path.exists() && !options.no_default_grader && !dry_run {
        let bp = baseline_path.to_string_lossy().to_string();
        match std::process::Command::new(super::super::loop_grader::resolve_sruja_binary().as_str())
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
        let sruja_bin = super::super::loop_grader::resolve_sruja_binary();
        let steps = super::super::loop_grader::default_grader_steps(
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

    if let ("default", Some(ref vc)) = (grader_source.as_str(), &verifier) {
        let sruja_bin = vc
            .options
            .allowed_executables
            .first()
            .map(|s| s.as_str())
            .unwrap_or("sruja");
        if let Err(problems) = super::super::loop_grader::verify_grader_health(repo_path, sruja_bin)
        {
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

    let checkpoint_dir = crate::utils::run_snapshots::run_dir(repo_path, &run_id);

    let pipeline = if options.plan_only {
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

    let calibration_ask_plan: Option<sruja_agent::AskPlan> = match &gate {
        GateOutcome::Proceed { plan, .. } => Some((**plan).clone()),
        GateOutcome::Halt { .. } => None,
    };

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

    let (event_tx, mut event_rx) = mpsc::channel::<LoopEvent>(128);
    let show_plan = options.show_plan;

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
        status_bar.finish_phase();
    });

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
                checkpoints.sort_by_key(|b| std::cmp::Reverse(b.0));
                if let Some((_, path)) = checkpoints.first() {
                    found_checkpoint = Some(path.clone());
                }
            }
            if let Some(ref cp_dir) = found_checkpoint {
                eprintln!("  Resuming from checkpoint: {}", cp_dir.display());
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

    drop(event_tx);
    let _ = render_task.await;

    if options.verbose {
        if let Some(ref cp) = checkpoint {
            cp.print_restore_hint();
        }
    }

    let mut result = loop_result?;
    result.grader_source = grader_source;

    if options.plan_only {
        for step in &result.final_result.step_results {
            if step.status == sruja_agent::cognition::StepStatus::Ok && !step.output.is_empty() {
                println!("\n{}", step.output);
            }
        }
    }

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
