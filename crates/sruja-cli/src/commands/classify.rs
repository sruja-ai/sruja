//! Classify command: generate .sruja/classification.json for a repository.
//!
//! This command scans the repository structure and produces a classification
//! that describes the logical layers, boundaries, and forbidden patterns.
//!
//! Currently uses heuristic classification. Future versions will support
//! LLM-assisted classification via `--llm` flag.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::commands::context::logic::{
    classify_crate_tiers, count_crates, infer_boundaries_from_deps,
};
use crate::commands::CliError;

#[derive(Debug, Clone)]
pub struct ClassifyOptions<'a> {
    pub repo: &'a str,
    pub force: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Classification {
    pub schema_version: String,
    pub project_type: String,
    pub summary: ClassificationSummary,
    pub layers: Vec<ClassifiedLayer>,
    pub boundaries: Vec<ClassifiedBoundary>,
    pub forbidden_patterns: Vec<String>,
    pub classified_at: String,
    pub classified_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassificationSummary {
    pub crates: Option<usize>,
    pub source_files: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassifiedLayer {
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassifiedBoundary {
    pub from: String,
    pub to: String,
    pub allowed: bool,
    pub reason: String,
}

fn count_source_files(repo_root: &str) -> usize {
    let extensions = [
        "rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "kt", "scala", "c", "cpp", "cc", "h",
        "hpp", "rb", "php", "cs", "swift", "zig",
    ];
    let skip_dirs = [
        "node_modules",
        "target",
        "dist",
        "build",
        ".git",
        "vendor",
        "__pycache__",
    ];

    let mut count = 0;
    let root = Path::new(repo_root);

    fn walk_dir(dir: &Path, extensions: &[&str], skip_dirs: &[&str], count: &mut usize) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if skip_dirs.contains(&name.as_ref()) {
                    continue;
                }
                walk_dir(&path, extensions, skip_dirs, count);
            } else if let Some(ext) = path.extension() {
                if extensions.contains(&ext.to_string_lossy().as_ref()) {
                    *count += 1;
                }
            }
        }
    }

    walk_dir(root, &extensions, &skip_dirs, &mut count);
    count
}

fn classify_heuristic(repo_root: &str) -> Classification {
    let crate_count = count_crates(repo_root);
    let source_files = count_source_files(repo_root);

    let is_rust_workspace = crate_count.is_some();

    let (layers, boundaries, forbidden_patterns) = if is_rust_workspace {
        let tiers = classify_crate_tiers(repo_root);
        let layers: Vec<ClassifiedLayer> = tiers
            .into_iter()
            .map(|(name, members)| ClassifiedLayer { name, members })
            .collect();

        let boundaries: Vec<ClassifiedBoundary> = infer_boundaries_from_deps(repo_root)
            .into_iter()
            .map(|b| ClassifiedBoundary {
                from: b.from,
                to: b.to,
                allowed: b.allowed,
                reason: b.reason,
            })
            .collect();

        let forbidden_patterns = vec![
            "Lower-tier crates must not depend on higher-tier crates".to_string(),
            "sruja-cli is the top-level aggregator — no other crate should depend on it"
                .to_string(),
            "WASM-only crates must not use native-only APIs (tree-sitter, fastembed)".to_string(),
        ];

        (layers, boundaries, forbidden_patterns)
    } else {
        // For non-Rust projects, produce a minimal classification.
        // LLM integration will handle proper classification.
        (vec![], vec![], vec![])
    };

    let now = chrono::Utc::now().to_rfc3339();

    Classification {
        schema_version: "classification/v1".to_string(),
        project_type: if is_rust_workspace {
            "rust-workspace".to_string()
        } else {
            "unknown".to_string()
        },
        summary: ClassificationSummary {
            crates: crate_count,
            source_files,
        },
        layers,
        boundaries,
        forbidden_patterns,
        classified_at: now,
        classified_by: "heuristic".to_string(),
    }
}

pub fn classify(options: ClassifyOptions<'_>) -> Result<(), CliError> {
    let repo_root = Path::new(options.repo);
    if !repo_root.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", options.repo),
        )));
    }

    let classification_path = repo_root.join(".sruja").join("classification.json");

    if classification_path.exists() && !options.force {
        eprintln!(
            "Classification already exists at {}. Use --force to overwrite.",
            classification_path.display()
        );
        return Ok(());
    }

    // Ensure .sruja directory exists
    let sruja_dir = repo_root.join(".sruja");
    if !sruja_dir.exists() {
        fs::create_dir_all(&sruja_dir)?;
    }

    let classification = classify_heuristic(options.repo);

    let json = serde_json::to_string_pretty(&classification)
        .map_err(|e| CliError::validation(format!("Failed to serialize classification: {}", e)))?;

    fs::write(&classification_path, json)?;

    eprintln!(
        "Classification written to {}",
        classification_path.display()
    );
    eprintln!("  Project type: {}", classification.project_type);
    if let Some(crates) = classification.summary.crates {
        eprintln!("  Crates: {}", crates);
    }
    eprintln!("  Source files: {}", classification.summary.source_files);
    eprintln!("  Layers: {}", classification.layers.len());
    eprintln!("  Boundaries: {}", classification.boundaries.len());
    eprintln!();
    eprintln!("Edit this file to customize layers, boundaries, and forbidden patterns.");
    eprintln!(
        "Then run `sruja sync-ide-rules -r {}` to update IDE context.",
        options.repo
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_rust_workspace() {
        let repo = tempfile::tempdir().unwrap();
        let crates_dir = repo.path().join("crates").join("test-crate");
        let src_dir = crates_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            crates_dir.join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();
        fs::write(src_dir.join("lib.rs"), "pub fn hello() {}\n").unwrap();

        let classification = classify_heuristic(repo.path().to_str().unwrap());
        assert_eq!(classification.project_type, "rust-workspace");
        assert_eq!(classification.summary.crates, Some(1));
    }

    #[test]
    fn classify_non_rust_project() {
        let repo = tempfile::tempdir().unwrap();
        fs::write(repo.path().join("main.py"), "print('hello')\n").unwrap();

        let classification = classify_heuristic(repo.path().to_str().unwrap());
        assert_eq!(classification.project_type, "unknown");
        assert_eq!(classification.summary.crates, None);
    }
}
