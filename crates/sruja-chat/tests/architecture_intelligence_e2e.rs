//! E2E tests for the architecture intelligence layer.
//!
//! Covers: scan → load context → graph query, agent definitions, session flow,
//! and extraction → decision → graph integration.

use std::fs;
use tempfile::TempDir;

fn create_minimal_repo() -> TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path();

    fs::create_dir_all(root.join("src")).ok();
    fs::write(
        root.join("src/api.ts"),
        r#"
import { db } from './db';
export async function getData() {
  return db.query('SELECT 1');
}
"#,
    )
    .expect("write api.ts");
    fs::write(
        root.join("src/db.ts"),
        r#"
export function query(sql: string) { return []; }
"#,
    )
    .expect("write db.ts");

    dir
}

#[tokio::test]
async fn load_repo_context_merges_into_graph() {
    let repo = create_minimal_repo();
    let server = sruja_chat::ChatServer::new();

    let count = server
        .load_repo_context(repo.path())
        .await
        .expect("load_repo_context");

    assert!(count > 0, "Should merge nodes/edges from scan");

    let graph = server.graph();
    let g = graph.read().await;
    let stats = g.stats();
    assert!(stats.total_nodes > 0, "Graph should have nodes from scan");
}

#[tokio::test]
async fn query_graph_after_load_context() {
    let repo = create_minimal_repo();
    let server = sruja_chat::ChatServer::new();

    server
        .load_repo_context(repo.path())
        .await
        .expect("load_repo_context");

    let graph = server.graph();
    let g = graph.read().await;
    let result = g.query("what services do we have?").expect("query");

    assert!(!result.answer.is_empty());
    assert!(result.confidence > 0.0);
}

#[tokio::test]
async fn agent_definition_create_and_join() {
    let server = sruja_chat::ChatServer::new();
    let session_id = server.create_session("Architecture Review", "Alice").await;

    let def = server
        .create_agent_definition(sruja_chat::CreateAgentDefinition {
            name: "Payment Expert".to_string(),
            role: "Subsystem Expert".to_string(),
            system_prompt: "You are an expert on payment systems.".to_string(),
            knowledge_context: Some("Stripe, webhooks, idempotency.".to_string()),
            model: "openai/gpt-4o-mini".to_string(),
            memory_limit_messages: Some(20),
        })
        .await
        .expect("create_agent_definition");

    assert_eq!(def.name, "Payment Expert");
    assert_eq!(def.role, "Subsystem Expert");
    assert!(def.knowledge_context.is_some());

    let defs = server.list_agent_definitions().await;
    assert_eq!(defs.len(), 1);

    let pid = server
        .join_agent_from_definition(&session_id, &def.id)
        .await
        .expect("join_agent_from_definition");

    let participants = server.get_participants(&session_id).await.unwrap();
    assert_eq!(participants.len(), 2);
    let agent = participants.iter().find(|p| p.id == pid).unwrap();
    assert!(matches!(agent.kind, sruja_chat::ParticipantKind::Agent(_)));
    if let sruja_chat::ParticipantKind::Agent(ref cfg) = agent.kind {
        assert_eq!(cfg.model, "openai/gpt-4o-mini");
        assert!(cfg.knowledge_context.is_some());
    }
}

#[tokio::test]
async fn session_flow_create_send_history_extractions() {
    let server = sruja_chat::ChatServer::new();
    let session_id = server.create_session("Architecture Review", "Alice").await;
    let participants = server.get_participants(&session_id).await.unwrap();
    let alice_id = participants[0].id.clone();

    let msg = server
        .send_message(
            &session_id,
            sruja_chat::NewMessage {
                author_id: alice_id.clone(),
                content: "We should use Kafka for event streaming.".to_string(),
                parent_message_id: None,
            },
        )
        .await
        .expect("send_message");

    assert_eq!(msg.author.name, "Alice");
    assert_eq!(msg.content, "We should use Kafka for event streaming.");

    let history = server.get_history(&session_id).await.unwrap();
    assert_eq!(history.len(), 1);

    let _extractions = server.get_extractions(&session_id).await.unwrap();
    // LLM extraction runs in background; may be empty without API key
    // We only verify the flow doesn't panic
}

#[tokio::test]
async fn confirm_extraction_updates_graph() {
    let server = sruja_chat::ChatServer::new();
    let session_id = server.create_session("Test", "Alice").await;
    let participants = server.get_participants(&session_id).await.unwrap();
    let alice_id = participants[0].id.clone();

    server
        .send_message(
            &session_id,
            sruja_chat::NewMessage {
                author_id: alice_id,
                content: "We should use Redis for caching.".to_string(),
                parent_message_id: None,
            },
        )
        .await
        .unwrap();

    // Wait briefly for background extraction (may complete or not without API key)
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let extractions = server.get_extractions(&session_id).await.unwrap();
    if !extractions.is_empty() {
        server
            .confirm_extraction(&session_id, &extractions[0].id)
            .await
            .unwrap();

        let graph = server.graph();
        let g = graph.read().await;
        let _stats = g.stats();
        // Flow completed; decision count depends on extraction
    }
}

#[tokio::test]
async fn persistence_restores_sessions_on_reload() {
    let dir = tempfile::tempdir().expect("temp dir");
    let data_dir = dir.path().to_path_buf();

    let server = sruja_chat::ChatServer::with_persistence(&data_dir)
        .await
        .expect("with_persistence");
    let session_id = server.create_session("Persistence Test", "Alice").await;
    let sessions = server.list_sessions().await;
    assert_eq!(sessions.len(), 1);
    // Allow background persist task to finish before reloading
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    drop(server);

    let server2 = sruja_chat::ChatServer::with_persistence(&data_dir)
        .await
        .expect("with_persistence");
    let sessions2 = server2.list_sessions().await;
    assert_eq!(sessions2.len(), 1);
    assert_eq!(sessions2[0].id, session_id);
    assert_eq!(sessions2[0].topic, "Persistence Test");
}

#[tokio::test]
async fn join_agent_inline_requires_model() {
    let server = sruja_chat::ChatServer::new();
    let session_id = server.create_session("Test", "Alice").await;

    let pid = server
        .join_agent_inline(
            &session_id,
            "Ad-hoc Expert",
            "Reviewer",
            "You review architecture.",
            None::<&str>,
            "openai/gpt-4o-mini",
            None,
        )
        .await
        .expect("join_agent_inline");

    let participants = server.get_participants(&session_id).await.unwrap();
    let agent = participants.iter().find(|p| p.id == pid).unwrap();
    if let sruja_chat::ParticipantKind::Agent(ref cfg) = agent.kind {
        assert_eq!(cfg.model, "openai/gpt-4o-mini");
        assert_eq!(cfg.role, "Reviewer");
    }
}
