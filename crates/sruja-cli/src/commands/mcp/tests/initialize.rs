use super::*;
use serde_json::{json, Value};

#[test]
fn mcp_initialize_result_includes_capabilities() {
    let server = McpServer::new(".".to_string());
    let resp = server.handle_initialize(
        Some(json!(1)),
        Some(&json!({ "protocolVersion": MCP_PROTOCOL_VERSION })),
    );

    assert_eq!(resp.get("jsonrpc").and_then(|v| v.as_str()), Some("2.0"));
    assert_eq!(resp.get("id").and_then(|v| v.as_i64()), Some(1));
    assert_eq!(
        resp.pointer("/result/protocolVersion")
            .and_then(|v| v.as_str()),
        Some(MCP_PROTOCOL_VERSION)
    );
    assert!(resp.pointer("/result/capabilities/tools").is_some());
    assert!(resp.pointer("/result/capabilities/resources").is_some());
    assert!(resp.pointer("/result/capabilities/prompts").is_some());
    assert_eq!(
        resp.pointer("/result/capabilities/experimental/watchDrift")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
