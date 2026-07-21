use super::*;
use serde_json::json;

#[tokio::test]
async fn mcp_watch_drift_env_enables_notification() {
    let mut server = McpServer::new(".".to_string());
    let _guard = ENV_LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .expect("env lock");
    std::env::set_var(ENV_MCP_WATCH_DRIFT, "1");
    let _ = server
        .handle_message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "protocolVersion": MCP_PROTOCOL_VERSION }
        }))
        .await;
    let _ = server
        .handle_message(json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }))
        .await;
    std::env::remove_var(ENV_MCP_WATCH_DRIFT);
    let pending = server.drain_pending_notifications();
    assert!(
        pending
            .iter()
            .any(|n| n.get("method").and_then(|m| m.as_str()) == Some("notifications/drift_state")),
        "expected drift_state from SRUJA_MCP_WATCH_DRIFT, got: {pending:?}"
    );
}

#[tokio::test]
async fn mcp_watch_drift_emits_notification_after_initialized() {
    let mut server = McpServer::new(".".to_string());
    let _ = server
        .handle_message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "initializationOptions": { "watch_drift": true }
            }
        }))
        .await;
    let _ = server
        .handle_message(json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }))
        .await;
    let pending = server.drain_pending_notifications();
    assert!(
        pending.iter().any(|n| {
            n.get("method").and_then(|m| m.as_str()) == Some("notifications/drift_state")
                && n.pointer("/params/schema_version").and_then(|v| v.as_str())
                    == Some("drift_state/v1")
        }),
        "expected drift_state notification, got: {pending:?}"
    );
}
