use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_dir(repo_root: &Path, run_id: &str) -> PathBuf {
    repo_root.join(".sruja").join("runs").join(run_id)
}

pub fn write_json_snapshot(
    repo_root: &Path,
    run_id: &str,
    filename: &str,
    value: &Value,
) -> std::io::Result<PathBuf> {
    let dir = run_dir(repo_root, run_id);
    fs::create_dir_all(&dir)?;
    let path = dir.join(filename);
    fs::write(
        &path,
        serde_json::to_string_pretty(value).unwrap_or_default(),
    )?;
    Ok(path)
}

pub fn blake3_hex(s: &str) -> String {
    blake3::hash(s.as_bytes()).to_hex().to_string()
}
