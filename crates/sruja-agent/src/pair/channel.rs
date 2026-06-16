//! Communication channel between driver and navigator.

use std::sync::{Arc, Mutex};

/// Message types exchanged between pair partners.
#[derive(Debug, Clone)]
pub enum ChannelMessage {
    /// Navigator observes something about the task.
    Observation {
        agent: super::PairRole,
        content: String,
    },
    /// Driver proposes a plan.
    PlanReview {
        agent: super::PairRole,
        feedback: String,
        approved: bool,
    },
    /// Driver makes a code change.
    Change {
        agent: super::PairRole,
        description: String,
        files_affected: Vec<String>,
    },
    /// Navigator reviews a change.
    Review {
        agent: super::PairRole,
        approved: bool,
        feedback: String,
    },
    /// Navigator suggests a fix.
    Suggestion {
        agent: super::PairRole,
        suggestion: String,
    },
    /// Roles are swapped.
    RoleSwap {
        from: super::PairRole,
        to: super::PairRole,
    },
    /// Navigator suggests cleanup.
    Cleanup {
        agent: super::PairRole,
        suggestions: Vec<String>,
    },
}

/// Shared communication channel between pair partners.
#[derive(Debug, Clone)]
pub struct Channel {
    messages: Arc<Mutex<Vec<ChannelMessage>>>,
}

impl Channel {
    /// Create a new empty channel.
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Send a message through the channel.
    pub fn send(&self, msg: ChannelMessage) {
        if let Ok(mut messages) = self.messages.lock() {
            messages.push(msg);
        }
    }

    /// Receive all pending messages (drains the queue).
    pub fn receive_all(&self) -> Vec<ChannelMessage> {
        if let Ok(mut messages) = self.messages.lock() {
            std::mem::take(&mut *messages)
        } else {
            Vec::new()
        }
    }

    /// Peek at messages without draining.
    pub fn peek(&self) -> Vec<ChannelMessage> {
        if let Ok(messages) = self.messages.lock() {
            messages.clone()
        } else {
            Vec::new()
        }
    }

    /// Number of pending messages.
    pub fn pending_count(&self) -> usize {
        self.messages.lock().map(|m| m.len()).unwrap_or(0)
    }

    /// Get all reviews (filter by Review variant).
    pub fn get_reviews(&self) -> Vec<(bool, String)> {
        self.peek()
            .into_iter()
            .filter_map(|msg| match msg {
                ChannelMessage::Review {
                    approved, feedback, ..
                } => Some((approved, feedback)),
                _ => None,
            })
            .collect()
    }

    /// Check if any review rejected a change.
    pub fn has_rejections(&self) -> bool {
        self.peek().iter().any(|msg| {
            matches!(
                msg,
                ChannelMessage::Review {
                    approved: false,
                    ..
                }
            )
        })
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::PairRole;
    use super::*;

    #[test]
    fn channel_send_receive() {
        let channel = Channel::new();
        channel.send(ChannelMessage::Change {
            agent: PairRole::Driver,
            description: "Added rate limiter".to_string(),
            files_affected: vec!["src/ratelimit.rs".to_string()],
        });

        assert_eq!(channel.pending_count(), 1);

        let messages = channel.receive_all();
        assert_eq!(messages.len(), 1);
        assert_eq!(channel.pending_count(), 0);
    }

    #[test]
    fn channel_peek_does_not_drain() {
        let channel = Channel::new();
        channel.send(ChannelMessage::Review {
            agent: PairRole::Navigator,
            approved: true,
            feedback: "Looks good".to_string(),
        });

        let _ = channel.peek();
        assert_eq!(channel.pending_count(), 1); // still there
    }

    #[test]
    fn channel_reviews() {
        let channel = Channel::new();
        channel.send(ChannelMessage::Review {
            agent: PairRole::Navigator,
            approved: true,
            feedback: "Good".to_string(),
        });
        channel.send(ChannelMessage::Review {
            agent: PairRole::Navigator,
            approved: false,
            feedback: "Missing tests".to_string(),
        });

        let reviews = channel.get_reviews();
        assert_eq!(reviews.len(), 2);
        assert!(channel.has_rejections());
    }

    #[test]
    fn channel_role_swap() {
        let channel = Channel::new();
        channel.send(ChannelMessage::RoleSwap {
            from: PairRole::Driver,
            to: PairRole::Navigator,
        });

        let msgs = channel.receive_all();
        assert!(matches!(msgs[0], ChannelMessage::RoleSwap { .. }));
    }
}
