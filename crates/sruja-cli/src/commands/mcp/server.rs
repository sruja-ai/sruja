use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

use super::config::{
    get_mcp_tool_profile, mcp_env_truthy, mcp_log_enabled, mcp_readonly_enabled,
    mcp_tools_for_list_with_readonly, mcp_trace_events_enabled, ToolProfile, ENV_MCP_WATCH_DRIFT,
    MCP_PROTOCOL_VERSION,
};
use super::run_tool::run_tool;
use super::trace::append_mcp_tool_call_event;
use super::transport::{mcp_repo_from_params, not_initialized_error};
use crate::commands::mcp_prompts::{prompts_get_result, prompts_list_result};
use crate::commands::mcp_resources::{resources_list_result, resources_read_result};

pub(crate) struct McpServer {
    initialized: bool,
    client_ready: bool,
    watch_drift: bool,
    tool_profile: ToolProfile,
    default_repo: String,
    graph_cache: std::sync::Arc<tokio::sync::Mutex<HashMap<String, sruja_scan::Graph>>>,
    pending_notifications: Vec<Value>,
}

impl McpServer {
    pub(crate) fn new(default_repo: String) -> Self {
        Self {
            initialized: false,
            client_ready: false,
            watch_drift: false,
            tool_profile: get_mcp_tool_profile(),
            default_repo,
            graph_cache: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            pending_notifications: Vec::new(),
        }
    }

    pub(crate) fn drain_pending_notifications(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.pending_notifications)
    }

    fn enqueue_notification(&mut self, method: &str, params: Value) {
        self.pending_notifications.push(json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }));
    }

    fn watch_drift_from_initialize_params(params: Option<&Value>) -> bool {
        if mcp_env_truthy(ENV_MCP_WATCH_DRIFT) {
            return true;
        }
        params
            .and_then(|p| p.get("initializationOptions"))
            .and_then(|o| o.get("watch_drift"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    fn tool_profile_from_initialize_params(params: Option<&Value>) -> ToolProfile {
        // InitializationOptions take priority
        if let Some(profile) = params
            .and_then(|p| p.get("initializationOptions"))
            .and_then(|o| o.get("tool_profile"))
            .and_then(|v| v.as_str())
        {
            match profile {
                "minimal" => return ToolProfile::Minimal,
                "coding" => return ToolProfile::Coding,
                "arch" => return ToolProfile::Arch,
                "full" | "legacy" => return ToolProfile::Full,
                "default" => return ToolProfile::Default,
                _ => {}
            }
        }
        // Fall back to env var + default
        get_mcp_tool_profile()
    }

    fn maybe_enqueue_drift_state_notification(&mut self) {
        if !self.watch_drift {
            return;
        }
        let repo_path = Path::new(&self.default_repo);
        if !repo_path.exists() {
            return;
        }
        let Ok(graph) = crate::commands::scan_repo_cached(repo_path) else {
            return;
        };
        let payload =
            crate::commands::drift_state::build_drift_state_payload(&self.default_repo, &graph);
        self.enqueue_notification("notifications/drift_state", payload);
    }

    pub(crate) async fn handle_message(&mut self, message: Value) -> Option<Value> {
        let id = message.get("id").cloned();
        let method = match message.get("method").and_then(|m| m.as_str()) {
            Some(m) => m,
            None => {
                return id.map(|id| {
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32600, "message": "Invalid Request" }
                    })
                })
            }
        };

        match method {
            "initialize" => {
                self.watch_drift = Self::watch_drift_from_initialize_params(message.get("params"));
                self.tool_profile =
                    Self::tool_profile_from_initialize_params(message.get("params"));
                self.initialized = true;
                Some(self.handle_initialize(id, message.get("params")))
            }
            "notifications/initialized" => {
                self.client_ready = true;
                self.maybe_enqueue_drift_state_notification();
                None
            }
            "ping" => id.map(|id| json!({ "jsonrpc": "2.0", "id": id, "result": {} })),
            "tools/list" => {
                if !self.initialized {
                    return id.map(|id| {
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32602, "message": "Server not initialized. Call initialize first." }
                        })
                    });
                }
                id.map(|id| self.handle_tools_list(id))
            }
            "tools/call" => {
                let id = id?;
                if !self.initialized {
                    return Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32602, "message": "Server not initialized. Call initialize first." }
                    }));
                }
                Some(self.handle_tools_call(id, message.get("params")).await)
            }
            "resources/list" => {
                let id = id?;
                if !self.initialized {
                    return Some(not_initialized_error(id));
                }
                Some(self.handle_resources_list(id, message.get("params")))
            }
            "resources/read" => {
                let id = id?;
                if !self.initialized {
                    return Some(not_initialized_error(id));
                }
                Some(self.handle_resources_read(id, message.get("params")).await)
            }
            "prompts/list" => {
                let id = id?;
                if !self.initialized {
                    return Some(not_initialized_error(id));
                }
                Some(self.handle_prompts_list(id))
            }
            "prompts/get" => {
                let id = id?;
                if !self.initialized {
                    return Some(not_initialized_error(id));
                }
                Some(self.handle_prompts_get(id, message.get("params")).await)
            }
            _ => id.map(|id| {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": format!("Unknown method: {method}") }
                })
            }),
        }
    }

    pub(crate) fn handle_initialize(&self, id: Option<Value>, params: Option<&Value>) -> Value {
        let id = id.unwrap_or_else(|| json!(0));
        let requested_protocol = params
            .and_then(|p| p.get("protocolVersion"))
            .and_then(|v| v.as_str());
        let protocol = match requested_protocol {
            Some(p) if p == MCP_PROTOCOL_VERSION => p,
            _ => MCP_PROTOCOL_VERSION,
        };
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": protocol,
                "capabilities": {
                    "tools": { "listChanged": false },
                    "resources": { "subscribe": false, "listChanged": false },
                    "prompts": { "listChanged": false },
                    "experimental": {
                        "watchDrift": true
                    }
                },
                "serverInfo": {
                    "name": "sruja",
                    "title": "Sruja",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })
    }

    pub(crate) fn handle_tools_list(&self, id: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": mcp_tools_for_list_with_readonly(
                    mcp_readonly_enabled(),
                    self.tool_profile
                )
            }
        })
    }

    fn handle_resources_list(&self, id: Value, params: Option<&Value>) -> Value {
        let repo = mcp_repo_from_params(params, &self.default_repo);
        match resources_list_result(&repo) {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err(e) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32603, "message": e.to_string() }
            }),
        }
    }

    async fn handle_resources_read(&self, id: Value, params: Option<&Value>) -> Value {
        let Some(params) = params else {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": "Missing params" }
            });
        };
        let uri = match params.get("uri").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32602, "message": "Missing resource uri" }
                });
            }
        };
        let repo = mcp_repo_from_params(Some(params), &self.default_repo);
        match resources_read_result(&repo, uri).await {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err(e) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32603, "message": e.to_string() }
            }),
        }
    }

    fn handle_prompts_list(&self, id: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": prompts_list_result()
        })
    }

    async fn handle_prompts_get(&self, id: Value, params: Option<&Value>) -> Value {
        let Some(params) = params else {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": "Missing params" }
            });
        };
        let name = match params.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32602, "message": "Missing prompt name" }
                });
            }
        };
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let repo = mcp_repo_from_params(Some(params), &self.default_repo);
        match prompts_get_result(&repo, name, &arguments).await {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err(e) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32603, "message": e.to_string() }
            }),
        }
    }

    async fn handle_tools_call(&self, id: Value, params: Option<&Value>) -> Value {
        let Some(params) = params else {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": "Missing params" }
            });
        };

        let name = match params.get("name").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32602, "message": "Missing tool name" }
                })
            }
        };

        let args = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let repo_for_log = args
            .get("path")
            .or_else(|| args.get("repo"))
            .and_then(|v| v.as_str())
            .unwrap_or(&self.default_repo)
            .to_string();

        let run_id_for_log = args
            .get("run_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let t0 = std::time::Instant::now();
        let result = run_tool(name, &args, &self.default_repo, &self.graph_cache).await;
        let elapsed_ms = t0.elapsed().as_millis() as u64;

        if mcp_log_enabled() {
            let ok = result.is_ok();
            let err_one_line = result
                .as_ref()
                .err()
                .map(|e| e.to_string().lines().collect::<Vec<_>>().join(" "));
            let line = json!({
                "mcp_tool_call": true,
                "tool": name,
                "repo": repo_for_log,
                "run_id": run_id_for_log.as_deref(),
                "ms": elapsed_ms,
                "ok": ok,
                "error": err_one_line,
            });
            eprintln!("{}", line);
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
                &args,
                run_id_for_log.as_deref(),
                ok,
                err_one_line.as_deref(),
                elapsed_ms,
            );
        }

        match result {
            Ok(text) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": text }],
                    "isError": false
                }
            }),
            Err(err) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": err.to_string() }],
                    "isError": true
                }
            }),
        }
    }
}
