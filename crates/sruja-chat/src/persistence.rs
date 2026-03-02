//! Persistence layer for sessions, graph, and agent definitions.
//!
//! Stores data as JSON files in a configurable data directory.

use crate::{AgentDefinition, ChatSession};
use sruja_graph::KnowledgeGraph;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Error)]
pub enum PersistError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Graph error: {0}")]
    Graph(String),
}

/// Default data directory: ~/.sruja/data or SRUJA_DATA_DIR env.
pub fn default_data_dir() -> PathBuf {
    std::env::var("SRUJA_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".sruja")
                .join("data")
        })
}

/// Ensure data directory exists.
pub async fn ensure_data_dir(path: &Path) -> Result<(), PersistError> {
    fs::create_dir_all(path).await?;
    Ok(())
}

/// Persistence handles loading and saving ChatServer state.
#[derive(Clone)]
pub struct Persistence {
    data_dir: PathBuf,
}

impl Persistence {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
        }
    }

    pub fn with_default_dir() -> Self {
        Self::new(default_data_dir())
    }

    fn sessions_path(&self) -> PathBuf {
        self.data_dir.join("sessions.json")
    }

    fn graph_path(&self) -> PathBuf {
        self.data_dir.join("graph.json")
    }

    fn agents_path(&self) -> PathBuf {
        self.data_dir.join("agent_definitions.json")
    }

    fn workspace_path(&self) -> PathBuf {
        self.data_dir.join("workspace.json")
    }

    /// Initialize data directory.
    pub async fn init(&self) -> Result<(), PersistError> {
        ensure_data_dir(&self.data_dir).await
    }

    /// Save sessions to disk.
    pub async fn save_sessions(
        &self,
        sessions: &HashMap<String, ChatSession>,
    ) -> Result<(), PersistError> {
        self.init().await?;
        let path = self.sessions_path();
        let json = serde_json::to_string_pretty(sessions)?;
        let mut f = fs::File::create(&path).await?;
        f.write_all(json.as_bytes()).await?;
        f.flush().await?;
        Ok(())
    }

    /// Load sessions from disk.
    pub async fn load_sessions(&self) -> Result<HashMap<String, ChatSession>, PersistError> {
        let path = self.sessions_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(HashMap::new());
        }
        let bytes = fs::read(&path).await?;
        let sessions: HashMap<String, ChatSession> = serde_json::from_slice(&bytes)?;
        Ok(sessions)
    }

    /// Save graph to disk.
    pub async fn save_graph(&self, graph: &KnowledgeGraph) -> Result<(), PersistError> {
        self.init().await?;
        let path = self.graph_path();
        let json = graph
            .to_json()
            .map_err(|e| PersistError::Graph(e.to_string()))?;
        let mut f = fs::File::create(&path).await?;
        f.write_all(json.as_bytes()).await?;
        f.flush().await?;
        Ok(())
    }

    /// Load graph from disk.
    pub async fn load_graph(&self) -> Result<KnowledgeGraph, PersistError> {
        let path = self.graph_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(KnowledgeGraph::new());
        }
        let bytes = fs::read(&path).await?;
        let json = String::from_utf8_lossy(&bytes);
        sruja_graph::KnowledgeGraph::from_json(&json)
            .map_err(|e| PersistError::Graph(e.to_string()))
    }

    /// Save agent definitions to disk.
    pub async fn save_agent_definitions(
        &self,
        defs: &HashMap<String, AgentDefinition>,
    ) -> Result<(), PersistError> {
        self.init().await?;
        let path = self.agents_path();
        let json = serde_json::to_string_pretty(defs)?;
        let mut f = fs::File::create(&path).await?;
        f.write_all(json.as_bytes()).await?;
        f.flush().await?;
        Ok(())
    }

    /// Load agent definitions from disk.
    pub async fn load_agent_definitions(
        &self,
    ) -> Result<HashMap<String, AgentDefinition>, PersistError> {
        let path = self.agents_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(HashMap::new());
        }
        let bytes = fs::read(&path).await?;
        let defs: HashMap<String, AgentDefinition> = serde_json::from_slice(&bytes)?;
        Ok(defs)
    }

    /// Save workspace (repo path, last session, etc.).
    pub async fn save_workspace(&self, workspace: &WorkspaceState) -> Result<(), PersistError> {
        self.init().await?;
        let path = self.workspace_path();
        let json = serde_json::to_string_pretty(workspace)?;
        let mut f = fs::File::create(&path).await?;
        f.write_all(json.as_bytes()).await?;
        f.flush().await?;
        Ok(())
    }

    /// Load workspace state.
    pub async fn load_workspace(&self) -> Result<WorkspaceState, PersistError> {
        let path = self.workspace_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(WorkspaceState::default());
        }
        let bytes = fs::read(&path).await?;
        let state: WorkspaceState = serde_json::from_slice(&bytes)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ParticipantKind, ParticipantRole};
    use chrono::Utc;

    #[tokio::test]
    async fn persistence_roundtrip_sessions_and_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let persist = Persistence::new(dir.path());

        persist.init().await.unwrap();

        let mut sessions = HashMap::new();
        let owner = crate::Participant {
            id: "p1".to_string(),
            name: "Alice".to_string(),
            role: ParticipantRole::Owner,
            kind: ParticipantKind::Human,
            joined_at: Utc::now(),
        };
        let session = ChatSession::new("s1", "Test", owner);
        sessions.insert("s1".to_string(), session);

        persist.save_sessions(&sessions).await.unwrap();
        let loaded = persist.load_sessions().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.get("s1").unwrap().topic, "Test");

        let ws = WorkspaceState {
            repo_path: Some("/path/to/repo".to_string()),
            last_session_id: Some("s1".to_string()),
        };
        persist.save_workspace(&ws).await.unwrap();
        let loaded_ws = persist.load_workspace().await.unwrap();
        assert_eq!(loaded_ws.repo_path.as_deref(), Some("/path/to/repo"));
        assert_eq!(loaded_ws.last_session_id.as_deref(), Some("s1"));
    }
}

/// Workspace state: repo path, last session, etc.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceState {
    /// Last loaded repo path.
    pub repo_path: Option<String>,
    /// Last active session ID.
    pub last_session_id: Option<String>,
}
