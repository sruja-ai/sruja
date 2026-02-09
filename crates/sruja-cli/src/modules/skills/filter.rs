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

/// Output format for filtered skills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Markdown,
    Json,
    Concise,
}

/// Filter criteria for loading skills (simplified)
#[derive(Debug, Clone, Default)]
pub struct SkillFilter {
    pub levels: Option<HashSet<Level>>,
    pub output_format: OutputFormat,
    pub limit: Option<usize>,
}

impl SkillFilter {
    pub fn new() -> Self {
        Self::default()
    }
}
