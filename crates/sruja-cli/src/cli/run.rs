use crate::commands;
use crate::commands::CliError;

use super::app::was_invoked_as;
use super::app::ContextIntent;
use super::commands::Commands;
use super::subcommands::{
    AgentCommand, AuthorCommand, DecisionCommand, DiscoverCommand, DslCommand, EventCommand,
    EvolutionCommand, FederationCommand, GuardCommand, IndexCommand, InspectCommand, IntentCommand,
    MemoryCommand, ProposeCommand, RunCommand, WorkflowCommand,
};
use super::Cli;

pub async fn run_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref rules_path) = cli.classification_rules {
        sruja_scan::set_classification_rules_path(Some(std::path::PathBuf::from(rules_path)));
    }

    let command = cli.command;
    let result = match command {
        Commands::Version => commands::version(),
        Commands::Workflow { cmd } => match cmd {
            WorkflowCommand::Init {
                repo,
                title,
                id,
                target_elements,
                strict_gates,
                with_aidlc,
                aidlc_profile,
                install_aidlc_rules,
            } => commands::workflow_init(
                &repo,
                &title,
                id.as_deref(),
                target_elements,
                strict_gates,
                commands::WorkflowInitOptions {
                    with_aidlc,
                    aidlc_profile,
                    install_rules: install_aidlc_rules,
                },
            ),
            WorkflowCommand::List { repo } => commands::workflow_list(&repo),
            WorkflowCommand::Status { repo, id, check } => {
                commands::workflow_status(&repo, id.as_deref(), check)
            }
            WorkflowCommand::RecordImpact { repo, id, depth } => {
                commands::workflow_record_impact(&repo, &id, depth)
            }
            WorkflowCommand::Approve {
                repo,
                id,
                phase,
                by,
            } => commands::workflow_approve(&repo, &id, &phase, by.as_deref()),
            WorkflowCommand::Advance { repo, id } => commands::workflow_advance(&repo, &id),
            WorkflowCommand::InstallRules { repo } => commands::workflow_install_rules(&repo),
            WorkflowCommand::Validate { repo, id } => {
                commands::workflow_validate(&repo, id.as_deref())
            }
            WorkflowCommand::Audit {
                repo,
                id,
                event,
                by,
            } => commands::workflow_audit(&repo, &id, &event, by.as_deref()),
            WorkflowCommand::Trace {
                repo,
                id,
                format,
                check,
            } => commands::workflow_trace(&repo, &id, &format, check),
            WorkflowCommand::Run {
                repo,
                id,
                vision,
                dry_run,
            } => commands::workflow_run(&repo, &id, std::path::Path::new(&vision), dry_run),
            WorkflowCommand::DesignReview {
                repo,
                id,
                output,
                enrich_cmd,
            } => {
                commands::review_design(
                    &repo,
                    &id,
                    output.as_deref().map(std::path::Path::new),
                    enrich_cmd.as_deref(),
                )
                .await
            }
        },
        Commands::Propose { cmd } => match cmd {
            ProposeCommand::Create {
                repo,
                description,
                workflow_id,
                add_elements,
                add_relationships,
                remove_elements,
                remove_relationships,
                format,
            } => {
                commands::propose_create(
                    &repo,
                    commands::ProposeCreateRequest {
                        description,
                        workflow_id,
                        add_elements,
                        add_relationships,
                        remove_elements,
                        remove_relationships,
                        format,
                    },
                )
                .await
            }
            ProposeCommand::List { repo, format } => commands::propose_list(&repo, &format).await,
            ProposeCommand::Approve {
                proposal_id,
                repo,
                dry_run,
                format,
            } => commands::propose_approve(&repo, &proposal_id, dry_run, &format).await,
        },
        Commands::Author { cmd } => match cmd {
            AuthorCommand::Evidence {
                repo,
                format,
                output,
                quiet,
            } => commands::author_evidence(&repo, &format, output.as_deref(), quiet).await,
            AuthorCommand::Propose {
                repo,
                enrich_cmd,
                enrich_timeout_ms,
                enrich_max_bytes,
            } => {
                commands::author_propose(&repo, &enrich_cmd, enrich_timeout_ms, enrich_max_bytes)
                    .await
            }
        },
        Commands::Scan { path, output } => commands::scan(&path, &output).await,
        Commands::Impact {
            repo,
            target,
            depth,
            format,
        } => commands::impact(&repo, &target, depth, &format).await,
        Commands::Why {
            question,
            repo,
            format,
            reasoned,
            llmguided,
        } => commands::why(&repo, &question, &format, reasoned, llmguided).await,
        Commands::Lint {
            file,
            format,
            baseline,
            write_baseline,
        } => {
            commands::lint(
                &file,
                &format,
                baseline.as_deref(),
                write_baseline.as_deref(),
            )
            .await
        }
        Commands::Export {
            format,
            file,
            extended,
            view_level,
            target,
            view,
            all_views,
            inject,
            hydrate,
            from_scan,
            repo,
            output_dir,
        } => {
            commands::export(
                &format,
                &file,
                commands::ExportOptions {
                    extended,
                    view_level,
                    target,
                    view_name: view,
                    all_views,
                    inject,
                    hydrate,
                    from_scan,
                    repo,
                    output_dir,
                },
            )
            .await
        }
        Commands::Fmt { file, check } => commands::fmt(&file, check).await,
        Commands::Lsp { .. } => commands::lsp().await,
        Commands::Mcp { root } => commands::mcp(&root).await,
        Commands::Critique {
            repo,
            files,
            description,
            proposal,
            base,
            head,
            staged,
            format,
            enrich,
            enrich_provider,
            enrich_cmd,
            enrich_model,
            enrich_base_url,
            enrich_timeout_ms,
            enrich_max_bytes,
            fail_on,
        } => {
            commands::critique(
                &repo,
                files,
                description,
                proposal,
                base,
                head,
                staged,
                &format,
                enrich,
                enrich_provider.as_deref(),
                enrich_cmd.as_deref(),
                enrich_model.as_deref(),
                enrich_base_url.as_deref(),
                enrich_timeout_ms,
                enrich_max_bytes,
                fail_on.as_deref(),
            )
            .await
        }
        Commands::Compile { file } => commands::compile(&file).await,
        Commands::Validate {
            file,
            constraints,
            fail_on_violations,
            format_json,
        } => commands::validate(&file, constraints, fail_on_violations, format_json).await,
        Commands::List { file } => commands::list_elements(&file).await,
        Commands::Tree { file } => commands::tree(&file).await,
        Commands::Diff {
            file1,
            file2,
            format,
        } => commands::diff(&file1, &file2, &format).await,
        Commands::Explain {
            element_id,
            file,
            json,
        } => commands::explain(&element_id, file.as_deref(), json).await,
        Commands::Import { format, file } => commands::import(&format, &file).await,
        Commands::Drift {
            repo,
            architecture,
            format,
            violations_only,
            fail_on,
            ci,
            violations_baseline,
            baseline_mode,
            structural_only,
            advisory,
        } => {
            if ci {
                let ci_format = if format == "text" {
                    "github-actions".to_string()
                } else {
                    format
                };
                commands::check(&repo, &ci_format, violations_baseline.as_deref()).await
            } else {
                commands::drift(commands::scan_domain::scan::drift::DriftRequest {
                    repo_root: &repo,
                    architecture_path: architecture.as_deref(),
                    format: &format,
                    violations_only,
                    fail_on: fail_on.as_deref(),
                    baseline_mode: baseline_mode.as_deref(),
                    structural_only,
                    advisory,
                })
                .await
            }
        }
        Commands::DriftPr {
            repo,
            base,
            head,
            format,
        } => commands::drift_pr(&repo, base.as_deref(), head.as_deref(), &format).await,
        Commands::DriftState { repo } => commands::drift_state(&repo),
        Commands::Quickstart {
            path,
            format,
            generate_baseline,
            fail_on,
            advisory,
        } => {
            commands::quickstart(
                &path,
                &format,
                generate_baseline,
                fail_on.as_deref(),
                advisory,
            )
            .await
        }
        Commands::Init {
            path,
            prompt,
            auto,
            force,
            hook,
            ci,
            dry_run,
            schema,
        } => commands::init(&path, prompt, auto, force, hook, ci, dry_run, &schema).await,
        Commands::Status {
            path,
            format,
            evolution,
        } => commands::status(&path, &format, evolution).await,
        Commands::Watch { path, clear, focus } => commands::watch(&path, clear, focus).await,
        Commands::Sync { path, format } => commands::sync(&path, &format).await,
        Commands::Review {
            path,
            format,
            show_all,
            critique,
        } => commands::review(&path, &format, show_all, critique).await,
        Commands::Learn {
            path,
            file,
            since,
            skip_proposals,
            apply_proposals,
            format,
        } => {
            let skip = skip_proposals || !apply_proposals;
            commands::learn(&path, file.as_deref(), since.as_deref(), skip, &format).await
        }
        Commands::Check {
            path,
            format,
            violations_baseline,
        } => {
            eprintln!("warning: 'sruja check' is deprecated, use 'sruja drift --ci'");
            commands::check(&path, &format, violations_baseline.as_deref()).await
        }
        Commands::Baseline { repo, output } => commands::baseline(&repo, &output).await,
        Commands::Publish {
            repo,
            repo_id,
            output,
        } => commands::publish(&repo, repo_id.as_deref(), &output).await,
        Commands::Compose {
            input,
            recursive,
            output,
        } => commands::compose(&input, recursive, &output).await,
        Commands::Intent { cmd } => match cmd {
            IntentCommand::Check {
                repo,
                intent,
                format,
                strict,
            } => {
                let intent_opt = intent.or_else(|| std::env::var("SRUJA_INTENT_PATH").ok());
                commands::intent_check(&repo, intent_opt.as_deref(), &format, strict).await
            }
            IntentCommand::Propose { repo, intent } => {
                commands::intent_propose(&repo, intent.as_deref()).await
            }
            IntentCommand::Evaluate { architecture } => commands::evaluate(&architecture).await,
            IntentCommand::History { repo } => commands::evolution_log(&repo).await,
        },
        Commands::Compliance {
            repo,
            architecture,
            intent,
            format,
            strict,
        } => {
            commands::compliance(
                &repo,
                architecture.as_deref(),
                intent.as_deref(),
                &format,
                strict,
            )
            .await
        }
        Commands::Ai {
            repo,
            task,
            file,
            element_id,
            query,
            base_ref,
            head_ref,
            staged,
            max_tokens,
            output,
            enrich,
            enrich_provider,
            enrich_cmd,
            enrich_model,
            enrich_base_url,
            enrich_timeout_ms,
            enrich_max_bytes,
        } => {
            commands::ai_brief(commands::AiBriefOptions {
                repo: &repo,
                task: task.as_deref(),
                file: file.as_deref(),
                element_id: element_id.as_deref(),
                query: query.as_deref(),
                base_ref: base_ref.as_deref(),
                head_ref: head_ref.as_deref(),
                staged,
                max_tokens,
                output: output.as_deref(),
                enrich,
                enrich_provider: enrich_provider.as_deref(),
                enrich_cmd: enrich_cmd.as_deref(),
                enrich_model: enrich_model.as_deref(),
                enrich_base_url: enrich_base_url.as_deref(),
                enrich_timeout_ms,
                enrich_max_bytes,
            })
            .await
        }
        Commands::Onboard {
            repo,
            format,
            max_items,
            enrich,
            enrich_provider,
            enrich_cmd,
            enrich_model,
            enrich_base_url,
            enrich_timeout_ms,
            enrich_max_bytes,
            output,
        } => {
            commands::onboard(
                &repo,
                &format,
                max_items,
                enrich,
                enrich_provider.as_deref(),
                enrich_cmd.as_deref(),
                enrich_model.as_deref(),
                enrich_base_url.as_deref(),
                enrich_timeout_ms,
                enrich_max_bytes,
                commands::LlmConfig {
                    provider: enrich_provider.as_deref(),
                    model: enrich_model.as_deref(),
                    base_url: enrich_base_url.as_deref(),
                },
                output.as_deref(),
            )
            .await
        }
        Commands::AiContext {
            run_id,
            repo,
            format,
            output,
            file,
            element_id,
            query,
            base_ref,
            head_ref,
            intent,
            depth,
            max_tokens,
            cache_friendly,
        } => {
            if was_invoked_as("context") {
                eprintln!("warning: 'sruja context' is deprecated, use 'sruja ai-context'");
            }
            commands::context_export(
                &repo,
                &format,
                output.as_deref(),
                commands::ContextRequest {
                    run_id: run_id.as_deref(),
                    file: file.as_deref(),
                    element_id: element_id.as_deref(),
                    query: query.as_deref(),
                    base_ref: base_ref.as_deref(),
                    head_ref: head_ref.as_deref(),
                    intent: intent.as_ref().map(ContextIntent::as_str),
                    depth,
                    max_tokens,
                    cache_friendly,
                },
            )
            .await
        }
        Commands::SyncIdeRules {
            repo,
            max_tokens,
            check,
        } => {
            if check {
                commands::sync_ide_rules_check(&repo, max_tokens).await
            } else {
                commands::sync_ide_rules(&repo, max_tokens).await
            }
        }
        Commands::Discover {
            cmd,
            context,
            explain,
            repomap,
            repo,
            format,
            max_files,
            max_tokens,
            export_report,
            enrich,
            enrich_provider,
            enrich_cmd,
            enrich_model,
            enrich_base_url,
            enrich_timeout_ms,
            enrich_max_bytes,
            update,
        } => {
            let effective = if let Some(ref sub) = cmd {
                sub.clone()
            } else if repomap {
                eprintln!(
                    "warning: 'discover --repomap' is deprecated, use 'sruja discover repomap'"
                );
                DiscoverCommand::Repomap
            } else if explain {
                eprintln!(
                    "warning: 'discover --explain' is deprecated, use 'sruja discover explain'"
                );
                DiscoverCommand::Explain
            } else if context {
                eprintln!(
                    "warning: 'discover --context' is deprecated, use 'sruja discover context'"
                );
                DiscoverCommand::Context
            } else {
                DiscoverCommand::Questions
            };

            match effective {
                DiscoverCommand::Repomap => {
                    commands::discover_repomap_cmd(&repo, max_files, max_tokens).await
                }
                DiscoverCommand::Explain => {
                    commands::discover_explain(
                        &repo,
                        &format,
                        export_report.as_deref(),
                        enrich,
                        enrich_provider.as_deref(),
                        enrich_cmd.as_deref(),
                        enrich_model.as_deref(),
                        enrich_base_url.as_deref(),
                        enrich_timeout_ms,
                        enrich_max_bytes,
                        update,
                    )
                    .await
                }
                DiscoverCommand::Context => {
                    if update {
                        let _ =
                            commands::scan_repo_cached_with_opts(std::path::Path::new(&repo), true);
                    }
                    commands::discover_context(&repo, &format).await
                }
                DiscoverCommand::Questions => {
                    if update {
                        let _ =
                            commands::scan_repo_cached_with_opts(std::path::Path::new(&repo), true);
                    }
                    commands::discover_questions()
                }
            }
        }
        Commands::Generate {
            repo,
            skill_path,
            prompt_only,
            output,
        } => {
            if !prompt_only {
                eprintln!(
                    "Only --prompt-only is supported. Use: sruja generate -r . --prompt-only -o prompt.txt"
                );
                eprintln!(
                    "Then use the prompt with any LLM; save output as architecture.sruja and run sruja lint."
                );
                std::process::exit(1);
            }
            commands::generate_prompt(&repo, skill_path.as_deref(), output.as_deref())
        }
        Commands::Index { cmd } => match cmd {
            IndexCommand::Semantic {
                repo,
                architecture,
                output,
            } => commands::semantic_index(&repo, architecture.as_deref(), &output).await,
            IndexCommand::Registry {
                repo,
                architecture,
                fix,
                format,
            } => commands::registry_index(&repo, architecture.as_deref(), fix, &format).await,
            IndexCommand::Dashboard { repo, output } => {
                commands::registry_dashboard(&repo, &output).await
            }
        },
        Commands::Query {
            query,
            repo,
            architecture,
            format,
        } => commands::query_registry(&repo, architecture.as_deref(), &query, &format).await,
        Commands::Completions { shell } => commands::completions(shell),
        Commands::Health {
            repo,
            architecture,
            format,
        } => commands::health(&repo, architecture.as_deref(), &format).await,
        Commands::ContextScore {
            repo,
            format,
            fail_under,
        } => commands::context_score(&repo, &format, fail_under).await,
        Commands::ContextGraph { repo, output, open } => {
            commands::context_graph(&repo, &output, open).await
        }
        Commands::Focus {
            run_id,
            repo,
            file,
            element_id,
            format,
            enrich,
            enrich_provider,
            enrich_cmd,
            enrich_model,
            enrich_base_url,
            enrich_timeout_ms,
            enrich_max_bytes,
            base_ref,
            head_ref,
        } => {
            commands::focus(
                &repo,
                file.as_deref(),
                element_id.as_deref(),
                &format,
                run_id.as_deref(),
                enrich,
                enrich_provider.as_deref(),
                enrich_cmd.as_deref(),
                enrich_model.as_deref(),
                enrich_base_url.as_deref(),
                enrich_timeout_ms,
                enrich_max_bytes,
                base_ref.as_deref(),
                head_ref.as_deref(),
            )
            .await
        }
        Commands::Ingest {
            sources,
            repo,
            category,
            elements,
        } => commands::ingest(&repo, &sources, category.as_deref(), elements.as_deref()).await,
        Commands::Memory { cmd } => match cmd {
            MemoryCommand::Reindex { repo } => commands::memory_reindex(&repo),
            MemoryCommand::Search {
                repo,
                query,
                element_id,
                decision_id,
                hitl_kind,
                limit,
            } => commands::memory_search(
                &repo,
                &query,
                element_id.as_deref(),
                decision_id.as_deref(),
                hitl_kind.as_deref(),
                limit,
            ),
            MemoryCommand::Timeline {
                repo,
                anchor_id,
                anchor_timestamp,
                before,
                after,
                decision_id,
                element_id,
            } => commands::memory_timeline(
                &repo,
                anchor_id.as_deref(),
                anchor_timestamp.as_deref(),
                before,
                after,
                decision_id.as_deref(),
                element_id.as_deref(),
            ),
        },
        Commands::Event { cmd } => match cmd {
            EventCommand::Append { repo, json } => {
                commands::event_append(&repo, json.as_deref()).await
            }
            EventCommand::List {
                repo,
                format,
                limit,
                kind,
                details_substring,
                decision_id,
                trace_id,
                element_id,
                decision_lineage_only,
            } => {
                commands::event_list(
                    &repo,
                    &format,
                    limit,
                    kind.as_deref(),
                    details_substring.as_deref(),
                    decision_id.as_deref(),
                    trace_id.as_deref(),
                    element_id.as_deref(),
                    decision_lineage_only,
                )
                .await
            }
        },
        Commands::Decision { cmd } => match cmd {
            DecisionCommand::New {
                repo,
                title,
                typ,
                scope,
            } => commands::decision_new(&repo, &title, &typ, scope.as_deref()).await,
            DecisionCommand::List { repo, format } => commands::decision_list(&repo, &format).await,
            DecisionCommand::Show { repo, id } => commands::decision_show(&repo, &id).await,
            DecisionCommand::Trace { repo, id, limit } => {
                commands::decision_trace(&repo, &id, limit).await
            }
            DecisionCommand::Link { repo, id, element } => {
                commands::decision_link(&repo, &id, &element).await
            }
            DecisionCommand::Accept { repo, id } => commands::decision_accept(&repo, &id).await,
            DecisionCommand::Supersede { repo, id, by } => {
                commands::decision_supersede(&repo, &id, &by).await
            }
        },
        Commands::Agent { cmd } => match cmd {
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
                commands::agent_merge(&repo, &ids, &context, &hypothesis, &guardrail, &outcome)
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
                enrich,
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                continue_on_error,
                trajectories,
            } => {
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
                    enrich,
                    enrich_provider: enrich_provider.as_deref(),
                    enrich_cmd: enrich_cmd.as_deref(),
                    enrich_model: enrich_model.as_deref(),
                    enrich_base_url: enrich_base_url.as_deref(),
                    enrich_timeout_ms,
                    enrich_max_bytes,
                    continue_on_error,
                    trajectories,
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
                enrich,
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
            } => {
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
                        enrich,
                        enrich_provider: enrich_provider.as_deref(),
                        enrich_cmd: enrich_cmd.as_deref(),
                        enrich_model: enrich_model.as_deref(),
                        enrich_base_url: enrich_base_url.as_deref(),
                        enrich_timeout_ms,
                        enrich_max_bytes,
                        continue_on_error: false,
                        trajectories: None,
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
                commands::agent_apply(std::path::Path::new(&plan), &repo, &format).await
            }
        },
        Commands::VerifyTask {
            repo,
            profile,
            file,
            max_runtime_ms,
            format,
        } => {
            let output = commands::verify_task(commands::VerifyTaskOptions {
                repo: &repo,
                profile: &profile,
                file: file.as_deref(),
                max_runtime_ms,
            })
            .await?;
            let all_passed = output.all_passed;
            println!("{}", commands::format_verify_task(&output, &format));
            if !all_passed {
                return Err(Box::new(CliError::validation("Verification failed")));
            }
            Ok(())
        }
        Commands::Run { cmd } => match cmd {
            RunCommand::Show {
                repo,
                run_id,
                format,
            } => commands::run_show(&repo, &run_id, &format).await,
            RunCommand::Export {
                repo,
                run_id,
                out,
                events_limit,
            } => commands::run_export(&repo, &run_id, out.as_deref(), events_limit).await,
        },
        Commands::Evaluate { architecture } => {
            eprintln!("warning: 'sruja evaluate' is deprecated, use 'sruja intent evaluate'");
            commands::evaluate(&architecture).await
        }
        Commands::Evolution { cmd } => {
            eprintln!("warning: 'sruja evolution' is deprecated, use 'sruja intent history'");
            match cmd {
                EvolutionCommand::Log { repo } => commands::evolution_log(&repo).await,
            }
        }
        Commands::Dsl { cmd } => match cmd {
            DslCommand::List { file } => commands::list_elements(&file).await,
            DslCommand::Tree { file } => commands::tree(&file).await,
            DslCommand::Diff {
                file1,
                file2,
                format,
            } => commands::diff(&file1, &file2, &format).await,
            DslCommand::Explain {
                element_id,
                file,
                json,
            } => commands::explain(&element_id, file.as_deref(), json).await,
            DslCommand::Import { format, file } => commands::import(&format, &file).await,
            DslCommand::Compile { file } => commands::compile(&file).await,
            DslCommand::Validate {
                file,
                constraints,
                fail_on_violations,
                format_json,
            } => commands::validate(&file, constraints, fail_on_violations, format_json).await,
            DslCommand::Generate {
                repo,
                skill_path,
                prompt_only,
                output,
            } => {
                if !prompt_only {
                    eprintln!("Only --prompt-only is supported. Use: sruja dsl generate -r . --prompt-only -o prompt.txt");
                    eprintln!("Then use the prompt with any LLM; save output as architecture.sruja and run sruja lint.");
                    std::process::exit(1);
                }
                commands::generate_prompt(&repo, skill_path.as_deref(), output.as_deref())
            }
        },
        Commands::Inspect { cmd } => match cmd {
            InspectCommand::Health {
                repo,
                architecture,
                format,
            } => commands::health(&repo, architecture.as_deref(), &format).await,
            InspectCommand::Impact {
                target,
                repo,
                depth,
                format,
            } => commands::impact(&repo, &target, depth, &format).await,
            InspectCommand::Why {
                question,
                repo,
                format,
                reasoned,
                llmguided,
            } => commands::why(&repo, &question, &format, reasoned, llmguided).await,
            InspectCommand::Query {
                query,
                repo,
                architecture,
                format,
            } => commands::query_registry(&repo, architecture.as_deref(), &query, &format).await,
            InspectCommand::ContextScore {
                repo,
                format,
                fail_under,
            } => commands::context_score(&repo, &format, fail_under).await,
            InspectCommand::ContextGraph { repo, output, open } => {
                commands::context_graph(&repo, &output, open).await
            }
            InspectCommand::Onboard {
                repo,
                format,
                max_items,
                enrich,
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                output,
            } => {
                commands::onboard(
                    &repo,
                    &format,
                    max_items,
                    enrich,
                    enrich_provider.as_deref(),
                    enrich_cmd.as_deref(),
                    enrich_model.as_deref(),
                    enrich_base_url.as_deref(),
                    enrich_timeout_ms,
                    enrich_max_bytes,
                    commands::LlmConfig {
                        provider: enrich_provider.as_deref(),
                        model: enrich_model.as_deref(),
                        base_url: enrich_base_url.as_deref(),
                    },
                    output.as_deref(),
                )
                .await
            }
            InspectCommand::Quickstart {
                path,
                format,
                generate_baseline,
                fail_on,
                advisory,
            } => {
                commands::quickstart(
                    &path,
                    &format,
                    generate_baseline,
                    fail_on.as_deref(),
                    advisory,
                )
                .await
            }
            InspectCommand::Watch { path, clear, focus } => {
                commands::watch(&path, clear, focus).await
            }
            InspectCommand::Learn {
                path,
                file,
                since,
                skip_proposals,
                apply_proposals,
                format,
            } => {
                let skip = skip_proposals || !apply_proposals;
                commands::learn(&path, file.as_deref(), since.as_deref(), skip, &format).await
            }
            InspectCommand::Ingest {
                sources,
                repo,
                category,
                elements,
            } => commands::ingest(&repo, &sources, category.as_deref(), elements.as_deref()).await,
        },
        Commands::Guard { cmd } => match cmd {
            GuardCommand::Critique {
                repo,
                files,
                description,
                proposal,
                base,
                head,
                staged,
                format,
                enrich,
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                fail_on,
            } => {
                commands::critique(
                    &repo,
                    files,
                    description,
                    proposal,
                    base,
                    head,
                    staged,
                    &format,
                    enrich,
                    enrich_provider.as_deref(),
                    enrich_cmd.as_deref(),
                    enrich_model.as_deref(),
                    enrich_base_url.as_deref(),
                    enrich_timeout_ms,
                    enrich_max_bytes,
                    fail_on.as_deref(),
                )
                .await
            }
            GuardCommand::Compliance {
                repo,
                architecture,
                intent,
                format,
                strict,
            } => {
                commands::compliance(
                    &repo,
                    architecture.as_deref(),
                    intent.as_deref(),
                    &format,
                    strict,
                )
                .await
            }
            GuardCommand::Baseline { repo, output } => commands::baseline(&repo, &output).await,
            GuardCommand::DriftPr {
                repo,
                base,
                head,
                format,
            } => commands::drift_pr(&repo, base.as_deref(), head.as_deref(), &format).await,
        },
        Commands::Federation { cmd } => match cmd {
            FederationCommand::Publish {
                repo,
                repo_id,
                output,
            } => commands::publish(&repo, repo_id.as_deref(), &output).await,
            FederationCommand::Compose {
                input,
                recursive,
                output,
            } => commands::compose(&input, recursive, &output).await,
        },
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            e.report();
            std::process::exit(e.exit_code());
        }
    }
}
