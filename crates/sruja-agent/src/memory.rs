use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentOutcome {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEntry {
    pub timestamp: DateTime<Utc>,
    pub context: String,
    pub hypothesis: String,
    pub outcome: ExperimentOutcome,
    pub reason: Option<String>,
    pub guardrail_advice: String,
    pub affected_elements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgenticMemory {
    pub learnings: Vec<LearningEntry>,
}

impl AgenticMemory {
    pub fn load(repo_root: &Path) -> Result<Self, MemoryError> {
        let path = Self::get_path(repo_root);
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let memory = serde_json::from_str(&content)?;
        Ok(memory)
    }

    pub fn save(&self, repo_root: &Path) -> Result<(), MemoryError> {
        let path = Self::get_path(repo_root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn add_learning(&mut self, learning: LearningEntry) {
        self.learnings.push(learning);
    }

    pub fn find_relevant(&self, element_id: &str) -> Vec<&LearningEntry> {
        self.learnings
            .iter()
            .filter(|l| {
                l.affected_elements.iter().any(|e| e == element_id || element_id.starts_with(e))
                    || l.context.to_lowercase().contains(&element_id.to_lowercase())
            })
            .collect()
    }

    fn get_path(repo_root: &Path) -> PathBuf {
        repo_root.join(".sruja").join("agent_memory.json")
    }
}
