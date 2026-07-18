use sruja_cli::cli::app::Cli;
use sruja_cli::cli::commands::Commands;

/// Test 1: 'compress-stats' parses successfully as a valid subcommand.
#[test]
fn compress_stats_parses_successfully() {
    std::thread::Builder::new()
        .name("clap_parse_compress_stats".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::try_parse_from(["sruja", "compress-stats"])
                .expect("compress-stats should be a valid subcommand");
            match cli.command {
                Commands::CompressStats {
                    repo,
                    run_id,
                    format,
                } => {
                    assert_eq!(repo, ".");
                    assert!(run_id.is_none());
                    assert_eq!(format, "text");
                }
                other => panic!(
                    "expected Commands::CompressStats, got {:?}",
                    std::mem::discriminant(&other)
                ),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

/// Test 2: --run-id flag is accepted as optional.
#[test]
fn compress_stats_accepts_optional_run_id() {
    std::thread::Builder::new()
        .name("clap_parse_compress_stats_run_id".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            // Without --run-id: run_id should be None
            let cli1 = Cli::try_parse_from(["sruja", "compress-stats"])
                .expect("compress-stats without run-id");
            match cli1.command {
                Commands::CompressStats { run_id, .. } => {
                    assert!(run_id.is_none(), "run_id should be None when not provided");
                }
                other => panic!(
                    "expected Commands::CompressStats, got {:?}",
                    std::mem::discriminant(&other)
                ),
            }

            // With --run-id: run_id should be Some(...)
            let cli2 = Cli::try_parse_from(["sruja", "compress-stats", "--run-id", "run-abc123"])
                .expect("compress-stats with --run-id");
            match cli2.command {
                Commands::CompressStats { run_id, .. } => {
                    assert_eq!(
                        run_id,
                        Some("run-abc123".to_string()),
                        "run_id should be Some when --run-id is provided"
                    );
                }
                other => panic!(
                    "expected Commands::CompressStats, got {:?}",
                    std::mem::discriminant(&other)
                ),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

/// Test 3: --format flag defaults to 'text' and accepts explicit values.
#[test]
fn compress_stats_format_defaults_to_text() {
    std::thread::Builder::new()
        .name("clap_parse_compress_stats_format".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            // Default format should be "text"
            let cli1 = Cli::try_parse_from(["sruja", "compress-stats"])
                .expect("compress-stats default format");
            match cli1.command {
                Commands::CompressStats { format, .. } => {
                    assert_eq!(format, "text", "--format should default to 'text'");
                }
                other => panic!(
                    "expected Commands::CompressStats, got {:?}",
                    std::mem::discriminant(&other)
                ),
            }

            // Explicit --format json should be accepted
            let cli2 = Cli::try_parse_from(["sruja", "compress-stats", "--format", "json"])
                .expect("compress-stats with --format json");
            match cli2.command {
                Commands::CompressStats { format, .. } => {
                    assert_eq!(format, "json", "--format json should be accepted");
                }
                other => panic!(
                    "expected Commands::CompressStats, got {:?}",
                    std::mem::discriminant(&other)
                ),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}

/// Test 4: --repo flag accepts a custom repository path.
#[test]
fn compress_stats_accepts_custom_repo() {
    std::thread::Builder::new()
        .name("clap_parse_compress_stats_repo".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            // Default repo should be "."
            let cli1 = Cli::try_parse_from(["sruja", "compress-stats"])
                .expect("compress-stats without --repo");
            match cli1.command {
                Commands::CompressStats { repo, .. } => {
                    assert_eq!(repo, ".", "repo should default to '.'");
                }
                other => panic!(
                    "expected Commands::CompressStats, got {:?}",
                    std::mem::discriminant(&other)
                ),
            }

            // With --repo: repo should be the provided value
            let cli2 = Cli::try_parse_from(["sruja", "compress-stats", "--repo", "/tmp/my-repo"])
                .expect("compress-stats with --repo");
            match cli2.command {
                Commands::CompressStats { repo, .. } => {
                    assert_eq!(
                        repo, "/tmp/my-repo",
                        "repo should be the value provided via --repo"
                    );
                }
                other => panic!(
                    "expected Commands::CompressStats, got {:?}",
                    std::mem::discriminant(&other)
                ),
            }
        })
        .expect("spawn")
        .join()
        .expect("join");
}
