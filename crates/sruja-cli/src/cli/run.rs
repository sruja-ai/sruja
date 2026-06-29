use crate::commands;
use crate::commands::CliError;

use super::app::ContextIntent;
use super::commands::Commands;
use super::subcommands::{
    AgentCommand, AidlcCommand, AuthorCommand, DecisionCommand, DiscoverCommand, DslCommand,
    EvalCommand, EventCommand, FederationCommand, GraphCommand, GuardCommand, HumanCommand,
    IndexCommand, InspectCommand, IntentCommand, MemoryCommand, ProposeCommand, RunCommand,
    WorkflowCommand,
};
use super::Cli;

pub async fn run_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref rules_path) = cli.classification_rules {
        sruja_scan::set_classification_rules_path(Some(std::path::PathBuf::from(rules_path)));
    }

    let command = cli.command;
    let result = match command {
        Commands::Version => commands::version(),
        Commands::Fmt { check, file } => commands::fmt(&file, check).await,
        Commands::Export {
            format,
            file,
            from_scan,
            repo,
            output_dir,
        } => {
            commands::export(
                &format,
                &file,
                from_scan,
                Some(&repo),
                output_dir.as_deref(),
            )
            .await
        }
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
                profile,
                template,
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
                    profile,
                    template,
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
            WorkflowCommand::CaptureRequirements {
                repo,
                id,
                from_issue,
                enrich_cmd,
            } => commands::workflow_capture_requirements(
                &repo,
                id.as_deref(),
                from_issue.as_deref(),
                enrich_cmd.as_deref(),
            ),
            WorkflowCommand::RecordTestResults {
                repo,
                id,
                profile,
                from_file,
            } => {
                commands::workflow_record_test_results(
                    &repo,
                    id.as_deref(),
                    profile.as_deref(),
                    from_file.as_deref(),
                )
                .await
            }
            WorkflowCommand::RecordReadiness { repo, id } => {
                commands::workflow_record_readiness(&repo, id.as_deref())
            }
            WorkflowCommand::Summary { repo, id, format } => {
                commands::workflow_summary(&repo, id.as_deref(), &format)
            }
            WorkflowCommand::NextSteps { repo, id } => {
                commands::workflow_next_steps(&repo, id.as_deref())
            }
        },
        Commands::Aidlc { cmd } => match cmd {
            AidlcCommand::Init {
                repo,
                title,
                id,
                profile,
                template,
                target_elements,
            } => commands::workflow_init(
                &repo,
                &title,
                id.as_deref(),
                target_elements,
                true,
                commands::WorkflowInitOptions {
                    with_aidlc: true,
                    aidlc_profile: profile.clone(),
                    install_rules: true,
                    profile,
                    template,
                },
            ),
            AidlcCommand::Status { repo, id, check } => {
                commands::workflow_status(&repo, id.as_deref(), check)
            }
            AidlcCommand::Validate { repo, id } => {
                commands::workflow_validate(&repo, id.as_deref())
            }
            AidlcCommand::NextSteps { repo, id } => {
                commands::workflow_next_steps(&repo, id.as_deref())
            }
            AidlcCommand::InstallRules { repo } => commands::workflow_install_rules(&repo),
            AidlcCommand::Summary { repo, id, format } => {
                commands::workflow_summary(&repo, id.as_deref(), &format)
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

        Commands::Lsp { .. } => commands::lsp().await,
        Commands::Mcp { root, v2 } => {
            if v2 {
                commands::mcp_v2(&root).await
            } else {
                commands::mcp(&root).await
            }
        }

        Commands::Check {
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
            exclude_barrel_files,
            pr,
            base,
            head,
            compliance,
            intent,
            strict,
        } => {
            if pr {
                commands::drift_pr(&repo, base.as_deref(), head.as_deref(), &format).await
            } else if compliance {
                commands::compliance(
                    &repo,
                    architecture.as_deref(),
                    intent.as_deref(),
                    &format,
                    strict,
                )
                .await
            } else if format == "drift-state" {
                commands::drift_state(&repo)
            } else if ci {
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
                    violations_baseline: violations_baseline.as_deref(),
                    baseline_mode: baseline_mode.as_deref(),
                    structural_only,
                    advisory,
                    exclude_barrel_files,
                })
                .await
            }
        }
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
            scan,
            force,
            hook,
            ci,
            dry_run,
            schema,
            sync_rules,
        } => {
            commands::init(
                &path, prompt, auto, scan, force, hook, ci, dry_run, &schema, sync_rules,
            )
            .await
        }
        Commands::Density { path, format } => commands::density(&path, &format).await,
        Commands::Status {
            path,
            format,
            evolution,
        } => commands::status(&path, &format, evolution).await,

        Commands::Sync { path, format } => commands::sync(&path, &format).await,
        Commands::Review {
            path,
            format,
            show_all,
            critique,
        } => commands::review(&path, &format, show_all, critique).await,

        Commands::Baseline { repo, output } => commands::baseline(&repo, &output).await,

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
        Commands::Onboard {
            repo,
            format,
            max_items,
            ref enrich,
            output,
        } => {
            commands::onboard(
                &repo,
                &format,
                max_items,
                &enrich.as_ref(),
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
            commands::utility_domain::sync_ide_rules(
                commands::utility_domain::SyncIdeRulesOptions {
                    repo: &repo,
                    max_tokens,
                    check,
                },
            )
            .await
        }
        Commands::Classify { repo, force } => {
            commands::utility_domain::classify(commands::utility_domain::ClassifyOptions {
                repo: &repo,
                force,
            })
        }
        Commands::GenerateSkill { repo, output } => {
            commands::utility_domain::generate_skill_prompt(
                commands::utility_domain::GenerateSkillPromptOptions {
                    repo: &repo,
                    output: output.as_deref(),
                },
            )
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
            ref enrich,
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
                        &enrich.as_ref(),
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
        Commands::Explore { repo } => commands::explore(&repo).await,
        Commands::ContextGraph { repo, output, open } => {
            commands::context_graph(&repo, &output, open).await
        }
        Commands::Focus {
            run_id,
            repo,
            file,
            element_id,
            task,
            query,
            format,
            ref enrich,
            base_ref,
            head_ref,
            compact,
            staged: _,
            max_tokens: _,
            output: _,
            cache_friendly: _,
        } => {
            if task.is_some() || query.is_some() {
                let enrich_ref = enrich.as_ref();
                commands::ai_brief(commands::AiBriefOptions {
                    repo: &repo,
                    task: task.as_deref(),
                    file: file.as_deref(),
                    element_id: element_id.as_deref(),
                    query: query.as_deref(),
                    base_ref: base_ref.as_deref(),
                    head_ref: head_ref.as_deref(),
                    staged: false,
                    max_tokens: 8000,
                    output: None,
                    enrich: &enrich_ref,
                })
                .await
            } else {
                commands::focus(
                    &repo,
                    file.as_deref(),
                    element_id.as_deref(),
                    &format,
                    run_id.as_deref(),
                    &enrich.as_ref(),
                    base_ref.as_deref(),
                    head_ref.as_deref(),
                    compact,
                )
                .await
            }
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
            ref enrich,
        } => {
            let enrich_ref = enrich.as_ref();
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
                enrich: &enrich_ref,
            })
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
            MemoryCommand::SkillStats { repo, format } => {
                commands::memory_skill_stats(&repo, &format)
            }
            MemoryCommand::Archive {
                repo,
                decay_threshold,
                min_age_days,
                force,
            } => commands::memory_archive(&repo, decay_threshold, min_age_days, force),
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
        Commands::Graph { cmd } => match cmd {
            GraphCommand::History {
                repo,
                since,
                element,
                kind,
                format,
            } => commands::graph_history(
                &repo,
                since.as_deref(),
                element.as_deref(),
                kind.as_deref(),
                &format,
            ),
        },
        Commands::Requirements {
            repo,
            format,
            priority,
            status,
        } => {
            commands::requirements_list(&repo, &format, priority.as_deref(), status.as_deref())
                .await
        }
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
            AgentCommand::Distill {
                repo,
                goal,
                outcome,
                elements,
                detail,
                guardrail,
            } => {
                commands::agent_distill(
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
            } => {
                commands::agent_loop(&commands::AgentLoopOptions {
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
                    checkpoint,
                    no_checkpoint,
                    changelog,
                })
                .await
            }
        },
        Commands::VerifyTask {
            repo,
            profile,
            file,
            max_runtime_ms,
            evidence_pack,
            evidence_pack_dir,
            format,
        } => {
            let output = commands::verify_task(commands::VerifyTaskOptions {
                repo: &repo,
                profile: &profile,
                file: file.as_deref(),
                max_runtime_ms,
                evidence_pack,
                evidence_pack_dir: evidence_pack_dir.as_deref(),
            })
            .await?;
            let all_passed = output.all_passed;
            println!("{}", commands::format_verify_task(&output, &format));
            if !all_passed {
                return Err(Box::new(CliError::validation("Verification failed")));
            }
            Ok(())
        }
        Commands::Confidence {
            repo,
            profile,
            file,
            max_runtime_ms,
            evidence_pack,
            evidence_pack_dir,
            format,
        } => {
            let report = commands::confidence(commands::ConfidenceOptions {
                repo: &repo,
                profile: &profile,
                file: file.as_deref(),
                max_runtime_ms,
                evidence_pack,
                evidence_pack_dir: evidence_pack_dir.as_deref(),
            })
            .await?;
            println!("{}", commands::format_confidence(&report, &format));
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
                ref enrich,
                output,
            } => {
                commands::onboard(
                    &repo,
                    &format,
                    max_items,
                    &enrich.as_ref(),
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
        Commands::Watch { path, clear, focus } => commands::watch(&path, clear, focus).await,
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
                ref enrich,
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
                    &enrich.as_ref(),
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
        Commands::Critique {
            repo,
            files,
            description,
            proposal,
            base,
            head,
            staged,
            format,
            ref enrich,
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
                &enrich.as_ref(),
                fail_on.as_deref(),
            )
            .await
        }
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
        Commands::Human { cmd } => match cmd {
            HumanCommand::Trace {
                query,
                repo,
                depth,
                team,
                format,
            } => commands::trace_cmd::trace(&query, &repo, depth, team.as_deref(), &format).await,
            HumanCommand::Explain {
                target,
                repo,
                format,
                persist,
            } => commands::explain_cmd::explain_element(&target, &repo, &format, persist).await,
            HumanCommand::Map {
                repo,
                format,
                team,
                focus,
                all,
            } => {
                commands::map_cmd::system_map(
                    &repo,
                    &format,
                    team.as_deref(),
                    focus.as_deref(),
                    all,
                )
                .await
            }
            HumanCommand::Before {
                file,
                repo,
                format,
                ci,
                threshold,
            } => commands::before::before(&repo, &file, &format, ci, threshold).await,
            HumanCommand::Daily { repo, format } => {
                commands::review(&repo, &format, false, false).await
            }
            HumanCommand::CognitiveDebt { repo, format, ci } => {
                commands::cognitive_debt::cognitive_debt(&repo, &format, ci).await
            }
            HumanCommand::WhatIf {
                query,
                repo,
                format,
                ci,
                threshold,
            } => commands::what_if::what_if(&query, &repo, &format, ci, threshold).await,
        },
        Commands::Eval { cmd } => match cmd {
            EvalCommand::Run {
                instance,
                repo,
                max_iterations,
                dry_run,
                format,
            } => commands::eval::run_eval_instance(
                &instance,
                &repo,
                max_iterations,
                dry_run,
                &format,
            )
            .await
            .map_err(|e| CliError::validation(e.to_string())),
            EvalCommand::List { tasks_dir } => commands::eval::list_eval_instances(&tasks_dir)
                .map_err(|e| CliError::validation(e.to_string())),
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
