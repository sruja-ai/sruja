use std::path::Path;

/// Calculate context age in hours from `.sruja/context.json` modification time.
pub fn context_age_hours(repo_path: &Path) -> u64 {
    let ctx_path = repo_path.join(".sruja").join("context.json");
    if !ctx_path.exists() {
        let scan_path = repo_path.join(crate::commands::SCAN_CACHE_PATH);
        let legacy_path = repo_path.join(".sruja/graph.json");
        let graph_path = if scan_path.exists() {
            scan_path
        } else {
            legacy_path
        };
        if !graph_path.exists() {
            return 999;
        }
        return file_age_hours(&graph_path);
    }
    file_age_hours(&ctx_path)
}

pub fn file_age_hours(path: &Path) -> u64 {
    if let Ok(metadata) = std::fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                return elapsed.as_secs() / 3600;
            }
        }
    }
    999
}
