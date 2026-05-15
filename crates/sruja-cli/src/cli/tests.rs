use super::subcommands::{AgentCommand, DiscoverCommand, IntentCommand};
use super::{Cli, Commands, ContextIntent};
use clap::Parser;

#[test]
fn context_intent_as_str_mappings() {
    assert_eq!(ContextIntent::AddFeature.as_str(), "add-feature");
    assert_eq!(ContextIntent::Refactor.as_str(), "refactor");
    assert_eq!(ContextIntent::FixBug.as_str(), "fix-bug");
    assert_eq!(ContextIntent::AddTest.as_str(), "add-test");
}

#[test]
fn parses_ai_context_defaults() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "ai-context"]).expect("parse");
            match cli.command {
                Commands::AiContext {
                    run_id: _,
                    format,
                    repo,
                    output,
                    file,
                    element_id,
                    query,
                    base_ref,
                    head_ref,
                    intent,
                    depth,
                    max_tokens,
                } => {
                    assert_eq!(format, "cursor-rules");
                    assert!(repo.is_empty());
                    assert!(output.is_none());
                    assert!(file.is_none());
                    assert!(element_id.is_none());
                    assert!(query.is_none());
                    assert!(base_ref.is_none());
                    assert!(head_ref.is_none());
                    assert!(intent.is_none());
                    assert_eq!(depth, 2);
                    assert_eq!(max_tokens, 10000);
                }
                _ => panic!("expected AiContext command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_context_alias() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "context"]).expect("parse via alias");
            assert!(matches!(cli.command, Commands::AiContext { .. }));
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_discover_subcommands() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "discover", "explain"]).expect("parse");
            match cli.command {
                Commands::Discover { cmd, .. } => {
                    assert!(matches!(cmd, Some(DiscoverCommand::Explain)));
                }
                _ => panic!("expected Discover command"),
            }

            let cli2 = Cli::try_parse_from(["sruja", "discover", "repomap"]).expect("parse");
            match cli2.command {
                Commands::Discover { cmd, .. } => {
                    assert!(matches!(cmd, Some(DiscoverCommand::Repomap)));
                }
                _ => panic!("expected Discover command"),
            }

            let cli3 = Cli::try_parse_from(["sruja", "discover"]).expect("parse bare");
            match cli3.command {
                Commands::Discover { cmd, .. } => {
                    assert!(
                        cmd.is_none(),
                        "bare discover should have no subcommand (defaults to questions)"
                    );
                }
                _ => panic!("expected Discover command"),
            }

            let cli4 =
                Cli::try_parse_from(["sruja", "discover", "questions"]).expect("parse questions");
            match cli4.command {
                Commands::Discover { cmd, .. } => {
                    assert!(matches!(cmd, Some(DiscoverCommand::Questions)));
                }
                _ => panic!("expected Discover command"),
            }

            let cli5 =
                Cli::try_parse_from(["sruja", "discover", "context"]).expect("parse context");
            match cli5.command {
                Commands::Discover { cmd, .. } => {
                    assert!(matches!(cmd, Some(DiscoverCommand::Context)));
                }
                _ => panic!("expected Discover command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_drift_ci_flag() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "drift", "--ci"]).expect("parse");
            match cli.command {
                Commands::Drift { ci, .. } => assert!(ci),
                _ => panic!("expected Drift command"),
            }

            let cli2 = Cli::try_parse_from(["sruja", "drift"]).expect("parse");
            match cli2.command {
                Commands::Drift { ci, .. } => assert!(!ci),
                _ => panic!("expected Drift command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_intent_evaluate_and_history() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "intent", "evaluate"]).expect("parse");
            match cli.command {
                Commands::Intent { cmd } => {
                    assert!(matches!(cmd, IntentCommand::Evaluate { .. }));
                }
                _ => panic!("expected Intent command"),
            }

            let cli2 = Cli::try_parse_from(["sruja", "intent", "history"]).expect("parse");
            match cli2.command {
                Commands::Intent { cmd } => {
                    assert!(matches!(cmd, IntentCommand::History { .. }));
                }
                _ => panic!("expected Intent command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_ai_brief_defaults() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "ai"]).expect("parse");
            match cli.command {
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
                    assert_eq!(repo, ".");
                    assert!(task.is_none());
                    assert!(file.is_none());
                    assert!(element_id.is_none());
                    assert!(query.is_none());
                    assert!(base_ref.is_none());
                    assert!(head_ref.is_none());
                    assert!(!staged);
                    assert_eq!(max_tokens, 8000);
                    assert!(output.is_none());
                    assert!(!enrich);
                    assert!(enrich_provider.is_none());
                    assert!(enrich_cmd.is_none());
                    assert!(enrich_model.is_none());
                    assert!(enrich_base_url.is_none());
                    assert_eq!(enrich_timeout_ms, 15000);
                    assert_eq!(enrich_max_bytes, 20000);
                }
                _ => panic!("expected Ai command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_ai_brief_focus_options() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from([
                "sruja",
                "ai",
                "--task",
                "Fix parser diagnostics",
                "--file",
                "crates/sruja-language/src/parser/mod.rs",
                "--element-id",
                "Sruja.Language",
                "--query",
                "parser",
                "--base-ref",
                "main",
                "--head-ref",
                "HEAD",
                "--staged",
                "--max-tokens",
                "12000",
                "-o",
                "brief.md",
            ])
            .expect("parse");
            match cli.command {
                Commands::Ai {
                    task,
                    file,
                    element_id,
                    query,
                    base_ref,
                    head_ref,
                    staged,
                    max_tokens,
                    output,
                    ..
                } => {
                    assert_eq!(task.as_deref(), Some("Fix parser diagnostics"));
                    assert_eq!(
                        file.as_deref(),
                        Some("crates/sruja-language/src/parser/mod.rs")
                    );
                    assert_eq!(element_id.as_deref(), Some("Sruja.Language"));
                    assert_eq!(query.as_deref(), Some("parser"));
                    assert_eq!(base_ref.as_deref(), Some("main"));
                    assert_eq!(head_ref.as_deref(), Some("HEAD"));
                    assert!(staged);
                    assert_eq!(max_tokens, 12000);
                    assert_eq!(output.as_deref(), Some("brief.md"));
                }
                _ => panic!("expected Ai command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_agent_run_defaults() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from([
                "sruja",
                "agent",
                "run",
                "--goal",
                "Add agent loop",
                "--file",
                "crates/sruja-cli/src/cli.rs",
            ])
            .expect("parse");
            match cli.command {
                Commands::Agent { cmd } => match cmd {
                    AgentCommand::Run {
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
                        continue_on_error,
                        ..
                    } => {
                        assert_eq!(repo, ".");
                        assert_eq!(goal, "Add agent loop");
                        assert_eq!(file.as_deref(), Some("crates/sruja-cli/src/cli.rs"));
                        assert!(element_id.is_none());
                        assert!(query.is_none());
                        assert_eq!(mode, "plan");
                        assert_eq!(ai_mode, "standard");
                        assert_eq!(format, "text");
                        assert!(max_steps.is_none());
                        assert!(max_runtime_ms_per_step.is_none());
                        assert!(!enrich);
                        assert!(!continue_on_error);
                    }
                    _ => panic!("expected Agent run subcommand"),
                },
                _ => panic!("expected Agent command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}
