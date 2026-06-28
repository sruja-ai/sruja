use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::budget::PipelineBudgets;
use super::types::StageDef;

/// Complete pipeline configuration loaded from `.sruja/pipeline.toml`.
///
/// Everything is config-driven — role names, models, stage order, budgets,
/// area patterns, verify steps. The config can be auto-generated from a goal
/// using [`generate_from_goal`], then hand-edited before running.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineManifest {
    /// Stage definitions in execution order.
    pub stages: Vec<StageDef>,
    /// Model assignments keyed by stage model reference.
    pub models: HashMap<String, ModelEntry>,
    /// Area partitioning rules for parallel work.
    #[serde(default)]
    pub areas: Vec<AreaDef>,
    /// Budgets and convergence thresholds.
    #[serde(default)]
    pub budgets: PipelineBudgets,
    /// Verify steps for programmatic checks (Judge Phase-1 + Fixer post-fix).
    #[serde(default)]
    pub verify: Vec<VerifyStepDef>,
    /// Path (relative to repo root) where agent prompt markdown files live.
    #[serde(default = "default_agents_dir")]
    pub agents_dir: String,
    /// Cross-cycle lesson cap per role.
    #[serde(default = "default_max_lessons_per_role")]
    pub max_lessons_per_role: usize,
    /// The goal this pipeline was generated for (optional, informational).
    #[serde(default)]
    pub goal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelEntry {
    Single(String),
    Pair(Vec<String>),
}

impl ModelEntry {
    pub fn resolve(&self) -> &[String] {
        match self {
            Self::Single(m) => std::slice::from_ref(m),
            Self::Pair(v) => v.as_slice(),
        }
    }
}

/// An area of the project, defined by file glob patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaDef {
    pub name: String,
    pub patterns: Vec<String>,
}

/// A verifiable step (tests, lint, typecheck) used by Judge Phase-1 and Fixer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyStepDef {
    pub id: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_verify_timeout")]
    pub timeout_ms: u64,
}

fn default_verify_timeout() -> u64 { 180_000 }
fn default_agents_dir() -> String { ".sruja/agents".to_string() }
fn default_max_lessons_per_role() -> usize { 15 }

impl Default for PipelineManifest {
    fn default() -> Self {
        Self {
            stages: vec![],
            models: HashMap::new(),
            areas: vec![],
            budgets: PipelineBudgets::default(),
            verify: vec![],
            agents_dir: default_agents_dir(),
            max_lessons_per_role: default_max_lessons_per_role(),
            goal: String::new(),
        }
    }
}

impl PipelineManifest {
    /// Load a pipeline definition by name from `.sruja/pipelines/{name}.toml`.
    /// Returns the default (empty) manifest when the file doesn't exist.
    pub fn load(repo: &Path, name: &str) -> Self {
        let path = Self::path_for(repo, name);
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!("warning: failed to parse {}: {e}", path.display());
            Self::default()
        })
    }

    /// Save this pipeline definition to `.sruja/pipelines/{name}.toml`.
    pub fn save(&self, repo: &Path, name: &str) -> Result<(), std::io::Error> {
        let path = Self::path_for(repo, name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        std::fs::write(path, content)
    }

    /// Full path for a pipeline definition file.
    fn path_for(repo: &Path, name: &str) -> std::path::PathBuf {
        repo.join(".sruja").join("pipelines").join(format!("{name}.toml"))
    }

    /// Resolve model names for a given model key.
    pub fn resolve_models(&self, model_key: &str) -> Vec<String> {
        match self.models.get(model_key) {
            Some(entry) => entry.resolve().to_vec(),
            None => vec![model_key.to_string()],
        }
    }

    /// Get the prompt file path for a stage (resolved relative to repo root).
    pub fn prompt_path(&self, stage: &StageDef, repo: &Path) -> Option<std::path::PathBuf> {
        stage.prompt_file.as_ref().map(|f| {
            if f.starts_with('/') {
                std::path::PathBuf::from(f)
            } else {
                repo.join(&self.agents_dir).join(f)
            }
        })
    }

    /// Enabled stages in order.
    pub fn enabled_stages(&self) -> Vec<&StageDef> {
        self.stages.iter().filter(|s| s.enabled).collect()
    }

    /// Check if this manifest has any stages configured.
    pub fn has_stages(&self) -> bool {
        self.stages.iter().any(|s| s.enabled)
    }
}

// ---------------------------------------------------------------------------
// Auto-generation from a goal
// ---------------------------------------------------------------------------

/// Generate a default pipeline manifest from a natural-language goal.
///
/// This creates a sensible default pipeline with analyzer → prober →
/// confirmer → fixer → auditor → retester → judge stages, all using
/// the same model (for single-model setups). Users can then edit
/// `.sruja/pipeline.toml` to assign different models per role.
pub fn generate_from_goal(goal: &str) -> PipelineManifest {
    let slug = goal_to_slug(goal);

    let mut models = HashMap::new();

    // All roles default to a single model name that users can customize.
    // In single-model setups, all roles use the same model.
    for role in &["analyzer", "prober", "confirmer", "fixer", "auditor", "retester", "judge"] {
        models.insert(role.to_string(), ModelEntry::Single("model-name".to_string()));
    }

    PipelineManifest {
        goal: goal.to_string(),
        stages: vec![
            StageDef {
                id: "analyzer".into(), enabled: true, parallel: false,
                model: "analyzer".into(), prompt_file: Some(format!("{slug}_analyzer.md")),
                phase_1_verify: false,
            },
            StageDef {
                id: "self_review".into(), enabled: true, parallel: false,
                model: "analyzer".into(), prompt_file: Some(format!("{slug}_self_review.md")),
                phase_1_verify: false,
            },
            StageDef {
                id: "prober".into(), enabled: true, parallel: false,
                model: "prober".into(), prompt_file: Some(format!("{slug}_prober.md")),
                phase_1_verify: false,
            },
            StageDef {
                id: "confirmer".into(), enabled: true, parallel: false,
                model: "confirmer".into(), prompt_file: Some(format!("{slug}_confirmer.md")),
                phase_1_verify: false,
            },
            StageDef {
                id: "fixer".into(), enabled: true, parallel: false,
                model: "fixer".into(), prompt_file: Some(format!("{slug}_fixer.md")),
                phase_1_verify: false,
            },
            StageDef {
                id: "auditor".into(), enabled: true, parallel: false,
                model: "auditor".into(), prompt_file: Some(format!("{slug}_auditor.md")),
                phase_1_verify: false,
            },
            StageDef {
                id: "retester".into(), enabled: true, parallel: false,
                model: "retester".into(), prompt_file: Some(format!("{slug}_retester.md")),
                phase_1_verify: false,
            },
            StageDef {
                id: "judge".into(), enabled: true, parallel: false,
                model: "judge".into(), prompt_file: Some(format!("{slug}_judge.md")),
                phase_1_verify: true,
            },
        ],
        models,
        areas: vec![AreaDef { name: "all".into(), patterns: vec!["**/*".into()] }],
        budgets: PipelineBudgets {
            max_cycles: 2,
            convergence_score_threshold: 4.0,
            ..PipelineBudgets::default()
        },
        verify: vec![],
        agents_dir: default_agents_dir(),
        max_lessons_per_role: default_max_lessons_per_role(),
    }
}

/// Generate a prompt file for a pipeline stage based on the goal and stage role.
pub fn generate_prompt_file(goal: &str, stage_id: &str) -> String {
    let stage_prompt = match stage_id {
        "analyzer" => format!(
            "You are the **Analyzer** for this pipeline.\n\n\
             Goal: {goal}\n\n\
             Scan the project and identify gaps between what's implemented \
             and what the goal requires. For each gap, cite evidence (file:line). \
             Return a JSON object with a `gaps` array."
        ),
        "self_review" | "analyzer_self_review" => format!(
            "You are the **Analyzer** doing a self-review.\n\n\
             Goal: {goal}\n\n\
             Critique each gap you identified. Drop unsubstantiated ones, \
             adjust severity, strengthen descriptions. Return only the gaps \
             that survive review."
        ),
        "prober" => format!(
            "You are the **Prober** — you write test cases that expose gaps.\n\n\
             Goal: {goal}\n\n\
             For each gap, write specific test cases: input, expected behavior, \
             why it would fail before the fix. Return a JSON object with a `bugs` array."
        ),
        "confirmer" => format!(
            "You are the **Confirmer** — independently validate test cases.\n\n\
             Goal: {goal}\n\n\
             For each test case: confirm it's valid, reject false positives, \
             adjust severity. Record lessons for rejections."
        ),
        "fixer" => format!(
            "You are the **Crafter** — you implement fixes.\n\n\
             Goal: {goal}\n\n\
             Fix each bug at the root cause. Write tests. Run the project's \
             test suite before declaring done. No shortcuts."
        ),
        "auditor" => format!(
            "You are the **Auditor** — you code-review fixes.\n\n\
             Goal: {goal}\n\n\
             Check: root cause fixed? Test added? No shortcuts? \
             Approve or request changes. Record lessons for rejections."
        ),
        "retester" => format!(
            "You are the **ReTester** — independently verify fixes.\n\n\
             Goal: {goal}\n\n\
             Verify each fix resolves the bug. Classify: resolved, incomplete, or regression."
        ),
        "judge" => format!(
            "You are the **Judge** — score the project.\n\n\
             Goal: {goal}\n\n\
             Score 5 dimensions 0-5: functional correctness, code quality, \
             test coverage, UX quality, cost efficiency. Read actual files. \
             Cite evidence. Return JSON."
        ),
        _ => format!(
            "You are a pipeline agent.\n\nGoal: {goal}\n\nExecute your role."
        ),
    };

    format!("---\nmode: full\n---\n\n{stage_prompt}")
}

/// Convert a goal string to a filesystem-safe slug.
fn goal_to_slug(goal: &str) -> String {
    goal.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_lowercase()
        .chars()
        .take(40)
        .collect()
}

// ---------------------------------------------------------------------------
// Example TOML (for --generate output guidance)
// ---------------------------------------------------------------------------

pub fn example_toml() -> String {
    r##"# .sruja/pipeline.toml — Multi-agent pipeline configuration
# Auto-generated. Edit to customize stages, models, budgets, and areas.
#
# To use different models per role, set them in [models]:
#   [models]
#   analyzer = "gpt-4o"
#   fixer = ["gpt-4o-mini", "claude-sonnet"]  # parallel pair

[models]
analyzer = "model-name"
prober = "model-name"
confirmer = "model-name"
fixer = "model-name"
auditor = "model-name"
retester = "model-name"
judge = "model-name"

[budgets]
max_cycles = 2
convergence_score_threshold = 4.0
"##.to_string()
}
