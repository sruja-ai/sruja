use crate::commands;
use crate::commands::CliError;

use super::super::app::ContextIntent;
use super::super::commands::*;
use super::super::subcommands::{
    AuthorCommand, DecisionCommand, DiscoverCommand, EvalCommand, EventCommand, FederationCommand,
    GraphCommand, GuardCommand, HumanCommand, IndexCommand, IntentCommand, MemoryCommand,
    ProposeCommand, RunCommand,
};

pub(super) async fn handle(command: Commands) -> Result<(), CliError> {
    match command {
        Commands::Version => commands::version(),
        Commands::Fmt { check, file } => commands::fmt(&file, check).await,
        Commands::Export(ExportArgs {
            format,
            file,
            from_scan,
            repo,
            output_dir,
        }) => {
            commands::export(
                &format,
                &file,
                from_scan,
                Some(&repo),
                output_dir.as_deref(),
            )
            .await
        }
        Commands::List(ListArgs { file }) => commands::list_elements(&file).await,
        Commands::Tree(TreeArgs { file }) => commands::tree(&file).await,
        Commands::Diff(DiffArgs {
            file1,
            file2,
            format,
        }) => commands::diff(&file1, &file2, &format).await,
        Commands::Explain(ExplainArgs {
            element_id,
            file,
            json,
        }) => commands::explain(&element_id, file.as_deref(), json).await,
        Commands::Propose(ProposeArgs { cmd }) => match cmd {
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
            ProposeCommand::List { repo, format } => {
                commands::propose_list(&repo, &format).await
            }
            ProposeCommand::Approve {
                proposal_id,
                repo,
                dry_run,
                format,
            } => commands::propose_approve(&repo, &proposal_id, dry_run, &format).await,
        },
        Commands::Author(AuthorArgs { cmd }) => match cmd {
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
                commands::author_propose(
                    &repo,
                    &enrich_cmd,
                    enrich_timeout_ms,
                    enrich_max_bytes,
                )
                .await
            }
        },
        Commands::Scan(ScanArgs { path, output }) => commands::scan(&path, &output).await,
        Commands::Impact(ImpactArgs {
            repo,
            target,
            depth,
            format,
        }) => commands::impact(&repo, &target, depth, &format).await,
        Commands::Why(WhyArgs {
            question,
            repo,
            format,
            reasoned,
            llmguided,
        }) => commands::why(&repo, &question, &format, reasoned, llmguided).await,
        Commands::Lint(LintArgs {
            file,
            format,
            baseline,
            write_baseline,
        }) => {
            commands::lint(
                &file,
                &format,
                baseline.as_deref(),
                write_baseline.as_deref(),
            )
            .await
        }
        Commands::Lsp { .. } => commands::lsp().await,
        Commands::Mcp(McpArgs { root, v2 }) => {
            if v2 {
                commands::mcp_v2(&root).await
            } else {
                commands::mcp(&root).await
            }
        }
        Commands::Check(CheckArgs {
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
        }) => {
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
        Commands::Quickstart(QuickstartArgs {
            path,
            format,
            generate_baseline,
            fail_on,
            advisory,
        }) => {
            commands::quickstart(
                &path,
                &format,
                generate_baseline,
                fail_on.as_deref(),
                advisory,
            )
            .await
        }
        Commands::Init(InitArgs {
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
        }) => {
            commands::init(
                &path, prompt, auto, scan, force, hook, ci, dry_run, &schema, sync_rules, false,
            )
            .await
        }
        Commands::Density(DensityArgs { path, format }) => commands::density(&path, &format).await,
        Commands::Status(StatusArgs {
            path,
            format,
            evolution,
        }) => commands::status(&path, &format, evolution).await,
        Commands::Sync(SyncArgs { path, format }) => commands::sync(&path, &format).await,
        Commands::Review(ReviewArgs {
            path,
            format,
            show_all,
            critique,
        }) => commands::review(&path, &format, show_all, critique).await,
        Commands::Baseline(BaselineArgs { repo, output }) => commands::baseline(&repo, &output).await,
        Commands::Intent(IntentArgs { cmd }) => match cmd {
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
        Commands::Compliance(ComplianceArgs {
            repo,
            architecture,
            intent,
            format,
            strict,
        }) => {
            commands::compliance(
                &repo,
                architecture.as_deref(),
                intent.as_deref(),
                &format,
                strict,
            )
            .await
        }
        Commands::Onboard(OnboardArgs {
            repo,
            format,
            max_items,
            ref enrich,
            output,
        }) => {
            commands::onboard(
                &repo,
                &format,
                max_items,
                &enrich.as_ref(),
                output.as_deref(),
            )
            .await
        }
        Commands::AiContext(AiContextArgs {
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
        }) => {
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
        Commands::SyncIdeRules(SyncIdeRulesArgs {
            repo,
            max_tokens,
            check,
        }) => {
            commands::utility_domain::sync_ide_rules(
                commands::utility_domain::SyncIdeRulesOptions {
                    repo: &repo,
                    max_tokens,
                    check,
                },
            )
            .await
        }
        Commands::Classify(ClassifyArgs { repo, force }) => {
            commands::utility_domain::classify(commands::utility_domain::ClassifyOptions {
                repo: &repo,
                force,
            })
        }
        Commands::GenerateSkill(GenerateSkillArgs { repo, output }) => {
            commands::utility_domain::generate_skill_prompt(
                commands::utility_domain::GenerateSkillPromptOptions {
                    repo: &repo,
                    output: output.as_deref(),
                },
            )
        }
        Commands::Discover(DiscoverArgs {
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
        }) => {
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
                        let _ = commands::scan_repo_cached_with_opts(
                            std::path::Path::new(&repo),
                            true,
                        );
                    }
                    commands::discover_context(&repo, &format).await
                }
                DiscoverCommand::Questions => {
                    if update {
                        let _ = commands::scan_repo_cached_with_opts(
                            std::path::Path::new(&repo),
                            true,
                        );
                    }
                    commands::discover_questions()
                }
            }
        }
        Commands::Generate(GenerateArgs {
            repo,
            skill_path,
            prompt_only,
            output,
        }) => {
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
        Commands::Index(IndexArgs { cmd }) => match cmd {
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
        Commands::Query(QueryArgs {
            query,
            repo,
            architecture,
            format,
        }) => commands::query_registry(&repo, architecture.as_deref(), &query, &format).await,
        Commands::Completions { shell } => commands::completions(shell),
        Commands::Health(HealthArgs {
            repo,
            architecture,
            format,
        }) => commands::health(&repo, architecture.as_deref(), &format).await,
        Commands::ContextScore(ContextScoreArgs {
            repo,
            format,
            fail_under,
        }) => commands::context_score(&repo, &format, fail_under).await,
        Commands::Explore(ExploreArgs { repo }) => commands::explore(&repo).await,
        Commands::ContextGraph(ContextGraphArgs { repo, output, open }) => {
            commands::context_graph(&repo, &output, open).await
        }
        Commands::Focus(FocusArgs {
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
        }) => {
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
        Commands::Lookup(LookupArgs { name, repo, format }) => commands::lookup(&name, &repo, &format).await,
        Commands::Ai(AiArgs {
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
        }) => {
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
        Commands::Ingest(IngestArgs {
            sources,
            repo,
            category,
            elements,
        }) => commands::ingest(&repo, &sources, category.as_deref(), elements.as_deref()).await,
        Commands::Memory(MemoryArgs { cmd }) => match cmd {
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
        Commands::Event(EventArgs { cmd }) => match cmd {
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
        Commands::Decision(DecisionArgs { cmd }) => match cmd {
            DecisionCommand::New {
                repo,
                title,
                typ,
                scope,
            } => commands::decision_new(&repo, &title, &typ, scope.as_deref()).await,
            DecisionCommand::List { repo, format } => {
                commands::decision_list(&repo, &format).await
            }
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
        Commands::Graph(GraphArgs { cmd }) => match cmd {
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
        Commands::Requirements(RequirementsArgs {
            repo,
            format,
            priority,
            status,
        }) => {
            commands::requirements_list(
                &repo,
                &format,
                priority.as_deref(),
                status.as_deref(),
            )
            .await
        }
        Commands::Auto(AutoArgs {
            goal,
            repo,
            max_iterations,
            dry_run,
            yes,
            pipeline,
            resume,
            format,
            show_details,
        }) => {
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
        Commands::Plan(PlanArgs {
            goal,
            repo,
            file,
            element_id,
            query,
            pipeline,
            output,
            json,
            compact,
        }) => {
            commands::plan_run(
                &repo,
                &goal,
                file.as_deref(),
                element_id.as_deref(),
                query.as_deref(),
                pipeline,
                output.as_deref(),
                json,
                compact,
            )
            .await
        }
        Commands::Verify(VerifyArgs {
            repo,
            profile,
            file,
            confidence,
            plan,
            json,
            continue_on_error,
        }) => {
            commands::verify_run(
                &repo,
                &profile,
                file.as_deref(),
                confidence,
                plan.as_deref(),
                json,
                continue_on_error,
            )
            .await
        }
        Commands::VerifyTask(VerifyTaskArgs {
            repo,
            profile,
            file,
            max_runtime_ms,
            evidence_pack,
            evidence_pack_dir,
            format,
        }) => {
            eprintln!("⚠️  `sruja verify-task` is deprecated. Use `sruja verify --profile \"{profile}\"` instead.");
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
                return Err(CliError::validation("Verification failed"));
            }
            Ok(())
        }
        Commands::Confidence(ConfidenceArgs {
            repo,
            profile,
            file,
            max_runtime_ms,
            evidence_pack,
            evidence_pack_dir,
            format,
        }) => {
            eprintln!(
                "⚠️  `sruja confidence` is deprecated. Use `sruja verify --confidence` instead."
            );
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
        Commands::Run(RunArgs { cmd }) => match cmd {
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
        Commands::Watch(WatchArgs { path, clear, focus }) => commands::watch(&path, clear, focus).await,
        Commands::Learn(LearnArgs {
            path,
            file,
            since,
            skip_proposals,
            apply_proposals,
            format,
        }) => {
            let skip = skip_proposals || !apply_proposals;
            commands::learn(&path, file.as_deref(), since.as_deref(), skip, &format).await
        }
        Commands::Guard(GuardArgs { cmd }) => match cmd {
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
        Commands::Critique(CritiqueArgs {
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
        }) => {
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
        Commands::Federation(FederationArgs { cmd }) => match cmd {
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
        Commands::Publish(PublishArgs {
            repo,
            repo_id,
            output,
        }) => commands::publish(&repo, repo_id.as_deref(), &output).await,
        Commands::Compose(ComposeArgs {
            input,
            recursive,
            output,
        }) => commands::compose(&input, recursive, &output).await,
        Commands::Human(HumanArgs { cmd }) => match cmd {
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
        Commands::Eval(EvalArgs { cmd }) => match cmd {
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
        _ => unreachable!(),
    }
}
