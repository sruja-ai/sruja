use colored::Colorize;
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc;

use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebouncedEvent};

use super::scan::{drift_json_string, status_result};
use super::CliError;

pub async fn watch(repo_root: &str, clear: bool) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let (tx, mut rx) = mpsc::channel(10);
    let tx_clone = tx.clone();

    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        move |res: Result<Vec<DebouncedEvent>, _>| match res {
            Ok(events) => {
                let relevant: Vec<_> = events
                    .into_iter()
                    .filter(|e| {
                        let p = e.path.to_string_lossy();
                        !p.contains("/.git/")
                            && !p.contains("/target/")
                            && !p.contains("/node_modules/")
                            && !p.contains("/docs/")
                            && !p.contains("/book/")
                            && !p.contains("/.sruja/")
                    })
                    .collect();

                if !relevant.is_empty() {
                    let _ = tx_clone.blocking_send(relevant);
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        },
    )
    .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    debouncer
        .watcher()
        .watch(repo_path, RecursiveMode::Recursive)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    if clear {
        print!("{}[2J{}[1;1H", 27 as char, 27 as char);
    }
    println!(
        "{} Watching {} for changes...",
        "👀".cyan(),
        repo_root.bold()
    );
    println!("   Press Ctrl+C to exit. \n");

    evaluate_and_display(repo_root, clear).await?;

    while let Some(events) = rx.recv().await {
        while rx.try_recv().is_ok() {}

        let path_summary = if events.len() == 1 {
            let p = events[0]
                .path
                .strip_prefix(repo_path)
                .unwrap_or(&events[0].path);
            format!("File changed: {}", p.display())
        } else {
            format!("{} files changed", events.len())
        };

        if !clear {
            println!("\n{} {}...", "↻".yellow(), path_summary);
        }

        evaluate_and_display(repo_root, clear).await?;
    }

    Ok(())
}

async fn evaluate_and_display(repo_root: &str, clear: bool) -> Result<(), CliError> {
    if clear {
        print!("{}[2J{}[1;1H", 27 as char, 27 as char);
    }

    let time = chrono::Local::now().format("%H:%M:%S");
    println!(
        "{} {} {}",
        "─".repeat(20).truecolor(100, 100, 100),
        time.to_string().truecolor(100, 100, 100),
        "─".repeat(20).truecolor(100, 100, 100)
    );

    if let Err(e) = super::sync_cmd::sync(repo_root, "quiet").await {
        eprintln!("{} Failed to sync context: {}", "✗".red(), e);
        return Ok(());
    }

    match status_result(repo_root).await {
        Ok(status) => {
            let health_str = if let Some(score) = status.health_score {
                format!("Health: {}/100", score)
            } else {
                "".to_string()
            };

            let status_color = match status.truth_status.as_str() {
                "reviewed" => status.truth_status.green(),
                "drifted" => status.truth_status.red(),
                _ => status.truth_status.yellow(),
            };

            println!(
                "{} {} | {} | {} violations",
                "✓".green(),
                status_color.bold(),
                health_str.cyan(),
                status.violations_count.to_string().yellow()
            );

            if status.violations_count > 0 {
                println!("\n{}", "Violations:".bold());

                if let Ok(json_str) = drift_json_string(repo_root, None, false).await {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        if let Some(violations) = value.get("violations").and_then(|v| v.as_array())
                        {
                            let display_count = std::cmp::min(10, violations.len());
                            for v in violations.iter().take(display_count) {
                                let kind =
                                    v.get("kind").and_then(|k| k.as_str()).unwrap_or("unknown");
                                let msg = v.get("message").and_then(|m| m.as_str()).unwrap_or("");

                                let location = if let Some(sources) =
                                    v.get("sources").and_then(|s| s.as_array())
                                {
                                    if let Some(src) = sources.first() {
                                        if let Some(file) = src.get("file").and_then(|f| f.as_str())
                                        {
                                            if let Some(line) =
                                                src.get("line").and_then(|l| l.as_u64())
                                            {
                                                format!("{}:{}", file, line)
                                            } else {
                                                file.to_string()
                                            }
                                        } else {
                                            "".to_string()
                                        }
                                    } else {
                                        "".to_string()
                                    }
                                } else {
                                    "".to_string()
                                };

                                let loc_str = if location.is_empty() {
                                    "".to_string()
                                } else {
                                    format!(" ({})", location)
                                };

                                println!(
                                    "  {} {}{}",
                                    "•".red(),
                                    kind.bold(),
                                    loc_str.truecolor(150, 150, 150)
                                );
                                println!("    {}", msg);
                            }
                            if violations.len() > display_count {
                                println!("  ... and {} more", violations.len() - display_count);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{} Evaluation failed: {}", "✗".red(), e);
        }
    }

    println!();
    Ok(())
}
