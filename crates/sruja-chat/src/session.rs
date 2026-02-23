//! Chat session management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sruja_extract::Extraction;
use sruja_graph::SessionId;
use std::collections::HashMap;

use crate::{Message, Participant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: SessionId,
    pub topic: String,
    pub participants: Vec<Participant>,
    pub messages: Vec<Message>,
    pub extractions: HashMap<String, Extraction>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Closed,
    Archived,
}

impl ChatSession {
    pub fn new(id: &str, topic: impl Into<String>, owner: Participant) -> Self {
        let now = Utc::now();
        Self {
            id: id.to_string(),
            topic: topic.into(),
            participants: vec![owner],
            messages: Vec::new(),
            extractions: HashMap::new(),
            created_at: now,
            updated_at: now,
            status: SessionStatus::Active,
        }
    }

    pub fn add_participant(&mut self, participant: Participant) {
        if !self.participants.iter().any(|p| p.id == participant.id) {
            self.participants.push(participant);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_participant(&mut self, participant_id: &str) -> Option<Participant> {
        if let Some(pos) = self
            .participants
            .iter()
            .position(|p| p.id == participant_id)
        {
            let participant = self.participants.remove(pos);
            self.updated_at = Utc::now();
            Some(participant)
        } else {
            None
        }
    }

    pub fn get_participant(&self, id: &str) -> Option<&Participant> {
        self.participants.iter().find(|p| p.id == id)
    }

    pub fn is_owner(&self, participant_id: &str) -> bool {
        self.participants
            .iter()
            .find(|p| p.id == participant_id)
            .map(|p| p.role == crate::ParticipantRole::Owner)
            .unwrap_or(false)
    }

    pub fn close(&mut self) {
        self.status = SessionStatus::Closed;
        self.updated_at = Utc::now();
    }

    pub fn archive(&mut self) {
        self.status = SessionStatus::Archived;
        self.updated_at = Utc::now();
    }

    pub fn stats(&self) -> SessionStats {
        SessionStats {
            participant_count: self.participants.len(),
            message_count: self.messages.len(),
            extraction_count: self.extractions.len(),
            duration_minutes: (self.updated_at - self.created_at).num_minutes() as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub participant_count: usize,
    pub message_count: usize,
    pub extraction_count: usize,
    pub duration_minutes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ParticipantKind, ParticipantRole};

    fn test_owner() -> Participant {
        Participant {
            id: "owner-1".to_string(),
            name: "Alice".to_string(),
            role: ParticipantRole::Owner,
            kind: ParticipantKind::Human,
            joined_at: Utc::now(),
        }
    }

    #[test]
    fn test_new_session() {
        let owner = test_owner();
        let session = ChatSession::new("sess-1", "Architecture Discussion", owner);

        assert_eq!(session.topic, "Architecture Discussion");
        assert_eq!(session.participants.len(), 1);
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn test_add_participant() {
        let owner = test_owner();
        let mut session = ChatSession::new("sess-1", "Test", owner);

        let contributor = Participant {
            id: "contrib-1".to_string(),
            name: "Bob".to_string(),
            role: ParticipantRole::Contributor,
            kind: ParticipantKind::Human,
            joined_at: Utc::now(),
        };

        session.add_participant(contributor);
        assert_eq!(session.participants.len(), 2);
    }

    #[test]
    fn test_is_owner() {
        let owner = test_owner();
        let session = ChatSession::new("sess-1", "Test", owner);

        assert!(session.is_owner("owner-1"));
        assert!(!session.is_owner("other"));
    }
}
