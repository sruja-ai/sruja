//! Density command: show current density tier and progression hints.

use serde::Serialize;
use std::path::Path;

use super::CliError;
use crate::utils::colors;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum DensityTier {
    Sparse = 0,
    Medium = 1,
    Dense = 2,
    Rich = 3,
}

impl std::fmt::Display for DensityTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DensityTier::Sparse => write!(f, "Sparse"),
            DensityTier::Medium => write!(f, "Medium"),
            DensityTier::Dense => write!(f, "Dense"),
            DensityTier::Rich => write!(f, "Rich"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DensityReport {
    pub tier: DensityTier,
    pub tier_name: String,
    pub checks: Vec<DensityCheck>,
    pub next_step: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DensityCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

pub fn compute_density(repo: &Path) -> DensityReport {
    let mut checks = Vec::new();

    // Tier 0: Sparse - code scan exists
    let scan_exists =
        repo.join(".sruja/cache/scan.json").exists() || repo.join(".sruja/graph.json").exists();
    checks.push(DensityCheck {
        name: "Code scan".to_string(),
        passed: scan_exists,
        detail: if scan_exists {
            "Graph exists".to_string()
        } else {
            "Run `sruja start`".to_string()
        },
    });

    // Tier 1: Medium - enriched graph with context score
    let context_json = repo.join(".sruja/context.json");
    let has_context_score = context_json.exists();
    checks.push(DensityCheck {
        name: "Context score".to_string(),
        passed: has_context_score,
        detail: if has_context_score {
            "Computed".to_string()
        } else {
            "Run `sruja sync`".to_string()
        },
    });

    // Tier 2: Dense - repo.sruja exists
    let repo_sruja = find_repo_sruja(repo);
    checks.push(DensityCheck {
        name: "Declared intent (repo.sruja)".to_string(),
        passed: repo_sruja.is_some(),
        detail: if repo_sruja.is_some() {
            "Found".to_string()
        } else {
            "Author repo.sruja".to_string()
        },
    });

    // Tier 3: Rich - decisions, learnings, temporal
    let has_decisions = repo.join(".sruja/decisions").is_dir();
    checks.push(DensityCheck {
        name: "Decision records".to_string(),
        passed: has_decisions,
        detail: if has_decisions {
            "Found".to_string()
        } else {
            "Create decision records".to_string()
        },
    });

    let has_learnings = repo.join(".sruja/agent_memory.json").exists();
    checks.push(DensityCheck {
        name: "Agent learnings".to_string(),
        passed: has_learnings,
        detail: if has_learnings {
            "Found".to_string()
        } else {
            "Record learnings via MCP".to_string()
        },
    });

    let has_snapshots = repo.join(".sruja/graph_snapshots.jsonl").exists();
    checks.push(DensityCheck {
        name: "Temporal tracking".to_string(),
        passed: has_snapshots,
        detail: if has_snapshots {
            "Found".to_string()
        } else {
            "Run `sruja sync` multiple times".to_string()
        },
    });

    // Compute tier
    let tier = if has_snapshots && has_learnings && has_decisions && repo_sruja.is_some() {
        DensityTier::Rich
    } else if repo_sruja.is_some() {
        DensityTier::Dense
    } else if has_context_score {
        DensityTier::Medium
    } else {
        DensityTier::Sparse
    };

    let next_step = suggest_next_step(tier);

    DensityReport {
        tier,
        tier_name: tier.to_string(),
        checks,
        next_step,
    }
}

fn suggest_next_step(tier: DensityTier) -> Option<String> {
    match tier {
        DensityTier::Sparse => Some("Run `sruja sync -r .` to reach Tier 1 (Medium).".to_string()),
        DensityTier::Medium => Some(
            "Author repo.sruja to reach Tier 2 (Dense). Run `sruja sync -r .` for author evidence."
                .to_string(),
        ),
        DensityTier::Dense => {
            Some("Record decisions and learnings to reach Tier 3 (Rich).".to_string())
        }
        DensityTier::Rich => None,
    }
}

fn find_repo_sruja(repo: &Path) -> Option<std::path::PathBuf> {
    for name in &["repo.sruja", "architecture.sruja", "arch.sruja"] {
        let path = repo.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    // Check docs/architecture/
    let arch_dir = repo.join("docs/architecture");
    if arch_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&arch_dir) {
            for entry in entries.flatten() {
                if entry
                    .path()
                    .extension()
                    .map(|e| e == "sruja")
                    .unwrap_or(false)
                {
                    return Some(entry.path());
                }
            }
        }
    }
    None
}

pub fn density_hint(repo: &Path) -> Option<String> {
    let density = compute_density(repo);
    match density.tier {
        DensityTier::Sparse => Some(
            "Tip: Run `sruja sync -r .` to enrich the graph and reach Tier 1 (Medium).".to_string(),
        ),
        DensityTier::Medium => {
            Some("Tip: Author `repo.sruja` to declare intent and reach Tier 2 (Dense).".to_string())
        }
        DensityTier::Dense => {
            Some("Tip: Record decisions and learnings to reach Tier 3 (Rich).".to_string())
        }
        DensityTier::Rich => None,
    }
}

pub async fn density(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let report = compute_density(repo_path);

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        _ => {
            let tier_label = format!("Tier {} ({})", report.tier as u8, report.tier_name);
            let colored_tier = match report.tier {
                DensityTier::Sparse => colors::warning(&tier_label),
                DensityTier::Medium => colors::info(&tier_label),
                DensityTier::Dense => colors::success(&tier_label),
                DensityTier::Rich => colors::success(&tier_label),
            };
            println!("Current Density: {}\n", colored_tier);

            for check in &report.checks {
                let icon = if check.passed {
                    colors::success("v")
                } else {
                    colors::error("x")
                };
                println!(
                    "  [{}] {}: {}",
                    icon,
                    check.name,
                    colors::dim(&check.detail)
                );
            }

            if let Some(ref next) = report.next_step {
                println!("\nNext step: {}", colors::info(next));
            } else {
                println!(
                    "\n{}",
                    colors::success("You have reached maximum density (Tier 3 Rich)!")
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_density_sparse() {
        let dir = tempdir().unwrap();
        let report = compute_density(dir.path());
        assert_eq!(report.tier, DensityTier::Sparse);
        assert!(report.next_step.is_some());
    }

    #[test]
    fn test_density_medium() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".sruja/cache")).unwrap();
        std::fs::write(dir.path().join(".sruja/cache/scan.json"), "{}").unwrap();
        std::fs::write(dir.path().join(".sruja/context.json"), "{}").unwrap();

        let report = compute_density(dir.path());
        assert_eq!(report.tier, DensityTier::Medium);
    }

    #[test]
    fn test_density_dense() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".sruja/cache")).unwrap();
        std::fs::write(dir.path().join(".sruja/cache/scan.json"), "{}").unwrap();
        std::fs::write(dir.path().join(".sruja/context.json"), "{}").unwrap();
        std::fs::write(dir.path().join("repo.sruja"), "").unwrap();

        let report = compute_density(dir.path());
        assert_eq!(report.tier, DensityTier::Dense);
    }

    #[test]
    fn test_density_rich() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".sruja/cache")).unwrap();
        std::fs::write(dir.path().join(".sruja/cache/scan.json"), "{}").unwrap();
        std::fs::write(dir.path().join(".sruja/context.json"), "{}").unwrap();
        std::fs::write(dir.path().join("repo.sruja"), "").unwrap();
        std::fs::create_dir(dir.path().join(".sruja/decisions")).unwrap();
        std::fs::write(dir.path().join(".sruja/agent_memory.json"), "{}").unwrap();
        std::fs::write(dir.path().join(".sruja/graph_snapshots.jsonl"), "").unwrap();

        let report = compute_density(dir.path());
        assert_eq!(report.tier, DensityTier::Rich);
        assert!(report.next_step.is_none());
    }

    #[test]
    fn test_density_hint() {
        let dir = tempdir().unwrap();
        let hint = density_hint(dir.path());
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("sruja sync"));
    }

    #[test]
    fn test_find_repo_sruja() {
        let dir = tempdir().unwrap();
        assert!(find_repo_sruja(dir.path()).is_none());

        std::fs::write(dir.path().join("repo.sruja"), "").unwrap();
        assert!(find_repo_sruja(dir.path()).is_some());
    }

    #[test]
    fn test_tier_ordering() {
        assert!(DensityTier::Sparse < DensityTier::Medium);
        assert!(DensityTier::Medium < DensityTier::Dense);
        assert!(DensityTier::Dense < DensityTier::Rich);
    }
}
