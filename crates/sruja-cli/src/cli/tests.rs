use super::subcommands::{
    AgentCommand, AuthorCommand, DiscoverCommand, IntentCommand, ProposeCommand,
};
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
                Commands::AiContext(ref args) => {
                    assert_eq!(args.format, "cursor-rules");
                    assert!(args.repo.is_empty());
                    assert!(args.output.is_none());
                    assert!(args.file.is_none());
                    assert!(args.element_id.is_none());
                    assert!(args.query.is_none());
                    assert!(args.base_ref.is_none());
                    assert!(args.head_ref.is_none());
                    assert!(args.intent.is_none());
                    assert_eq!(args.depth, 2);
                    assert_eq!(args.max_tokens, 10000);
                    assert!(!args.cache_friendly);
                }
                _ => panic!("expected AiContext command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_check_drift_state_format() {
    std::thread::Builder::new()
        .name("clap_parse_drift_state".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "check", "-r", ".", "--format", "drift-state"])
                .expect("parse");
            match cli.command {
                Commands::Check(ref args) => {
                    assert_eq!(args.repo, ".");
                    assert_eq!(args.format, "drift-state");
                }
                _ => panic!("expected Check command with drift-state format"),
            }
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
                Commands::Discover(ref args) => {
                    assert!(matches!(args.cmd, Some(DiscoverCommand::Explain)));
                }
                _ => panic!("expected Discover command"),
            }

            let cli2 = Cli::try_parse_from(["sruja", "discover", "repomap"]).expect("parse");
            match cli2.command {
                Commands::Discover(ref args) => {
                    assert!(matches!(args.cmd, Some(DiscoverCommand::Repomap)));
                }
                _ => panic!("expected Discover command"),
            }

            let cli3 = Cli::try_parse_from(["sruja", "discover"]).expect("parse bare");
            match cli3.command {
                Commands::Discover(ref args) => {
                    assert!(
                        args.cmd.is_none(),
                        "bare discover should have no subcommand (defaults to questions)"
                    );
                }
                _ => panic!("expected Discover command"),
            }

            let cli4 =
                Cli::try_parse_from(["sruja", "discover", "questions"]).expect("parse questions");
            match cli4.command {
                Commands::Discover(ref args) => {
                    assert!(matches!(args.cmd, Some(DiscoverCommand::Questions)));
                }
                _ => panic!("expected Discover command"),
            }

            let cli5 =
                Cli::try_parse_from(["sruja", "discover", "context"]).expect("parse context");
            match cli5.command {
                Commands::Discover(ref args) => {
                    assert!(matches!(args.cmd, Some(DiscoverCommand::Context)));
                }
                _ => panic!("expected Discover command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_author_subcommands() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "author", "evidence"]).expect("parse");
            match cli.command {
                Commands::Author(ref args) => {
                    assert!(matches!(args.cmd, AuthorCommand::Evidence { .. }));
                }
                _ => panic!("expected Author command"),
            }

            let cli2 = Cli::try_parse_from([
                "sruja",
                "author",
                "propose",
                "-r",
                ".",
                "--enrich-cmd",
                "cat",
            ])
            .expect("parse propose");
            match cli2.command {
                Commands::Author(ref args) => {
                    assert!(matches!(args.cmd, AuthorCommand::Propose { .. }));
                }
                _ => panic!("expected Author command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_propose_subcommands() {
    std::thread::Builder::new()
        .name("clap_parse_propose".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from([
                "sruja",
                "propose",
                "create",
                "-r",
                ".",
                "-d",
                "test",
                "-e",
                "A:system:Payments",
                "-f",
                "json",
            ])
            .expect("parse propose create");
            match cli.command {
                Commands::Propose(ref args) => {
                    assert!(matches!(args.cmd, ProposeCommand::Create { .. }));
                }
                _ => panic!("expected Propose command"),
            }

            let cli2 = Cli::try_parse_from(["sruja", "propose", "list", "-r", ".", "-f", "text"])
                .expect("parse propose list");
            match cli2.command {
                Commands::Propose(ref args) => {
                    assert!(matches!(args.cmd, ProposeCommand::List { .. }));
                }
                _ => panic!("expected Propose command"),
            }

            let cli3 = Cli::try_parse_from([
                "sruja",
                "propose",
                "approve",
                "p1",
                "-r",
                ".",
                "--dry-run",
                "-f",
                "json",
            ])
            .expect("parse propose approve");
            match cli3.command {
                Commands::Propose(ref args) => {
                    assert!(matches!(args.cmd, ProposeCommand::Approve { .. }));
                }
                _ => panic!("expected Propose command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_check_ci_flag() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "check", "--ci"]).expect("parse");
            match cli.command {
                Commands::Check(ref args) => assert!(args.ci),
                _ => panic!("expected Check command"),
            }

            let cli2 = Cli::try_parse_from(["sruja", "check"]).expect("parse");
            match cli2.command {
                Commands::Check(ref args) => assert!(!args.ci),
                _ => panic!("expected Check command"),
            }

            let cli3 = Cli::try_parse_from(["sruja", "drift", "--ci"]).expect("parse alias");
            match cli3.command {
                Commands::Check(ref args) => assert!(args.ci),
                _ => panic!("expected Check command via drift alias"),
            }

            let cli4 = Cli::try_parse_from([
                "sruja",
                "check",
                "-r",
                ".",
                "--structural-only",
                "--advisory",
            ])
            .expect("parse");
            match cli4.command {
                Commands::Check(ref args) => {
                    assert!(args.structural_only);
                    assert!(args.advisory);
                }
                _ => panic!("expected Check command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_start_and_init_alias() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "start"]).expect("parse");
            match cli.command {
                Commands::Init(ref args) => assert_eq!(args.path, "."),
                _ => panic!("expected Init command via start"),
            }

            let cli2 = Cli::try_parse_from(["sruja", "init"]).expect("parse");
            match cli2.command {
                Commands::Init(ref args) => assert_eq!(args.path, "."),
                _ => panic!("expected Init command via init alias"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_ingest_command() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "ingest"]).expect("parse");
            match cli.command {
                Commands::Ingest(ref args) => {
                    assert!(args.sources.is_empty());
                    assert_eq!(args.repo, ".");
                    assert!(args.category.is_none());
                    assert!(args.elements.is_none());
                }
                _ => panic!("expected Ingest command"),
            }

            let cli2 = Cli::try_parse_from([
                "sruja",
                "ingest",
                "docs/adr/",
                "--category",
                "adr",
                "--elements",
                "Sruja.CLI",
            ])
            .expect("parse");
            match cli2.command {
                Commands::Ingest(ref args) => {
                    assert_eq!(args.sources, vec!["docs/adr/".to_string()]);
                    assert_eq!(args.category.as_deref(), Some("adr"));
                    assert_eq!(args.elements.as_deref(), Some("Sruja.CLI"));
                }
                _ => panic!("expected Ingest command"),
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
                Commands::Intent(ref args) => {
                    assert!(matches!(args.cmd, IntentCommand::Evaluate { .. }));
                }
                _ => panic!("expected Intent command"),
            }

            let cli2 = Cli::try_parse_from(["sruja", "intent", "history"]).expect("parse");
            match cli2.command {
                Commands::Intent(ref args) => {
                    assert!(matches!(args.cmd, IntentCommand::History { .. }));
                }
                _ => panic!("expected Intent command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_ai_as_focus_alias() {
    std::thread::Builder::new()
        .name("clap_parse_large_stack".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "ai"]).expect("parse");
            match cli.command {
                Commands::Ai(ref args) => {
                    assert_eq!(args.repo, ".");
                    assert!(args.task.is_none());
                    assert!(args.file.is_none());
                    assert!(args.element_id.is_none());
                    assert!(args.query.is_none());
                    assert!(args.base_ref.is_none());
                    assert!(args.head_ref.is_none());
                    assert!(!args.staged);
                    assert_eq!(args.max_tokens, 8000);
                    assert!(args.output.is_none());
                    assert!(!args.enrich.enrich);
                }
                _ => panic!("expected Ai command"),
            }

            let cli2 = Cli::try_parse_from(["sruja", "focus"]).expect("parse focus");
            match cli2.command {
                Commands::Focus(ref args) => {
                    assert_eq!(args.format, "text");
                }
                _ => panic!("expected Focus command"),
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
                Commands::Ai(ref args) => {
                    assert_eq!(args.task.as_deref(), Some("Fix parser diagnostics"));
                    assert_eq!(
                        args.file.as_deref(),
                        Some("crates/sruja-language/src/parser/mod.rs")
                    );
                    assert_eq!(args.element_id.as_deref(), Some("Sruja.Language"));
                    assert_eq!(args.query.as_deref(), Some("parser"));
                    assert_eq!(args.base_ref.as_deref(), Some("main"));
                    assert_eq!(args.head_ref.as_deref(), Some("HEAD"));
                    assert!(args.staged);
                    assert_eq!(args.max_tokens, 12000);
                    assert_eq!(args.output.as_deref(), Some("brief.md"));
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
                Commands::Agent(ref args) => match &args.cmd {
                    AgentCommand::Run {
                        ref repo,
                        ref goal,
                        ref file,
                        ref element_id,
                        ref query,
                        ref mode,
                        ref ai_mode,
                        ref format,
                        ref max_steps,
                        ref max_runtime_ms_per_step,
                        ref enrich,
                        ref continue_on_error,
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
                        assert!(!enrich.enrich);
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

#[test]
fn parses_focus_compact_flag() {
    std::thread::Builder::new()
        .name("clap_parse_focus_compact".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "focus", "--element-id", "Auth", "--compact"])
                .expect("parse");
            match cli.command {
                Commands::Focus(ref args) => {
                    assert_eq!(args.element_id.as_deref(), Some("Auth"));
                    assert!(args.compact);
                }
                _ => panic!("expected Focus command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

#[test]
fn parses_lookup_command() {
    std::thread::Builder::new()
        .name("clap_parse_lookup".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from([
                "sruja",
                "lookup",
                "Sruja.CLI",
                "-r",
                ".",
                "--format",
                "json",
            ])
            .expect("parse");
            match cli.command {
                Commands::Lookup(ref args) => {
                    assert_eq!(args.name, "Sruja.CLI");
                    assert_eq!(args.repo, ".");
                    assert_eq!(args.format, "json");
                }
                _ => panic!("expected Lookup command"),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}
