//! Declarative loop manifest (`.sruja/loop.toml`).
//!
//! The manifest is the **user-facing contract** for autonomous loop runs:
//! structured goal, budget, scope, and deterministic verification steps.
//! It is loaded by the CLI and resolved with the standard priority chain:
//! CLI flags > manifest > defaults.

use serde::{Deserialize, Serialize};

use crate::goal::GoalSpec;
use crate::verify::VerifyStep;

fn default_max_iterations() -> usize {
    3
}
fn default_true() -> bool {
    true
}

/// Declarative configuration for `sruja agent loop`, loaded from `.sruja/loop.toml`.
///
/// ## Example
///
/// ```toml
/// [goal]
/// statement = "Add JWT authentication to all /api/* endpoints"
/// acceptance_criteria = [
///   "all existing tests pass",
///   "new tests cover token validation",
/// ]
/// constraints = ["do not modify the public API", "no new dependencies"]
///
/// max_iterations = 5
/// spend_cap_usd = 2.0
/// shell_allowlist = ["cargo", "git"]
///
/// [[verify]]
/// id = "tests"
/// command = "cargo"
/// args = ["test", "--workspace"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopManifest {
    /// Structured goal specification (statement + acceptance criteria + constraints).
    /// If `statement` is empty, the CLI `--goal` flag is required.
    #[serde(default)]
    pub goal: GoalSpec,

    /// Maximum plan→execute→critique iterations.
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,

    /// Write tests before implementation (TDD mode).
    #[serde(default = "default_true")]
    pub tdd: bool,

    /// Run the critic after every tool execution.
    #[serde(default = "default_true")]
    pub review_every_change: bool,

    /// Block all file mutations (dry-run mode).
    #[serde(default)]
    pub dry_run: bool,

    /// Shell commands the agent is allowed to execute.
    #[serde(default)]
    pub shell_allowlist: Vec<String>,

    /// USD spend cap for the entire loop run.
    #[serde(default)]
    pub spend_cap_usd: Option<f64>,

    /// Detect and terminate on repeated critique patterns (oscillation).
    #[serde(default = "default_true")]
    pub detect_oscillation: bool,

    /// Deterministic verification steps run after the loop completes.
    ///
    /// These are the **independent grader** — the agent that writes code
    /// cannot fake a passing `cargo test`. If any step fails, the loop
    /// result reports verification failure regardless of LLM critique.
    #[serde(default, rename = "verify")]
    pub verify_steps: Vec<VerifyStep>,
}

impl Default for LoopManifest {
    fn default() -> Self {
        Self {
            goal: GoalSpec::default(),
            max_iterations: default_max_iterations(),
            tdd: default_true(),
            review_every_change: default_true(),
            dry_run: false,
            shell_allowlist: Vec::new(),
            spend_cap_usd: None,
            detect_oscillation: default_true(),
            verify_steps: Vec::new(),
        }
    }
}

impl LoopManifest {
    /// Load from a TOML string.
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Load from a `.sruja/loop.toml` file path. Returns `Default` if the
    /// file does not exist (non-fatal — the manifest is optional).
    /// Logs a warning if the file exists but cannot be parsed.
    pub fn load_from_path(repo: &std::path::Path) -> Self {
        let path = repo.join(".sruja/loop.toml");
        match std::fs::read_to_string(&path) {
            Ok(content) => match Self::from_toml_str(&content) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse {}: {e}. Using default loop config.",
                        path.display()
                    );
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_manifest() {
        let toml_str = r#"
max_iterations = 5
spend_cap_usd = 1.5
"#;
        let m = LoopManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(m.max_iterations, 5);
        assert_eq!(m.spend_cap_usd, Some(1.5));
        assert!(m.goal.statement.is_empty());
        assert!(m.tdd);
    }

    #[test]
    fn parse_full_manifest() {
        let toml_str = r#"
max_iterations = 3
tdd = true
shell_allowlist = ["cargo", "git"]

[goal]
statement = "Add JWT auth"
acceptance_criteria = ["tests pass", "tokens validated"]
constraints = ["no new deps"]

[[verify]]
id = "tests"
command = "cargo"
args = ["test", "--workspace"]

[[verify]]
id = "lint"
command = "cargo"
args = ["clippy", "--", "-D", "warnings"]
"#;
        let m = LoopManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(m.goal.statement, "Add JWT auth");
        assert_eq!(m.goal.acceptance_criteria.len(), 2);
        assert_eq!(m.goal.constraints, vec!["no new deps"]);
        assert_eq!(m.verify_steps.len(), 2);
        assert_eq!(m.verify_steps[0].id, "tests");
        assert_eq!(m.verify_steps[1].args[2], "-D");
        assert_eq!(m.shell_allowlist, vec!["cargo", "git"]);
    }

    #[test]
    fn empty_file_gives_default() {
        let m = LoopManifest::from_toml_str("").unwrap();
        assert_eq!(m.max_iterations, 3);
        assert!(m.tdd);
        assert!(m.review_every_change);
    }

    #[test]
    fn missing_file_gives_default() {
        let m = LoopManifest::load_from_path(std::path::Path::new("/nonexistent"));
        assert_eq!(m.max_iterations, 3);
    }
}
