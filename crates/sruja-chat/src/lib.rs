//! Multi-Party Chat System for Architecture Discussions
//!
//! This crate provides real-time chat capabilities for architecture conversations,
//! with automatic extraction of decisions, requirements, and constraints.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sruja_extract::{ConversationMessage, Extraction, ExtractionEngine};
use sruja_graph::{merge_scan_into_graph, KnowledgeGraph, SessionId};
use sruja_scan::scan_repo;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod agent;
pub mod definition;
pub mod graph_rag;
pub mod persistence;
pub mod session;
pub mod store;

pub use agent::generate_agent_reply;
pub use definition::{AgentDefinition, CreateAgentDefinition};
pub use persistence::{PersistError, Persistence, WorkspaceState};
pub use session::ChatSession;
pub use store::MessageStore;

/// Errors that can occur during chat operations.
#[derive(Debug, Error)]
pub enum ChatError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Participant not found: {0}")]
    ParticipantNotFound(String),

    #[error("Agent definition not found: {0}")]
    AgentDefinitionNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Store error: {0}")]
    Store(String),

    #[error("Scan error: {0}")]
    Scan(String),
}

pub type MessageId = String;
pub type ParticipantId = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Participant {
    pub id: ParticipantId,
    pub name: String,
    pub role: ParticipantRole,
    #[serde(default)]
    pub kind: ParticipantKind,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticipantRole {
    Owner,
    Contributor,
    Observer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum ParticipantKind {
    #[default]
    Human,
    Agent(AgentConfig),
}


/// Per-participant agent config. Built from AgentDefinition or specified inline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Links to admin definition if added from pool
    #[serde(default)]
    pub definition_id: Option<String>,
    pub role: String,
    pub system_prompt: String,
    #[serde(default)]
    pub knowledge_context: Option<String>,
    /// LLM model (required; from definition or inline)
    pub model: String,
    #[serde(default)]
    pub memory_limit_messages: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub session_id: SessionId,
    /// If Some, this message is a reply in a thread; the main channel shows only top-level messages.
    #[serde(default)]
    pub parent_message_id: Option<MessageId>,
    pub author: Participant,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub extractions: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMessage {
    pub author_id: ParticipantId,
    pub content: String,
    /// If Some, sends as a reply in that message's thread. Main channel captures summaries from threads.
    #[serde(default)]
    pub parent_message_id: Option<MessageId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: SessionId,
    pub topic: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub participant_count: usize,
    pub message_count: usize,
    pub extraction_count: usize,
}

/// Default system prompt for self-brainstorming (no agents in session).
const BRAINSTORM_SYSTEM_PROMPT: &str = "You are a helpful assistant for architecture discussions. \
    Support brainstorming, clarify ideas, and ask thoughtful questions. Respond concisely.";

fn default_model_from_env() -> String {
    std::env::var("SRUJA_DEFAULT_MODEL").unwrap_or_else(|_| "openai/gpt-4o-mini".to_string())
}

/// Multi-party chat server for architecture discussions.
///
/// Supports human and AI participants, agent definitions, and
/// persistence of chat history.
///
/// # Example
///
/// ```no_run
/// use sruja_chat::ChatServer;
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// use sruja_graph::KnowledgeGraph;
///
/// # #[tokio::main]
/// # async fn main() {
/// let graph = Arc::new(RwLock::new(KnowledgeGraph::new()));
/// let server = ChatServer::with_graph(graph);
/// let session_id = server.create_session("Architecture Review", "Alice").await;
/// # }
/// ```
#[derive(Clone)]
pub struct ChatServer {
    sessions: Arc<DashMap<SessionId, ChatSession>>,
    agent_definitions: Arc<DashMap<String, AgentDefinition>>,
    extraction_engine: ExtractionEngine,
    graph: Arc<RwLock<KnowledgeGraph>>,
    default_model: String,
    persistence: Option<Arc<persistence::Persistence>>,
}

impl ChatServer {
    pub fn new() -> Self {
        Self::with_graph(Arc::new(RwLock::new(KnowledgeGraph::new())))
    }

    pub fn with_graph(graph: Arc<RwLock<KnowledgeGraph>>) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            agent_definitions: Arc::new(DashMap::new()),
            extraction_engine: ExtractionEngine::new(),
            graph,
            default_model: default_model_from_env(),
            persistence: None,
        }
    }

    /// Create a ChatServer with persistence. Loads state from disk if it exists.
    pub async fn with_persistence(
        data_dir: impl AsRef<std::path::Path>,
    ) -> Result<Self, persistence::PersistError> {
        let persist = persistence::Persistence::new(data_dir);
        persist.init().await?;

        let sessions: HashMap<SessionId, ChatSession> = persist.load_sessions().await?;
        let agent_definitions: HashMap<String, AgentDefinition> =
            persist.load_agent_definitions().await?;
        let graph = Arc::new(RwLock::new(persist.load_graph().await?));

        Ok(Self {
            sessions: Arc::new(sessions.into_iter().collect()),
            agent_definitions: Arc::new(agent_definitions.into_iter().collect()),
            extraction_engine: ExtractionEngine::new(),
            graph,
            default_model: default_model_from_env(),
            persistence: Some(Arc::new(persist)),
        })
    }

    /// Spawn a background task to persist state. No-op if persistence is disabled.
    fn spawn_persist(&self) {
        let Some(ref persist) = self.persistence else {
            return;
        };
        let sessions = Arc::clone(&self.sessions);
        let agent_definitions = Arc::clone(&self.agent_definitions);
        let graph = Arc::clone(&self.graph);
        let persist = Arc::clone(persist);
        tokio::spawn(async move {
            let s: HashMap<_, _> = sessions
                .iter()
                .map(|r| (r.key().clone(), r.value().clone()))
                .collect();
            let a: HashMap<_, _> = agent_definitions
                .iter()
                .map(|r| (r.key().clone(), r.value().clone()))
                .collect();
            let g = graph.read().await.clone();
            let _ = persist.save_sessions(&s).await;
            let _ = persist.save_agent_definitions(&a).await;
            let _ = persist.save_graph(&g).await;
        });
    }

    pub async fn create_session(
        &self,
        topic: impl Into<String>,
        owner_name: impl Into<String>,
    ) -> SessionId {
        let session_id = Uuid::new_v4().to_string();
        let owner = Participant {
            id: Uuid::new_v4().to_string(),
            name: owner_name.into(),
            role: ParticipantRole::Owner,
            kind: ParticipantKind::Human,
            joined_at: Utc::now(),
        };

        let session = ChatSession::new(&session_id, topic, owner);

        self.sessions.insert(session_id.clone(), session);
        self.spawn_persist();

        session_id
    }

    pub async fn join_session(
        &self,
        session_id: &SessionId,
        name: impl Into<String>,
    ) -> Result<ParticipantId, ChatError> {
        let mut session_ref = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;

        let participant = Participant {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            role: ParticipantRole::Contributor,
            kind: ParticipantKind::Human,
            joined_at: Utc::now(),
        };

        let pid = participant.id.clone();
        session_ref.participants.push(participant);
        session_ref.updated_at = Utc::now();
        drop(session_ref);
        self.spawn_persist();

        Ok(pid)
    }

    /// Create an admin-configured agent definition.
    pub async fn create_agent_definition(
        &self,
        input: CreateAgentDefinition,
    ) -> Result<AgentDefinition, ChatError> {
        let now = Utc::now();
        let id = Uuid::new_v4().to_string();
        let def = AgentDefinition {
            id: id.clone(),
            name: input.name,
            role: input.role,
            system_prompt: input.system_prompt,
            knowledge_context: input.knowledge_context,
            model: input.model,
            memory_limit_messages: input.memory_limit_messages,
            created_at: now,
            updated_at: now,
        };
        self.agent_definitions.insert(id, def.clone());
        self.spawn_persist();
        Ok(def)
    }

    pub async fn list_agent_definitions(&self) -> Vec<AgentDefinition> {
        self.agent_definitions
            .iter()
            .map(|r| r.value().clone())
            .collect()
    }

    pub async fn get_agent_definition(&self, id: &str) -> Option<AgentDefinition> {
        self.agent_definitions.get(id).map(|r| r.value().clone())
    }

    /// Add an agent from the admin pool to a session.
    /// Retrieve Graph RAG context from the knowledge graph for the given question.
    pub async fn retrieve_graph_context(&self, question: &str) -> String {
        graph_rag::retrieve_graph_context(&self.graph, question, 5).await
    }

    pub async fn join_agent_from_definition(
        &self,
        session_id: &SessionId,
        definition_id: &str,
    ) -> Result<ParticipantId, ChatError> {
        let def = self
            .get_agent_definition(definition_id)
            .await
            .ok_or_else(|| ChatError::AgentDefinitionNotFound(definition_id.to_string()))?;

        let config = AgentConfig {
            definition_id: Some(def.id.clone()),
            role: def.role.clone(),
            system_prompt: def.system_prompt.clone(),
            knowledge_context: def.knowledge_context.clone(),
            model: def.model.clone(),
            memory_limit_messages: def.memory_limit_messages,
        };
        self.add_agent_to_session(session_id, def.name, config)
            .await
    }

    /// Add an ad-hoc agent (e.g. for testing). Model is required.
    pub async fn join_agent_inline(
        &self,
        session_id: &SessionId,
        name: impl Into<String>,
        role: impl Into<String>,
        system_prompt: impl Into<String>,
        knowledge_context: Option<impl Into<String>>,
        model: impl Into<String>,
        memory_limit_messages: Option<usize>,
    ) -> Result<ParticipantId, ChatError> {
        let config = AgentConfig {
            definition_id: None,
            role: role.into(),
            system_prompt: system_prompt.into(),
            knowledge_context: knowledge_context.map(Into::into),
            model: model.into(),
            memory_limit_messages,
        };
        self.add_agent_to_session(session_id, name, config).await
    }

    async fn add_agent_to_session(
        &self,
        session_id: &SessionId,
        name: impl Into<String>,
        config: AgentConfig,
    ) -> Result<ParticipantId, ChatError> {
        let mut session_ref = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;

        let participant = Participant {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            role: ParticipantRole::Contributor,
            kind: ParticipantKind::Agent(config),
            joined_at: Utc::now(),
        };

        let pid = participant.id.clone();
        session_ref.participants.push(participant);
        session_ref.updated_at = Utc::now();
        drop(session_ref);
        self.spawn_persist();

        Ok(pid)
    }

    pub async fn send_message(
        &self,
        session_id: &SessionId,
        new_message: NewMessage,
    ) -> Result<Message, ChatError> {
        let (message, conv_message, author_is_human, agent_ids) = {
            let mut session = self
                .sessions
                .get_mut(session_id)
                .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;

            let author = session
                .participants
                .iter()
                .find(|p| p.id == new_message.author_id)
                .cloned()
                .ok_or_else(|| ChatError::ParticipantNotFound(new_message.author_id.clone()))?;

            let message_id = Uuid::new_v4().to_string();
            let timestamp = Utc::now();

            let conv_message = ConversationMessage {
                id: message_id.clone(),
                session_id: session_id.clone(),
                author: author.name.clone(),
                content: new_message.content.clone(),
                timestamp,
            };

            let message = Message {
                id: message_id,
                session_id: session_id.clone(),
                parent_message_id: new_message.parent_message_id.clone(),
                author: author.clone(),
                content: new_message.content,
                timestamp,
                extractions: vec![],
                metadata: HashMap::new(),
            };

            session.messages.push(message.clone());
            session.updated_at = Utc::now();

            let author_is_human = matches!(author.kind, ParticipantKind::Human);
            let agent_ids: Vec<ParticipantId> = if author_is_human {
                let agents: Vec<ParticipantId> = session
                    .participants
                    .iter()
                    .filter(|p| matches!(p.kind, ParticipantKind::Agent(_)))
                    .map(|p| p.id.clone())
                    .collect();
                if agents.is_empty() {
                    let default_model = self.default_model.clone();
                    let brainstorm = Participant {
                        id: Uuid::new_v4().to_string(),
                        name: "Brainstorm".to_string(),
                        role: ParticipantRole::Contributor,
                        kind: ParticipantKind::Agent(AgentConfig {
                            definition_id: None,
                            role: "General Assistant".to_string(),
                            system_prompt: BRAINSTORM_SYSTEM_PROMPT.to_string(),
                            knowledge_context: None,
                            model: default_model,
                            memory_limit_messages: None,
                        }),
                        joined_at: Utc::now(),
                    };
                    let pid = brainstorm.id.clone();
                    session.participants.push(brainstorm);
                    vec![pid]
                } else {
                    agents
                }
            } else {
                vec![]
            };

            (message, conv_message, author_is_human, agent_ids)
        };

        self.spawn_persist();

        let session_id = session_id.to_string();
        let session_id_for_agents = session_id.clone();
        let engine = self.extraction_engine.clone();
        let sessions = Arc::clone(&self.sessions);
        let graph = Arc::clone(&self.graph);
        let server = Arc::new(self.clone());
        let server_for_persist = Arc::clone(&server);
        tokio::spawn(async move {
            if let Ok(extractions) = engine.extract_from_message_async(&conv_message).await {
                let _ = Self::merge_extractions_impl(
                    &sessions,
                    &graph,
                    &session_id,
                    &conv_message.id,
                    &extractions,
                )
                .await;
                server_for_persist.spawn_persist();
            }
        });

        if author_is_human && !agent_ids.is_empty() {
            let server2 = Arc::clone(&server);
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("agent runtime");
                rt.block_on(async move {
                    if let Ok(history) = server2.get_history(&session_id_for_agents).await {
                        let graph_ctx = if let Some(last) = history.last() {
                            server2.retrieve_graph_context(&last.content).await
                        } else {
                            String::new()
                        };
                        let graph_ctx_ref = if graph_ctx.is_empty() {
                            None
                        } else {
                            Some(graph_ctx.as_str())
                        };
                        for agent_id in agent_ids {
                            if let Ok(participants) =
                                server2.get_participants(&session_id_for_agents).await
                            {
                                if let Some(agent) = participants.iter().find(|p| p.id == agent_id)
                                {
                                    if let Ok(reply) =
                                        generate_agent_reply(agent, &history, graph_ctx_ref).await
                                    {
                                        if !reply.is_empty() {
                                            let _ = server2
                                                .send_message(
                                                    &session_id_for_agents,
                                                    NewMessage {
                                                        author_id: agent_id,
                                                        content: reply,
                                                        parent_message_id: None,
                                                    },
                                                )
                                                .await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            });
        }

        Ok(message)
    }

    async fn merge_extractions_impl(
        sessions: &Arc<DashMap<SessionId, ChatSession>>,
        graph: &Arc<RwLock<KnowledgeGraph>>,
        session_id: &SessionId,
        message_id: &MessageId,
        extractions: &[Extraction],
    ) -> Result<(), ChatError> {
        let thread_root = {
            let session = sessions
                .get(session_id)
                .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;
            session
                .messages
                .iter()
                .find(|m| m.id == *message_id)
                .and_then(|m| m.parent_message_id.clone())
        };

        let mut extractions_with_thread: Vec<Extraction> = extractions.to_vec();
        if thread_root.is_some() {
            for ext in extractions_with_thread.iter_mut() {
                ext.thread_root_message_id = thread_root.clone();
            }
        }

        let extraction_ids: Vec<String> = extractions_with_thread
            .iter()
            .map(|e| e.id.clone())
            .collect();

        {
            let mut session = sessions
                .get_mut(session_id)
                .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;

            for extraction in extractions_with_thread {
                session
                    .extractions
                    .insert(extraction.id.clone(), extraction);
            }

            if let Some(msg) = session.messages.iter_mut().find(|m| m.id == *message_id) {
                msg.extractions = extraction_ids;
            }
            session.updated_at = Utc::now();
        }

        for extraction in extractions {
            if let Some(decision) = extraction.to_decision(session_id) {
                let mut g = graph.write().await;
                let _ = g.add_decision(decision);
            }
        }

        Ok(())
    }

    pub async fn get_history(&self, session_id: &SessionId) -> Result<Vec<Message>, ChatError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;

        Ok(session.messages.clone())
    }

    /// Main thread only: top-level messages (no parent). Child threads hold the detailed discussion.
    pub async fn get_main_thread(&self, session_id: &SessionId) -> Result<Vec<Message>, ChatError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;

        Ok(session
            .messages
            .iter()
            .filter(|m| m.parent_message_id.is_none())
            .cloned()
            .collect())
    }

    /// Replies in a thread. Main channel surfaces key decisions from these via extractions.
    pub async fn get_thread_replies(
        &self,
        session_id: &SessionId,
        parent_message_id: &MessageId,
    ) -> Result<Vec<Message>, ChatError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;

        Ok(session
            .messages
            .iter()
            .filter(|m| m.parent_message_id.as_deref() == Some(parent_message_id.as_str()))
            .cloned()
            .collect())
    }

    /// Reply count for a main-thread message.
    pub async fn get_thread_reply_count(
        &self,
        session_id: &SessionId,
        parent_message_id: &MessageId,
    ) -> Result<usize, ChatError> {
        let replies = self
            .get_thread_replies(session_id, parent_message_id)
            .await?;
        Ok(replies.len())
    }

    pub async fn get_extractions(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<Extraction>, ChatError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;

        Ok(session.extractions.values().cloned().collect())
    }

    /// Key decisions and extractions from a thread, surfaced in the main channel.
    pub async fn get_extractions_for_thread_root(
        &self,
        session_id: &SessionId,
        thread_root_message_id: &MessageId,
    ) -> Result<Vec<Extraction>, ChatError> {
        let all = self.get_extractions(session_id).await?;
        Ok(all
            .into_iter()
            .filter(|e| {
                e.thread_root_message_id.as_deref() == Some(thread_root_message_id.as_str())
            })
            .collect())
    }

    pub async fn confirm_extraction(
        &self,
        session_id: &SessionId,
        extraction_id: &str,
    ) -> Result<(), ChatError> {
        let mut session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;

        if let Some(extraction) = session.extractions.get_mut(extraction_id) {
            extraction.status = sruja_extract::ExtractionStatus::Confirmed;

            if let Some(decision) = extraction.to_decision(session_id) {
                let mut graph = self.graph.write().await;
                let _ = graph.accept_decision(&decision.id);
            }
        }
        drop(session);
        self.spawn_persist();

        Ok(())
    }

    pub async fn reject_extraction(
        &self,
        session_id: &SessionId,
        extraction_id: &str,
    ) -> Result<(), ChatError> {
        let mut session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;

        if let Some(extraction) = session.extractions.get_mut(extraction_id) {
            extraction.status = sruja_extract::ExtractionStatus::Rejected;
        }
        drop(session);
        self.spawn_persist();

        Ok(())
    }

    pub async fn get_participants(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<Participant>, ChatError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;
        Ok(session.participants.clone())
    }

    pub async fn get_session_info(&self, session_id: &SessionId) -> Result<SessionInfo, ChatError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| ChatError::SessionNotFound(session_id.clone()))?;

        Ok(SessionInfo {
            id: session_id.clone(),
            topic: session.topic.clone(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            participant_count: session.participants.len(),
            message_count: session.messages.len(),
            extraction_count: session.extractions.len(),
        })
    }

    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions
            .iter()
            .map(|s| SessionInfo {
                id: s.id.clone(),
                topic: s.topic.clone(),
                created_at: s.created_at,
                updated_at: s.updated_at,
                participant_count: s.participants.len(),
                message_count: s.messages.len(),
                extraction_count: s.extractions.len(),
            })
            .collect()
    }

    /// Load architecture context from a repo by scanning source code.
    /// Merges inferred nodes and edges into the knowledge graph.
    pub async fn load_repo_context(&self, repo_path: &Path) -> Result<usize, ChatError> {
        let scan_graph = scan_repo(repo_path).map_err(|e| ChatError::Scan(e.to_string()))?;
        let path_str = repo_path.display().to_string();

        let mut graph = self.graph.write().await;
        let count = merge_scan_into_graph(&mut graph, &scan_graph, &path_str);
        drop(graph);
        self.spawn_persist();

        // Update workspace with repo path
        if let Some(ref p) = self.persistence {
            let mut state = p.load_workspace().await.unwrap_or_default();
            state.repo_path = Some(path_str);
            let _ = p.save_workspace(&state).await;
        }

        Ok(count)
    }

    /// Return a reference to the knowledge graph for querying.
    pub fn graph(&self) -> Arc<tokio::sync::RwLock<KnowledgeGraph>> {
        Arc::clone(&self.graph)
    }

    /// Save workspace state (repo path, last session). No-op if persistence disabled.
    pub async fn save_workspace(
        &self,
        state: &persistence::WorkspaceState,
    ) -> Result<(), persistence::PersistError> {
        if let Some(ref p) = self.persistence {
            p.save_workspace(state).await?;
        }
        Ok(())
    }

    /// Load workspace state. Returns default if persistence disabled or not found.
    pub async fn load_workspace(
        &self,
    ) -> Result<persistence::WorkspaceState, persistence::PersistError> {
        if let Some(ref p) = self.persistence {
            p.load_workspace().await
        } else {
            Ok(persistence::WorkspaceState::default())
        }
    }
}

impl Default for ChatServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let server = ChatServer::new();
        let session_id = server.create_session("Architecture Review", "Alice").await;

        assert!(!session_id.is_empty());

        let info = server.get_session_info(&session_id).await.unwrap();
        assert_eq!(info.topic, "Architecture Review");
        assert_eq!(info.participant_count, 1);
    }

    #[tokio::test]
    async fn test_join_and_send_message() {
        let server = ChatServer::new();
        let session_id = server.create_session("Test", "Alice").await;

        let bob_id = server.join_session(&session_id, "Bob").await.unwrap();

        let message = server
            .send_message(
                &session_id,
                NewMessage {
                    author_id: bob_id,
                    content: "We should use Kafka for events".to_string(),
                    parent_message_id: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(message.author.name, "Bob");
        assert_eq!(message.content, "We should use Kafka for events");
        // Extractions come from LLM; may be empty in tests without API key
    }

    #[tokio::test]
    async fn test_extraction_workflow() {
        let server = ChatServer::new();
        let session_id = server.create_session("Test", "Alice").await;

        let participants = server.get_participants(&session_id).await.unwrap();
        let alice_id = participants[0].id.clone();

        server
            .send_message(
                &session_id,
                NewMessage {
                    author_id: alice_id.clone(),
                    content: "We should use Redis for caching".to_string(),
                    parent_message_id: None,
                },
            )
            .await
            .unwrap();

        let extractions = server.get_extractions(&session_id).await.unwrap();
        // LLM extraction may return empty without API key; when present, test confirm flow
        if !extractions.is_empty() {
            let extraction_id = &extractions[0].id;
            server
                .confirm_extraction(&session_id, extraction_id)
                .await
                .unwrap();
            let extractions = server.get_extractions(&session_id).await.unwrap();
            assert_eq!(
                extractions[0].status,
                sruja_extract::ExtractionStatus::Confirmed
            );
        }
    }
}
