use crate::commands;
use crate::commands::CliError;

use super::super::subcommands::AgentCommand;

pub(super) async fn handle(cmd: AgentCommand) -> Result<(), CliError> {
    match cmd {
        AgentCommand::Task {
            goal,
            repo,
            max_iterations,
            dry_run,
            yes,
            pipeline,
            resume,
            format,
            show_details,
        } => {
            commands::auto_run(
                &repo,
                &goal,
                max_iterations,
                dry_run,
                yes,
                pipeline.as_deref(),
                resume,
                &format,
                show_details,
            )
            .await
        }
        AgentCommand::Setup {
            repo,
            provider,
            api_key,
            model,
        } => commands::agent_setup::agent_setup(
            &repo,
            provider.as_deref(),
            api_key.as_deref(),
            model.as_deref(),
        ),
        AgentCommand::History {
            repo,
            element_id,
            format,
        } => commands::agent_history(&repo, element_id.as_deref(), &format).await,
        AgentCommand::Record {
            repo,
            context,
            hypothesis,
            outcome,
            guardrail,
            reason,
            elements,
            hitl_kind,
        } => {
            commands::agent_record(
                &repo,
                &context,
                &hypothesis,
                &outcome,
                &guardrail,
                reason.as_deref(),
                elements.as_deref(),
                hitl_kind.as_deref(),
            )
            .await
        }
        AgentCommand::Clear { repo, force } => commands::agent_clear(&repo, force).await,
        AgentCommand::Clusters {
            repo,
            entry_id,
            tag,
            format,
        } => {
            commands::agent_clusters(&repo, entry_id.as_deref(), tag.as_deref(), &format).await
        }
        AgentCommand::Curate { repo, format } => commands::agent_curate(&repo, &format).await,
        AgentCommand::Update {
            repo,
            id,
            context,
            hypothesis,
            outcome,
            guardrail,
            reason,
        } => {
            commands::agent_update(
                &repo,
                &id,
                context.as_deref(),
                hypothesis.as_deref(),
                outcome.as_deref(),
                guardrail.as_deref(),
                reason.as_deref(),
            )
            .await
        }
        AgentCommand::Delete { repo, id, force } => {
            commands::agent_delete(&repo, &id, force).await
        }
        AgentCommand::Merge {
            repo,
            ids,
            context,
            hypothesis,
            guardrail,
            outcome,
        } => {
            commands::agent_merge(&repo, &ids, &context, &hypothesis, &guardrail, &outcome).await
        }
        AgentCommand::Learn {
            repo,
            goal,
            outcome,
            elements,
            detail,
            guardrail,
        } => {
            commands::agent_learn(
                &repo,
                &goal,
                &outcome,
                elements.as_deref(),
                detail.as_deref(),
                guardrail.as_deref(),
            )
            .await
        }
        AgentCommand::SessionSummary {
            repo,
            goal,
            success,
            element_id,
            summary,
        } => {
            commands::agent_session_summary(
                &repo,
                &goal,
                success,
                element_id.as_deref(),
                summary.as_deref(),
            )
            .await
        }
        AgentCommand::ProposeFact {
            repo,
            subject,
            predicate,
            object,
            claim,
            confidence,
            evidence,
        } => {
            commands::agent_propose_fact(
                &repo,
                &subject,
                &predicate,
                &object,
                &claim,
                confidence,
                evidence.as_deref(),
            )
            .await
        }
        AgentCommand::Run {
            run_id,
            repo,
            goal,
            file,
            element_id,
            query,
            mode,
            ai_mode,
            format,
            max_steps,
            max_runtime_ms_per_step,
            ref enrich,
            continue_on_error,
            force_sync,
        } => {
            eprintln!("⚠️  `sruja agent run` is deprecated. Use `sruja plan \"{goal}\"` or `sruja verify` instead.");
            let enrich_ref = enrich.as_ref();
            commands::agent_run(commands::AgentRunOptions {
                repo: &repo,
                goal: &goal,
                file: file.as_deref(),
                element_id: element_id.as_deref(),
                query: query.as_deref(),
                mode: &mode,
                ai_mode: &ai_mode,
                format: &format,
                run_id: run_id.as_deref(),
                max_steps,
                max_runtime_ms_per_step,
                enrich: &enrich_ref,
                continue_on_error,
                force_sync,
            })
            .await
        }
        AgentCommand::Plan {
            run_id,
            repo,
            goal,
            file,
            element_id,
            query,
            out,
            print,
            ai_mode,
            ref enrich,
        } => {
            eprintln!(
                "⚠️  `sruja agent plan` is deprecated. Use `sruja plan \"{goal}\"` instead."
            );
            let enrich_ref = enrich.as_ref();
            let out_path = out.as_deref().map(std::path::Path::new);
            commands::agent_plan(
                commands::AgentRunOptions {
                    repo: &repo,
                    goal: &goal,
                    file: file.as_deref(),
                    element_id: element_id.as_deref(),
                    query: query.as_deref(),
                    mode: "plan",
                    ai_mode: &ai_mode,
                    format: "json",
                    run_id: run_id.as_deref(),
                    max_steps: None,
                    max_runtime_ms_per_step: None,
                    enrich: &enrich_ref,
                    continue_on_error: false,
                    force_sync: false,
                },
                out_path,
                print,
            )
            .await
        }
        AgentCommand::Reflect {
            repo,
            run_id,
            write,
            format,
        } => commands::agent_reflect(&repo, run_id.as_deref(), write, &format).await,
        AgentCommand::Apply { repo, plan, format } => {
            eprintln!("⚠️  `sruja agent apply` is deprecated. Use `sruja verify --plan \"{plan}\"` instead.");
            commands::agent_apply(std::path::Path::new(&plan), &repo, &format).await
        }
        AgentCommand::Loop {
            repo,
            goal,
            max_iterations,
            no_tdd,
            dry_run,
            model,
            base_url,
            spend_cap,
            no_oscillation_detection,
            format,
            yes,
            no_default_grader,
            steer,
            resume,
            show_plan,
            checkpoint,
            no_checkpoint,
            changelog,
            plan_only,
            show_pipeline,
        } => {
            eprintln!(
                "⚠️  `sruja agent loop` is deprecated. Use `sruja auto \"{goal}\"` instead."
            );
            commands::run_agent_loop(&commands::AgentLoopOptions {
                repo: &repo,
                goal: &goal,
                max_iterations,
                no_tdd,
                dry_run,
                model: model.as_deref(),
                base_url: base_url.as_deref(),
                spend_cap_usd: spend_cap,
                no_oscillation_detection,
                format: &format,
                force_proceed: yes,
                no_default_grader,
                steer,
                resume,
                show_plan,
                plan_only,
                checkpoint,
                no_checkpoint,
                changelog,
                show_pipeline,
                pipeline_override: None,
                verbose: false,
            })
            .await
        }
    }
}
