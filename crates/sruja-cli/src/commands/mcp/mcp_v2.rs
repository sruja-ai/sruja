//! MCP v2 server using rmcp SDK.
//!
//! Consolidates the existing MCP server onto rmcp by reusing the `run_tool` dispatch
//! and existing tool definitions, resources, and prompts.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use rmcp::model::{
    Annotated, CallToolResult, Content, GetPromptResult, Implementation, ListPromptsResult,
    ListResourcesResult, ListToolsResult, Prompt, PromptArgument, PromptMessage,
    PromptMessageContent, PromptMessageRole, RawResource, ReadResourceResult, ResourceContents,
    ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::MaybeSendFuture;
use rmcp::{ErrorData, ServerHandler, ServiceExt};

use super::config::{
    get_mcp_tool_profile, mcp_log_enabled, mcp_readonly_enabled, mcp_tools_for_list_with_readonly,
    mcp_trace_events_enabled,
};
use super::run_tool::run_tool;
use super::trace::append_mcp_tool_call_event;
use crate::commands::{mcp_prompts, mcp_resources};
use crate::commands::CliError;

struct SrujaMcpServer {
    default_repo: String,
    tool_profile: super::config::ToolProfile,
    graph_cache: Arc<tokio::sync::Mutex<HashMap<String, sruja_scan::Graph>>>,
}

impl SrujaMcpServer {
    fn new(default_repo: String) -> Self {
        Self {
            default_repo,
            tool_profile: get_mcp_tool_profile(),
            graph_cache: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}

fn convert_tool_def(tool_value: &Value) -> Option<Tool> {
    let obj = tool_value.as_object()?;
    let name = obj.get("name").and_then(|v| v.as_str())?.to_string();
    let description = obj
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let input_schema: rmcp::model::JsonObject = obj
        .get("inputSchema")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let mut tool = Tool::new(name, description, input_schema);
    if let Some(ann) = obj.get("annotations") {
        if let Some(ro) = ann.get("readOnlyHint").and_then(|v| v.as_bool()) {
            let mut tool_ann = rmcp::model::ToolAnnotations::default();
            tool_ann.read_only_hint = Some(ro);
            tool.annotations = Some(tool_ann);
        }
    }
    Some(tool)
}

#[allow(clippy::manual_async_fn)]
impl ServerHandler for SrujaMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
        )
        .with_server_info(Implementation::new(
            "sruja",
            env!("CARGO_PKG_VERSION"),
        ))
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + MaybeSendFuture + '_ {
        async move {
            let tools_json =
                mcp_tools_for_list_with_readonly(mcp_readonly_enabled(), self.tool_profile);

            let tools: Vec<Tool> = tools_json.iter().filter_map(convert_tool_def).collect();

            Ok(ListToolsResult::with_all_items(tools))
        }
    }

    fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, ErrorData>> + MaybeSendFuture + '_ {
        async move {
            let name = &request.name;
            let arguments = request
                .arguments
                .as_ref()
                .map(|m| Value::Object(m.clone()))
                .unwrap_or(json!({}));

            let repo_for_log = arguments
                .get("path")
                .or_else(|| arguments.get("repo"))
                .and_then(|v| v.as_str())
                .unwrap_or(&self.default_repo)
                .to_string();

            let run_id_for_log = arguments
                .get("run_id")
                .and_then(|v| v.as_str())
                .map(String::from);

            let t0 = std::time::Instant::now();
            let result = run_tool(name, &arguments, &self.default_repo, &self.graph_cache).await;
            let elapsed_ms = t0.elapsed().as_millis() as u64;

            if mcp_log_enabled() {
                let ok = result.is_ok();
                let err_one_line = result
                    .as_ref()
                    .err()
                    .map(|e| e.to_string().lines().collect::<Vec<_>>().join(" "));
                eprintln!(
                    "{}",
                    json!({
                        "mcp_tool_call": true,
                        "tool": name,
                        "repo": repo_for_log,
                        "run_id": run_id_for_log.as_deref(),
                        "ms": elapsed_ms,
                        "ok": ok,
                        "error": err_one_line,
                    })
                );
            }

            if mcp_trace_events_enabled() && !name.starts_with("sruja_record_") {
                let ok = result.is_ok();
                let err_one_line = result
                    .as_ref()
                    .err()
                    .map(|e| e.to_string().lines().collect::<Vec<_>>().join(" "));
                let _ = append_mcp_tool_call_event(
                    &repo_for_log,
                    name,
                    &arguments,
                    run_id_for_log.as_deref(),
                    ok,
                    err_one_line.as_deref(),
                    elapsed_ms,
                );
            }

            match result {
                Ok(text) => Ok(CallToolResult::success(vec![Content::text(text)])),
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
            }
        }
    }

    fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, ErrorData>> + MaybeSendFuture + '_ {
        async move {
            match mcp_resources::resources_list_result(&self.default_repo) {
                Ok(result) => {
                    let resources: Vec<Annotated<RawResource>> = result
                        .get("resources")
                        .and_then(|r| r.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|r| {
                                    Some(Annotated::new(
                                        RawResource {
                                            uri: r.get("uri")?.as_str()?.to_string(),
                                            name: r.get("name")?.as_str()?.to_string(),
                                            title: None,
                                            description: r
                                                .get("description")
                                                .and_then(|v| v.as_str())
                                                .map(String::from),
                                            mime_type: None,
                                            size: None,
                                            icons: None,
                                            meta: None,
                                        },
                                        None,
                                    ))
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    Ok(ListResourcesResult::with_all_items(resources))
                }
                Err(e) => Err(ErrorData::internal_error(e.to_string(), None)),
            }
        }
    }

    fn read_resource(
        &self,
        request: rmcp::model::ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, ErrorData>> + MaybeSendFuture + '_ {
        async move {
            let uri = &request.uri;
            match mcp_resources::resources_read_result(&self.default_repo, uri).await {
                Ok(result) => {
                    let contents: Vec<ResourceContents> = result
                        .get("contents")
                        .and_then(|c| c.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|c| {
                                    let uri = c.get("uri")?.as_str()?.to_string();
                                    let text = c
                                        .get("text")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string();
                                    Some(ResourceContents::TextResourceContents {
                                        uri,
                                        mime_type: None,
                                        text,
                                        meta: None,
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    Ok(ReadResourceResult::new(contents))
                }
                Err(e) => Err(ErrorData::internal_error(e.to_string(), None)),
            }
        }
    }

    fn list_prompts(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<ListPromptsResult, ErrorData>> + MaybeSendFuture + '_ {
        async move {
            let result = mcp_prompts::prompts_list_result();
            let prompts: Vec<Prompt> = result
                .get("prompts")
                .and_then(|p| p.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|p| {
                            let name = p.get("name")?.as_str()?.to_string();
                            let description = p
                                .get("description")
                                .and_then(|v| v.as_str())
                                .map(String::from);
                            let arguments: Option<Vec<PromptArgument>> = p
                                .get("arguments")
                                .and_then(|a| a.as_array())
                                .map(|args| {
                                    args.iter()
                                        .filter_map(|a| {
                                            let mut pa =
                                                PromptArgument::new(
                                                    a.get("name")?.as_str()?.to_string(),
                                                );
                                            if let Some(desc) = a
                                                .get("description")
                                                .and_then(|v| v.as_str())
                                            {
                                                pa = pa.with_description(desc);
                                            }
                                            if let Some(req) =
                                                a.get("required").and_then(|v| v.as_bool())
                                            {
                                                pa = pa.with_required(req);
                                            }
                                            Some(pa)
                                        })
                                        .collect()
                                });

                            Some(Prompt::new(name, description, arguments))
                        })
                        .collect()
                })
                .unwrap_or_default();

            Ok(ListPromptsResult::with_all_items(prompts))
        }
    }

    fn get_prompt(
        &self,
        request: rmcp::model::GetPromptRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<GetPromptResult, ErrorData>> + MaybeSendFuture + '_ {
        async move {
            let name = &request.name;
            let arguments = request
                .arguments
                .as_ref()
                .map(|m| Value::Object(m.clone()))
                .unwrap_or(json!({}));

            match mcp_prompts::prompts_get_result(&self.default_repo, name, &arguments).await {
                Ok(result) => {
                    let messages: Vec<PromptMessage> = result
                        .get("messages")
                        .and_then(|m| m.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|m| {
                                    let role = m.get("role")?.as_str()?;
                                    let text = m
                                        .get("content")
                                        .and_then(|c| c.get("text"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string();
                                    let role = match role {
                                        "assistant" => PromptMessageRole::Assistant,
                                        _ => PromptMessageRole::User,
                                    };
                                    Some(PromptMessage::new(
                                        role,
                                        PromptMessageContent::Text { text },
                                    ))
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    Ok(GetPromptResult::new(messages))
                }
                Err(e) => Err(ErrorData::internal_error(e.to_string(), None)),
            }
        }
    }
}

pub async fn mcp_v2(root: &str) -> Result<(), CliError> {
    let server = SrujaMcpServer::new(root.to_string());

    let transport = rmcp::transport::stdio();

    let service = server
        .serve(transport)
        .await
        .map_err(|e| CliError::validation(format!("mcp v2 serve error: {e}")))?;

    service
        .waiting()
        .await
        .map_err(|e| CliError::validation(format!("mcp v2 waiting error: {e}")))?;

    Ok(())
}