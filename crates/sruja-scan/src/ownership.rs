//! Ownership inference from code: CODEOWNERS, package manifests, git history.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipMap {
    /// path pattern or module id -> owner team/individual
    pub owners: HashMap<String, String>,
    pub source: OwnershipSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnershipSource {
    Codeowners,
    PackageManifest,
    GitHistory,
    Fallback,
}

/// Parse a CODEOWNERS file (GitHub, GitLab, Bitbucket format).
/// Returns a map of glob pattern -> owner.
pub fn parse_codeowners(repo_path: &Path) -> Option<OwnershipMap> {
    let codeowners_paths = [
        "CODEOWNERS",
        ".github/CODEOWNERS",
        ".gitlab/CODEOWNERS",
        "docs/CODEOWNERS",
    ];

    let (content, _path) = codeowners_paths.iter().find_map(|p| {
        let full = repo_path.join(p);
        std::fs::read_to_string(&full)
            .ok()
            .map(|c| (c, p.to_string()))
    })?;

    let mut owners = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let pattern = parts[0];
        let owner = parts[1..]
            .iter()
            .find(|p| p.starts_with('@') || p.contains('/'));

        if let Some(owner) = owner {
            owners.insert(pattern.to_string(), owner.to_string());
        }
    }

    if owners.is_empty() {
        return None;
    }

    Some(OwnershipMap {
        owners,
        source: OwnershipSource::Codeowners,
    })
}

/// Extract owners from package manifests (Cargo.toml, package.json).
pub fn parse_manifest_owners(repo_path: &Path) -> OwnershipMap {
    let mut owners = HashMap::new();

    let cargo_path = repo_path.join("Cargo.toml");
    if let Ok(content) = std::fs::read_to_string(&cargo_path) {
        if let Some(authors) = extract_cargo_authors(&content) {
            owners.insert("cargo workspace".to_string(), authors);
        }
    }

    let npm_path = repo_path.join("package.json");
    if let Ok(content) = std::fs::read_to_string(&npm_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(author) = val.get("author").and_then(|a| a.as_str()) {
                owners.insert("npm package".to_string(), author.to_string());
            }
            if let Some(maintainers) = val.get("maintainers").and_then(|m| m.as_array()) {
                let names: Vec<String> = maintainers
                    .iter()
                    .filter_map(|m| {
                        m.as_str().map(|s| s.to_string()).or_else(|| {
                            m.get("name")
                                .and_then(|n| n.as_str())
                                .map(|s| s.to_string())
                        })
                    })
                    .collect();
                if !names.is_empty() {
                    owners.insert("npm maintainers".to_string(), names.join(", "));
                }
            }
        }
    }

    OwnershipMap {
        owners,
        source: OwnershipSource::PackageManifest,
    }
}

/// Infer owner from git history (most frequent author).
pub fn infer_from_git(repo_path: &Path) -> OwnershipMap {
    let mut owners = HashMap::new();

    let output = std::process::Command::new("git")
        .args(["log", "--format=%ae", "--all", "-n", "100"])
        .current_dir(repo_path)
        .output()
        .ok();

    if let Some(output) = output {
        if output.status.success() {
            let emails: Vec<&str> = std::str::from_utf8(&output.stdout)
                .unwrap_or("")
                .lines()
                .filter(|l| !l.is_empty())
                .collect();

            if !emails.is_empty() {
                let mut counts: HashMap<&str, usize> = HashMap::new();
                for email in &emails {
                    *counts.entry(email).or_insert(0) += 1;
                }

                if let Some((top_author, _)) = counts.into_iter().max_by_key(|(_, c)| *c) {
                    owners.insert("git primary".to_string(), top_author.to_string());
                }
            }
        }
    }

    OwnershipMap {
        owners,
        source: OwnershipSource::GitHistory,
    }
}

/// Resolve ownership for a given path/module from all sources.
pub fn resolve_ownership(repo_path: &Path) -> HashMap<String, String> {
    let mut result = HashMap::new();

    if let Some(map) = parse_codeowners(repo_path) {
        result.extend(map.owners);
        return result;
    }

    let manifest = parse_manifest_owners(repo_path);
    result.extend(manifest.owners);

    if result.is_empty() {
        let git = infer_from_git(repo_path);
        result.extend(git.owners);
    }

    result
}

fn extract_cargo_authors(content: &str) -> Option<String> {
    let in_package = content.contains("[package]");
    if !in_package {
        return None;
    }

    let mut authors_line = None;
    let mut in_package_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[package]" {
            in_package_section = true;
            continue;
        }
        if trimmed.starts_with('[') && trimmed != "[package]" {
            in_package_section = false;
            continue;
        }
        if in_package_section && trimmed.starts_with("authors") {
            authors_line = Some(line.to_string());
            break;
        }
    }

    authors_line.map(|line| {
        let authors: Vec<String> = line
            .split('=')
            .nth(1)
            .unwrap_or("")
            .split(',')
            .map(|a| {
                a.trim()
                    .trim_matches('"')
                    .trim_matches('[')
                    .trim_matches(']')
                    .trim()
                    .to_string()
            })
            .filter(|a| !a.is_empty())
            .collect();
        authors.join(", ")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_codeowners_format() {
        let dir = tempfile::tempdir().unwrap();
        let codeowners = dir.path().join("CODEOWNERS");
        std::fs::write(&codeowners, "# Owners\n*.rs @rust-team\n/src/ @core-team\n").unwrap();

        let result = parse_codeowners(dir.path()).unwrap();
        assert_eq!(result.source, OwnershipSource::Codeowners);
        assert_eq!(result.owners.get("*.rs"), Some(&"@rust-team".to_string()));
        assert_eq!(result.owners.get("/src/"), Some(&"@core-team".to_string()));
    }

    #[test]
    fn test_parse_manifest_owners_npm() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("package.json");
        std::fs::write(&pkg, r#"{"name": "test", "author": "test@example.com"}"#).unwrap();

        let result = parse_manifest_owners(dir.path());
        assert_eq!(
            result.owners.get("npm package"),
            Some(&"test@example.com".to_string())
        );
    }

    #[test]
    fn test_parse_codeowners_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let result = parse_codeowners(dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_codeowners_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let codeowners = dir.path().join("CODEOWNERS");
        std::fs::write(&codeowners, "# Only comments\n").unwrap();

        let result = parse_codeowners(dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_codeowners_github_path() {
        let dir = tempfile::tempdir().unwrap();
        let github_dir = dir.path().join(".github");
        std::fs::create_dir(&github_dir).unwrap();
        let codeowners = github_dir.join("CODEOWNERS");
        std::fs::write(&codeowners, "*.rs @team\n").unwrap();

        let result = parse_codeowners(dir.path());
        assert!(result.is_some());
        assert_eq!(result.unwrap().owners.get("*.rs"), Some(&"@team".to_string()));
    }

    #[test]
    fn test_parse_codeowners_gitlab_path() {
        let dir = tempfile::tempdir().unwrap();
        let gitlab_dir = dir.path().join(".gitlab");
        std::fs::create_dir(&gitlab_dir).unwrap();
        let codeowners = gitlab_dir.join("CODEOWNERS");
        std::fs::write(&codeowners, "*.rs @team\n").unwrap();

        let result = parse_codeowners(dir.path());
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_codeowners_docs_path() {
        let dir = tempfile::tempdir().unwrap();
        let docs_dir = dir.path().join("docs");
        std::fs::create_dir(&docs_dir).unwrap();
        let codeowners = docs_dir.join("CODEOWNERS");
        std::fs::write(&codeowners, "*.md @docs-team\n").unwrap();

        let result = parse_codeowners(dir.path());
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_codeowners_multiple_patterns() {
        let dir = tempfile::tempdir().unwrap();
        let codeowners = dir.path().join("CODEOWNERS");
        std::fs::write(
            &codeowners,
            "# Ownership\n*.rs @rust-team\n*.ts @frontend-team\n/src/ @core-team\n",
        )
        .unwrap();

        let result = parse_codeowners(dir.path()).unwrap();
        assert_eq!(result.owners.len(), 3);
        assert_eq!(result.owners.get("*.rs"), Some(&"@rust-team".to_string()));
        assert_eq!(result.owners.get("*.ts"), Some(&"@frontend-team".to_string()));
        assert_eq!(result.owners.get("/src/"), Some(&"@core-team".to_string()));
    }

    #[test]
    fn test_parse_codeowners_line_without_owner() {
        let dir = tempfile::tempdir().unwrap();
        let codeowners = dir.path().join("CODEOWNERS");
        std::fs::write(&codeowners, "*.rs\n@team\n").unwrap();

        let result = parse_codeowners(dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_manifest_owners_cargo() {
        let dir = tempfile::tempdir().unwrap();
        let cargo = dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo,
            "[package]\nname = \"test\"\nauthors = [\"Alice\", \"Bob\"]\n",
        )
        .unwrap();

        let result = parse_manifest_owners(dir.path());
        let authors = result.owners.get("cargo workspace").unwrap();
        assert!(authors.contains("Alice"));
        assert!(authors.contains("Bob"));
    }

    #[test]
    fn test_parse_manifest_owners_cargo_no_package() {
        let dir = tempfile::tempdir().unwrap();
        let cargo = dir.path().join("Cargo.toml");
        std::fs::write(&cargo, "[workspace]\nmembers = [\"crate1\"]\n").unwrap();

        let result = parse_manifest_owners(dir.path());
        assert!(result.owners.is_empty());
    }

    #[test]
    fn test_parse_manifest_owners_npm_maintainers() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("package.json");
        std::fs::write(
            &pkg,
            r#"{"name": "test", "maintainers": ["Alice", "Bob"]}"#,
        )
        .unwrap();

        let result = parse_manifest_owners(dir.path());
        assert_eq!(
            result.owners.get("npm maintainers"),
            Some(&"Alice, Bob".to_string())
        );
    }

    #[test]
    fn test_parse_manifest_owners_npm_maintainers_objects() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("package.json");
        std::fs::write(
            &pkg,
            r#"{"name": "test", "maintainers": [{"name": "Alice"}, {"name": "Bob"}]}"#,
        )
        .unwrap();

        let result = parse_manifest_owners(dir.path());
        assert_eq!(
            result.owners.get("npm maintainers"),
            Some(&"Alice, Bob".to_string())
        );
    }

    #[test]
    fn test_parse_manifest_owners_no_files() {
        let dir = tempfile::tempdir().unwrap();
        let result = parse_manifest_owners(dir.path());
        assert!(result.owners.is_empty());
        assert_eq!(result.source, OwnershipSource::PackageManifest);
    }

    #[test]
    fn test_parse_manifest_owners_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("package.json");
        std::fs::write(&pkg, "not valid json").unwrap();

        let result = parse_manifest_owners(dir.path());
        assert!(result.owners.is_empty());
    }

    #[test]
    fn test_extract_cargo_authors_with_package() {
        let content = "[package]\nname = \"test\"\nauthors = [\"Alice\", \"Bob\"]\n";
        let result = extract_cargo_authors(content);
        assert!(result.is_some());
        let authors = result.unwrap();
        assert!(authors.contains("Alice"));
        assert!(authors.contains("Bob"));
    }

    #[test]
    fn test_extract_cargo_authors_without_package() {
        let content = "[workspace]\nmembers = [\"crate1\"]\n";
        let result = extract_cargo_authors(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_cargo_authors_no_authors() {
        let content = "[package]\nname = \"test\"\nversion = \"0.1.0\"\n";
        let result = extract_cargo_authors(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_cargo_authors_single_author() {
        let content = "[package]\nname = \"test\"\nauthors = [\"Alice\"]\n";
        let result = extract_cargo_authors(content);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Alice"));
    }

    #[test]
    fn test_resolve_ownership_with_codeowners() {
        let dir = tempfile::tempdir().unwrap();
        let codeowners = dir.path().join("CODEOWNERS");
        std::fs::write(&codeowners, "*.rs @rust-team\n").unwrap();

        let result = resolve_ownership(dir.path());
        assert_eq!(result.get("*.rs"), Some(&"@rust-team".to_string()));
    }

    #[test]
    fn test_resolve_ownership_with_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("package.json");
        std::fs::write(&pkg, r#"{"name": "test", "author": "test@example.com"}"#).unwrap();

        let result = resolve_ownership(dir.path());
        assert_eq!(
            result.get("npm package"),
            Some(&"test@example.com".to_string())
        );
    }

    #[test]
    fn test_ownership_source_equality() {
        assert_eq!(OwnershipSource::Codeowners, OwnershipSource::Codeowners);
        assert_eq!(
            OwnershipSource::PackageManifest,
            OwnershipSource::PackageManifest
        );
        assert_eq!(OwnershipSource::GitHistory, OwnershipSource::GitHistory);
        assert_eq!(OwnershipSource::Fallback, OwnershipSource::Fallback);
        assert_ne!(OwnershipSource::Codeowners, OwnershipSource::PackageManifest);
    }

    #[test]
    fn test_ownership_map_clone() {
        let map = OwnershipMap {
            owners: HashMap::from([("*.rs".to_string(), "@team".to_string())]),
            source: OwnershipSource::Codeowners,
        };

        let cloned = map.clone();
        assert_eq!(map.owners, cloned.owners);
        assert_eq!(map.source, cloned.source);
    }
}
