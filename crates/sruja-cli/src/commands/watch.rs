//! Watch command: sync and evaluate drift live on file changes.

use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebouncedEvent};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode};

use super::CliError;
use super::violation_shared::*;
use crate::utils::{architecture_path, colors};
use sruja_diff::Violation;
use sruja_scan::scan_repo;

const DEBOUNCE_MS: u64 = 500;
const THROTTLE_COOLDOWN: Duration = Duration::from_secs(2);
const MAX_RETRIES: u32 = 2;

struct WatchState {
    last_run: Option<Instant>,
    previous_fingerprints: HashSet<String>,
    retry_count: u32,
    clear: bool,
    health_history: Vec<u8>,
    focus: Option<String>,
    last_health: u8,
}

pub async fn watch(repo_root: &str, clear: bool, focus: Option<String>) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let mut state = WatchState {
        last_run: None,
        previous_fingerprints: HashSet::new(),
        retry_count: 0,
        clear,
        health_history: Vec::new(),
        focus: focus.clone(),
        last_health: 100,
    };
    
    // Initial evaluation
    if let Ok(report) = evaluate_drift(repo_path).await {
        state.previous_fingerprints = report.violations.iter().map(fingerprint_violation).collect();
        state.last_health = report.health_score;
        state.health_history.push(report.health_score);
    }
    state.last_run = Some(Instant::now());

    render_header(repo_root, &state);

    let state = Arc::new(Mutex::new(state));
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let (refresh_tx, mut refresh_rx) = tokio::sync::mpsc::channel(1);

    // Watcher thread
    let watcher_tx = tx.clone();
    let mut debouncer = new_debouncer(
        Duration::from_millis(DEBOUNCE_MS),
        move |res: notify_debouncer_mini::DebounceEventResult| match res {
            Ok(events) => {
                let _ = watcher_tx.blocking_send(events);
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        },
    )
    .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    debouncer
        .watcher()
        .watch(repo_path, RecursiveMode::Recursive)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    // Input thread for shortcuts
    let input_refresh_tx = refresh_tx.clone();
    tokio::spawn(async move {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(Event::Key(key_event)) = event::read() {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            std::process::exit(0);
                        }
                        KeyCode::Char('r') => {
                            let _ = input_refresh_tx.send(()).await;
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    loop {
        tokio::select! {
            Some(events) = rx.recv() => {
                // Clear any pending events to debounce further
                while rx.try_recv().is_ok() {}
                if let Err(e) = handle_events(repo_root, events, Arc::clone(&state)).await {
                    eprintln!("{} {}", colors::error("Error:"), e);
                }
            }
            Some(_) = refresh_rx.recv() => {
                let mut s = state.lock().await;
                println!("  {} Manual refresh triggered...", colors::info("↻"));
                if let Err(e) = run_sync_and_report(repo_root, &mut s).await {
                    eprintln!("{} Manual refresh failed: {}", colors::error("✗"), e);
                }
            }
        }
    }
}

fn render_header(repo_root: &str, state: &WatchState) {
    if state.clear {
        print!("{}[2J{}[1;1H", 27 as char, 27 as char);
    }
    
    let repo_name = Path::new(repo_root).file_name().and_then(|n| n.to_str()).unwrap_or(repo_root);
    colors::print_header(&format!("👁  SRUJA WATCH: {}", repo_name.to_uppercase()));
    
    let health_status = if state.last_health >= 90 {
        colors::badge("HEALTHY", "success")
    } else if state.last_health >= 70 {
        colors::badge("STABLE", "info")
    } else {
        colors::badge("DRIFTED", "error")
    };

    let spark = colors::sparkline(&state.health_history);
    println!("  {} Health: {}/100 {} {}", colors::info("•"), colors::style(state.last_health).bold(), spark, health_status);
    
    if let Some(ref f) = state.focus {
        println!("  {} Focus:  {}", colors::info("•"), colors::warning(f));
    }
    
    println!("  {} Action: {}", colors::info("•"), colors::dim("[r] refresh  [q] quit  [ctrl+c] exit"));
    println!("{}", colors::dim("  ").to_string() + &colors::dim("─".repeat(50)).to_string());
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
            if path_str.contains("/.sruja/") || path_str.contains("/.git/") || path_str.contains("/target/") {
                return false;
            }
            if let Some(ref focus) = s.focus {
                return focus.split(',').any(|f| path_str.contains(f.trim()));
            }
            true
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
        match run_sync_and_report(repo_root, &mut s).await {
            Ok(_) => {
                s.last_run = Some(Instant::now());
                s.retry_count = 0;
                return Ok(());
            }
            Err(e) => {
                current_retry += 1;
                if current_retry > MAX_RETRIES {
                    return Err(e);
                }
                tokio::time::sleep(Duration::from_millis(500 * current_retry as u64)).await;
            }
        }
    }

    Ok(())
}

async fn run_sync_and_report(repo_root: &str, state: &mut WatchState) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let report = evaluate_drift(repo_path).await?;
    
    state.last_health = report.health_score;
    state.health_history.push(report.health_score);
    if state.health_history.len() > 20 {
        state.health_history.remove(0);
    }

    if state.clear {
        render_header(repo_root, state);
    }

    let current_fingerprints: HashSet<String> = report.violations.iter().map(fingerprint_violation).collect();

    let new_violations: Vec<_> = report.violations
        .iter()
        .filter(|v| !state.previous_fingerprints.contains(&fingerprint_violation(v)))
        .collect();

    let resolved: Vec<_> = state.previous_fingerprints
        .iter()
        .filter(|fp| !current_fingerprints.contains(*fp))
        .collect();

    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    
    if !new_violations.is_empty() || !resolved.is_empty() {
        let mut diff_parts = Vec::new();
        if !new_violations.is_empty() {
            diff_parts.push(colors::error(format!("+{}", new_violations.len())).to_string());
        }
        if !resolved.is_empty() {
            diff_parts.push(colors::success(format!("-{}", resolved.len())).to_string());
        }
        
        let diff_text = if !diff_parts.is_empty() {
            format!(" ({})", diff_parts.join(", "))
        } else {
            "".to_string()
        };

        println!(
            "{} [{}] {} │ health: {} │ violations: {}{}",
            colors::dim("→"),
            colors::info(now),
            if report.health_score >= 90 { colors::success("STABLE") } else { colors::error("DRIFT ") },
            colors::style(report.health_score).bold(),
            report.violations.len(),
            diff_text
        );

        for v in new_violations {
            println!(
                "      {} {}: {} {}",
                colors::error("NEW"),
                colors::style(kind_slug(v.kind)).bold(),
                v.message,
                colors::dim(v.location.as_deref().unwrap_or(""))
            );
        }

        for fp in resolved {
            // Find kind from fingerprint (rough but works for UI)
            let kind = fp.split('|').next().unwrap_or("violation");
            println!(
                "      {} Fixed: {}",
                colors::success("✨"),
                colors::style(kind).bold().italic()
            );
        }
    } else {
        // Quiet update if nothing changed but we still want to show activity
        if !state.clear {
            println!("  {} [{}] Architecture in sync. No changes detected.", colors::dim("•"), colors::dim(now));
        }
    }

    state.previous_fingerprints = current_fingerprints;
    
    // Trigger background sync
    let _ = super::sync_cmd::sync(repo_root, "quiet").await;

    Ok(())
}

struct WatchReport {
    violations: Vec<Violation>,
    #[allow(dead_code)]
    truth_status: String,
    health_score: u8,
}

async fn evaluate_drift(repo_path: &Path) -> Result<WatchReport, CliError> {
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
        Ok(WatchReport {
            violations: diff.violations,
            truth_status: truth.to_string(),
            health_score: diff.summary.health_score,
        })
    } else {
        let drift = sruja_diff::detect_architectural_drift(&graph);
        Ok(WatchReport {
            violations: drift.violations,
            truth_status: "unknown".to_string(),
            health_score: drift.health_score,
        })
    }
}
