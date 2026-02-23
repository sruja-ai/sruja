//! Message storage (stub for future persistence)

use crate::{Message, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct MessageStore {
    messages: HashMap<SessionId, Vec<StoredMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub session_id: SessionId,
    #[serde(default)]
    pub parent_message_id: Option<String>,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub extractions: Vec<String>,
}

impl MessageStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store(&mut self, message: &Message) {
        let stored = StoredMessage {
            id: message.id.clone(),
            session_id: message.session_id.clone(),
            parent_message_id: message.parent_message_id.clone(),
            author_id: message.author.id.clone(),
            author_name: message.author.name.clone(),
            content: message.content.clone(),
            timestamp: message.timestamp,
            extractions: message.extractions.clone(),
        };

        self.messages
            .entry(message.session_id.clone())
            .or_default()
            .push(stored);
    }

    pub fn get_session_messages(&self, session_id: &SessionId) -> Vec<&StoredMessage> {
        self.messages
            .get(session_id)
            .map(|m| m.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_message(&self, session_id: &SessionId, message_id: &str) -> Option<&StoredMessage> {
        self.messages
            .get(session_id)?
            .iter()
            .find(|m| m.id == message_id)
    }

    pub fn search(&self, query: &str) -> Vec<&StoredMessage> {
        let query_lower = query.to_lowercase();
        self.messages
            .values()
            .flat_map(|msgs| msgs.iter())
            .filter(|m| m.content.to_lowercase().contains(&query_lower))
            .collect()
    }

    pub fn count(&self) -> usize {
        self.messages.values().map(|m| m.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Participant, ParticipantRole};
    use std::collections::HashMap;

    fn test_message(content: &str) -> Message {
        Message {
            id: "msg-1".to_string(),
            session_id: "sess-1".to_string(),
            parent_message_id: None,
            author: Participant {
                id: "user-1".to_string(),
                name: "Alice".to_string(),
                role: ParticipantRole::Owner,
                kind: crate::ParticipantKind::Human,
                joined_at: Utc::now(),
            },
            content: content.to_string(),
            timestamp: Utc::now(),
            extractions: vec![],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_store_message() {
        let mut store = MessageStore::new();
        let message = test_message("Hello world");

        store.store(&message);

        let messages = store.get_session_messages(&"sess-1".to_string());
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_search() {
        let mut store = MessageStore::new();
        store.store(&test_message("We should use Kafka"));
        store.store(&test_message("Hello world"));

        let results = store.search("Kafka");
        assert_eq!(results.len(), 1);
    }
}
