//! Agent completion for chat responses.
//!
//! Uses Rig with OpenRouter for LLM calls. Supports Graph RAG context from
//! the architecture knowledge graph.

use crate::{Message, Participant};
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::{message::Message as RigMessage, Chat};
use rig::providers::openrouter;

/// Build system message from config: core prompt + optional knowledge context + optional graph RAG context.
fn build_system_message(cfg: &crate::AgentConfig, graph_context: Option<&str>) -> String {
    let mut s = cfg.system_prompt.clone();
    if let Some(ref ctx) = cfg.knowledge_context {
        if !ctx.is_empty() {
            s.push_str("\n\n## Knowledge Context\n");
            s.push_str(ctx);
        }
    }
    if let Some(ctx) = graph_context {
        if !ctx.is_empty() {
            s.push_str("\n\n## Architecture Graph Context (use for RAG)\n");
            s.push_str(ctx);
        }
    }
    s
}

/// Convert Sruja chat history to Rig messages (excludes the final prompt).
fn to_rig_messages(history: &[Message], memory_limit: Option<usize>) -> Vec<RigMessage> {
    let history = match memory_limit {
        Some(limit) if history.len() > limit => &history[history.len() - limit..],
        _ => history,
    };
    let mut rig_msgs = Vec::with_capacity(history.len());
    for msg in history {
        let role = match msg.author.kind {
            crate::ParticipantKind::Human => RigMessage::user(&msg.content),
            crate::ParticipantKind::Agent(_) => RigMessage::assistant(&msg.content),
        };
        rig_msgs.push(role);
    }
    rig_msgs
}

/// Generate an agent's reply given the conversation history and optional graph RAG context.
pub async fn generate_agent_reply(
    agent: &Participant,
    history: &[Message],
    graph_context: Option<&str>,
) -> Result<String, String> {
    let (system_message, model, memory_limit) = match &agent.kind {
        crate::ParticipantKind::Human => return Err("Participant is not an agent".to_string()),
        crate::ParticipantKind::Agent(cfg) => (
            build_system_message(cfg, graph_context),
            cfg.model.clone(),
            cfg.memory_limit_messages,
        ),
    };

    let rig_history = to_rig_messages(history, memory_limit);

    // Last message is the human's prompt we're responding to; rest is chat history
    let (chat_history, prompt) = if let Some(last_msg) = history.last() {
        let prompt = last_msg.content.clone();
        let chat_history = if rig_history.len() > 1 {
            rig_history[..rig_history.len() - 1].to_vec()
        } else {
            vec![]
        };
        (chat_history, prompt)
    } else {
        (vec![], String::new())
    };

    if prompt.is_empty() {
        return Ok(String::new());
    }

    let client = openrouter::Client::from_env();
    let rig_agent = client.agent(&model).preamble(&system_message).build();

    let response = rig_agent
        .chat(prompt, chat_history)
        .await
        .map_err(|e| format!("Agent LLM failed: {}", e))?;

    Ok(response.trim().to_string())
}
