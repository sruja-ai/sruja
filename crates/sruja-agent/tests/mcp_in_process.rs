//! In-process MCP test harness.
//!
//! Tests the MCP client/server integration using tokio::io::duplex to create
//! an in-process rmcp server and client pair — no subprocess needed.

#![cfg(feature = "mcp-client")]

use rmcp::model::{CallToolResult, Content, ServerCapabilities, ServerInfo, Tool};use rmcp::service::MaybeSendFuture;
use rmcp::{ErrorData, ServerHandler, ServiceExt};

/// A fake MCP server that exposes a simple echo tool and a read-only query tool.
#[derive(Clone)]
struct FakeMcpServer {
    tools: Vec<Tool>,
}

impl FakeMcpServer {
    fn new() -> Self {
        let empty_schema = rmcp::model::JsonObject::new();
        let echo = Tool::new("echo", "Echo back the input", empty_schema.clone());
        let query = {
            let mut t = Tool::new("query", "Query data", empty_schema);
            let mut ann = rmcp::model::ToolAnnotations::default();
            ann.read_only_hint = Some(true);
            t.annotations = Some(ann);
            t
        };
        Self {
            tools: vec![echo, query],
        }
    }
}

#[allow(clippy::manual_async_fn)]
impl ServerHandler for FakeMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListToolsResult, ErrorData>> + MaybeSendFuture + '_ {
        let tools = self.tools.clone();
        async move {
            Ok(rmcp::model::ListToolsResult::with_all_items(tools))
        }
    }

    fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + MaybeSendFuture + '_ {
        async move {
            let name = &request.name;
            let text = match name.as_ref() {
                "echo" => {
                    let msg = request
                        .arguments
                        .as_ref()
                        .and_then(|a| a.get("message"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("(empty)");
                    format!("echo: {msg}")
                }
                "query" => "query result: 42".to_string(),
                _ => return Ok(CallToolResult::error(vec![Content::text(format!("unknown tool: {name}"))])),
            };
            Ok(CallToolResult::success(vec![Content::text(text)]))
        }
    }
}

#[tokio::test]
async fn test_in_process_mcp_client_discovers_and_calls_tools() {
    let (client_io, server_io) = tokio::io::duplex(4096);

    // Start fake server on one end of the duplex
    let server = FakeMcpServer::new();
    let server_handle = tokio::spawn(async move {
        server.serve(server_io).await.unwrap().waiting().await.unwrap();
    });

    // Connect rmcp client on the other end
    let client = ().serve(client_io).await.expect("client init");

    // List tools
    let tools_result = client
        .list_tools(Default::default())
        .await
        .expect("list_tools");
    assert_eq!(tools_result.tools.len(), 2);
    assert_eq!(tools_result.tools[0].name, "echo");
    assert_eq!(tools_result.tools[1].name, "query");

    // Call echo tool
    let mut args = rmcp::model::JsonObject::new();
    args.insert("message".to_string(), serde_json::json!("hello world"));
    let echo_result = client
        .call_tool(rmcp::model::CallToolRequestParams::new("echo").with_arguments(args))
        .await
        .expect("call echo");
    assert!(!echo_result.is_error.unwrap_or(false));
    let text = &echo_result.content[0];
    assert!(format!("{text:?}").contains("echo: hello world"));

    // Call query tool
    let query_result = client
        .call_tool(rmcp::model::CallToolRequestParams::new("query"))
        .await
        .expect("call query");
    assert!(!query_result.is_error.unwrap_or(false));
    let text = &query_result.content[0];
    assert!(format!("{text:?}").contains("query result: 42"));

    // Clean up
    drop(client);
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_in_process_mcp_unknown_tool_returns_error_result() {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let server = FakeMcpServer::new();
    let server_handle = tokio::spawn(async move {
        server.serve(server_io).await.unwrap().waiting().await.unwrap();
    });

    let client = ().serve(client_io).await.expect("client init");

    let result = client
        .call_tool(rmcp::model::CallToolRequestParams::new("nonexistent"))
        .await
        .expect("call should succeed at protocol level");

    // Should be isError=true (tool-level error), not a JSON-RPC error
    assert!(result.is_error.unwrap_or(false));

    drop(client);
    let _ = server_handle.await;
}

/// Verify that read-only hint annotation is preserved through the wire.
#[tokio::test]
async fn test_read_only_hint_preserved() {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let server = FakeMcpServer::new();
    let server_handle = tokio::spawn(async move {
        server.serve(server_io).await.unwrap().waiting().await.unwrap();
    });

    let client = ().serve(client_io).await.expect("client init");

    let tools = client.list_tools(Default::default()).await.unwrap();

    let query_tool = tools
        .tools
        .iter()
        .find(|t| t.name == "query")
        .expect("query tool should exist");
    assert!(query_tool
        .annotations
        .as_ref()
        .and_then(|a| a.read_only_hint)
        .unwrap_or(false));

    drop(client);
    let _ = server_handle.await;
}