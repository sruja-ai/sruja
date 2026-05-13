use crate::commands::CliError;
use crate::utils::run_snapshots::run_dir;
use std::fs;
use std::path::Path;

pub async fn run_show(repo: &str, run_id: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let dir = run_dir(repo_path, run_id);
    if !dir.exists() {
        return Err(CliError::validation(format!(
            "Run snapshot not found: {}",
            dir.display()
        )));
    }

    let mut files: Vec<String> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .collect();
    files.sort();

    if format == "json" {
        let mut snapshots = serde_json::Map::new();
        for f in &files {
            let p = dir.join(f);
            let content = fs::read_to_string(&p).unwrap_or_default();
            let v: serde_json::Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| serde_json::json!({ "error": "invalid_json", "raw": content }));
            snapshots.insert(f.clone(), v);
        }
        let out = serde_json::json!({
            "schema_version": "run_show/v1",
            "repo": repo,
            "run_id": run_id,
            "files": files,
            "snapshots": snapshots,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("Run: {}", run_id);
    println!("Repo: {}", repo_path.display());
    println!("Snapshots:");
    for f in &files {
        println!("- {}", f);
    }
    Ok(())
}
