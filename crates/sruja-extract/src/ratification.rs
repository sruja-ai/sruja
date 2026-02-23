//! Decision ratification detection

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RatificationStatus {
    Pending,
    Ratified { by: Vec<String> },
    Rejected { by: Vec<String> },
}

impl RatificationStatus {
    pub fn is_ratified(&self) -> bool {
        matches!(self, RatificationStatus::Ratified { .. })
    }
}

pub fn detect_ratification(
    messages: &[crate::ConversationMessage],
    extraction_time: chrono::DateTime<chrono::Utc>,
) -> RatificationStatus {
    let ratification_keywords = [
        "+1",
        "agreed",
        "agree",
        "sounds good",
        "let's do it",
        "approved",
        "lgtm",
        "ship it",
        "done",
        ":+1:",
        "👍",
    ];

    let rejection_keywords = [
        "-1",
        "no",
        "disagree",
        "not a good idea",
        "bad idea",
        "i don't think so",
        "let's reconsider",
        ":-1:",
        "👎",
    ];

    let mut ratifiers: Vec<String> = Vec::new();
    let mut rejecters: Vec<String> = Vec::new();

    for message in messages {
        if message.timestamp <= extraction_time {
            continue;
        }

        let content_lower = message.content.to_lowercase();

        if ratification_keywords
            .iter()
            .any(|kw| content_lower.contains(kw))
        {
            ratifiers.push(message.author.clone());
        }

        if rejection_keywords
            .iter()
            .any(|kw| content_lower.contains(kw))
        {
            rejecters.push(message.author.clone());
        }
    }

    if !rejecters.is_empty() && rejecters.len() >= ratifiers.len() {
        RatificationStatus::Rejected { by: rejecters }
    } else if !ratifiers.is_empty() {
        RatificationStatus::Ratified { by: ratifiers }
    } else {
        RatificationStatus::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{generate_id, ConversationMessage};
    use chrono::{TimeZone, Utc};

    fn test_message(author: &str, content: &str, offset_secs: i64) -> ConversationMessage {
        ConversationMessage {
            id: generate_id(),
            session_id: "test".to_string(),
            author: author.to_string(),
            content: content.to_string(),
            timestamp: Utc.timestamp_opt(1000 + offset_secs, 0).unwrap(),
        }
    }

    #[test]
    fn test_pending_ratification() {
        let base_time = Utc.timestamp_opt(1000, 0).unwrap();
        let messages = vec![];

        let status = detect_ratification(&messages, base_time);
        assert_eq!(status, RatificationStatus::Pending);
    }

    #[test]
    fn test_successful_ratification() {
        let base_time = Utc.timestamp_opt(1000, 0).unwrap();
        let messages = vec![test_message("bob", "+1 sounds good", 10)];

        let status = detect_ratification(&messages, base_time);
        assert!(status.is_ratified());
    }

    #[test]
    fn test_rejection() {
        let base_time = Utc.timestamp_opt(1000, 0).unwrap();
        let messages = vec![test_message("bob", "-1 bad idea", 10)];

        let status = detect_ratification(&messages, base_time);
        assert!(matches!(status, RatificationStatus::Rejected { .. }));
    }
}
