//! Skill filtering logic (simplified version)
//!
//! Simple filtering without complex metadata structures.

use std::collections::HashSet;

/// Experience level for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    Beginner,
    Intermediate,
    Advanced,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Beginner => write!(f, "beginner"),
            Level::Intermediate => write!(f, "intermediate"),
            Level::Advanced => write!(f, "advanced"),
        }
    }
}

impl Level {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "beginner" => Some(Level::Beginner),
            "intermediate" => Some(Level::Intermediate),
            "advanced" => Some(Level::Advanced),
            _ => None,
        }
    }
}

/// Rule category priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Critical,
    High,
    Medium,
    Low,
    Reference,
}

impl Category {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "critical" => Some(Category::Critical),
            "high" => Some(Category::High),
            "medium" => Some(Category::Medium),
            "low" => Some(Category::Low),
            "reference" => Some(Category::Reference),
            _ => None,
        }
    }
}

/// Output format for filtered skills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Markdown,
    Json,
    Concise,
}

/// Project context from analysis
#[derive(Debug, Clone, Default)]
pub struct ProjectContext {
    pub is_async: bool,
    pub web: bool,
    pub embedded: bool,
    pub wasm: bool,
    pub cli: bool,
    pub library: bool,
    pub complexity_score: f32,
}

/// Filter criteria for loading skills (simplified)
#[derive(Debug, Clone, Default)]
pub struct SkillFilter {
    pub levels: Option<HashSet<Level>>,
    pub categories: Option<HashSet<Category>>,
    pub output_format: OutputFormat,
    pub limit: Option<usize>,
    pub project_context: Option<ProjectContext>,
}

impl SkillFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn matches(&self, _rule_id: &str, _metadata: &()) -> bool {
        // Simplified: all rules match for now
        // Future: add filtering based on metadata
        true
    }
}
