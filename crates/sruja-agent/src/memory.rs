//! Agentic memory management for Sruja.
//!
//! This module provides the `AgenticMemory` system, which stores and retrieves
//! "learnings" (hypotheses, outcomes, and guardrails) derived from AI-driven
//! architectural experiments.

use chrono::{DateTime, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during memory operations.
#[derive(Debug, Error)]
pub enum MemoryError {
    /// An I/O error occurred while reading or writing the memory file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// A serialization or deserialization error occurred.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// The outcome of an architectural experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentOutcome {
    /// The experiment was successful (e.g., the change improved the architecture).
    #[serde(rename = "success")]
    Success,
    /// The experiment failed (e.g., the change caused a regression or drift).
    #[serde(rename = "failed")]
    Failed,
}

/// A single entry in the agentic memory representing a learned architectural lesson.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEntry {
    /// When this learning was recorded.
    pub timestamp: DateTime<Utc>,
    /// The context in which the experiment was performed (e.g., "Refactoring API layer").
    pub context: String,
    /// The hypothesis being tested (e.g., "Moving logic to sruja-engine will reduce CLI bloat").
    pub hypothesis: String,
    /// Whether the experiment succeeded or failed.
    pub outcome: ExperimentOutcome,
    /// The reason for the outcome (optional).
    pub reason: Option<String>,
    /// Actionable advice for future agents to avoid repeating mistakes or to replicate success.
    pub guardrail_advice: String,
    /// IDs of architectural elements affected by this experiment.
    pub affected_elements: Vec<String>,
}

impl LearningEntry {
    /// Checks if this learning entry is relevant to a specific architectural element.
    pub fn is_relevant_to(&self, element_id: &str) -> bool {
        let element_id_lower = element_id.to_lowercase();

        // 1. Check affected elements (direct match or hierarchy match)
        let element_match = self.affected_elements.iter().any(|e| {
            e == element_id
                || element_id.starts_with(&format!("{}.", e))
                || e.starts_with(&format!("{}.", element_id))
        });

        if element_match {
            return true;
        }

        // 2. Check context for keywords
        self.context.to_lowercase().contains(&element_id_lower)
    }
}

/// Persistent store for architectural learnings and agentic guardrails.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgenticMemory {
    /// The list of recorded learnings.
    pub learnings: Vec<LearningEntry>,
}

impl AgenticMemory {
    /// Loads agentic memory from the specified repository root.
    ///
    /// Memory is stored in `.sruja/agent_memory.json`. If the file does not exist,
    /// an empty memory is returned.
    pub fn load(repo_root: &Path) -> Result<Self, MemoryError> {
        let path = Self::get_path(repo_root);
        Self::load_from_path(&path)
    }

    /// Loads agentic memory from a specific path.
    pub fn load_from_path(path: &Path) -> Result<Self, MemoryError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let file = File::open(path)?;
        file.lock_shared()?;
        let mut content = String::new();
        let mut reader = std::io::BufReader::new(&file);
        reader.read_to_string(&mut content)?;
        file.unlock()?;
        let memory = serde_json::from_str(&content)?;
        Ok(memory)
    }

    /// Saves the current memory to the specified repository root.
    ///
    /// This will create the `.sruja` directory if it doesn't exist.
    pub fn save(&self, repo_root: &Path) -> Result<(), MemoryError> {
        let path = Self::get_path(repo_root);
        self.save_to_path(&path)
    }

    /// Saves the current memory to a specific path.
    pub fn save_to_path(&self, path: &Path) -> Result<(), MemoryError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        file.lock_exclusive()?;
        let content = serde_json::to_string_pretty(self)?;
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        {
            let mut writer = std::io::BufWriter::new(&mut file);
            writer.write_all(content.as_bytes())?;
            writer.flush()?;
        }
        file.unlock()?;
        Ok(())
    }

    /// Clears the agentic memory for the specified repository.
    pub fn clear(repo_root: &Path) -> Result<(), MemoryError> {
        let path = Self::get_path(repo_root);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Checks if the agentic memory file exists for the given repository.
    pub fn exists(repo_root: &Path) -> bool {
        Self::get_path(repo_root).exists()
    }

    /// Adds a new learning entry to the memory.
    pub fn add_learning(&mut self, learning: LearningEntry) {
        self.learnings.push(learning);
    }

    /// Finds learning entries relevant to a specific architectural element ID.
    ///
    /// Relevancy is determined by:
    /// 1. Direct match in `affected_elements`.
    /// 2. Parent match (element_id starts with an affected element).
    /// 3. String match in the `context` field.
    pub fn find_relevant(&self, element_id: &str) -> Vec<&LearningEntry> {
        self.learnings
            .iter()
            .filter(|l| l.is_relevant_to(element_id))
            .collect()
    }

    pub fn get_path(repo_root: &Path) -> PathBuf {
        repo_root.join(".sruja").join("agent_memory.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_add_and_find_relevant() {
        let mut memory = AgenticMemory::default();
        let entry = LearningEntry {
            timestamp: Utc::now(),
            context: "Refactoring API".to_string(),
            hypothesis: "Test hypothesis".to_string(),
            outcome: ExperimentOutcome::Success,
            reason: None,
            guardrail_advice: "Keep doing this".to_string(),
            affected_elements: vec!["Sruja.API".to_string(), "Sruja.CLI".to_string()],
        };

        memory.add_learning(entry.clone());

        // Exact match
        assert_eq!(memory.find_relevant("Sruja.API").len(), 1);
        // Parent match
        assert_eq!(memory.find_relevant("Sruja.API.V1").len(), 1);
        // Context match
        assert_eq!(memory.find_relevant("api").len(), 1);
        // No match
        assert_eq!(memory.find_relevant("Other").len(), 0);
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let mut memory = AgenticMemory::default();
        let entry = LearningEntry {
            timestamp: Utc::now(),
            context: "Test".to_string(),
            hypothesis: "Hypo".to_string(),
            outcome: ExperimentOutcome::Failed,
            reason: Some("Error".to_string()),
            guardrail_advice: "Don't".to_string(),
            affected_elements: vec!["ID1".to_string()],
        };

        memory.add_learning(entry);
        memory.save(repo_root).unwrap();

        let loaded = AgenticMemory::load(repo_root).unwrap();
        assert_eq!(loaded.learnings.len(), 1);
        assert_eq!(loaded.learnings[0].context, "Test");
    }

    #[test]
    fn test_load_nonexistent() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();

        let loaded = AgenticMemory::load(repo_root).unwrap();
        assert_eq!(loaded.learnings.len(), 0);
    }

    #[test]
    fn test_clear_and_exists() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();

        assert!(!AgenticMemory::exists(repo_root));

        let memory = AgenticMemory::default();
        memory.save(repo_root).unwrap();

        assert!(AgenticMemory::exists(repo_root));

        AgenticMemory::clear(repo_root).unwrap();

        assert!(!AgenticMemory::exists(repo_root));
    }

    #[test]
    fn test_load_invalid_json() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let path = AgenticMemory::get_path(repo_root);

        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "invalid json").unwrap();

        let result = AgenticMemory::load(repo_root);
        assert!(matches!(result, Err(MemoryError::Serialization(_))));
    }

    #[test]
    fn test_find_relevant_edge_cases() {
        let mut memory = AgenticMemory::default();
        let entry = LearningEntry {
            timestamp: Utc::now(),
            context: "Some Context".to_string(),
            hypothesis: "".to_string(),
            outcome: ExperimentOutcome::Success,
            reason: None,
            guardrail_advice: "".to_string(),
            affected_elements: vec!["Sruja.API.V1".to_string()],
        };
        memory.add_learning(entry);

        // e.starts_with(&format!("{}.", element_id)) -> element_id is prefix of e
        // e is "Sruja.API.V1"
        // element_id is "Sruja.API"
        assert_eq!(memory.find_relevant("Sruja.API").len(), 1);

        // element_id.starts_with(&format!("{}.", e)) -> e is prefix of element_id
        // e is "Sruja.API.V1"
        // element_id is "Sruja.API.V1.Endpoint"
        assert_eq!(memory.find_relevant("Sruja.API.V1.Endpoint").len(), 1);

        // Exact match
        assert_eq!(memory.find_relevant("Sruja.API.V1").len(), 1);

        // No match
        assert_eq!(memory.find_relevant("Sruja.API.V2").len(), 0);
    }

    #[test]
    fn test_load_save_custom_path() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("custom_memory.json");
        let mut memory = AgenticMemory::default();
        memory.add_learning(LearningEntry {
            timestamp: Utc::now(),
            context: "Custom".to_string(),
            hypothesis: "H".to_string(),
            outcome: ExperimentOutcome::Success,
            reason: None,
            guardrail_advice: "G".to_string(),
            affected_elements: vec![],
        });

        memory.save_to_path(&path).unwrap();
        assert!(path.exists());

        let loaded = AgenticMemory::load_from_path(&path).unwrap();
        assert_eq!(loaded.learnings.len(), 1);
        assert_eq!(loaded.learnings[0].context, "Custom");
    }

    #[test]
    fn test_learning_entry_relevance() {
        let entry = LearningEntry {
            timestamp: Utc::now(),
            context: "API Refactoring".to_string(),
            hypothesis: "".to_string(),
            outcome: ExperimentOutcome::Success,
            reason: None,
            guardrail_advice: "".to_string(),
            affected_elements: vec!["System.Core".to_string()],
        };

        // Direct match
        assert!(entry.is_relevant_to("System.Core"));
        // Hierarchy match (child)
        assert!(entry.is_relevant_to("System.Core.UI"));
        // Hierarchy match (parent)
        assert!(entry.is_relevant_to("System"));
        // Context match
        assert!(entry.is_relevant_to("api"));
        // No match
        assert!(!entry.is_relevant_to("Database"));
    }

    #[test]
    fn test_save_to_path_replaces_longer_existing_content() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("memory.json");
        let mut memory = AgenticMemory::default();
        memory.add_learning(LearningEntry {
            timestamp: Utc::now(),
            context: "Short".to_string(),
            hypothesis: "H".to_string(),
            outcome: ExperimentOutcome::Success,
            reason: None,
            guardrail_advice: "G".to_string(),
            affected_elements: vec![],
        });

        std::fs::write(&path, "{".repeat(4096)).unwrap();

        memory.save_to_path(&path).unwrap();

        let loaded = AgenticMemory::load_from_path(&path).unwrap();
        assert_eq!(loaded.learnings.len(), 1);
        assert_eq!(loaded.learnings[0].context, "Short");
    }
}
