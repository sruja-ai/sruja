//! Watch command: sync and evaluate drift live on file changes.

use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebouncedEvent};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

use super::CliError;
use super::violation_shared::*;
use crate::utils::{architecture_path, colors};
use sruja_diff::Violation;
use sruja_scan::scan_repo;

const DEBOUNCE_MS: u64 = 500;
const THROTTLE_COOLDOWN: Duration = Duration::from_secs(2);
const MAX_RETRIES: u32 = 3;

struct WatchState {
    last_run: Option<Instant>,
    previous_fingerprints: HashSet<String>,
    retry_count: u32,
    clear: bool,
}

pub async fn watch(repo_root: &str, clear: bool) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    if clear {
        print!("{}[2J{}[1;1H", 27 as char, 27 as char);
    }

    colors::print_header("👁 Sruja Watcher");
    println!("  {} {}", colors::dim("Root:"), repo_root);
    println!("  {} Monitoring for changes...", colors::info("•"));
    println!("  {} Press Ctrl+C to exit.", colors::dim("ℹ"));
    println!();

    // Initial run
    let mut state = WatchState {
        last_run: None,
        previous_fingerprints: HashSet::new(),
        retry_count: 0,
        clear,
    };
    
    // Initial evaluation
    if let Ok((violations, _)) = evaluate_drift(repo_path).await {
        state.previous_fingerprints = violations.iter().map(fingerprint_violation).collect();
    }
    state.last_run = Some(Instant::now());

    let state = Arc::new(Mutex::new(state));
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    let mut debouncer = new_debouncer(
        Duration::from_millis(DEBOUNCE_MS),
        move |res: notify_debouncer_mini::DebounceEventResult| match res {
            Ok(events) => {
                let _ = tx.blocking_send(events);
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        },
    )
    .map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    debouncer
        .watcher()
        .watch(repo_path, RecursiveMode::Recursive)
        .map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    while let Some(events) = rx.recv().await {
        while rx.try_recv().is_ok() {}

        if let Err(e) = handle_events(repo_root, events, Arc::clone(&state)).await {
            eprintln!("{} {}", colors::error("Error during watch update:"), e);
        }
    }

    Ok(())
}

async fn handle_events(
    repo_root: &str,
    events: Vec<DebouncedEvent>,
    state: Arc<Mutex<WatchState>>,
) -> Result<(), CliError> {
    let mut s = state.lock().await;

    let relevant_events: Vec<_> = events
        .iter()
        .filter(|e| {
            let path_str = e.path.to_string_lossy();
            !path_str.contains("/.sruja/") && !path_str.contains("/.git/") && !path_str.contains("/target/")
        })
        .collect();

    if relevant_events.is_empty() {
        return Ok(());
    }

    if let Some(last) = s.last_run {
        if last.elapsed() < THROTTLE_COOLDOWN {
            return Ok(());
        }
    }

    let mut current_retry = 0;
    while current_retry <= MAX_RETRIES {
        if current_retry > 0 {
            let backoff = Duration::from_secs(current_retry as u64 * 2);
            println!("  {} Retrying in {}...", colors::warning("⚠"), colors::elapsed_display(backoff));
            tokio::time::sleep(backoff).await;
        }

        match run_sync_and_report(repo_root, &mut s).await {
            Ok(_) => {
                s.last_run = Some(Instant::now());
                s.retry_count = 0;
                return Ok(());
            }
            Err(e) => {
                eprintln!("{} Evaluation failed (retry {}/{}): {}", colors::error("✗"), current_retry, MAX_RETRIES, e);
                current_retry += 1;
            }
        }
    }

    Ok(())
}

async fn run_sync_and_report(repo_root: &str, state: &mut WatchState) -> Result<(), CliError> {
    if state.clear {
        print!("{}[2J{}[1;1H", 27 as char, 27 as char);
        colors::print_header("👁 Sruja Watcher (Automatic Refresh)");
        println!();
    }

    let repo_path = Path::new(repo_root);
    
    // 1. Evaluate drift
    let (violations, truth_status) = evaluate_drift(repo_path).await?;
    
    // 2. Health score
    let _health_score = if violations.is_empty() { 100 } else { 0 }; // Simplified health for watch for now, or use sruja_diff::DriftReport

    // 3. Compare violations
    let current_fingerprints: HashSet<String> = violations.iter().map(fingerprint_violation).collect();

    let new_violations: Vec<_> = violations
        .iter()
        .filter(|v| !state.previous_fingerprints.contains(&fingerprint_violation(v)))
        .collect();

    let resolved_count = state
        .previous_fingerprints
        .iter()
        .filter(|fp| !current_fingerprints.contains(*fp))
        .count();

    // 4. Report
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    let status_icon = if truth_status == "reviewed" {
        colors::success("✓")
    } else {
        colors::error("✗")
    };

    let diff_text = if !new_violations.is_empty() || resolved_count > 0 {
        format!(
            " ({}{}, {}{})",
            if !new_violations.is_empty() { colors::error("+") } else { colors::dim("+") },
            new_violations.len(),
            if resolved_count > 0 { colors::success("-") } else { colors::dim("-") },
            resolved_count
        )
    } else {
        "".to_string()
    };

    println!(
        "{} [{}] {} {} │ violations: {}{}",
        colors::dim("→"),
        colors::dim(now),
        status_icon,
        if truth_status == "reviewed" { colors::success("in sync") } else { colors::error("drifted") },
        violations.len(),
        diff_text
    );

    if !new_violations.is_empty() {
        for v in new_violations {
            let summ = summarize_violation(v);
            println!(
                "      {} {}: {} {}",
                colors::error("+"),
                colors::style(&summ.kind).bold(),
                summ.message,
                colors::dim(summ.location.as_deref().unwrap_or(""))
            );
        }
    }

    state.previous_fingerprints = current_fingerprints;
    
    // Also trigger a background sync to keep context.json up to date if possible, but safely.
    let _ = super::sync_cmd::sync(repo_root, "quiet").await;

    Ok(())
}

async fn evaluate_drift(repo_path: &Path) -> Result<(Vec<Violation>, String), CliError> {
    let graph = scan_repo(repo_path).map_err(|e| CliError::scan(e.to_string()))?;
    let baseline_path = architecture_path::resolve_architecture_path(repo_path);

    if let Some(ref bp) = baseline_path {
        let content = std::fs::read_to_string(bp)?;
        let parser = sruja_language::Parser::new(bp.to_string_lossy().as_ref());
        let program = parser.parse(&content).map_err(|diags| {
            CliError::parse_with_diagnostics(bp.to_string_lossy().to_string(), diags)
        })?;
        let proposed = sruja_diff::program_to_graph(&program);
        let diff = sruja_diff::compare_graphs(&graph, &proposed);
        let truth = match diff.truth_status {
            sruja_diff::TruthStatus::Reviewed => "reviewed",
            sruja_diff::TruthStatus::Drifted => "drifted",
            _ => "unknown",
        };
        Ok((diff.violations, truth.to_string()))
    } else {
        let drift = sruja_diff::detect_architectural_drift(&graph);
        Ok((drift.violations, "unknown".to_string()))
    }
}
