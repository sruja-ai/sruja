//! File watcher for repo change detection.
//!
//! Watches a directory and invokes a callback when files change (debounced).

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use notify::RecommendedWatcher as NotifyRecommendedWatcher;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WatchError {
    #[error("Watch error: {0}")]
    Watch(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Spawn a debounced file watcher on the given path. When changes are detected,
/// the callback is invoked. Returns a handle that stops the watcher when dropped.
pub fn watch_repo_debounced<F>(
    path: impl AsRef<Path>,
    debounce_ms: u64,
    on_change: F,
) -> Result<WatchHandle, WatchError>
where
    F: Fn() + Send + 'static,
{
    let path = path.as_ref().to_path_buf();
    if !path.exists() {
        return Err(WatchError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path does not exist: {}", path.display()),
        )));
    }

    let (tx, rx) = mpsc::channel::<DebounceEventResult>();
    let mut debouncer =
        new_debouncer(Duration::from_millis(debounce_ms), move |res: DebounceEventResult| {
            let _ = tx.send(res);
        })
        .map_err(|e| WatchError::Watch(e.to_string()))?;

    debouncer.watcher().watch(&path, RecursiveMode::Recursive).map_err(|e| WatchError::Watch(e.to_string()))?;

    std::thread::spawn(move || {
        for res in rx {
            match res {
                Ok(events) => {
                    if !events.is_empty() {
                        tracing::debug!("Repo change detected: {} events", events.len());
                        on_change();
                    }
                }
                Err(e) => {
                    tracing::warn!("Watch error: {:?}", e);
                }
            }
        }
    });

    Ok(WatchHandle {
        _watcher: debouncer,
        _path: path,
    })
}

/// Handle that keeps the watcher running. Drop to stop watching.
pub struct WatchHandle {
    _watcher: Debouncer<NotifyRecommendedWatcher>,
    _path: std::path::PathBuf,
}
