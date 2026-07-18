use crate::commands::CliError;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Metadata for a single run entry, derived from the runs directory.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RunEntry {
    pub run_id: String,
    pub timestamp: String,
    pub status: String,
}

/// Read run entries from the `.sruja/runs/` directory under `repo_root`.
///
/// Each subdirectory is treated as one run. The run ID is the directory name.
/// Timestamp is read from `metadata.json` (key `created_at` or `timestamp`),
/// falling back to directory modification time. Convergence/status is read
/// from `snapshot.json` (key `status` or `convergence`), defaulting to
/// `"unknown"`.
///
/// Results are sorted by timestamp descending and truncated to `limit` entries.
pub fn ls_runs(repo_root: &Path, limit: Option<usize>) -> Result<Vec<RunEntry>, CliError> {
    let runs_base = repo_root.join(".sruja").join("runs");

    if !runs_base.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<RunEntry> = Vec::new();

    let dir_entries = fs::read_dir(&runs_base).map_err(CliError::Io)?;

    for entry in dir_entries {
        let entry = entry.map_err(CliError::Io)?;
        let file_type = entry.file_type().map_err(CliError::Io)?;

        if !file_type.is_dir() {
            continue;
        }

        let run_id = entry
            .file_name()
            .to_string_lossy()
            .into_owned();

        let dir_path = entry.path();

        // --- timestamp ---
        let mut timestamp = String::new();

        // Try metadata.json first
        let meta_path = dir_path.join("metadata.json");
        if meta_path.exists() {
            if let Ok(content) = fs::read_to_string(&meta_path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(ts) = v
                        .get("created_at")
                        .or_else(|| v.get("timestamp"))
                        .and_then(|v| v.as_str())
                    {
                        timestamp = ts.to_string();
                    }
                }
            }
        }

        // Fallback: directory mtime as ISO-like string
        if timestamp.is_empty() {
            if let Ok(meta) = fs::metadata(&dir_path) {
                if let Ok(modified) = meta.modified() {
                    let dur = modified
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default();
                    timestamp = format!("{}", dur.as_secs());
                }
            }
        }

        // --- status / convergence ---
        let mut status = "unknown".to_string();

        // Try snapshot.json
        let snapshot_path = dir_path.join("snapshot.json");
        if snapshot_path.exists() {
            if let Ok(content) = fs::read_to_string(&snapshot_path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(s) = v
                        .get("status")
                        .or_else(|| v.get("convergence"))
                        .or_else(|| v.get("result"))
                        .and_then(|v| v.as_str())
                    {
                        status = s.to_string();
                    }
                }
            }
        }

        // Also try metadata.json for status
        if status == "unknown" && meta_path.exists() {
            if let Ok(content) = fs::read_to_string(&meta_path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(s) = v
                        .get("status")
                        .or_else(|| v.get("convergence"))
                        .and_then(|v| v.as_str())
                    {
                        status = s.to_string();
                    }
                }
            }
        }

        entries.push(RunEntry {
            run_id,
            timestamp,
            status,
        });
    }

    // Sort by timestamp descending (lexicographic works for ISO-8601 and epoch strings)
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Apply limit
    if let Some(limit) = limit {
        entries.truncate(limit);
    }

    Ok(entries)
}

/// Public async command handler exposed to the CLI.
///
/// Reads `.sruja/runs/` under `repo`, lists run entries sorted by timestamp
/// descending, applies the given `limit`, and prints output in `format`
/// (`"json"` or text table).
pub async fn ls_runs_cmd(repo: &str, limit: usize, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let effective_limit = if limit == 0 { None } else { Some(limit) };
    let entries = ls_runs(repo_path, effective_limit)?;

    if format == "json" {
        let out = serde_json::json!({
            "schema_version": "ls_runs/v1",
            "repo": repo,
            "count": entries.len(),
            "runs": entries,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    // Text table output
    if entries.is_empty() {
        println!("No runs found in .sruja/runs/");
        return Ok(());
    }

    // Header
    println!(
        "{:<45} {:<30} {}",
        "RUN ID", "TIMESTAMP", "STATUS"
    );
    println!("{}", "-".repeat(90));

    for entry in &entries {
        println!(
            "{:<45} {:<30} {}",
            entry.run_id, entry.timestamp, entry.status
        );
    }

    println!();
    println!("{} run(s) listed.", entries.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_run(root: &Path, run_id: &str, timestamp: &str, status: &str) {
        let dir = root.join(".sruja").join("runs").join(run_id);
        fs::create_dir_all(&dir).unwrap();
        let meta = serde_json::json!({
            "created_at": timestamp,
            "status": status,
        });
        fs::write(
            dir.join("metadata.json"),
            serde_json::to_string_pretty(&meta).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_list_runs_with_metadata() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        create_run(root, "run-001", "2025-01-01T10:00:00Z", "success");
        create_run(root, "run-002", "2025-01-02T12:00:00Z", "failure");
        create_run(root, "run-003", "2025-01-03T08:00:00Z", "success");

        let runs = ls_runs(root, None).expect("ls_runs should succeed");
        assert_eq!(runs.len(), 3, "expected 3 run entries");

        let ids: Vec<&str> = runs.iter().map(|r| r.run_id.as_str()).collect();
        assert!(ids.contains(&"run-001"));
        assert!(ids.contains(&"run-002"));
        assert!(ids.contains(&"run-003"));

        let run1 = runs.iter().find(|r| r.run_id == "run-001").unwrap();
        assert_eq!(run1.timestamp, "2025-01-01T10:00:00Z");
        assert_eq!(run1.status, "success");

        let run2 = runs.iter().find(|r| r.run_id == "run-002").unwrap();
        assert_eq!(run2.status, "failure");
    }

    #[test]
    fn test_sorted_descending_by_timestamp() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        create_run(root, "run-a", "2025-06-01T00:00:00Z", "success");
        create_run(root, "run-b", "2025-06-02T00:00:00Z", "success");
        create_run(root, "run-c", "2025-06-03T00:00:00Z", "success");
        create_run(root, "run-d", "2025-06-04T00:00:00Z", "success");
        create_run(root, "run-e", "2025-06-05T00:00:00Z", "success");

        let runs = ls_runs(root, None).expect("ls_runs should succeed");
        assert_eq!(runs.len(), 5);
        // Should be sorted descending
        assert_eq!(runs[0].run_id, "run-e");
        assert_eq!(runs[1].run_id, "run-d");
        assert_eq!(runs[2].run_id, "run-c");
        assert_eq!(runs[3].run_id, "run-b");
        assert_eq!(runs[4].run_id, "run-a");
    }

    #[test]
    fn test_limit_flag_truncates_results() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        create_run(root, "run-a", "2025-06-01T00:00:00Z", "success");
        create_run(root, "run-b", "2025-06-02T00:00:00Z", "success");
        create_run(root, "run-c", "2025-06-03T00:00:00Z", "success");
        create_run(root, "run-d", "2025-06-04T00:00:00Z", "success");
        create_run(root, "run-e", "2025-06-05T00:00:00Z", "success");

        let runs = ls_runs(root, Some(2)).expect("ls_runs should succeed");
        assert_eq!(runs.len(), 2, "limit=2 should return exactly 2 entries");

        // Most recent first
        assert_eq!(runs[0].run_id, "run-e");
        assert_eq!(runs[1].run_id, "run-d");
    }

    #[test]
    fn test_limit_larger_than_count_returns_all() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        create_run(root, "run-x", "2025-03-01T00:00:00Z", "success");
        create_run(root, "run-y", "2025-03-02T00:00:00Z", "success");

        let runs = ls_runs(root, Some(100)).expect("ls_runs should succeed");
        assert_eq!(
            runs.len(),
            2,
            "limit larger than count should return all entries"
        );
    }

    #[test]
    fn test_limit_zero_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        create_run(root, "run-1", "2025-01-01T00:00:00Z", "success");

        let runs = ls_runs(root, Some(0)).expect("ls_runs should succeed");
        assert!(runs.is_empty(), "limit=0 should return no entries");
    }

    // ---------------------------------------------------------------
    // Edge cases
    // ---------------------------------------------------------------

    #[test]
    fn test_empty_runs_directory_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::create_dir_all(root.join(".sruja").join("runs")).unwrap();
        let runs = ls_runs(root, None).expect("should succeed on empty dir");
        assert!(runs.is_empty());
    }

    #[test]
    fn test_missing_runs_directory_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // No .sruja/runs/ at all
        let runs = ls_runs(root, None).expect("should succeed when runs dir missing");
        assert!(runs.is_empty());
    }

    #[test]
    fn test_run_without_metadata_uses_mtime() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let dir = root.join(".sruja").join("runs").join("bare-run");
        fs::create_dir_all(&dir).unwrap();

        let runs = ls_runs(root, None).expect("should succeed");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].run_id, "bare-run");
        // mtime fallback should produce a numeric-looking timestamp
        assert!(!runs[0].timestamp.is_empty());
    }

    #[test]
    fn test_file_entries_in_runs_dir_are_skipped() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let runs_dir = root.join(".sruja").join("runs");
        fs::create_dir_all(&runs_dir).unwrap();

        // Create a file (not a directory) in runs/
        fs::write(runs_dir.join("not-a-run.txt"), "ignored").unwrap();
        create_run(root, "real-run", "2025-01-01T00:00:00Z", "success");

        let runs = ls_runs(root, None).expect("should succeed");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].run_id, "real-run");
    }

    #[test]
    fn test_status_from_snapshot_json() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let dir = root.join(".sruja").join("runs").join("snap-run");
        fs::create_dir_all(&dir).unwrap();

        // metadata.json without status
        fs::write(
            dir.join("metadata.json"),
            r#"{"created_at": "2025-05-01T00:00:00Z"}"#,
        )
        .unwrap();

        // snapshot.json with convergence info
        let snapshot = serde_json::json!({
            "convergence": "converged",
            "score": 95,
        });
        fs::write(
            dir.join("snapshot.json"),
            serde_json::to_string_pretty(&snapshot).unwrap(),
        )
        .unwrap();

        let runs = ls_runs(root, None).expect("should succeed");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, "converged");
    }
}
