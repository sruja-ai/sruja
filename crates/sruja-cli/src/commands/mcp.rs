use serde_json::{json, Value};
use sruja_agent::{AgenticMemory, ExperimentOutcome, LearningEntry};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

use super::CliError;
use crate::commands::mcp_prompts::{prompts_get_result, prompts_list_result};
use crate::commands::mcp_resources::{resources_list_result, resources_read_result};
use crate::commands::{agent_run_to_string, AgentRunOptions};
use crate::integrations::{
    resolve_enrichment_plan, resolve_openai_auth, run_cmd_enrichment, run_openai_markdown,
};

const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

/// When set to `1`, `true`, `yes`, or `on` (case-insensitive), MCP lists only read/query tools and rejects mutating tool calls.
const ENV_MCP_READONLY: &str = "SRUJA_MCP_READONLY";
/// When set to `1`, `true`, `yes`, or `on`, emit one JSON line per `tools/call` on stderr for observability.
const ENV_MCP_LOG: &str = "SRUJA_MCP_LOG";
/// When set to `1`, `true`, `yes`, or `on`, append a `context_event/v2` row per `tools/call`.
const ENV_MCP_TRACE_EVENTS: &str = "SRUJA_MCP_TRACE_EVENTS";

fn mcp_env_truthy(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => {
            let v = v.trim();
            v.eq_ignore_ascii_case("1")
                || v.eq_ignore_ascii_case("true")
                || v.eq_ignore_ascii_case("yes")
                || v.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

fn mcp_readonly_enabled() -> bool {
    mcp_env_truthy(ENV_MCP_READONLY)
}

fn mcp_log_enabled() -> bool {
    mcp_env_truthy(ENV_MCP_LOG)
}

fn mcp_trace_events_enabled() -> bool {
    mcp_env_truthy(ENV_MCP_TRACE_EVENTS)
}

/// Tools that write under `.sruja`, mutate git state, run user-supplied gate commands, or may apply repo changes.
const MCP_MUTATING_TOOLS: &[&str] = &[
    "sruja_propose_topology_change",
    "sruja_commit_evolution",
    "sruja_add_element",
    "sruja_add_relationship",
    "sruja_propose_change",
    "sruja_ai_scratchpad",
    "sruja_sandbox",
    "sruja_evaluate_proposal",
    "sruja_record_learning",
    "sruja_record_learn_feedback",
    "sruja_agent_run",
    "sruja_record_context_event",
    "sruja_record_decision_event",
    "sruja_create_decision_record",
    "sruja_link_decision_to_element",
];

fn is_mutating_mcp_tool(name: &str) -> bool {
    MCP_MUTATING_TOOLS.contains(&name)
}

fn mcp_tools_for_list() -> Vec<Value> {
    mcp_tools_for_list_with_readonly(mcp_readonly_enabled())
}

fn mcp_tools_for_list_with_readonly(readonly: bool) -> Vec<Value> {
    let defs = tool_definitions();
    if !readonly {
        return defs;
    }
    defs.into_iter()
        .filter(|t| {
            let tool_name = t.get("name").and_then(|n| n.as_str()).unwrap_or("");
            !is_mutating_mcp_tool(tool_name)
        })
        .collect()
}

pub async fn mcp(root: &str) -> Result<(), CliError> {
    let stdin = io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout);

    let mut server = McpServer::new(root.to_string());

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let message: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("mcp parse error: {err}");
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": 0,
                    "error": { "code": -32700, "message": "Parse error" }
                });
                write_message(&mut out, &resp).await?;
                continue;
            }
        };

        if let Some(response) = server.handle_message(message).await {
            write_message(&mut out, &response).await?;
        }
        for notification in server.drain_pending_notifications() {
            write_message(&mut out, &notification).await?;
        }
    }

    Ok(())
}

async fn write_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    message: &Value,
) -> Result<(), CliError> {
    let serialized = serde_json::to_string(message)?;
    writer.write_all(serialized.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

fn not_initialized_error(id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32602, "message": "Server not initialized. Call initialize first." }
    })
}

fn mcp_repo_from_params(params: Option<&Value>, default_repo: &str) -> String {
    params
        .and_then(|p| {
            p.get("path")
                .or_else(|| p.get("repo"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or(default_repo)
        .to_string()
}

struct McpServer {
    initialized: bool,
    client_ready: bool,
    watch_drift: bool,
    default_repo: String,
    graph_cache: std::sync::Arc<tokio::sync::Mutex<HashMap<String, sruja_scan::Graph>>>,
    pending_notifications: Vec<Value>,
}

impl McpServer {
    fn new(default_repo: String) -> Self {
        Self {
            initialized: false,
            client_ready: false,
            watch_drift: false,
            default_repo,
            graph_cache: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            pending_notifications: Vec::new(),
        }
    }

    fn drain_pending_notifications(&mut self) -> Vec<Value> {
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
        params
            .and_then(|p| p.get("initializationOptions"))
            .and_then(|o| o.get("watch_drift"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
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

    async fn handle_message(&mut self, message: Value) -> Option<Value> {
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

    fn handle_initialize(&self, id: Option<Value>, params: Option<&Value>) -> Value {
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

    fn handle_tools_list(&self, id: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": mcp_tools_for_list()
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

fn append_mcp_tool_call_event(
    repo: &str,
    tool: &str,
    args: &Value,
    run_id: Option<&str>,
    ok: bool,
    error: Option<&str>,
    elapsed_ms: u64,
) -> Result<(), String> {
    let repo_path = std::path::Path::new(repo);

    let elements = args
        .get("elements")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .or_else(|| {
            args.get("element_id")
                .and_then(|v| v.as_str())
                .map(|s| vec![s.to_string()])
        });

    let args_keys = args
        .as_object()
        .map(|m| {
            let mut keys = m.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            keys
        })
        .unwrap_or_default();

    let record = crate::commands::context_events::ContextEventRecord {
        schema_version: crate::commands::context_events::CONTEXT_EVENTS_SCHEMA_V2.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        kind: "context_retrieved".to_string(),
        outcome: if ok {
            "ok".to_string()
        } else {
            "fail".to_string()
        },
        policy_fingerprint: crate::commands::context_events::policy_fingerprint(repo_path),
        strict: None,
        details: serde_json::json!({
            "repo": repo,
            "tool": tool,
            "elapsed_ms": elapsed_ms,
            "ok": ok,
            "error": error,
            "args_keys": args_keys,
        }),
        trace_id: run_id.map(|s| s.to_string()),
        decision_id: None,
        run_id: run_id.map(|s| s.to_string()),
        workflow_id: None,
        actor: Some("agent".to_string()),
        source: Some("mcp".to_string()),
        tool: Some(tool.to_string()),
        elements,
        subject_ids: None,
        evidence_refs: None,
        summary: Some(format!("mcp tools/call: {tool}")),
    };

    crate::commands::context_events::validate_context_event_record(&record)?;
    crate::commands::context_events::append_context_event(repo_path, record);
    Ok(())
}

fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "sruja_get_repomap",
            "title": "Sruja RepoMap",
            "description": "Generate a token-optimized repository map with tree-sitter signatures.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_list_architecture_index",
            "title": "Sruja Architecture Index",
            "description": "Progressive disclosure: list architecture element IDs with minimal metadata (Index layer). Prefers declared architecture (repo.sruja) and falls back to scanned graph nodes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "max_tokens": { "type": "integer", "description": "Maximum estimated tokens for the response (default: 2000)", "minimum": 200 },
                    "kinds": { "type": "array", "items": { "type": "string" }, "description": "Optional element kind filter (e.g. system, container, component, database, queue, person)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_topology",
            "title": "Sruja Topology",
            "description": "Progressive disclosure: get upstream/downstream topology for an element (Topology layer). Prefers declared relationships (repo.sruja) and falls back to scanned graph blast radius.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "id": { "type": "string", "description": "Element ID (prefer fully-qualified IDs)" },
                    "depth": { "type": "integer", "description": "Traversal depth (default: 1)", "minimum": 1, "maximum": 4 },
                    "max_tokens": { "type": "integer", "description": "Maximum estimated tokens for the response (default: 5000)", "minimum": 500 }
                },
                "required": ["id"]
            }
        }),
        json!({
            "name": "sruja_get_elements",
            "title": "Sruja Get Elements",
            "description": "Progressive disclosure: fetch element details for a list of IDs (Detail layer). Prefers declared elements (repo.sruja) and falls back to scanned graph nodes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "ids": { "type": "array", "items": { "type": "string" }, "description": "Element IDs to fetch" },
                    "max_tokens": { "type": "integer", "description": "Maximum estimated tokens for the response (default: 8000)", "minimum": 500 }
                },
                "required": ["ids"]
            }
        }),
        json!({
            "name": "sruja_get_diagnostic_full",
            "title": "Sruja Diagnostic Full Text",
            "description": "Fetch the full text of a truncated diagnostic stored under .sruja/vfs/diagnostics/ (URI from head/tail truncation payloads).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "uri": { "type": "string", "description": "sruja-vfs://diagnostics/<filename> or bare filename" }
                },
                "required": ["uri"]
            }
        }),
        json!({
            "name": "sruja_suggest_context_prune",
            "title": "Sruja Suggest Context Prune",
            "description": "Graph-aware prune suggestion: which session element IDs to compress vs keep based on topology distance to active focus. Host applies compression.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "active_element_ids": { "type": "array", "items": { "type": "string" }, "description": "Currently focused architecture element IDs" },
                    "session_element_ids": { "type": "array", "items": { "type": "string" }, "description": "Element IDs mentioned in the session context" },
                    "depth": { "type": "integer", "description": "Topology hops from active focus (default 2)", "minimum": 1, "maximum": 4 }
                },
                "required": ["active_element_ids", "session_element_ids"]
            }
        }),
        json!({
            "name": "sruja_get_drift_state",
            "title": "Sruja Drift State Injector",
            "description": "Compact structured drift payload (drift_state/v1) for host context injection—prefer over pasting full drift reports.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_architecture_context",
            "title": "Sruja Architecture Context",
            "description": "Export high-level architecture context and project rules. Provide a file or element_id to get a localized, task-scoped context map.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "run_id": { "type": "string", "description": "Optional run ID for tracing (defaults to auto-generated)" },
                    "file": { "type": "string", "description": "Optional file focus for task-scoped context (relative to repo root)" },
                    "element_id": { "type": "string", "description": "Optional architecture element ID focus (e.g. MySystem.Api)" },
                    "intent": { "type": "string", "description": "Optional intent hint (add-feature, refactor, fix-bug)" }
                }
            }
        }),
        json!({
            "name": "sruja_explain_discovery",
            "title": "Sruja Discovery Explanation",
            "description": "Explain what Sruja discovered in the repo, why it inferred that shape, and what to review next.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "format": { "type": "string", "description": "Output format: text (default) or json" }
                }
            }
        }),
        json!({
            "name": "sruja_check_drift",
            "title": "Sruja Drift Check",
            "description": "Detect architectural drift in the codebase (returns JSON with violations and suggestions).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "architecture": { "type": "string", "description": "Optional path to a .sruja architecture file" }
                }
            }
        }),
        json!({
            "name": "sruja_evaluate_mutation",
            "title": "Sruja Evaluate Mutation",
            "description": "Evaluate declared fitness functions on a .sruja file to check architectural and behavioral scores.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "architecture": { "type": "string", "description": "Path to the .sruja architecture file (defaults to repo.sruja)" }
                }
            }
        }),
        json!({
            "name": "sruja_propose_topology_change",
            "title": "Sruja Propose Topology Change",
            "description": "Propose an architectural topology mutation and run impact analysis and drift checks before committing.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "description": { "type": "string", "description": "Description of the proposed change" },
                    "add_elements": { "type": "array", "items": { "type": "string" }, "description": "List of elements in format 'id:kind:label[:tech]'" },
                    "add_relationships": { "type": "array", "items": { "type": "string" }, "description": "List of relationships in format 'source->target[:label]'" }
                },
                "required": ["description"]
            }
        }),
        json!({
            "name": "sruja_commit_evolution",
            "title": "Sruja Commit Evolution",
            "description": "Commit an evolutionary mutation log record into the history log.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "id": { "type": "string", "description": "Fitness ID of the evaluated function" },
                    "target": { "type": "string", "description": "Target criteria of the function" },
                    "result": { "type": "string", "description": "Evaluation result (PASS/FAIL/ERROR)" },
                    "detail": { "type": "string", "description": "Detailed log or command output" }
                },
                "required": ["id", "target", "result"]
            }
        }),
        json!({
            "name": "sruja_add_element",
            "title": "Sruja Add Element",
            "description": "Add a new element (system, container, component, database, person) to the architecture.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "id": { "type": "string", "description": "Unique ID for the element (e.g. MySystem.Api)" },
                    "kind": { "type": "string", "description": "Kind: system, container, component, database, person" },
                    "title": { "type": "string", "description": "Human-readable title" },
                    "description": { "type": "string", "description": "Description of the element" },
                    "technology": { "type": "string", "description": "Technology used (for containers/components)" }
                },
                "required": ["id", "kind", "title"]
            }
        }),
        json!({
            "name": "sruja_add_relationship",
            "title": "Sruja Add Relationship",
            "description": "Add a new relationship between two elements.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "source": { "type": "string", "description": "Source element ID" },
                    "target": { "type": "string", "description": "Target element ID" },
                    "label": { "type": "string", "description": "Relationship label (e.g. HTTPS, SQL)" },
                    "technology": { "type": "string", "description": "Technology used (optional)" }
                },
                "required": ["source", "target"]
            }
        }),
        json!({
            "name": "sruja_get_system_context",
            "title": "Sruja System Context",
            "description": "Get the full multi-repo system architecture from the composed system.index.json. Returns all systems, containers, components, databases, their relationships, and cross-repo conflicts.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Start path to search for system.index.json (walks up to find it, defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_list_elements",
            "title": "Sruja List Elements",
            "description": "List architectural elements from the composed system index, filtered by kind (system, container, component, database, queue, person). Returns elements across all federated repos with their canonical IDs and lineage.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Start path to search for system.index.json (defaults to .)" },
                    "kind": { "type": "string", "description": "Element kind to filter by: system, container, component, database, queue, person. If omitted, returns all elements." }
                }
            }
        }),
        json!({
            "name": "sruja_get_neighbors",
            "title": "Sruja Get Neighbors",
            "description": "Get immediate upstream and downstream neighbors of a component.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "id": { "type": "string", "description": "Component ID" },
                    "depth": { "type": "integer", "description": "Search depth (default: 1)", "minimum": 1 }
                },
                "required": ["id"]
            }
        }),
        json!({
            "name": "sruja_find_path",
            "title": "Sruja Find Path",
            "description": "Find the path between two components in the architecture graph.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "source": { "type": "string", "description": "Source component ID" },
                    "target": { "type": "string", "description": "Target component ID" }
                },
                "required": ["source", "target"]
            }
        }),
        json!({
            "name": "sruja_get_entrypoints",
            "title": "Sruja Get Entrypoints",
            "description": "List all entrypoints (External APIs, Systems, or components with no incoming edges) in the codebase.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_data_stores",
            "title": "Sruja Get Data Stores",
            "description": "List all data stores (Databases, Queues) discovered in the architecture.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_architecture_summary",
            "title": "Sruja Architecture Summary",
            "description": "Get a compact, high-level summary of how the architecture works (layers, boundaries, and key flows).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_hydrated_context",
            "title": "Sruja Hydrated Context",
            "description": "Get architectural context for a component hydrated with its actual source code and immediate neighbors. Ideal for AI code reviews.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "id": { "type": "string", "description": "Component ID to hydrate" },
                    "max_tokens": { "type": "integer", "description": "Maximum tokens for the hydrated context (default: 20000)" },
                    "enrich": { "type": "boolean", "description": "If true, add optional enrichment (cmd/openai) grounded in the hydrated context. Default: false." },
                    "enrich_provider": { "type": "string", "description": "Enrichment provider: cmd|openai. Default: cmd." },
                    "enrich_cmd": { "type": "string", "description": "External enrichment command (stdin JSON -> stdout markdown)." },
                    "enrich_model": { "type": "string", "description": "Model name for provider=openai (default: gpt-4o-mini)." },
                    "enrich_base_url": { "type": "string", "description": "Base URL for provider=openai (default: https://api.openai.com/v1)." },
                    "enrich_timeout_ms": { "type": "integer", "description": "Timeout for enrichment (ms, default: 15000)." },
                    "enrich_max_bytes": { "type": "integer", "description": "Max bytes to read from enrichment stdout (default: 20000)." }
                },
                "required": ["id"]
            }
        }),
        json!({
            "name": "sruja_validate_change",
            "title": "Sruja Validate Change",
            "description": "Validate architectural impact of a set of changed files. Runs fast lint and drift checks on the impacted area. Ideal for self-validation before committing.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "files": { "type": "array", "items": { "type": "string" }, "description": "List of changed file paths" }
                },
                "required": ["files"]
            }
        }),
        json!({
            "name": "sruja_get_task_context",
            "title": "Sruja Task Context",
            "description": "Get high-fidelity architectural context for a specific task. Supports selection by element ID, file path, git diff (base/head refs), or search query. Returns focus elements, neighbors, impact analysis, hydrated source code, and a grounding_trace that explains how the focus and evidence were selected.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "element_id": { "type": "string", "description": "Specific architectural element ID (e.g. MySystem.Api)" },
                    "file": { "type": "string", "description": "File path to resolve architectural focus from" },
                    "query": { "type": "string", "description": "Search query for semantic architectural lookup" },
                    "base_ref": { "type": "string", "description": "Git base ref for diff-based context (baseline)" },
                    "head_ref": { "type": "string", "description": "Git head ref for diff-based context (current changes)" },
                    "depth": { "type": "integer", "description": "Depth of neighbor expansion (default: 1, max: 4)", "minimum": 1, "maximum": 4 },
                    "max_tokens": { "type": "integer", "description": "Maximum tokens for hydrated source code (default: 10000)" },
                    "enrich": { "type": "boolean", "description": "If true, add optional enrichment (cmd/openai) grounded in the task context. Default: false." },
                    "enrich_provider": { "type": "string", "description": "Enrichment provider: cmd|openai. Default: cmd." },
                    "enrich_cmd": { "type": "string", "description": "External enrichment command (stdin JSON -> stdout markdown)." },
                    "enrich_model": { "type": "string", "description": "Model name for provider=openai (default: gpt-4o-mini)." },
                    "enrich_base_url": { "type": "string", "description": "Base URL for provider=openai (default: https://api.openai.com/v1)." },
                    "enrich_timeout_ms": { "type": "integer", "description": "Timeout for enrichment (ms, default: 15000)." },
                    "enrich_max_bytes": { "type": "integer", "description": "Max bytes to read from enrichment stdout (default: 20000)." },
                    "cache_friendly": { "type": "boolean", "description": "If true, return invariant/tools/volatile JSON for prompt-cache-friendly payloads (default: false)." }
                }
            }
        }),
        json!({
            "name": "sruja_semantic_search",
            "title": "Sruja Semantic Search",
            "description": "Embedding-only similarity search over architecture elements (requires a built vector index). Prefer sruja_hybrid_query when you are unsure whether the answer is purely semantic or needs graph structure.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "query": { "type": "string", "description": "Search query (e.g. 'payment processing', 'database access')" },
                    "top_k": { "type": "integer", "description": "Number of results to return (default: 5)" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "sruja_query_graph",
            "title": "Sruja Query Graph",
            "description": "Natural-language Q&A over the architecture graph with scan-backed facts and optional enrich narrative. Prefer sruja_hybrid_query as the default entry point unless you specifically need this graph-query path or enrich tuning.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "query": { "type": "string", "description": "The natural language query (e.g. 'what connects auth to database?')" },
                    "enrich": { "type": "boolean", "description": "Add LLM narrative grounded in the matched subgraph context. Default: false." },
                    "enrich_provider": { "type": "string", "description": "Enrichment provider: cmd|openai. Default: cmd." },
                    "enrich_cmd": { "type": "string", "description": "External enrichment command (stdin JSON -> stdout markdown)." },
                    "enrich_model": { "type": "string", "description": "Model name for provider=openai (default: gpt-4o-mini)." },
                    "enrich_base_url": { "type": "string", "description": "Base URL for provider=openai (default: https://api.openai.com/v1)." },
                    "enrich_timeout_ms": { "type": "integer", "description": "Timeout for enrichment (ms, default: 15000)." },
                    "enrich_max_bytes": { "type": "integer", "description": "Max bytes to read from enrichment stdout (default: 20000)." }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "sruja_explain_element",
            "title": "Sruja Explain Element",
            "description": "Deep-dive on an architectural element, its centrality, neighbors, and extracted comments with optional LLM narrative.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "id": { "type": "string", "description": "Unique ID of the architectural element (e.g. MySystem.Api)" },
                    "enrich": { "type": "boolean", "description": "Add LLM narrative grounded in the element context. Default: false." },
                    "enrich_provider": { "type": "string", "description": "Enrichment provider: cmd|openai. Default: cmd." },
                    "enrich_cmd": { "type": "string", "description": "External enrichment command (stdin JSON -> stdout markdown)." },
                    "enrich_model": { "type": "string", "description": "Model name for provider=openai (default: gpt-4o-mini)." },
                    "enrich_base_url": { "type": "string", "description": "Base URL for provider=openai (default: https://api.openai.com/v1)." },
                    "enrich_timeout_ms": { "type": "integer", "description": "Timeout for enrichment (ms, default: 15000)." },
                    "enrich_max_bytes": { "type": "integer", "description": "Max bytes to read from enrichment stdout (default: 20000)." }
                },
                "required": ["id"]
            }
        }),
        json!({
            "name": "sruja_get_context_score",
            "title": "Sruja Context Score",
            "description": "Get the context engineering score (0-100) and AI-readiness breakdown for the repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "format": { "type": "string", "description": "Output format: text (default) or json" }
                }
            }
        }),
        json!({
            "name": "sruja_get_context_events",
            "title": "Sruja Context Events",
            "description": "Read recent append-only context lineage events from .sruja/context_events.jsonl (intent_check, drift, proposal_merge, learn_run, and context_event/v2 decision traces). Newest first.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "limit": { "type": "integer", "description": "Max events to return (default: 50)" },
                    "kind": { "type": "string", "description": "Optional filter on event kind" },
                    "details_substring": { "type": "string", "description": "Optional substring filter on JSON details" },
                    "decision_id": { "type": "string", "description": "Optional filter: decision_id field or details contains this id" },
                    "trace_id": { "type": "string", "description": "Optional filter: trace_id field or details contains" },
                    "element_id": { "type": "string", "description": "Optional filter: elements array or details mentions this architecture id" },
                    "decision_lineage_only": { "type": "boolean", "description": "If true, only decision/workflow lineage kinds (decision_opened, context_retrieved, ...)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_decisions",
            "title": "Sruja Decision Records",
            "description": "List Decision Record files (.sruja/decisions/*.md) with YAML front matter (generalized ADRs).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_decision_trace",
            "title": "Sruja Decision Trace",
            "description": "Return context_events.jsonl rows for a decision_id (append-only lineage).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "decision_id": { "type": "string", "description": "Decision id e.g. DR-2026-001" },
                    "limit": { "type": "integer", "description": "Max events (default: 50)" }
                },
                "required": ["decision_id"]
            }
        }),
        json!({
            "name": "sruja_record_context_event",
            "title": "Sruja Record Context Event",
            "description": "Append one context_event/v1 or v2 JSON object to .sruja/context_events.jsonl (same contract as `sruja event append`).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "event": { "type": "object", "description": "Full ContextEventRecord JSON (schema_version, timestamp, kind, outcome, ...)" }
                },
                "required": ["event"]
            }
        }),
        json!({
            "name": "sruja_record_decision_event",
            "title": "Sruja Record Decision Event",
            "description": "Append a context_event/v2 row for decision/workflow lineage (kind, decision_id, summary, optional trace fields).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "kind": { "type": "string", "description": "Event kind e.g. context_retrieved, validation_passed" },
                    "decision_id": { "type": "string", "description": "Decision id (optional for some kinds)" },
                    "outcome": { "type": "string", "description": "ok | fail | warn (default: ok)" },
                    "summary": { "type": "string", "description": "Human-readable one-line summary" },
                    "trace_id": { "type": "string" },
                    "run_id": { "type": "string" },
                    "workflow_id": { "type": "string" },
                    "actor": { "type": "string", "description": "agent | human | ci | system" },
                    "source": { "type": "string", "description": "mcp | cli | ci | editor | external" },
                    "tool": { "type": "string" },
                    "elements": { "type": "array", "items": { "type": "string" } },
                    "evidence_refs": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["kind", "summary"]
            }
        }),
        json!({
            "name": "sruja_create_decision_record",
            "title": "Sruja Create Decision Record",
            "description": "Create a proposed Decision Record under .sruja/decisions/ and emit decision_opened event.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "title": { "type": "string" },
                    "record_type": { "type": "string", "description": "architecture | product | operational | security | agent | exception" },
                    "scope": { "type": "string", "description": "repo | workflow | system | organization (default: repo)" }
                },
                "required": ["title", "record_type"]
            }
        }),
        json!({
            "name": "sruja_link_decision_to_element",
            "title": "Sruja Link Decision To Element",
            "description": "Append an architecture element id to a Decision Record YAML front matter.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "decision_id": { "type": "string" },
                    "element_id": { "type": "string" }
                },
                "required": ["decision_id", "element_id"]
            }
        }),
        json!({
            "name": "sruja_get_learned_facts",
            "title": "Sruja Learned Facts",
            "description": "Read hypotheses from .sruja/learned_facts.jsonl (deterministic scan + drift vs reviewed architecture). Treat as candidates, not repo.sruja truth.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "limit": { "type": "integer", "description": "Max facts to return (default: 200)" },
                    "status": { "type": "string", "description": "Optional filter: observed | inferred | proposed | reviewed | rejected | stale" }
                }
            }
        }),
        json!({
            "name": "sruja_get_evidence_graph",
            "title": "Sruja Evidence Graph",
            "description": "Load .sruja/evidence_graph.json (scan-derived graph snapshot written by `sruja learn`).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_evidence_for_claim",
            "title": "Sruja Evidence For Claim",
            "description": "Resolve a learned fact by id and attach matching scan nodes from the evidence graph when ids align.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "claim_id": { "type": "string", "description": "Learned fact id (e.g. fact_a1b2c3d4e5f67890)" }
                },
                "required": ["claim_id"]
            }
        }),
        json!({
            "name": "sruja_record_learn_feedback",
            "title": "Sruja Record Learn Feedback",
            "description": "Append approve/reject for a learned fact id to .sruja/learn_feedback.jsonl so future `sruja learn` runs can skip rejected proposals.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "fact_id": { "type": "string", "description": "Learned fact id" },
                    "decision": { "type": "string", "description": "approve | reject" },
                    "reason": { "type": "string", "description": "Optional human reason (especially for reject)" }
                },
                "required": ["fact_id", "decision"]
            }
        }),
        json!({
            "name": "sruja_get_agent_learnings",
            "title": "Sruja Agent Learnings",
            "description": "Return Agentic Memory entries relevant to an architecture element ID (same matching rules as focus memory_hits).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "element_id": { "type": "string", "description": "Architecture element ID (e.g. MySystem.Api)" }
                },
                "required": ["element_id"]
            }
        }),
        json!({
            "name": "sruja_get_focus_briefing",
            "title": "Sruja Focus Briefing",
            "description": "Get a task-scoped architectural briefing for a specific file or element. Includes blast radius, linked decisions, AI instructions, optional git-range temporal context, and agent memory hits.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "run_id": { "type": "string", "description": "Optional run ID for tracing (defaults to auto-generated)" },
                    "file": { "type": "string", "description": "File path to focus on" },
                    "element_id": { "type": "string", "description": "Element ID to focus on" },
                    "format": { "type": "string", "description": "Output format: text (default) or json" },
                    "base_ref": { "type": "string", "description": "Optional git base ref for temporal context (use with head_ref; head defaults to HEAD if omitted)" },
                    "head_ref": { "type": "string", "description": "Optional git head ref for temporal context (requires base_ref)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_operational_context",
            "title": "Sruja Operational Context",
            "description": "Get operational knowledge (gotchas, constraints, runbooks) and recent incidents for the repository or a specific element.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "element_id": { "type": "string", "description": "Optional element ID focus" }
                }
            }
        }),
        json!({
            "name": "sruja_propose_change",
            "title": "Sruja Propose Change",
            "description": "Propose an architectural change before writing code. Creates a validated proposal for human review.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path" },
                    "description": { "type": "string", "description": "What this change does and why" },
                    "add_elements": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "kind": { "type": "string" },
                                "label": { "type": "string" },
                                "technology": { "type": "string" }
                            },
                            "required": ["id", "kind", "label"]
                        }
                    },
                    "add_relationships": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "source": { "type": "string" },
                                "target": { "type": "string" },
                                "label": { "type": "string" }
                            },
                            "required": ["source", "target"]
                        }
                    },
                    "remove_elements": {
                        "type": "array",
                        "items": { "type": "string" }
                    }
                },
                "required": ["description"]
            }
        }),
        json!({
            "name": "sruja_critique",
            "title": "Sruja Adversarial Critique",
            "description": "Adversarial architectural review. Actively finds problems in proposed changes by cross-referencing policies, historical incidents, tribal knowledge, blast radius, and behavioral contracts.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path" },
                    "files": { "type": "array", "items": { "type": "string" }, "description": "Changed file paths to critique" },
                    "description": { "type": "string", "description": "What this change does (helps pattern matching)" },
                    "proposal_id": { "type": "string", "description": "Proposal ID if this is an approved proposal" },
                    "base_ref": { "type": "string", "description": "Git base ref for diff-based critique" },
                    "head_ref": { "type": "string", "description": "Git head ref for diff-based critique" }
                }
            }
        }),
        json!({
            "name": "sruja_get_state_machine",
            "title": "Sruja Get State Machine",
            "description": "Get the state machine definition for a component. Returns states, transitions, guards, and actions.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "element_id": { "type": "string", "description": "Component ID with state machine (e.g. MySystem.Api)" }
                },
                "required": ["element_id"]
            }
        }),
        json!({
            "name": "sruja_get_contract",
            "title": "Sruja Get Contract",
            "description": "Get the API contract (input/output spec) for a component. Ideal for generating client code or stubs.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "element_id": { "type": "string", "description": "Component ID with contract (e.g. MySystem.Api)" },
                    "contract_name": { "type": "string", "description": "Optional specific contract name if the component has multiple" }
                },
                "required": ["element_id"]
            }
        }),
        json!({
            "name": "sruja_preflight_check",
            "title": "Sruja Preflight Check",
            "description": "Before generating code, check what architectural constraints, policies, boundaries, and known risks apply to the target area. Call this BEFORE writing any code.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "target_files": { "type": "array", "items": { "type": "string" }, "description": "List of files you plan to modify" },
                    "intent": { "type": "string", "description": "What you plan to do (optional)" }
                },
                "required": ["target_files"]
            }
        }),
        json!({
            "name": "sruja_ai_scratchpad",
            "title": "Sruja AI Scratchpad",
            "description": "Read or write to the shared AI architectural scratchpad (legacy markdown format). Use sruja_record_learning for structured agentic memory.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "action": { "type": "string", "description": "Action: 'read' or 'append'" },
                    "content": { "type": "string", "description": "Markdown content to append (if action is append)." }
                },
                "required": ["action"]
            }
        }),
        json!({
            "name": "sruja_sandbox",
            "title": "Sruja Experiment Sandbox",
            "description": "Create, commit, or discard an isolated git worktree for safe architectural experimentation.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "action": { "type": "string", "description": "Action: 'create', 'commit', 'discard', 'list'" },
                    "name": { "type": "string", "description": "Name of the sandbox/experiment." }
                },
                "required": ["action"]
            }
        }),
        json!({
            "name": "sruja_evaluate_proposal",
            "title": "Sruja Evaluate Proposal",
            "description": "Evaluate the current state of the codebase against your architectural hypothesis. Calculates the current Context Score and runs an optional 'gate' command (e.g. 'make check').",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "gate_command": { "type": "string", "description": "Optional terminal command to run as a regression gate (e.g., 'make check' or 'cargo test')" }
                }
            }
        }),
        json!({
            "name": "sruja_record_learning",
            "title": "Sruja Record Learning",
            "description": "Record an architectural learning, failed hypothesis, or guardrail advice in the Agentic Memory. Used to prevent future agents from repeating mistakes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "context": { "type": "string", "description": "Context of the learning (e.g. 'Refactoring Auth')" },
                    "hypothesis": { "type": "string", "description": "What was being tried" },
                    "outcome": { "type": "string", "description": "Outcome: 'success' or 'failed'" },
                    "reason": { "type": "string", "description": "Why it failed (if applicable)" },
                    "guardrail_advice": { "type": "string", "description": "Explicit advice for future agents (e.g. 'Do not merge X into Y')" },
                    "affected_elements": { "type": "array", "items": { "type": "string" }, "description": "Architectural element IDs affected by this learning" },
                    "hitl_kind": { "type": "string", "description": "Optional: precedent | exception | correction | guardrail (human-in-the-loop classification)" }
                },
                "required": ["context", "hypothesis", "outcome", "guardrail_advice"]
            }
        }),
        json!({
            "name": "sruja_bm25_search",
            "title": "Sruja BM25 Search",
            "description": "Search ingested context documents (.sruja/context/) using BM25 keyword retrieval. Best for exact terms, acronyms, API identifiers, and error codes that embedding search may miss.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "query": { "type": "string", "description": "Search query (keywords, terms, identifiers)" },
                    "max_results": { "type": "integer", "description": "Maximum results to return (default: 5)" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "sruja_hybrid_query",
            "title": "Sruja Hybrid Query",
            "description": "Preferred default for most natural-language architecture questions: classifies query complexity and routes to graph-only, semantic-only, or hybrid retrieval. Use sruja_query_graph when you need that explicit pipeline; use sruja_semantic_search for embedding-only ranked nodes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "query": { "type": "string", "description": "Natural language query about the architecture" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "sruja_memory_clusters",
            "title": "Sruja Memory Clusters",
            "description": "View thematic clusters and tags from Zettelkasten-linked agentic memory. Shows how learnings relate to each other. Can filter by entry ID (for a specific cluster) or tag.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "entry_id": { "type": "string", "description": "Optional entry ID to show its cluster" },
                    "tag": { "type": "string", "description": "Optional tag to filter entries by" }
                }
            }
        }),
        json!({
            "name": "sruja_agent_run",
            "title": "Sruja Agent Run",
            "description": "Run Sruja's agent loop (observe→plan→optional apply→verify) and return structured JSON output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "run_id": { "type": "string", "description": "Optional run ID for tracing (defaults to auto-generated)" },
                    "goal": { "type": "string", "description": "Natural language goal for the agent" },
                    "file": { "type": "string", "description": "Optional file focus (exactly one of file/element_id/query)" },
                    "element_id": { "type": "string", "description": "Optional element id focus (exactly one of file/element_id/query)" },
                    "query": { "type": "string", "description": "Optional query focus (exactly one of file/element_id/query)" },
                    "mode": { "type": "string", "description": "plan|apply (default: plan)" },
                    "ai_mode": { "type": "string", "description": "standard|conservative|aggressive (default: standard)" },
                    "max_steps": { "type": "integer", "description": "Optional max steps override" },
                    "max_runtime_ms_per_step": { "type": "integer", "description": "Optional per-step timeout override (ms)" },
                    "enrich": { "type": "boolean", "description": "Optional enrichment grounded in facts (default: false)" },
                    "enrich_provider": { "type": "string", "description": "cmd|openai (or configured default)" },
                    "enrich_cmd": { "type": "string", "description": "External enrichment command (stdin JSON -> stdout markdown)" },
                    "enrich_model": { "type": "string", "description": "Model name for provider=openai" },
                    "enrich_base_url": { "type": "string", "description": "Base URL for provider=openai (OpenAI-compatible supported)" },
                    "enrich_timeout_ms": { "type": "integer", "description": "Enrichment timeout (ms)" },
                    "enrich_max_bytes": { "type": "integer", "description": "Max bytes to read from enrichment output" },
                    "continue_on_error": { "type": "boolean", "description": "If true, continue verification after errors" }
                },
                "required": ["goal"]
            }
        }),
    ]
}

async fn run_tool(
    name: &str,
    arguments: &Value,
    default_repo: &str,
    graph_cache: &Arc<Mutex<HashMap<String, sruja_scan::Graph>>>,
) -> Result<String, CliError> {
    let repo = arguments
        .get("path")
        .or_else(|| arguments.get("repo"))
        .and_then(|v| v.as_str())
        .unwrap_or(default_repo)
        .to_string();

    let run_id = arguments.get("run_id").and_then(|v| v.as_str());

    if mcp_readonly_enabled() && is_mutating_mcp_tool(name) {
        return Err(CliError::validation(format!(
            "MCP tool {name:?} is disabled when {} is set (read-only MCP profile)",
            ENV_MCP_READONLY
        )));
    }

    match name {
        "sruja_get_repomap" => {
            let repomap = super::discover::discover_repomap(&repo, 100, 5000)?;
            Ok(repomap)
        }
        "sruja_list_architecture_index" => {
            let max_tokens = arguments
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(2000)
                .max(200) as usize;
            let kinds: Option<Vec<String>> = arguments.get("kinds").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
            });

            let repo_path = Path::new(&repo);
            let (arch, warning) = load_architecture_program_best_effort(repo_path);
            let out = if let Some((source_file, program)) = arch {
                build_architecture_index_from_program(
                    &source_file,
                    &program,
                    kinds.as_deref(),
                    max_tokens,
                    warning.as_deref(),
                )?
            } else {
                let graph = get_or_scan_graph(graph_cache, &repo).await?;
                build_architecture_index_from_scan(
                    &graph,
                    kinds.as_deref(),
                    max_tokens,
                    warning.as_deref(),
                )?
            };
            Ok(out)
        }
        "sruja_get_topology" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing id"))?;
            let depth = arguments
                .get("depth")
                .and_then(|v| v.as_u64())
                .unwrap_or(1)
                .clamp(1, 4) as usize;
            let max_tokens = arguments
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(5000)
                .max(500) as usize;

            let repo_path = Path::new(&repo);
            let (arch, warning) = load_architecture_program_best_effort(repo_path);
            let out = if let Some((source_file, program)) = arch {
                build_topology_from_program(
                    &source_file,
                    &program,
                    id,
                    depth,
                    max_tokens,
                    warning.as_deref(),
                )?
            } else {
                let graph = get_or_scan_graph(graph_cache, &repo).await?;
                build_topology_from_scan(&graph, id, depth, max_tokens, warning.as_deref())?
            };
            Ok(out)
        }
        "sruja_get_elements" => {
            let ids = arguments
                .get("ids")
                .and_then(|v| v.as_array())
                .ok_or_else(|| CliError::validation("Missing ids"))?
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>();
            if ids.is_empty() {
                return Err(CliError::validation("ids must be non-empty"));
            }
            let max_tokens = arguments
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(8000)
                .max(500) as usize;

            let repo_path = Path::new(&repo);
            let (arch, warning) = load_architecture_program_best_effort(repo_path);
            let out = if let Some((source_file, program)) = arch {
                build_elements_from_program(
                    &source_file,
                    &program,
                    &ids,
                    max_tokens,
                    warning.as_deref(),
                )?
            } else {
                let graph = get_or_scan_graph(graph_cache, &repo).await?;
                build_elements_from_scan(&graph, &ids, max_tokens, warning.as_deref())?
            };
            Ok(out)
        }
        "sruja_get_diagnostic_full" => {
            let uri = arguments
                .get("uri")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing uri"))?;
            let content =
                crate::commands::diagnostic_vfs::read_vfs_diagnostic(Path::new(&repo), uri)?;
            Ok(content)
        }
        "sruja_suggest_context_prune" => {
            let active =
                crate::commands::context_prune::parse_id_list_arg(arguments, "active_element_ids")?;
            let session = crate::commands::context_prune::parse_id_list_arg(
                arguments,
                "session_element_ids",
            )?;
            let depth = arguments.get("depth").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            crate::commands::context_prune::suggest_context_prune_json(
                &graph, &active, &session, depth,
            )
        }
        "sruja_get_drift_state" => {
            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            crate::commands::drift_state::build_drift_state_json(&repo, &graph)
        }
        "sruja_get_architecture_context" => {
            let file = arguments
                .get("file")
                .and_then(|v| v.as_str())
                .map(String::from);
            let element_id = arguments
                .get("element_id")
                .and_then(|v| v.as_str())
                .map(String::from);
            let intent = arguments
                .get("intent")
                .and_then(|v| v.as_str())
                .map(String::from);
            let content = super::context::context_string(
                &repo,
                "markdown",
                super::context::ContextRequest {
                    run_id,
                    file: file.as_deref(),
                    element_id: element_id.as_deref(),
                    query: None,
                    base_ref: None,
                    head_ref: None,
                    intent: intent.as_deref(),
                    depth: 2,
                    max_tokens: 10000,
                    cache_friendly: false,
                },
            )
            .await?;
            Ok(content)
        }
        "sruja_get_architecture_summary" => {
            let content = super::context::context_string(
                &repo,
                "markdown",
                super::context::ContextRequest {
                    run_id,
                    file: None,
                    element_id: None,
                    query: None,
                    base_ref: None,
                    head_ref: None,
                    intent: None,
                    depth: 1,
                    max_tokens: 3000,
                    cache_friendly: false,
                },
            )
            .await?;
            Ok(content)
        }
        "sruja_get_neighbors" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing id"))?;
            let depth = arguments.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let radius = graph.blast_radius(id, depth);

            let mut out = format!("# Neighbors of {}\n\n", id);
            out.push_str("## Upstream (depend on this)\n");
            if radius.upstream.is_empty() {
                out.push_str("- None\n");
            } else {
                for n in radius.upstream {
                    out.push_str(&format!("- {} (depth: {})\n", n.id, n.depth));
                }
            }

            out.push_str("\n## Downstream (this depends on)\n");
            if radius.downstream.is_empty() {
                out.push_str("- None\n");
            } else {
                for n in radius.downstream {
                    out.push_str(&format!("- {} (depth: {})\n", n.id, n.depth));
                }
            }
            Ok(out)
        }
        "sruja_find_path" => {
            let source = arguments
                .get("source")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing source"))?;
            let target = arguments
                .get("target")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing target"))?;

            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            match graph.find_path(source, target) {
                Some(path) => Ok(format!(
                    "# Path from {} to {}\n\n{}",
                    source,
                    target,
                    path.join(" -> ")
                )),
                None => Ok(format!("No path found from {} to {}", source, target)),
            }
        }
        "sruja_agent_run" => {
            let goal = arguments
                .get("goal")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing goal"))?;
            let file = arguments.get("file").and_then(|v| v.as_str());
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());
            let query = arguments.get("query").and_then(|v| v.as_str());
            let mode = arguments
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("plan");
            let ai_mode = arguments
                .get("ai_mode")
                .and_then(|v| v.as_str())
                .unwrap_or("standard");
            let max_steps = arguments
                .get("max_steps")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);
            let max_runtime_ms_per_step = arguments
                .get("max_runtime_ms_per_step")
                .and_then(|v| v.as_u64());
            let enrich = arguments
                .get("enrich")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let enrich_provider = arguments.get("enrich_provider").and_then(|v| v.as_str());
            let enrich_cmd = arguments.get("enrich_cmd").and_then(|v| v.as_str());
            let enrich_model = arguments.get("enrich_model").and_then(|v| v.as_str());
            let enrich_base_url = arguments.get("enrich_base_url").and_then(|v| v.as_str());
            let enrich_timeout_ms = arguments
                .get("enrich_timeout_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(15_000);
            let enrich_max_bytes = arguments
                .get("enrich_max_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(20_000) as usize;
            let continue_on_error = arguments
                .get("continue_on_error")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let text = agent_run_to_string(AgentRunOptions {
                repo: &repo,
                goal,
                file,
                element_id,
                query,
                run_id,
                mode,
                ai_mode,
                format: "for-ai",
                max_steps,
                max_runtime_ms_per_step,
                enrich,
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                continue_on_error,
                trajectories: None,
            })
            .await?;
            Ok(text)
        }
        "sruja_get_entrypoints" => {
            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let mut entrypoints = Vec::new();

            let mut has_incoming = HashMap::new();
            for edge in &graph.edges {
                *has_incoming.entry(edge.target.as_str()).or_insert(0) += 1;
            }

            for node in &graph.nodes {
                let is_high_level = matches!(
                    node.kind.as_str(),
                    sruja_scan::NodeKind::SERVICE
                        | sruja_scan::NodeKind::EXTERNAL_API
                        | sruja_scan::NodeKind::SYSTEM
                );
                let no_incoming = has_incoming.get(node.id.as_str()).cloned().unwrap_or(0) == 0;

                if is_high_level || no_incoming {
                    entrypoints.push(format!("- {} ({})", node.id, node.kind));
                }
            }

            if entrypoints.is_empty() {
                Ok("No clear entrypoints discovered.".to_string())
            } else {
                entrypoints.sort();
                Ok(format!(
                    "# Architecture Entrypoints\n\n{}",
                    entrypoints.join("\n")
                ))
            }
        }
        "sruja_get_data_stores" => {
            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let mut stores = Vec::new();

            for node in &graph.nodes {
                if matches!(
                    node.kind.as_str(),
                    sruja_scan::NodeKind::DATABASE | sruja_scan::NodeKind::QUEUE
                ) {
                    let tech = node
                        .technology
                        .as_deref()
                        .map(|t| format!(" ({})", t))
                        .unwrap_or_default();
                    stores.push(format!("- {}: {}{}", node.id, node.kind, tech));
                }
            }

            if stores.is_empty() {
                Ok("No data stores (databases/queues) discovered.".to_string())
            } else {
                stores.sort();
                Ok(format!("# Discovered Data Stores\n\n{}", stores.join("\n")))
            }
        }
        "sruja_explain_discovery" => {
            let format = arguments
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("text");
            match format {
                "json" => super::discover::discover_explanation_json(&repo),
                "text" => super::discover::discover_explanation_string(&repo),
                _ => Err(CliError::validation(format!(
                    "Unknown format: {}. Use: text or json",
                    format
                ))),
            }
        }
        "sruja_evaluate_mutation" => {
            let architecture = arguments
                .get("architecture")
                .and_then(|v| v.as_str())
                .unwrap_or("repo.sruja");

            // Execute the evaluation logic
            crate::commands::evaluate(architecture).await?;
            Ok("Fitness functions evaluated successfully. Check output logs/terminal or evolution log.".to_string())
        }
        "sruja_propose_topology_change" => {
            let desc = arguments
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap();
            let add_elements: Vec<String> = arguments
                .get("add_elements")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let add_relationships: Vec<String> = arguments
                .get("add_relationships")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            // Run proposal create simulation and return feedback
            crate::commands::propose_create(
                &repo,
                desc,
                add_elements,
                add_relationships,
                Vec::new(),
            )
            .await?;
            Ok("Architecture topology change proposed successfully. Proposal ID and details generated.".to_string())
        }
        "sruja_commit_evolution" => {
            let id = arguments.get("id").and_then(|v| v.as_str()).unwrap();
            let target = arguments.get("target").and_then(|v| v.as_str()).unwrap();
            let result = arguments.get("result").and_then(|v| v.as_str()).unwrap();
            let detail = arguments
                .get("detail")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Write mutation record directly using our internal helper
            let sruja_dir = std::path::Path::new(&repo).join(".sruja");
            if !sruja_dir.exists() {
                std::fs::create_dir_all(&sruja_dir)?;
            }
            let log_path = sruja_dir.join("evolution.log");
            use std::fs::OpenOptions;
            use std::io::Write;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)?;

            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(
                file,
                "[{}] ID: {} | Target: {} | Result: {} | Output: {}",
                timestamp,
                id,
                target,
                result.to_uppercase(),
                detail
            )?;
            Ok("Evolution mutation successfully committed to history log.".to_string())
        }
        "sruja_check_drift" => {
            let architecture = arguments
                .get("architecture")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let content =
                super::scan::drift_json_string(&repo, architecture.as_deref(), false).await?;
            Ok(content)
        }
        "sruja_get_task_context" => {
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());
            let file = arguments.get("file").and_then(|v| v.as_str());
            let query = arguments.get("query").and_then(|v| v.as_str());
            let base_ref = arguments.get("base_ref").and_then(|v| v.as_str());
            let head_ref = arguments.get("head_ref").and_then(|v| v.as_str());
            let depth = arguments.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
            let max_tokens = arguments
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(10000) as usize;
            let enrich = arguments
                .get("enrich")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let enrich_provider = arguments.get("enrich_provider").and_then(|v| v.as_str());
            let enrich_cmd = arguments.get("enrich_cmd").and_then(|v| v.as_str());
            let enrich_model = arguments.get("enrich_model").and_then(|v| v.as_str());
            let enrich_base_url = arguments.get("enrich_base_url").and_then(|v| v.as_str());
            let enrich_timeout_ms = arguments
                .get("enrich_timeout_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(15000);
            let enrich_max_bytes = arguments
                .get("enrich_max_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(20000) as usize;
            let cache_friendly = arguments
                .get("cache_friendly")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let selectors = super::context::logic::TaskSelectors {
                element_id,
                file,
                query,
                base_ref,
                head_ref,
                depth: Some(depth),
            };

            let ctx =
                super::context::logic::build_task_context(&graph, &repo, selectors, max_tokens)?;
            if !enrich && enrich_cmd.is_none() {
                if cache_friendly {
                    let arch = super::context::logic::build_architecture_context(
                        &graph, &repo, None, None, depth, max_tokens,
                    )?;
                    let export =
                        super::context::logic::build_cache_friendly_task_export(&repo, &arch, ctx);
                    return Ok(serde_json::to_string_pretty(&export)?);
                }
                return Ok(serde_json::to_string_pretty(&ctx)?);
            }

            let wrapped = enrich_wrapper_json(
                Path::new(&repo),
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                "task_context",
                serde_json::to_value(&ctx)?,
            );
            Ok(serde_json::to_string_pretty(&wrapped)?)
        }
        "sruja_get_state_machine" => {
            let element_id = arguments
                .get("element_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing element_id"))?;
            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == element_id)
                .ok_or_else(|| CliError::validation(format!("Element {} not found", element_id)))?;

            if node.state_machines.is_empty() {
                return Ok(format!(
                    "No state machines found for element {}.",
                    element_id
                ));
            }

            Ok(serde_json::to_string_pretty(&node.state_machines)?)
        }
        "sruja_get_contract" => {
            let element_id = arguments
                .get("element_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing element_id"))?;
            let contract_name = arguments.get("contract_name").and_then(|v| v.as_str());
            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == element_id)
                .ok_or_else(|| CliError::validation(format!("Element {} not found", element_id)))?;

            if node.contracts.is_empty() {
                return Ok(format!("No contracts found for element {}.", element_id));
            }

            if let Some(name) = contract_name {
                let contract = node
                    .contracts
                    .iter()
                    .find(|c| c.name == name)
                    .ok_or_else(|| {
                        CliError::validation(format!(
                            "Contract {} not found on element {}",
                            name, element_id
                        ))
                    })?;
                Ok(serde_json::to_string_pretty(contract)?)
            } else {
                Ok(serde_json::to_string_pretty(&node.contracts)?)
            }
        }
        "sruja_preflight_check" => {
            let files = arguments
                .get("target_files")
                .and_then(|v| v.as_array())
                .ok_or_else(|| CliError::validation("Missing target_files array"))?;
            let file_list: Vec<String> = files
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            let intent_hint = arguments
                .get("intent")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let res = super::preflight::preflight(Path::new(&repo), file_list, intent_hint).await?;
            Ok(serde_json::to_string_pretty(&res)?)
        }
        "sruja_ai_scratchpad" => {
            let action = arguments
                .get("action")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing action"))?;

            let scratchpad_path = Path::new(&repo).join(".sruja").join("ai-scratchpad.md");

            match action {
                "read" => {
                    if scratchpad_path.exists() {
                        Ok(std::fs::read_to_string(scratchpad_path)?)
                    } else {
                        Ok("Scratchpad is empty. No learnings recorded yet.".to_string())
                    }
                }
                "append" => {
                    let content = arguments
                        .get("content")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| CliError::validation("Missing content for append"))?;

                    std::fs::create_dir_all(Path::new(&repo).join(".sruja"))?;
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(scratchpad_path)?;

                    use std::io::Write;
                    writeln!(file, "\n{}", content)?;
                    Ok("Successfully appended to AI scratchpad.".to_string())
                }
                _ => Err(CliError::validation(format!("Invalid action: {}", action))),
            }
        }
        "sruja_sandbox" => {
            let action = arguments
                .get("action")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing action"))?;

            let name = arguments.get("name").and_then(|v| v.as_str());
            let sruja_dir = Path::new(&repo).join(".sruja");
            let sandbox_dir = sruja_dir.join("sandboxes");
            std::fs::create_dir_all(&sandbox_dir)?;

            match action {
                "create" => {
                    let name =
                        name.ok_or_else(|| CliError::validation("Missing name for create"))?;
                    let target = sandbox_dir.join(name);
                    if target.exists() {
                        return Err(CliError::validation(format!(
                            "Sandbox '{}' already exists",
                            name
                        )));
                    }

                    let output = std::process::Command::new("git")
                        .args([
                            "worktree",
                            "add",
                            "-b",
                            &format!("sruja-sandbox/{}", name),
                            target.to_str().ok_or_else(|| {
                                CliError::validation("Target path is not valid UTF-8")
                            })?,
                        ])
                        .current_dir(&repo)
                        .output()?;

                    if !output.status.success() {
                        return Err(CliError::validation(format!(
                            "Git worktree failed: {}",
                            String::from_utf8_lossy(&output.stderr)
                        )));
                    }
                    Ok(format!("✅ Created isolated sandbox at {}. Run your tools and evaluations against this path.", target.display()))
                }
                "discard" => {
                    let name =
                        name.ok_or_else(|| CliError::validation("Missing name for discard"))?;
                    let target = sandbox_dir.join(name);

                    if !target.exists() {
                        return Err(CliError::validation(format!(
                            "Sandbox '{}' not found",
                            name
                        )));
                    }

                    std::process::Command::new("git")
                        .args([
                            "worktree",
                            "remove",
                            "--force",
                            target.to_str().ok_or_else(|| {
                                CliError::validation("Target path is not valid UTF-8")
                            })?,
                        ])
                        .current_dir(&repo)
                        .output()?;

                    std::process::Command::new("git")
                        .args(["branch", "-D", &format!("sruja-sandbox/{}", name)])
                        .current_dir(&repo)
                        .output()?;

                    Ok(format!("🗑️ Discarded sandbox '{}'.", name))
                }
                "commit" => {
                    let name =
                        name.ok_or_else(|| CliError::validation("Missing name for commit"))?;
                    let target = sandbox_dir.join(name);

                    if !target.exists() {
                        return Err(CliError::validation(format!(
                            "Sandbox '{}' not found",
                            name
                        )));
                    }

                    // Commit any pending changes in the worktree
                    std::process::Command::new("git")
                        .args(["add", "-A"])
                        .current_dir(&target)
                        .output()?;

                    std::process::Command::new("git")
                        .args(["commit", "-m", &format!("Sruja Sandbox: {}", name)])
                        .current_dir(&target)
                        .output()?;

                    Ok(format!("✅ Sandbox '{}' successfully committed to branch 'sruja-sandbox/{}'. A human can now merge this into the main branch.", name, name))
                }
                "list" => {
                    if let Ok(entries) = std::fs::read_dir(&sandbox_dir) {
                        let mut sandboxes = Vec::new();
                        for entry in entries.flatten() {
                            if entry.path().is_dir() {
                                sandboxes
                                    .push(format!("- {}", entry.file_name().to_string_lossy()));
                            }
                        }
                        if sandboxes.is_empty() {
                            Ok("No active sandboxes.".to_string())
                        } else {
                            Ok(format!("Active Sandboxes:\n{}", sandboxes.join("\n")))
                        }
                    } else {
                        Ok("No active sandboxes.".to_string())
                    }
                }
                _ => Err(CliError::validation(format!(
                    "Invalid sandbox action: {}",
                    action
                ))),
            }
        }
        "sruja_evaluate_proposal" => {
            let gate_cmd = arguments.get("gate_command").and_then(|v| v.as_str());

            let mut out = String::new();
            if let Some(cmd) = gate_cmd {
                out.push_str(&format!("Running gate: {}\n", cmd));

                let output = if cfg!(target_os = "windows") {
                    std::process::Command::new("cmd")
                        .args(["/C", cmd])
                        .current_dir(&repo)
                        .output()
                } else {
                    std::process::Command::new("sh")
                        .args(["-c", cmd])
                        .current_dir(&repo)
                        .output()
                };

                match output {
                    Ok(o) => {
                        let stdout = String::from_utf8_lossy(&o.stdout);
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        if o.status.success() {
                            out.push_str("✅ Gate Passed\n");
                        } else {
                            out.push_str("❌ Gate Failed\n\n");
                            out.push_str(&stdout);
                            out.push_str(&stderr);
                            out.push_str("\nRevert your changes or update your hypothesis in the Agentic Memory before trying again.");
                            return Ok(out);
                        }
                    }
                    Err(e) => {
                        out.push_str(&format!("❌ Gate Execution Failed: {}\n", e));
                        return Ok(out);
                    }
                }
            }

            // Calculate Context Score as the quality metric
            let kg = crate::graph_store::load_or_build_graph(Path::new(&repo))?;
            let scan_node_count = match sruja_scan::scan_repo(Path::new(&repo)) {
                Ok(g) => g.nodes.len(),
                Err(_) => kg.nodes.len(),
            };
            let score =
                sruja_graph::compute_context_score(&kg, scan_node_count, Path::new(&repo), 0);

            out.push_str(&format!("\n📈 Context Score: {}/100\n", score.score));
            out.push_str(&format!(
                "  - Architecture Coverage: {}/100\n",
                score.architecture_coverage.value
            ));
            out.push_str(&format!(
                "  - Decision Completeness: {}/100\n",
                score.decision_completeness.value
            ));
            out.push_str(&format!(
                "  - Evidence Freshness: {}/100\n",
                score.evidence_freshness.value
            ));
            out.push_str(&format!(
                "  - Relationship Density: {}/100\n",
                score.relationship_density.value
            ));

            if score.score == 100 {
                out.push_str("\n🎉 Perfect Score Achieved! Your hypothesis succeeded.");
            } else {
                out.push_str("\nReview the Agentic Memory or Context Map to find new optimization opportunities.");
            }

            Ok(out)
        }
        "sruja_record_learning" => {
            let context = arguments
                .get("context")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let hypothesis = arguments
                .get("hypothesis")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let outcome_str = arguments
                .get("outcome")
                .and_then(|v| v.as_str())
                .unwrap_or("success");
            let reason = arguments
                .get("reason")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let guardrail_advice = arguments
                .get("guardrail_advice")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let affected_elements = arguments
                .get("affected_elements")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let hitl_raw = arguments.get("hitl_kind").and_then(|v| v.as_str());
            let hitl_kind = if let Some(h) = hitl_raw {
                let v = h.trim().to_lowercase();
                match v.as_str() {
                    "precedent" | "exception" | "correction" | "guardrail" => Some(v),
                    "" => None,
                    _ => {
                        return Err(CliError::validation(format!(
                            "invalid hitl_kind: expected precedent|exception|correction|guardrail, got {h}"
                        )));
                    }
                }
            } else {
                None
            };

            let outcome = if outcome_str == "failed" {
                ExperimentOutcome::Failed
            } else {
                ExperimentOutcome::Success
            };

            let mut memory = AgenticMemory::load(Path::new(&repo))
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
            memory.add_learning(LearningEntry {
                id: String::new(),
                kind: Some(match outcome {
                    ExperimentOutcome::Success => sruja_agent::LearningKind::Playbook,
                    ExperimentOutcome::Failed => sruja_agent::LearningKind::Guardrail,
                }),
                timestamp: chrono::Utc::now(),
                run_id: arguments
                    .get("run_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                repo: Some(repo.clone()),
                selector: None,
                context,
                hypothesis,
                outcome,
                reason,
                guardrail_advice,
                affected_elements,
                evidence_refs: Vec::new(),
                confidence: None,
                tags: Vec::new(),
                hitl_kind,
                related_ids: Vec::new(),
            });
            memory
                .save(Path::new(&repo))
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

            Ok("Learning recorded in Agentic Memory successfully.".to_string())
        }
        "sruja_validate_change" => {
            let files = arguments
                .get("files")
                .and_then(|v| v.as_array())
                .ok_or_else(|| CliError::validation("Missing files array".to_string()))?;
            let file_list: Vec<String> = files
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let mut impacted_ids = std::collections::HashSet::new();
            for f in &file_list {
                for node in &graph.nodes {
                    if node
                        .path
                        .as_ref()
                        .is_some_and(|p| p.contains(f) || f.contains(p))
                    {
                        impacted_ids.insert(node.id.clone());
                    }
                }
            }

            let report = sruja_diff::detect_architectural_drift(&graph);
            let mut relevant_violations: Vec<_> = report
                .violations
                .into_iter()
                .filter(|v| {
                    v.location.as_ref().is_some_and(|l| {
                        file_list.iter().any(|f| l.contains(f)) || impacted_ids.contains(l)
                    })
                })
                .collect();

            let baseline_path = crate::utils::architecture_path::resolve_architecture_path(
                std::path::Path::new(&repo),
            );
            if let Some(p) = baseline_path {
                if let Ok(status) =
                    super::scan::drift::truth_status_from_baseline_compare(&graph, &p)
                {
                    if matches!(status, sruja_diff::TruthStatus::Drifted) {
                        // If we are drifted, run a full compare to get detailed delta violations
                        let content = std::fs::read_to_string(&p)?;
                        let parser = sruja_language::Parser::new(p.to_string_lossy().to_string());
                        if let Ok(program) = parser.parse(&content) {
                            let proposed = sruja_diff::program_to_graph(&program);
                            let diff = sruja_diff::compare_graphs(&graph, &proposed);
                            for v in diff.violations {
                                if !relevant_violations
                                    .iter()
                                    .any(|rv| rv.message == v.message && rv.location == v.location)
                                    && v.location.as_ref().is_some_and(|l| {
                                        file_list.iter().any(|f| l.contains(f))
                                            || impacted_ids.contains(l)
                                    })
                                {
                                    relevant_violations.push(v);
                                }
                            }
                        }
                    }
                }
            }

            if relevant_violations.is_empty() {
                Ok("✅ No architectural violations detected for the changed files.".to_string())
            } else {
                let mut out = "⚠️ Architectural violations detected:\n\n".to_string();
                for v in relevant_violations {
                    out.push_str(&format!(
                        "- [{:?}] {}{}: {}\n",
                        v.severity,
                        v.location.as_deref().unwrap_or("Unknown"),
                        v.rule_id
                            .as_ref()
                            .map(|r| format!(" ({})", r))
                            .unwrap_or_default(),
                        v.message
                    ));
                }
                out.push_str("\nPlease review these findings before committing.");
                Ok(out)
            }
        }
        "sruja_get_operational_context" => {
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());
            let graph = get_or_scan_graph(graph_cache, &repo).await?;

            let mut out = "# Operational Context\n\n".to_string();

            if let Some(id) = element_id {
                if let Some(node) = graph.nodes.iter().find(|n| n.id == id) {
                    out.push_str(&format!("## {}\n", id));
                    if !node.gotchas.is_empty() {
                        out.push_str("### Gotchas\n");
                        for g in &node.gotchas {
                            out.push_str(&format!("- {}\n", g));
                        }
                    }
                    if !node.operational_constraints.is_empty() {
                        out.push_str("### Constraints\n");
                        for c in &node.operational_constraints {
                            out.push_str(&format!("- {}\n", c));
                        }
                    }
                    if !node.runbooks.is_empty() {
                        out.push_str("### Runbooks\n");
                        for r in &node.runbooks {
                            out.push_str(&format!("- {}\n", r));
                        }
                    }
                } else {
                    return Err(CliError::validation(format!("Element not found: {}", id)));
                }
            } else {
                out.push_str("## Recent Incidents\n");
                if graph.incidents.is_empty() {
                    out.push_str("No incidents recorded.\n");
                } else {
                    for inc in &graph.incidents {
                        out.push_str(&format!("### {} - {}\n", inc.id, inc.title));
                        if let Some(s) = &inc.severity {
                            out.push_str(&format!("- **Severity**: {}\n", s));
                        }
                        if let Some(d) = &inc.date {
                            out.push_str(&format!("- **Date**: {}\n", d));
                        }
                        if !inc.affected.is_empty() {
                            out.push_str("- **Affected**: ");
                            out.push_str(&inc.affected.join(", "));
                            out.push('\n');
                        }
                        if let Some(c) = &inc.cause {
                            out.push_str(&format!("- **Cause**: {}\n", c));
                        }
                        if let Some(r) = &inc.resolution {
                            out.push_str(&format!("- **Resolution**: {}\n", r));
                        }
                        if let Some(l) = &inc.lesson {
                            out.push_str(&format!("- **Lesson**: {}\n", l));
                        }
                        out.push('\n');
                    }
                }

                out.push_str("\n## Tribal Knowledge (Gotchas & Constraints)\n");
                let mut found = false;
                for node in &graph.nodes {
                    if !node.gotchas.is_empty() || !node.operational_constraints.is_empty() {
                        found = true;
                        out.push_str(&format!("### {}\n", node.id));
                        for g in &node.gotchas {
                            out.push_str(&format!("- [Gotcha] {}\n", g));
                        }
                        for c in &node.operational_constraints {
                            out.push_str(&format!("- [Constraint] {}\n", c));
                        }
                    }
                }
                if !found {
                    out.push_str("No specific tribal knowledge recorded for elements.\n");
                }
            }

            Ok(out)
        }
        "sruja_propose_change" => {
            let description = arguments
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let mut add_elements = Vec::new();
            if let Some(elements) = arguments.get("add_elements").and_then(|v| v.as_array()) {
                for e in elements {
                    let id = e.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let kind = e.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                    let label = e.get("label").and_then(|v| v.as_str()).unwrap_or("");
                    let tech = e.get("technology").and_then(|v| v.as_str()).unwrap_or("");
                    add_elements.push(format!("{}:{}:{}:{}", id, kind, label, tech));
                }
            }

            let mut add_relationships = Vec::new();
            if let Some(rels) = arguments
                .get("add_relationships")
                .and_then(|v| v.as_array())
            {
                for r in rels {
                    let source = r.get("source").and_then(|v| v.as_str()).unwrap_or("");
                    let target = r.get("target").and_then(|v| v.as_str()).unwrap_or("");
                    let label = r.get("label").and_then(|v| v.as_str()).unwrap_or("");
                    add_relationships.push(format!("{}->{}:{}", source, target, label));
                }
            }

            let mut remove_elements = Vec::new();
            if let Some(elements) = arguments.get("remove_elements").and_then(|v| v.as_array()) {
                for e in elements {
                    if let Some(id) = e.as_str() {
                        remove_elements.push(id.to_string());
                    }
                }
            }

            super::propose_create(
                &repo,
                description,
                add_elements,
                add_relationships,
                remove_elements,
            )
            .await?;
            Ok("Proposal created successfully. Human review required via CLI.".to_string())
        }
        "sruja_add_element" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing id"))?;
            let kind = arguments
                .get("kind")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing kind"))?;
            let title = arguments
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing title"))?;
            let description = arguments.get("description").and_then(|v| v.as_str());
            let technology = arguments.get("technology").and_then(|v| v.as_str());

            add_element(&repo, id, kind, title, description, technology).await?;
            Ok(format!("Added {} {} to architecture", kind, id))
        }
        "sruja_add_relationship" => {
            let source = arguments
                .get("source")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing source"))?;
            let target = arguments
                .get("target")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing target"))?;
            let label = arguments.get("label").and_then(|v| v.as_str());
            let technology = arguments.get("technology").and_then(|v| v.as_str());

            add_relationship(&repo, source, target, label, technology).await?;
            Ok(format!("Added relationship {} -> {}", source, target))
        }
        "sruja_get_system_context" => {
            let start = std::path::Path::new(&repo);
            match super::federation::find_system_index(start) {
                Some(index_path) => {
                    let index = super::federation::load_system_index(&index_path)?;
                    let summary = format!(
                        "System index: {} repos, {} nodes, {} edges, {} conflicts\nSource: {}\n\n",
                        index.repos.len(),
                        index.nodes.len(),
                        index.edges.len(),
                        index.conflicts.len(),
                        index_path.display()
                    );
                    let json = serde_json::to_string_pretty(&index)
                        .map_err(|e| CliError::validation(e.to_string()))?;
                    Ok(format!("{}{}", summary, json))
                }
                None => Ok("No system.index.json found. Run `sruja compose` to create a multi-repo system index.".to_string()),
            }
        }
        "sruja_list_elements" => {
            let start = std::path::Path::new(&repo);
            match super::federation::find_system_index(start) {
                Some(index_path) => {
                    let index = super::federation::load_system_index(&index_path)?;
                    let filtered = match arguments.get("kind").and_then(|v| v.as_str()) {
                        Some(kind) => super::federation::filter_system_index_by_kind(&index, kind),
                        None => index,
                    };
                    let kind_label = arguments
                        .get("kind")
                        .and_then(|v| v.as_str())
                        .unwrap_or("all");
                    let mut out = format!(
                        "Found {} {} element(s) across {} repo(s)\n\n",
                        filtered.nodes.len(),
                        kind_label,
                        filtered.repos.len()
                    );
                    for node in &filtered.nodes {
                        out.push_str(&format!(
                            "- [{}] {} ({}){}\n  repo: {}\n",
                            node.kind,
                            node.label,
                            node.canonical_id,
                            node.technology
                                .as_ref()
                                .map(|t| format!(" [{}]", t))
                                .unwrap_or_default(),
                            node.repo_id
                        ));
                    }
                    if !filtered.edges.is_empty() {
                        out.push_str(&format!("\n{} relationship(s):\n", filtered.edges.len()));
                        for edge in &filtered.edges {
                            out.push_str(&format!(
                                "  {} -> {} {}\n",
                                edge.source,
                                edge.target,
                                edge.label.as_deref().unwrap_or("")
                            ));
                        }
                    }
                    if !filtered.conflicts.is_empty() {
                        out.push_str(&format!("\n⚠ {} conflict(s):\n", filtered.conflicts.len()));
                        for c in &filtered.conflicts {
                            out.push_str(&format!("  {}: {}\n", c.key, c.message));
                        }
                    }
                    Ok(out)
                }
                None => Ok("No system.index.json found. Run `sruja compose` to create a multi-repo system index.".to_string()),
            }
        }
        "sruja_get_hydrated_context" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing id"))?;
            let max_tokens = arguments
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(20000) as usize;
            let enrich = arguments
                .get("enrich")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let enrich_provider = arguments.get("enrich_provider").and_then(|v| v.as_str());
            let enrich_cmd = arguments.get("enrich_cmd").and_then(|v| v.as_str());
            let enrich_model = arguments.get("enrich_model").and_then(|v| v.as_str());
            let enrich_base_url = arguments.get("enrich_base_url").and_then(|v| v.as_str());
            let enrich_timeout_ms = arguments
                .get("enrich_timeout_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(15000);
            let enrich_max_bytes = arguments
                .get("enrich_max_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(20000) as usize;

            let hydrated = get_hydrated_context(&repo, id, max_tokens, graph_cache).await?;
            if !enrich && enrich_cmd.is_none() {
                return Ok(hydrated);
            }

            let wrapped = enrich_wrapper_json(
                Path::new(&repo),
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                "hydrated_context",
                json!({ "markdown": hydrated }),
            );
            Ok(serde_json::to_string_pretty(&wrapped)?)
        }
        "sruja_semantic_search" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing query"))?;
            let top_k = arguments.get("top_k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

            let vector_path = std::path::Path::new(&repo)
                .join(".sruja")
                .join("vectors.json");
            if !vector_path.exists() {
                return Ok("Semantic index not found. Please run `sruja index` first to generate embeddings.".to_string());
            }

            let index_json = tokio::fs::read_to_string(&vector_path).await?;
            let index: sruja_export::vector::VectorIndex = serde_json::from_str(&index_json)?;

            let mut searcher = sruja_export::vector::SemanticSearcher::new().map_err(|e| {
                CliError::Io(std::io::Error::other(format!(
                    "Failed to init searcher: {}",
                    e
                )))
            })?;

            let results = searcher.search(&index, query, top_k).map_err(|e| {
                CliError::Io(std::io::Error::other(format!("Search failed: {}", e)))
            })?;

            let mut out = format!("# Semantic Search Results for: \"{}\"\n\n", query);
            if results.is_empty() {
                out.push_str("No matching components found.\n");
            } else {
                for (id, score) in results {
                    let node = index.nodes.iter().find(|n| n.id == id);
                    let label = node.map(|n| n.label.as_str()).unwrap_or(&id);
                    let desc = node.map(|n| n.description.as_str()).unwrap_or("");
                    out.push_str(&format!(
                        "- **{}** (Score: {:.2})\n  ID: {}\n  Description: {}\n",
                        label, score, id, desc
                    ));
                }
            }
            Ok(out)
        }
        "sruja_query_graph" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing query"))?;
            let enrich = arguments
                .get("enrich")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let enrich_provider = arguments.get("enrich_provider").and_then(|v| v.as_str());
            let enrich_cmd = arguments.get("enrich_cmd").and_then(|v| v.as_str());
            let enrich_model = arguments.get("enrich_model").and_then(|v| v.as_str());
            let enrich_base_url = arguments.get("enrich_base_url").and_then(|v| v.as_str());
            let enrich_timeout_ms = arguments
                .get("enrich_timeout_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(15000);
            let enrich_max_bytes = arguments
                .get("enrich_max_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(20000) as usize;

            // Adaptive Hybrid Retrieval: single graph load for both classification and metadata
            let kg = crate::graph_store::load_or_build_graph(Path::new(&repo))?;
            let complexity = sruja_graph::classify_query(query, &kg);
            let vector_path = std::path::Path::new(&repo)
                .join(".sruja")
                .join("vectors.json");
            let has_semantic_index = vector_path.exists();
            let strategy = sruja_graph::select_strategy(complexity, has_semantic_index);

            let semantic_results = match strategy {
                sruja_graph::RetrievalStrategy::GraphOnly => Vec::new(),
                _ => {
                    if has_semantic_index {
                        let index_json = tokio::fs::read_to_string(&vector_path).await?;
                        let index: sruja_export::vector::VectorIndex =
                            serde_json::from_str(&index_json)?;
                        let mut searcher =
                            sruja_export::vector::SemanticSearcher::new().map_err(|e| {
                                CliError::Io(std::io::Error::other(format!(
                                    "Failed to init searcher: {}",
                                    e
                                )))
                            })?;
                        searcher.search(&index, query, 5).unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                }
            };

            let hybrid = sruja_graph::execute_hybrid(
                &kg,
                query,
                semantic_results
                    .iter()
                    .map(|(id, score)| sruja_graph::SemanticCandidate {
                        element_id: id.clone(),
                        score: *score,
                        label: None,
                    })
                    .collect(),
            );

            let mut matched_nodes = Vec::new();
            let mut relations = Vec::new();
            let mut seen_nodes = std::collections::HashSet::new();

            let push_kg_node =
                |node: &sruja_graph::ArchitectureNode, score: f32, matched: &mut Vec<Value>| {
                    matched.push(json!({
                        "id": node.id,
                        "label": node.label,
                        "kind": node.kind.as_str(),
                        "score": score,
                        "description": node.description.as_deref()
                    }));
                };

            for candidate in &hybrid.semantic_candidates {
                if let Some(node) = kg.nodes.get(&candidate.element_id) {
                    push_kg_node(node, candidate.score, &mut matched_nodes);
                    seen_nodes.insert(node.id.clone());
                }
            }

            if let Some(ref gr) = hybrid.graph_result {
                for ev in &gr.evidence {
                    if seen_nodes.insert(ev.reference.clone()) {
                        if let Some(node) = kg.nodes.get(&ev.reference) {
                            push_kg_node(node, 1.0, &mut matched_nodes);
                        }
                    }
                }
            }

            // 1-depth neighbor expansion via KnowledgeGraph edges
            let seed_ids: Vec<String> = seen_nodes.iter().cloned().collect();
            for seed_id in &seed_ids {
                for edge in kg.get_edges_from(seed_id) {
                    if seen_nodes.insert(edge.target.clone()) {
                        if let Some(node) = kg.nodes.get(&edge.target) {
                            push_kg_node(node, 0.0, &mut matched_nodes);
                        }
                    }
                }
                for edge in kg.get_edges_to(seed_id) {
                    if seen_nodes.insert(edge.source.clone()) {
                        if let Some(node) = kg.nodes.get(&edge.source) {
                            push_kg_node(node, 0.0, &mut matched_nodes);
                        }
                    }
                }
            }

            for edge in &kg.edges {
                if seen_nodes.contains(&edge.source) && seen_nodes.contains(&edge.target) {
                    relations.push(json!({
                        "source": edge.source,
                        "target": edge.target,
                        "kind": edge.kind.as_str()
                    }));
                }
            }

            let grounded = json!({
                "query": query,
                "complexity": format!("{:?}", complexity),
                "strategy": format!("{:?}", strategy),
                "matched_nodes": matched_nodes,
                "relationships": relations
            });

            if !enrich && enrich_cmd.is_none() {
                return Ok(serde_json::to_string_pretty(&grounded)?);
            }

            let wrapped = enrich_wrapper_json(
                Path::new(&repo),
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                "query_graph",
                grounded,
            );
            Ok(serde_json::to_string_pretty(&wrapped)?)
        }
        "sruja_explain_element" => {
            let element_id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing id"))?;
            let enrich = arguments
                .get("enrich")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let enrich_provider = arguments.get("enrich_provider").and_then(|v| v.as_str());
            let enrich_cmd = arguments.get("enrich_cmd").and_then(|v| v.as_str());
            let enrich_model = arguments.get("enrich_model").and_then(|v| v.as_str());
            let enrich_base_url = arguments.get("enrich_base_url").and_then(|v| v.as_str());
            let enrich_timeout_ms = arguments
                .get("enrich_timeout_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(15000);
            let enrich_max_bytes = arguments
                .get("enrich_max_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(20000) as usize;

            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == element_id)
                .ok_or_else(|| CliError::validation(format!("Element {} not found", element_id)))?;

            // Compute centrality (PageRank)
            let centrality = sruja_scan::graph::compute_all_centrality(&graph);
            let pr = centrality.get(&node.id).map(|s| s.pagerank).unwrap_or(0.0);

            // Immediate neighbors
            let radius = graph.blast_radius(&node.id, 1);
            let upstream: Vec<String> = radius.upstream.iter().map(|u| u.id.clone()).collect();
            let downstream: Vec<String> = radius.downstream.iter().map(|d| d.id.clone()).collect();

            // Notes / Explanatory comments from Comment discovery (explained by edges)
            let mut notes = Vec::new();
            for edge in &graph.edges {
                if edge.target == node.id && edge.kind.kind_str() == "explains" {
                    if let Some(src) = graph.nodes.iter().find(|n| n.id == edge.source) {
                        notes.push(json!({
                            "id": src.id,
                            "label": src.label,
                            "description": src.metadata.get("description").cloned()
                        }));
                    }
                }
            }

            // Compute community
            let raw_communities = sruja_scan::detect_communities(&graph);
            let community_infos = sruja_scan::summarize_communities(&graph, &raw_communities);
            let element_community = raw_communities.get(element_id).cloned();
            let community_detail = element_community.and_then(|cid| {
                community_infos.iter().find(|c| c.id == cid).map(|c| {
                    json!({
                        "id": c.id,
                        "suggested_label": c.suggested_label,
                        "cohesion": c.cohesion,
                        "member_count": c.member_count
                    })
                })
            });

            let grounded = json!({
                "element": {
                    "id": node.id,
                    "label": node.label,
                    "kind": node.kind.as_str(),
                    "pagerank": pr,
                    "description": node.metadata.get("description").cloned(),
                    "technology": node.technology,
                    "path": node.path,
                    "community": community_detail
                },
                "neighbors": {
                    "upstream": upstream,
                    "downstream": downstream
                },
                "notes": notes
            });

            if !enrich && enrich_cmd.is_none() {
                return Ok(serde_json::to_string_pretty(&grounded)?);
            }

            let wrapped = enrich_wrapper_json(
                Path::new(&repo),
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                "explain_element",
                grounded,
            );
            Ok(serde_json::to_string_pretty(&wrapped)?)
        }
        "sruja_get_context_score" => {
            let format = arguments
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("text");
            let kg = crate::graph_store::load_or_build_graph(Path::new(&repo))?;
            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let age_hours = crate::utils::context::context_age_hours(Path::new(&repo));
            let score = sruja_graph::compute_context_score(
                &kg,
                graph.nodes.len(),
                Path::new(&repo),
                age_hours,
            );

            if format == "json" {
                Ok(serde_json::to_string_pretty(&score)?)
            } else {
                Ok(format!(
                    "Context Score: {}/100\n\nBreakdown:\n- Coverage: {}%\n- Decisions: {}%\n- Freshness: {}%\n- Density: {}%\n- External: {}%",
                    score.score,
                    score.architecture_coverage.pct_u8(),
                    score.decision_completeness.pct_u8(),
                    score.evidence_freshness.pct_u8(),
                    score.relationship_density.pct_u8(),
                    score.external_context.pct_u8()
                ))
            }
        }
        "sruja_get_context_events" => {
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(50) as usize;
            let kind = arguments.get("kind").and_then(|v| v.as_str());
            let sub = arguments.get("details_substring").and_then(|v| v.as_str());
            let decision_id = arguments.get("decision_id").and_then(|v| v.as_str());
            let trace_id = arguments.get("trace_id").and_then(|v| v.as_str());
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());
            let decision_lineage_only = arguments
                .get("decision_lineage_only")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let events = crate::commands::context_events::read_context_events_query(
                Path::new(&repo),
                crate::commands::context_events::ContextEventQuery {
                    limit,
                    kind_filter: kind,
                    details_substring: sub,
                    decision_id,
                    trace_id,
                    run_id: None,
                    element_id,
                    decision_lineage_only,
                },
            )?;
            Ok(serde_json::to_string_pretty(&events)?)
        }
        "sruja_get_decisions" => {
            let items = crate::commands::list_decisions(Path::new(&repo))?;
            Ok(serde_json::to_string_pretty(&items)?)
        }
        "sruja_get_decision_trace" => {
            let decision_id = arguments
                .get("decision_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing decision_id".to_string()))?;
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(50) as usize;
            let events = crate::commands::context_events::read_context_events_query(
                Path::new(&repo),
                crate::commands::context_events::ContextEventQuery {
                    limit,
                    kind_filter: None,
                    details_substring: None,
                    decision_id: Some(decision_id),
                    trace_id: None,
                    run_id: None,
                    element_id: None,
                    decision_lineage_only: false,
                },
            )?;
            Ok(serde_json::to_string_pretty(&events)?)
        }
        "sruja_record_context_event" => {
            let ev = arguments
                .get("event")
                .ok_or_else(|| CliError::validation("missing event object".to_string()))?;
            let line = serde_json::to_string(ev)?;
            crate::commands::context_events::append_context_event_from_json_line(
                Path::new(&repo),
                &line,
            )
            .map_err(CliError::validation)?;
            Ok(r#"{"ok":true}"#.to_string())
        }
        "sruja_record_decision_event" => {
            let kind = arguments
                .get("kind")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing kind".to_string()))?;
            let summary = arguments
                .get("summary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing summary".to_string()))?;
            let decision_id = arguments
                .get("decision_id")
                .and_then(|v| v.as_str())
                .map(String::from);
            let outcome = arguments
                .get("outcome")
                .and_then(|v| v.as_str())
                .unwrap_or("ok")
                .to_string();
            let elements: Option<Vec<String>> = arguments
                .get("elements")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                });
            let evidence_refs: Option<Vec<String>> = arguments
                .get("evidence_refs")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                });
            let record = crate::commands::context_events::ContextEventRecord {
                schema_version: crate::commands::context_events::CONTEXT_EVENTS_SCHEMA_V2
                    .to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                kind: kind.to_string(),
                outcome,
                policy_fingerprint: crate::commands::context_events::policy_fingerprint(Path::new(
                    &repo,
                )),
                strict: None,
                details: serde_json::json!({}),
                trace_id: arguments
                    .get("trace_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                decision_id,
                run_id: arguments
                    .get("run_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                workflow_id: arguments
                    .get("workflow_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                actor: arguments
                    .get("actor")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                source: arguments
                    .get("source")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                tool: arguments
                    .get("tool")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                elements,
                subject_ids: None,
                evidence_refs,
                summary: Some(summary.to_string()),
            };
            crate::commands::context_events::validate_context_event_record(&record)
                .map_err(CliError::validation)?;
            crate::commands::context_events::append_context_event(Path::new(&repo), record);
            Ok(r#"{"ok":true}"#.to_string())
        }
        "sruja_create_decision_record" => {
            let title = arguments
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing title".to_string()))?;
            let record_type = arguments
                .get("record_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing record_type".to_string()))?;
            let scope = arguments.get("scope").and_then(|v| v.as_str());
            let id = crate::commands::create_decision_record(
                Path::new(&repo),
                title,
                record_type,
                scope,
                "sruja_create_decision_record",
                "agent",
                "mcp",
            )?;
            Ok(serde_json::json!({ "id": id }).to_string())
        }
        "sruja_link_decision_to_element" => {
            let decision_id = arguments
                .get("decision_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing decision_id".to_string()))?;
            let element_id = arguments
                .get("element_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing element_id".to_string()))?;
            crate::commands::decision::decision_link(&repo, decision_id, element_id).await?;
            Ok(r#"{"ok":true}"#.to_string())
        }
        "sruja_get_agent_learnings" => {
            let element_id = arguments
                .get("element_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing element_id"))?;
            let memory = AgenticMemory::load(Path::new(&repo))
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
            let relevant: Vec<&LearningEntry> = memory.find_relevant(element_id);
            Ok(serde_json::to_string_pretty(&relevant)?)
        }
        "sruja_get_focus_briefing" => {
            let file = arguments.get("file").and_then(|v| v.as_str());
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());

            let kg = crate::graph_store::load_or_build_graph(Path::new(&repo))?;
            let graph = get_or_scan_graph(graph_cache, &repo).await?;

            let target_id = super::focus::resolve_target(&kg, Path::new(&repo), file, element_id)?;
            let base_ref = arguments.get("base_ref").and_then(|v| v.as_str());
            let head_ref = arguments.get("head_ref").and_then(|v| v.as_str());
            let temporal = match (base_ref, head_ref) {
                (Some(b), Some(h)) => Some(super::focus::load_temporal_context(
                    Path::new(&repo),
                    b,
                    h,
                    &target_id,
                )?),
                (Some(b), None) => Some(super::focus::load_temporal_context(
                    Path::new(&repo),
                    b,
                    "HEAD",
                    &target_id,
                )?),
                (None, Some(_)) => {
                    return Err(CliError::validation(
                        "head_ref requires base_ref for focus temporal context".to_string(),
                    ));
                }
                (None, None) => None,
            };
            let mut briefing = super::focus::build_focus_briefing(
                &kg,
                &target_id,
                Path::new(&repo),
                graph.nodes.len(),
                temporal,
            );
            briefing.run_id = Some(
                run_id
                    .map(|s| s.to_string())
                    .unwrap_or_else(crate::utils::run_id::generate_run_id),
            );

            Ok(serde_json::to_string_pretty(&briefing)?)
        }
        "sruja_critique" => {
            let files: Vec<String> = arguments
                .get("files")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let description = arguments
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from);
            let proposal_id = arguments
                .get("proposal_id")
                .and_then(|v| v.as_str())
                .map(String::from);
            let base_ref = arguments
                .get("base_ref")
                .and_then(|v| v.as_str())
                .map(String::from);
            let head_ref = arguments
                .get("head_ref")
                .and_then(|v| v.as_str())
                .map(String::from);

            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let baseline_path =
                crate::utils::architecture_path::resolve_architecture_path(Path::new(&repo));
            let program = if let Some(path) = baseline_path {
                let content = std::fs::read_to_string(path).map_err(CliError::Io)?;
                let parser = sruja_language::Parser::new(&repo);
                parser.parse(&content).ok()
            } else {
                None
            };

            let engine = sruja_intent::CritiqueEngine::new(graph, program);
            let report = engine.critique(&sruja_intent::CritiqueRequest {
                changed_files: files,
                description,
                proposal_id,
                base_ref,
                head_ref,
            });

            Ok(sruja_intent::format_critique_json(&report))
        }
        "sruja_bm25_search" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing query"))?;
            let cfg_default = crate::integrations::load_repo_config(Path::new(&repo))
                .and_then(|c| c.context_engineering.bm25_max_results_mcp)
                .unwrap_or(5);
            let max_results = arguments
                .get("max_results")
                .and_then(|v| v.as_u64())
                .unwrap_or(cfg_default as u64) as usize;

            let index = sruja_graph::SparseIndex::build(Path::new(&repo));
            let hits = index.search(query, max_results);

            let out = json!({
                "query": query,
                "doc_count": index.doc_count(),
                "results": hits.iter().map(|h| json!({
                    "path": h.path,
                    "title": h.title,
                    "category": h.category,
                    "score": h.score,
                    "matched_terms": h.matched_terms,
                    "excerpt": h.excerpt,
                    "linked_elements": h.linked_elements,
                })).collect::<Vec<_>>(),
            });
            Ok(serde_json::to_string_pretty(&out)?)
        }
        "sruja_hybrid_query" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing query"))?;

            let kg = crate::graph_store::load_or_build_graph(Path::new(&repo))?;
            let complexity = sruja_graph::classify_query(query, &kg);
            let vector_path = std::path::Path::new(&repo)
                .join(".sruja")
                .join("vectors.json");
            let has_semantic = vector_path.exists();
            let strategy = sruja_graph::select_strategy(complexity, has_semantic);

            let semantic_candidates = match strategy {
                sruja_graph::RetrievalStrategy::GraphOnly => Vec::new(),
                _ => {
                    if has_semantic {
                        let index_json = tokio::fs::read_to_string(&vector_path).await?;
                        let index: sruja_export::vector::VectorIndex =
                            serde_json::from_str(&index_json)?;
                        let mut searcher =
                            sruja_export::vector::SemanticSearcher::new().map_err(|e| {
                                CliError::Io(std::io::Error::other(format!(
                                    "Failed to init searcher: {}",
                                    e
                                )))
                            })?;
                        searcher
                            .search(&index, query, 5)
                            .unwrap_or_default()
                            .into_iter()
                            .map(|(id, score)| sruja_graph::SemanticCandidate {
                                element_id: id,
                                score,
                                label: None,
                            })
                            .collect()
                    } else {
                        Vec::new()
                    }
                }
            };

            let result = sruja_graph::execute_hybrid(&kg, query, semantic_candidates);
            Ok(serde_json::to_string_pretty(&result)?)
        }
        "sruja_memory_clusters" => {
            let entry_id = arguments.get("entry_id").and_then(|v| v.as_str());
            let tag = arguments.get("tag").and_then(|v| v.as_str());

            let memory = AgenticMemory::load(Path::new(&repo))
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

            if let Some(eid) = entry_id {
                let cluster = memory.find_cluster(eid);
                return Ok(serde_json::to_string_pretty(&cluster)?);
            }

            if let Some(t) = tag {
                let entries = memory.find_by_tag(t);
                return Ok(serde_json::to_string_pretty(&entries)?);
            }

            let all_tags = memory.all_tags();
            let mut clusters = Vec::new();
            let mut visited = std::collections::HashSet::new();
            for entry in &memory.learnings {
                if visited.contains(&entry.id) {
                    continue;
                }
                let cluster = memory.find_cluster(&entry.id);
                let ids: Vec<String> = cluster.iter().map(|e| e.id.clone()).collect();
                for id in &ids {
                    visited.insert(id.clone());
                }
                clusters.push(json!({
                    "root_id": entry.id,
                    "size": cluster.len(),
                    "entry_ids": ids,
                }));
            }

            let out = json!({
                "total_entries": memory.learnings.len(),
                "total_tags": all_tags.len(),
                "tags": all_tags,
                "clusters": clusters,
            });
            Ok(serde_json::to_string_pretty(&out)?)
        }
        "sruja_get_learned_facts" => {
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(200) as usize;
            let status = arguments.get("status").and_then(|v| v.as_str());
            let facts =
                crate::commands::learn::read_learned_facts(Path::new(&repo), limit, status)?;
            Ok(serde_json::to_string_pretty(&facts)?)
        }
        "sruja_get_evidence_graph" => {
            let p = Path::new(&repo).join(".sruja").join("evidence_graph.json");
            if !p.exists() {
                return Err(CliError::validation(format!(
                    "No evidence graph at {}. Run `sruja learn -r {}` first.",
                    p.display(),
                    repo
                )));
            }
            let text = std::fs::read_to_string(&p).map_err(CliError::Io)?;
            Ok(text)
        }
        "sruja_get_evidence_for_claim" => {
            let claim_id = arguments
                .get("claim_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing claim_id".to_string()))?;
            let fact = crate::commands::learn::get_learned_fact_by_id(Path::new(&repo), claim_id)?
                .ok_or_else(|| CliError::validation(format!("Unknown claim_id {claim_id}")))?;
            let eg_path = Path::new(&repo).join(".sruja").join("evidence_graph.json");
            let related = if eg_path.exists() {
                let raw: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(&eg_path).map_err(CliError::Io)?)
                        .map_err(CliError::Json)?;
                let empty: Vec<serde_json::Value> = Vec::new();
                let nodes = raw
                    .pointer("/graph/nodes")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or(empty);
                let sid = fact.subject.as_str();
                let oid = fact.object.as_str();
                nodes
                    .into_iter()
                    .filter(|n| {
                        n.get("id")
                            .and_then(|v| v.as_str())
                            .is_some_and(|id| id == sid || id == oid)
                    })
                    .take(8)
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let out = json!({ "fact": fact, "related_scan_nodes": related });
            Ok(serde_json::to_string_pretty(&out)?)
        }
        "sruja_record_learn_feedback" => {
            let fact_id = arguments
                .get("fact_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing fact_id".to_string()))?;
            let decision = arguments
                .get("decision")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing decision".to_string()))?;
            let reason = arguments.get("reason").and_then(|v| v.as_str());
            crate::commands::learn::append_learn_feedback(
                Path::new(&repo),
                fact_id,
                decision,
                reason,
            )?;
            Ok(json!({ "ok": true, "fact_id": fact_id, "decision": decision }).to_string())
        }
        _ => Err(CliError::validation(format!("Unknown tool: {name}"))),
    }
}

// MCP tool payload maps many JSON fields; grouping would not reduce call-site noise meaningfully.
#[allow(clippy::too_many_arguments)]
fn enrich_wrapper_json(
    repo_path: &Path,
    enrich_provider: Option<&str>,
    enrich_cmd: Option<&str>,
    enrich_model: Option<&str>,
    enrich_base_url: Option<&str>,
    enrich_timeout_ms: u64,
    enrich_max_bytes: usize,
    kind: &str,
    grounded: Value,
) -> Value {
    let plan = resolve_enrichment_plan(
        repo_path,
        enrich_cmd,
        enrich_model,
        enrich_base_url,
        Some(enrich_timeout_ms),
        Some(enrich_max_bytes),
    );
    let provider = enrich_provider.unwrap_or(plan.provider.as_str());

    let input = json!({
        "schema_version": "mcp_enrichment_input/v1",
        "kind": kind,
        "grounded": grounded,
    });
    let stdin_payload = serde_json::to_vec(&input).unwrap_or_default();

    let enrichment = if provider == "cmd" {
        match plan.cmd.as_deref() {
            Some(cmd) => match run_cmd_enrichment(cmd, &stdin_payload, plan.limits) {
                Ok(md) => json!({
                    "status": "ok",
                    "provider": "external_cmd",
                    "model": Value::Null,
                    "error": Value::Null,
                    "narrative_markdown": md
                }),
                Err(e) => json!({
                    "status": "error",
                    "provider": "external_cmd",
                    "model": Value::Null,
                    "error": e,
                    "narrative_markdown": Value::Null
                }),
            },
            None => json!({
                "status": "skipped",
                "provider": "cmd",
                "model": Value::Null,
                "error": "No command configured. Provide enrich_cmd or set SRUJA_ENRICH_CMD / .sruja/config.toml [integrations].cmd.",
                "narrative_markdown": Value::Null
            }),
        }
    } else if provider == "openai" {
        let model = plan.model.as_deref().unwrap_or("gpt-4o-mini");
        let base_url = plan
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        match resolve_openai_auth() {
            Some(key) => {
                let user_prompt = format!(
                    r#"You are assisting an AI coding agent.

You MUST only use the JSON facts provided below. Do not invent modules, APIs, or file paths. If something is unknown, say "unknown".

Produce markdown with these sections:
- "Summary"
- "Risks / unknowns to verify" (bullets)
- "Suggested verification steps" (bullets)

JSON facts:
{}"#,
                    input
                );
                match run_openai_markdown(
                    "You are a careful repo assistant. Never fabricate.",
                    &user_prompt,
                    model,
                    base_url,
                    &key,
                ) {
                    Ok(md) => json!({
                        "status": "ok",
                        "provider": "openai",
                        "model": model,
                        "error": Value::Null,
                        "narrative_markdown": md
                    }),
                    Err(e) => json!({
                        "status": "error",
                        "provider": "openai",
                        "model": model,
                        "error": e,
                        "narrative_markdown": Value::Null
                    }),
                }
            }
            None => json!({
                "status": "skipped",
                "provider": "openai",
                "model": model,
                "error": "Missing API key (set OPENAI_API_KEY or SRUJA_ENRICH_API_KEY; SRUJA_LLM_API_KEY is deprecated).",
                "narrative_markdown": Value::Null
            }),
        }
    } else {
        json!({
            "status": "skipped",
            "provider": provider,
            "model": Value::Null,
            "error": "Unsupported provider. Use cmd (recommended) or openai.",
            "narrative_markdown": Value::Null
        })
    };

    json!({
        "schema_version": "mcp_enriched_output/v1",
        "grounded": input.get("grounded").cloned().unwrap_or(Value::Null),
        "enrichment": enrichment
    })
}

async fn get_or_scan_graph(
    cache: &Arc<Mutex<HashMap<String, sruja_scan::Graph>>>,
    repo_path: &str,
) -> Result<sruja_scan::Graph, CliError> {
    let mut cache = cache.lock().await;
    if let Some(g) = cache.get(repo_path) {
        return Ok(g.clone());
    }

    let g = super::scan_repo_cached(std::path::Path::new(repo_path))?;
    cache.insert(repo_path.to_string(), g.clone());
    Ok(g)
}

async fn add_element(
    repo: &str,
    id: &str,
    kind: &str,
    title: &str,
    description: Option<&str>,
    technology: Option<&str>,
) -> Result<(), CliError> {
    validate_ident(id, "id")?;
    validate_ident(kind, "kind")?;
    let target_file = find_best_sruja_file(repo)?;
    let mut content = tokio::fs::read_to_string(&target_file)
        .await
        .unwrap_or_default();

    if !content.ends_with('\n') && !content.is_empty() {
        content.push('\n');
    }

    content.push('\n');
    content.push_str(&format!(
        "{} = {} \"{}\"",
        id,
        kind,
        escape_dsl_string(title)?
    ));

    if description.is_some() || technology.is_some() {
        content.push_str(" {\n");
        if let Some(tech) = technology {
            content.push_str(&format!("  technology \"{}\"\n", escape_dsl_string(tech)?));
        }
        if let Some(desc) = description {
            content.push_str(&format!("  description \"{}\"\n", escape_dsl_string(desc)?));
        }
        content.push_str("}\n");
    } else {
        content.push('\n');
    }

    tokio::fs::write(&target_file, content).await?;
    Ok(())
}

async fn add_relationship(
    repo: &str,
    source: &str,
    target: &str,
    label: Option<&str>,
    technology: Option<&str>,
) -> Result<(), CliError> {
    validate_ident(source, "source")?;
    validate_ident(target, "target")?;
    let target_file = find_best_sruja_file(repo)?;
    let mut content = tokio::fs::read_to_string(&target_file)
        .await
        .unwrap_or_default();

    if !content.ends_with('\n') && !content.is_empty() {
        content.push('\n');
    }

    content.push('\n');
    let mut rel = format!("{} -> {}", source, target);
    if let Some(l) = label {
        rel.push_str(&format!(" \"{}\"", escape_dsl_string(l)?));
    }
    if let Some(t) = technology {
        rel.push_str(&format!(" [technology=\"{}\"]", escape_dsl_string(t)?));
    }
    rel.push('\n');
    content.push_str(&rel);

    tokio::fs::write(&target_file, content).await?;
    Ok(())
}

async fn get_hydrated_context(
    repo: &str,
    id: &str,
    max_tokens: usize,
    graph_cache: &Arc<Mutex<HashMap<String, sruja_scan::Graph>>>,
) -> Result<String, CliError> {
    let graph = get_or_scan_graph(graph_cache, repo).await?;
    let target_node = graph
        .nodes
        .iter()
        .find(|n| n.id == id)
        .ok_or_else(|| CliError::validation(format!("Component ID not found: {id}")))?;

    let blast = graph.blast_radius(id, 1);
    let repo_path = std::path::Path::new(repo);

    let mut out = format!("# Hydrated Architecture Context: {}\n\n", id);
    out.push_str(&format!("- **Title**: {}\n", target_node.label));
    out.push_str(&format!("- **Kind**: {}\n", target_node.kind));
    if let Some(tech) = &target_node.technology {
        out.push_str(&format!("- **Technology**: {}\n", tech));
    }

    // Neighbors summary
    out.push_str("\n## Relationships (Immediate Neighbors)\n");
    if blast.upstream.is_empty() && blast.downstream.is_empty() {
        out.push_str("- No direct relationships discovered.\n");
    } else {
        for n in &blast.upstream {
            out.push_str(&format!("- [Upstream] {} (depends on this)\n", n.id));
        }
        for n in &blast.downstream {
            out.push_str(&format!("- [Downstream] (this depends on) -> {}\n", n.id));
        }
    }

    out.push_str("\n## Source Implementation Hydration\n\n");

    let mut files_to_hydrate = Vec::new();

    // 1. Add target node sources
    for s in &target_node.sources {
        files_to_hydrate.push((target_node.id.clone(), s.path.clone()));
    }
    if target_node.sources.is_empty() {
        if let Some(p) = &target_node.path {
            files_to_hydrate.push((target_node.id.clone(), p.clone()));
        }
    }

    // 2. Add neighbor sources (metadata/interfaces only if possible, but for now just files)
    for neighbor in blast.upstream.iter().chain(blast.downstream.iter()) {
        if let Some(n) = graph.nodes.iter().find(|node| node.id == neighbor.id) {
            for s in &n.sources {
                files_to_hydrate.push((n.id.clone(), s.path.clone()));
            }
            if n.sources.is_empty() {
                if let Some(p) = &n.path {
                    files_to_hydrate.push((n.id.clone(), p.clone()));
                }
            }
        }
    }

    files_to_hydrate.sort_by(|a, b| a.1.cmp(&b.1));
    files_to_hydrate.dedup_by(|a, b| a.1 == b.1);

    let mut current_chars = 0;
    let max_chars = max_tokens * 4; // Estimating 4 chars per token

    for (node_id, rel_path) in files_to_hydrate {
        let full_path = repo_path.join(&rel_path);
        match tokio::fs::read_to_string(&full_path).await {
            Ok(content) => {
                let header = format!("### Component: {} (Path: {})\n\n", node_id, rel_path);
                if current_chars + header.len() + content.len() > max_chars {
                    out.push_str(&header);
                    out.push_str("... [File content truncated due to token budget] ...\n\n");
                    break;
                }
                out.push_str(&header);
                out.push_str("```\n");
                out.push_str(&content);
                out.push_str("\n```\n\n");
                current_chars += header.len() + content.len();
            }
            Err(e) => {
                out.push_str(&format!(
                    "### Component: {} (Path: {})\n\n*(Error reading file: {})*\n\n",
                    node_id, rel_path, e
                ));
            }
        }
    }

    Ok(out)
}

fn validate_ident(value: &str, field: &str) -> Result<(), CliError> {
    if value.is_empty() {
        return Err(CliError::validation(format!("Missing {}", field)));
    }
    if value.trim() != value {
        return Err(CliError::validation(format!(
            "Invalid {}: leading/trailing whitespace",
            field
        )));
    }
    if value
        .chars()
        .any(|c| c.is_whitespace() || c == '"' || c == '{' || c == '}' || c == '\\')
    {
        return Err(CliError::validation(format!(
            "Invalid {}: contains forbidden characters",
            field
        )));
    }
    Ok(())
}

fn escape_dsl_string(value: &str) -> Result<String, CliError> {
    if value.chars().any(|c| c == '\n' || c == '\r') {
        return Err(CliError::validation("Invalid string: contains newline"));
    }
    Ok(value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn find_best_sruja_file(repo: &str) -> Result<String, CliError> {
    let path = std::path::Path::new(repo);
    let repo_sruja = path.join("repo.sruja");
    if repo_sruja.exists() {
        return Ok(repo_sruja.to_string_lossy().to_string());
    }

    let files = crate::modules::file_operations::collect_sruja_files(path)?;
    if let Some(first) = files.first() {
        return Ok(first.clone());
    }

    Ok(repo_sruja.to_string_lossy().to_string())
}

fn load_architecture_program_best_effort(
    repo_path: &Path,
) -> (
    Option<(String, sruja_language::ast::Program)>,
    Option<String>,
) {
    let Some(arch_path) = crate::utils::architecture_path::resolve_architecture_path(repo_path)
    else {
        return (None, None);
    };

    let file = arch_path.to_string_lossy().to_string();
    let Ok(content) = std::fs::read_to_string(&arch_path) else {
        return (None, Some(format!("Cannot read architecture file: {file}")));
    };

    let parser = sruja_language::Parser::new(file.clone());
    match parser.parse(&content) {
        Ok(program) => (Some((file, program)), None),
        Err(diags) => (
            None,
            Some(format!(
                "Failed to parse architecture file: {} error(s)",
                diags.len()
            )),
        ),
    }
}

fn estimate_tokens(text: &str) -> usize {
    crate::commands::context::types::TokenBudget::estimate_tokens(text)
}

fn kind_matches_filter(kind: &str, filter: &[String]) -> bool {
    let k = kind.trim().to_lowercase();
    filter.iter().any(|f| f.trim().to_lowercase() == k)
}

fn trim_text(value: Option<&str>, max_chars: usize) -> Option<String> {
    let v = value?;
    let s = v.trim();
    if s.is_empty() {
        return None;
    }
    if s.chars().count() <= max_chars {
        Some(s.to_string())
    } else {
        Some(s.chars().take(max_chars).collect::<String>())
    }
}

#[derive(Debug, Clone)]
struct ResolvedId {
    id: String,
    ambiguous_matches: Vec<String>,
}

fn resolve_id_best_effort(id: &str, all_ids_sorted: &[String]) -> ResolvedId {
    let needle = id.trim();
    if needle.is_empty() {
        return ResolvedId {
            id: needle.to_string(),
            ambiguous_matches: Vec::new(),
        };
    }
    if all_ids_sorted.iter().any(|x| x == needle) {
        return ResolvedId {
            id: needle.to_string(),
            ambiguous_matches: Vec::new(),
        };
    }
    let suffix = format!(".{needle}");
    let matches: Vec<String> = all_ids_sorted
        .iter()
        .filter(|x| x.ends_with(&suffix))
        .cloned()
        .collect();
    match matches.len() {
        0 => ResolvedId {
            id: needle.to_string(),
            ambiguous_matches: Vec::new(),
        },
        1 => ResolvedId {
            id: matches[0].clone(),
            ambiguous_matches: Vec::new(),
        },
        _ => {
            let chosen = matches
                .iter()
                .min()
                .cloned()
                .unwrap_or_else(|| needle.to_string());
            ResolvedId {
                id: chosen,
                ambiguous_matches: matches,
            }
        }
    }
}

fn push_resolution_warnings(warnings: &mut Vec<String>, requested: &str, resolved: &ResolvedId) {
    if resolved.ambiguous_matches.len() > 1 {
        warnings.push(format!(
            "Ambiguous element id {requested:?}: suffix matched {:?}; using {:?}",
            resolved.ambiguous_matches, resolved.id
        ));
    }
}

fn bfs_radius(
    adjacency: &HashMap<String, Vec<String>>,
    target: &str,
    max_depth: usize,
) -> Vec<Value> {
    use std::collections::{HashSet, VecDeque};
    if max_depth == 0 {
        return Vec::new();
    }
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    let mut out: Vec<(String, usize)> = Vec::new();

    visited.insert(target.to_string());
    queue.push_back((target.to_string(), 0));

    while let Some((cur, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        let Some(nexts) = adjacency.get(&cur) else {
            continue;
        };
        for next in nexts {
            if visited.insert(next.clone()) {
                let nd = depth + 1;
                out.push((next.clone(), nd));
                queue.push_back((next.clone(), nd));
            }
        }
    }

    out.sort_by(|a, b| (a.1, a.0.as_str()).cmp(&(b.1, b.0.as_str())));
    out.into_iter()
        .map(|(id, depth)| json!({ "id": id, "depth": depth }))
        .collect()
}

fn enforce_max_tokens_on_json_array_fields(
    value: &mut Value,
    max_tokens: usize,
    shrink_fields: &[&str],
) -> Result<bool, CliError> {
    let mut truncated = false;
    loop {
        let text = serde_json::to_string_pretty(value)?;
        if estimate_tokens(&text) <= max_tokens {
            break;
        }
        let mut shrunk_any = false;
        for key in shrink_fields {
            if let Some(arr) = value.get_mut(*key).and_then(|v| v.as_array_mut()) {
                if !arr.is_empty() {
                    arr.pop();
                    shrunk_any = true;
                }
            }
        }
        if !shrunk_any {
            break;
        }
        truncated = true;
    }
    Ok(truncated)
}

fn sync_element_ids_from_array(response: &mut Value, elements_key: &str) {
    let mut ids = response
        .get(elements_key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    ids.sort();
    ids.dedup();
    response["element_ids"] = json!(ids);
}

fn push_token_budget_warning(response: &mut Value, max_tokens: usize) {
    let Ok(text) = serde_json::to_string_pretty(response) else {
        return;
    };
    if estimate_tokens(&text) <= max_tokens {
        return;
    }
    let msg = format!(
        "Response still exceeds max_tokens ({max_tokens}) after truncation; use a smaller max_tokens budget, fewer ids, or raise max_tokens."
    );
    let arr = match response.get_mut("warnings").and_then(|w| w.as_array_mut()) {
        Some(a) => a,
        None => {
            let mut existing = Vec::new();
            if let Some(v) = response.get("warnings") {
                if let Some(s) = v.as_str() {
                    existing.push(json!(s));
                } else if let Some(a) = v.as_array() {
                    existing.extend(a.iter().cloned());
                }
            }
            existing.push(json!(msg));
            response["warnings"] = Value::Array(existing);
            return;
        }
    };
    arr.push(json!(msg));
}

fn finalize_ladder_response(
    response: &mut Value,
    max_tokens: usize,
    shrink_fields: &[&str],
    sync_ids_from: Option<&str>,
) -> Result<(), CliError> {
    let truncated = enforce_max_tokens_on_json_array_fields(response, max_tokens, shrink_fields)?;
    if truncated {
        response["truncated"] = json!(true);
    }
    if let Some(key) = sync_ids_from {
        sync_element_ids_from_array(response, key);
    }
    push_token_budget_warning(response, max_tokens);
    set_estimated_tokens(response)
}

fn finalize_topology_response(response: &mut Value, max_tokens: usize) -> Result<(), CliError> {
    let truncated =
        enforce_max_tokens_on_json_array_fields(response, max_tokens, &["upstream", "downstream"])?;
    if truncated {
        response["truncated"] = json!(true);
    }
    response["element_ids"] = json!(collect_topology_element_ids(response));
    push_token_budget_warning(response, max_tokens);
    set_estimated_tokens(response)
}

fn attach_index_validation_log(
    response: &mut Value,
    source_file: &str,
    diagnostics: &[sruja_diagnostics::Diagnostic],
) -> Result<(), CliError> {
    if diagnostics.len() <= 8 {
        return Ok(());
    }
    let repo = Path::new(source_file)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let stem = Path::new(source_file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("architecture");
    let storage_name = format!("index-{stem}.txt");
    let text = crate::commands::diagnostic_vfs::diagnostics_to_text(diagnostics);
    let truncation = crate::commands::diagnostic_vfs::truncate_and_store_if_needed(
        repo,
        &storage_name,
        &text,
        crate::commands::diagnostic_vfs::INDEX_VALIDATION_LOG_TOKEN_BUDGET,
    )?;
    if let Some(validation) = response.get_mut("validation") {
        validation["diagnostic_log"] = serde_json::to_value(&truncation)?;
    }
    Ok(())
}

fn set_estimated_tokens(value: &mut Value) -> Result<(), CliError> {
    let text = serde_json::to_string_pretty(value)?;
    value["estimated_tokens"] = json!(estimate_tokens(&text));
    Ok(())
}

fn build_architecture_index_from_program(
    source_file: &str,
    program: &sruja_language::ast::Program,
    kind_filter: Option<&[String]>,
    max_tokens: usize,
    warning: Option<&str>,
) -> Result<String, CliError> {
    use sruja_diagnostics::codes;
    let (elements, relations) = sruja_language::collect_elements(program);

    let mut node_ids = elements.keys().cloned().collect::<Vec<_>>();
    node_ids.sort();
    let edges = relations
        .iter()
        .map(|r| (r.from.as_string(), r.to.as_string()))
        .collect::<Vec<_>>();
    let scc = sruja_graph::SccAnalyzer::new().analyze(&node_ids, &edges);
    let cyclic_nodes = scc
        .components
        .iter()
        .filter(|c| c.is_cyclic)
        .flat_map(|c| c.nodes.iter().cloned())
        .collect::<std::collections::HashSet<_>>();
    let cycle_samples = scc
        .components
        .iter()
        .filter(|c| c.is_cyclic)
        .take(3)
        .map(|c| json!({ "id": c.id, "size": c.nodes.len(), "nodes": c.nodes }))
        .collect::<Vec<_>>();

    let validator = sruja_engine::Validator::with_default_rules();
    let diagnostics = validator.validate_sync(program);
    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
        .count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Warning)
        .count();
    let policy_count = diagnostics
        .iter()
        .filter(|d| d.code == codes::CODE_POLICY_VIOLATION)
        .count();
    let policy_samples = diagnostics
        .iter()
        .filter(|d| d.code == codes::CODE_POLICY_VIOLATION)
        .take(3)
        .map(|d| {
            json!({
                "code": d.code,
                "message": trim_text(Some(d.message.as_str()), 180),
                "location": {
                    "file": d.location.file.clone(),
                    "line": d.location.line,
                    "column": d.location.column
                }
            })
        })
        .collect::<Vec<_>>();

    let mut entries = Vec::new();
    for (id, elem) in elements {
        let kind = elem.assignment.kind.to_string();
        if let Some(filter) = kind_filter {
            if !kind_matches_filter(&kind, filter) {
                continue;
            }
        }
        let title = elem
            .assignment
            .title
            .as_deref()
            .and_then(|t| trim_text(Some(t), 120))
            .unwrap_or_else(|| id.clone());
        let (description, technology) = elem
            .assignment
            .body
            .as_ref()
            .map(|b| (b.description.as_deref(), b.technology.as_deref()))
            .unwrap_or((None, None));
        entries.push(json!({
            "id": id.clone(),
            "kind": kind,
            "title": title,
            "technology": trim_text(technology, 80),
            "description": trim_text(description, 160),
            "in_cycle": cyclic_nodes.contains(&id)
        }));
    }
    entries.sort_by(|a, b| {
        a.get("id")
            .and_then(|v| v.as_str())
            .cmp(&b.get("id").and_then(|v| v.as_str()))
    });

    let element_ids = entries
        .iter()
        .filter_map(|e| e.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect::<Vec<_>>();

    let warnings = match warning {
        Some(w) => vec![w.to_string()],
        None => Vec::new(),
    };

    let mut response = json!({
        "schema_version": "architecture_index/v1",
        "source": { "kind": "dsl", "file": source_file },
        "element_ids": element_ids,
        "elements": entries,
        "validation": {
            "errors": error_count,
            "warnings": warning_count,
            "policy_violations": { "count": policy_count, "samples": policy_samples },
            "cycles": { "cyclic_sccs": scc.cyclic_sccs, "largest_scc_size": scc.largest_scc_size, "samples": cycle_samples }
        },
        "next_suggested_tool": "sruja_get_topology",
        "truncated": false,
        "estimated_tokens": 0,
        "warnings": warnings
    });

    attach_index_validation_log(&mut response, source_file, &diagnostics)?;
    finalize_ladder_response(
        &mut response,
        max_tokens,
        &["elements", "element_ids"],
        Some("elements"),
    )?;
    Ok(serde_json::to_string_pretty(&response)?)
}

fn build_architecture_index_from_scan(
    graph: &sruja_scan::Graph,
    kind_filter: Option<&[String]>,
    max_tokens: usize,
    warning: Option<&str>,
) -> Result<String, CliError> {
    let mut node_ids = graph.nodes.iter().map(|n| n.id.clone()).collect::<Vec<_>>();
    node_ids.sort();
    let edges = graph
        .edges
        .iter()
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect::<Vec<_>>();
    let scc = sruja_graph::SccAnalyzer::new().analyze(&node_ids, &edges);
    let cyclic_nodes = scc
        .components
        .iter()
        .filter(|c| c.is_cyclic)
        .flat_map(|c| c.nodes.iter().cloned())
        .collect::<std::collections::HashSet<_>>();

    let mut entries = Vec::new();
    let mut nodes = graph.nodes.clone();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    for n in nodes {
        let kind = n.kind.as_str().to_string();
        if let Some(filter) = kind_filter {
            if !kind_matches_filter(&kind, filter) {
                continue;
            }
        }
        entries.push(json!({
            "id": n.id,
            "kind": kind,
            "title": trim_text(Some(n.label.as_str()), 120),
            "technology": trim_text(n.technology.as_deref(), 80),
            "description": trim_text(n.path.as_deref(), 160),
            "in_cycle": cyclic_nodes.contains(&n.id)
        }));
    }

    let element_ids = entries
        .iter()
        .filter_map(|e| e.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect::<Vec<_>>();

    let warnings = match warning {
        Some(w) => vec![w.to_string()],
        None => Vec::new(),
    };

    let mut response = json!({
        "schema_version": "architecture_index/v1",
        "source": { "kind": "scan" },
        "element_ids": element_ids,
        "elements": entries,
        "validation": {
            "errors": 0,
            "warnings": 0,
            "policy_violations": { "count": 0, "samples": [] },
            "cycles": { "cyclic_sccs": scc.cyclic_sccs, "largest_scc_size": scc.largest_scc_size, "samples": [] }
        },
        "next_suggested_tool": "sruja_get_topology",
        "truncated": false,
        "estimated_tokens": 0,
        "warnings": warnings
    });

    finalize_ladder_response(
        &mut response,
        max_tokens,
        &["elements", "element_ids"],
        Some("elements"),
    )?;
    Ok(serde_json::to_string_pretty(&response)?)
}

fn build_topology_from_program(
    source_file: &str,
    program: &sruja_language::ast::Program,
    id: &str,
    depth: usize,
    max_tokens: usize,
    warning: Option<&str>,
) -> Result<String, CliError> {
    let (elements, relations) = sruja_language::collect_elements(program);
    let mut all_ids = elements.keys().cloned().collect::<Vec<_>>();
    all_ids.sort();
    let resolved = resolve_id_best_effort(id, &all_ids);
    let target = resolved.id.clone();

    let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
    let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
    for r in relations {
        let src = r.from.as_string();
        let tgt = r.to.as_string();
        outgoing.entry(src.clone()).or_default().push(tgt.clone());
        incoming.entry(tgt).or_default().push(src);
    }
    for v in outgoing.values_mut() {
        v.sort();
        v.dedup();
    }
    for v in incoming.values_mut() {
        v.sort();
        v.dedup();
    }

    let upstream = bfs_radius(&incoming, &target, depth);
    let downstream = bfs_radius(&outgoing, &target, depth);

    let mut warnings = match warning {
        Some(w) => vec![w.to_string()],
        None => Vec::new(),
    };
    push_resolution_warnings(&mut warnings, id, &resolved);

    let mut response = json!({
        "schema_version": "topology/v1",
        "source": { "kind": "dsl", "file": source_file },
        "target": target,
        "max_depth": depth,
        "upstream": upstream,
        "downstream": downstream,
        "element_ids": [],
        "next_suggested_tool": "sruja_get_elements",
        "truncated": false,
        "estimated_tokens": 0,
        "warnings": warnings
    });

    finalize_topology_response(&mut response, max_tokens)?;
    Ok(serde_json::to_string_pretty(&response)?)
}

fn build_topology_from_scan(
    graph: &sruja_scan::Graph,
    id: &str,
    depth: usize,
    max_tokens: usize,
    warning: Option<&str>,
) -> Result<String, CliError> {
    let mut all_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    all_ids.sort();
    let resolved = resolve_id_best_effort(id, &all_ids);
    let radius = graph.blast_radius(&resolved.id, depth);

    let mut warnings = match warning {
        Some(w) => vec![w.to_string()],
        None => Vec::new(),
    };
    push_resolution_warnings(&mut warnings, id, &resolved);

    let mut response = json!({
        "schema_version": "topology/v1",
        "source": { "kind": "scan" },
        "target": radius.target,
        "max_depth": radius.max_depth,
        "upstream": radius.upstream.iter().map(|n| json!({"id": n.id, "depth": n.depth})).collect::<Vec<_>>(),
        "downstream": radius.downstream.iter().map(|n| json!({"id": n.id, "depth": n.depth})).collect::<Vec<_>>(),
        "element_ids": [],
        "next_suggested_tool": "sruja_get_elements",
        "truncated": false,
        "estimated_tokens": 0,
        "warnings": warnings
    });

    finalize_topology_response(&mut response, max_tokens)?;
    Ok(serde_json::to_string_pretty(&response)?)
}

fn collect_topology_element_ids(topology: &Value) -> Vec<String> {
    let mut ids = Vec::new();
    if let Some(t) = topology.get("target").and_then(|v| v.as_str()) {
        ids.push(t.to_string());
    }
    for key in ["upstream", "downstream"] {
        if let Some(arr) = topology.get(key).and_then(|v| v.as_array()) {
            for n in arr {
                if let Some(id) = n.get("id").and_then(|v| v.as_str()) {
                    ids.push(id.to_string());
                }
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

fn build_elements_from_program(
    source_file: &str,
    program: &sruja_language::ast::Program,
    ids: &[String],
    max_tokens: usize,
    warning: Option<&str>,
) -> Result<String, CliError> {
    let (elements, _relations) = sruja_language::collect_elements(program);
    let mut all_ids = elements.keys().cloned().collect::<Vec<_>>();
    all_ids.sort();

    let mut out = Vec::new();
    let mut warnings = match warning {
        Some(w) => vec![w.to_string()],
        None => Vec::new(),
    };
    for req in ids {
        let resolved = resolve_id_best_effort(req, &all_ids);
        push_resolution_warnings(&mut warnings, req, &resolved);
        let Some(elem) = elements.get(&resolved.id) else {
            out.push(json!({ "id": resolved.id, "requested_id": req, "missing": true }));
            continue;
        };
        let title = elem
            .assignment
            .title
            .clone()
            .unwrap_or_else(|| resolved.id.clone());
        let (
            description,
            technology,
            owner,
            domain,
            criticality,
            gotchas,
            runbooks,
            sources,
            constraint_count,
            convention_count,
        ) = elem
            .assignment
            .body
            .as_ref()
            .map(|b| {
                let sources = b
                    .sources
                    .iter()
                    .take(5)
                    .map(|s| {
                        json!({
                            "kind": s.kind.as_str(),
                            "path": s.path,
                            "description": s.description
                        })
                    })
                    .collect::<Vec<_>>();
                (
                    b.description.clone(),
                    b.technology.clone(),
                    b.owner.clone(),
                    b.domain.clone(),
                    b.criticality.as_ref().map(|c| c.as_str().to_string()),
                    b.gotchas.iter().take(5).cloned().collect::<Vec<_>>(),
                    b.runbooks.iter().take(5).cloned().collect::<Vec<_>>(),
                    sources,
                    b.constraints.len(),
                    b.conventions.len(),
                )
            })
            .unwrap_or((
                None,
                None,
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                0,
                0,
            ));

        out.push(json!({
            "id": resolved.id,
            "kind": elem.assignment.kind.to_string(),
            "title": title,
            "description": description,
            "technology": technology,
            "tags": elem.assignment.tag_refs,
            "owner": owner,
            "domain": domain,
            "criticality": criticality,
            "gotchas": gotchas,
            "runbooks": runbooks,
            "constraint_count": constraint_count,
            "convention_count": convention_count,
            "sources": sources
        }));
    }

    out.sort_by(|a, b| {
        a.get("id")
            .and_then(|v| v.as_str())
            .cmp(&b.get("id").and_then(|v| v.as_str()))
    });

    let mut response = json!({
        "schema_version": "elements/v1",
        "source": { "kind": "dsl", "file": source_file },
        "requested_ids": ids,
        "elements": out,
        "element_ids": [],
        "next_suggested_tool": "sruja_get_task_context",
        "truncated": false,
        "estimated_tokens": 0,
        "warnings": warnings
    });

    finalize_ladder_response(
        &mut response,
        max_tokens,
        &["elements", "element_ids"],
        Some("elements"),
    )?;
    Ok(serde_json::to_string_pretty(&response)?)
}

fn build_elements_from_scan(
    graph: &sruja_scan::Graph,
    ids: &[String],
    max_tokens: usize,
    warning: Option<&str>,
) -> Result<String, CliError> {
    let mut by_id: HashMap<&str, &sruja_scan::Node> = HashMap::new();
    for n in &graph.nodes {
        by_id.insert(n.id.as_str(), n);
    }

    let mut all_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    all_ids.sort();

    let mut out = Vec::new();
    let mut warnings = match warning {
        Some(w) => vec![w.to_string()],
        None => Vec::new(),
    };
    for id in ids {
        let resolved = resolve_id_best_effort(id, &all_ids);
        push_resolution_warnings(&mut warnings, id, &resolved);
        if let Some(n) = by_id.get(resolved.id.as_str()) {
            out.push(json!({
                "id": n.id,
                "kind": n.kind.as_str(),
                "title": n.label,
                "description": n.path,
                "technology": n.technology,
                "tags": [],
                "owner": null,
                "domain": null,
                "criticality": null,
                "gotchas": [],
                "runbooks": [],
                "constraint_count": 0,
                "convention_count": 0,
                "sources": n.sources
            }));
        } else {
            out.push(json!({ "id": resolved.id, "requested_id": id, "missing": true }));
        }
    }

    let mut response = json!({
        "schema_version": "elements/v1",
        "source": { "kind": "scan" },
        "requested_ids": ids,
        "elements": out,
        "element_ids": [],
        "next_suggested_tool": "sruja_get_task_context",
        "truncated": false,
        "estimated_tokens": 0,
        "warnings": warnings
    });

    finalize_ladder_response(
        &mut response,
        max_tokens,
        &["elements", "element_ids"],
        Some("elements"),
    )?;
    Ok(serde_json::to_string_pretty(&response)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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

    #[test]
    fn mcp_resources_list_includes_invariant_uri() {
        let resources = crate::commands::mcp_resources::list_resources(".").expect("list");
        assert!(resources
            .iter()
            .any(|r| r.uri == "sruja://context/invariant.md"));
    }

    #[test]
    fn mcp_prompts_list_includes_mcp_guide() {
        let prompts = crate::commands::mcp_prompts::list_prompts();
        assert!(prompts.iter().any(|p| p.name == "sruja_mcp_guide"));
    }

    #[test]
    fn invariant_brief_fits_token_budget() {
        let ctx = crate::commands::context::types::ArchitectureContext {
            repo: "test".to_string(),
            summary: crate::commands::context::types::ContextSummary {
                total_modules: 10,
                total_services: 2,
                total_databases: 1,
                total_external_apis: 0,
            },
            layers: vec![],
            boundaries: vec![crate::commands::context::types::BoundaryRule {
                from: "ui".to_string(),
                to: "data".to_string(),
                allowed: false,
                reason: "use services".to_string(),
            }],
            forbidden_patterns: vec![],
            active_decisions: vec![],
            focus: None,
            system_context: None,
            max_tokens: 700,
        };
        let brief = crate::commands::context::format_invariant_brief(&ctx);
        assert!(brief.len() < 4_000, "brief should stay compact");
        assert!(brief.contains("sruja_list_architecture_index"));
    }

    #[test]
    fn mutating_mcp_tool_detection() {
        assert!(is_mutating_mcp_tool("sruja_record_learning"));
        assert!(is_mutating_mcp_tool("sruja_record_context_event"));
        assert!(is_mutating_mcp_tool("sruja_record_decision_event"));
        assert!(is_mutating_mcp_tool("sruja_create_decision_record"));
        assert!(is_mutating_mcp_tool("sruja_link_decision_to_element"));
        assert!(is_mutating_mcp_tool("sruja_sandbox"));
        assert!(is_mutating_mcp_tool("sruja_agent_run"));
        assert!(!is_mutating_mcp_tool("sruja_check_drift"));
        assert!(!is_mutating_mcp_tool("sruja_hybrid_query"));
        for ladder in [
            "sruja_list_architecture_index",
            "sruja_get_topology",
            "sruja_get_elements",
            "sruja_get_diagnostic_full",
            "sruja_suggest_context_prune",
            "sruja_get_drift_state",
        ] {
            assert!(
                !is_mutating_mcp_tool(ladder),
                "ladder tool {ladder} must be read-only"
            );
        }
    }

    #[test]
    fn mcp_readonly_list_excludes_all_mutating_tools() {
        let full = mcp_tools_for_list_with_readonly(false);
        let ro = mcp_tools_for_list_with_readonly(true);
        assert!(ro.len() < full.len());
        for t in &ro {
            let n = t.get("name").and_then(|x| x.as_str()).expect("name");
            assert!(
                !is_mutating_mcp_tool(n),
                "readonly list leaked mutating tool {n}"
            );
        }
        for m in MCP_MUTATING_TOOLS {
            assert!(!ro
                .iter()
                .any(|t| t.get("name").and_then(|n| n.as_str()) == Some(*m)));
        }
    }

    #[tokio::test]
    async fn mcp_tools_list_returns_sruja_tools() {
        let server = McpServer::new(".".to_string());
        let resp = server.handle_tools_list(json!(1));
        let tools = resp
            .pointer("/result/tools")
            .and_then(|v| v.as_array())
            .expect("tools list");

        let names: Vec<String> = tools
            .iter()
            .filter_map(|t| {
                t.get("name")
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        assert!(names.contains(&"sruja_get_repomap".to_string()));
        assert!(names.contains(&"sruja_get_architecture_context".to_string()));
        assert!(names.contains(&"sruja_explain_discovery".to_string()));
        assert!(names.contains(&"sruja_check_drift".to_string()));
        assert!(names.contains(&"sruja_list_architecture_index".to_string()));
        assert!(names.contains(&"sruja_get_topology".to_string()));
        assert!(names.contains(&"sruja_get_elements".to_string()));
        assert!(names.contains(&"sruja_get_diagnostic_full".to_string()));
        assert!(names.contains(&"sruja_suggest_context_prune".to_string()));
        assert!(names.contains(&"sruja_get_drift_state".to_string()));
        assert!(names.contains(&"sruja_get_context_events".to_string()));
        assert!(names.contains(&"sruja_get_decisions".to_string()));
        assert!(names.contains(&"sruja_get_decision_trace".to_string()));
        assert!(names.contains(&"sruja_record_context_event".to_string()));
        assert!(names.contains(&"sruja_record_decision_event".to_string()));
        assert!(names.contains(&"sruja_create_decision_record".to_string()));
        assert!(names.contains(&"sruja_link_decision_to_element".to_string()));
        assert!(names.contains(&"sruja_get_learned_facts".to_string()));
        assert!(names.contains(&"sruja_get_evidence_graph".to_string()));
        assert!(names.contains(&"sruja_get_agent_learnings".to_string()));
    }

    #[tokio::test]
    async fn mcp_tool_call_repomap_returns_markdown() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("main.rs"), "fn main() { println!(\"hello\"); }\n").expect("write");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_repomap",
            &json!({ "path": dir.path().to_string_lossy() }),
            ".",
            &cache,
        )
        .await
        .expect("repomap");
        assert!(out.contains("# Repository Map"));
    }

    #[tokio::test]
    async fn mcp_tool_call_discovery_explanation_returns_summary() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"express":"4.18.0"}}"#,
        )
        .expect("package");
        fs::write(
            src.join("server.ts"),
            "import { query } from './db';\nexport function start() { return query(); }\n",
        )
        .expect("server");
        fs::write(
            src.join("db.ts"),
            "export function query() { return []; }\n",
        )
        .expect("db");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_explain_discovery",
            &json!({ "path": dir.path().to_string_lossy() }),
            ".",
            &cache,
        )
        .await
        .expect("discovery explanation");

        assert!(out.contains("# Sruja Discovery Explanation"));
        assert!(out.contains("Why Sruja Thinks That"));
    }

    #[tokio::test]
    async fn mcp_tool_call_neighbors_returns_neighbors() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("main.rs"), "mod sub;\nfn main() {}\n").expect("main");
        fs::write(src.join("sub.rs"), "pub fn run() {}\n").expect("sub");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_neighbors",
            &json!({ "path": dir.path().to_string_lossy(), "id": "src_sub_rs" }),
            ".",
            &cache,
        )
        .await
        .expect("neighbors");

        assert!(out.contains("# Neighbors of src_sub_rs"));
        assert!(out.contains("Upstream"));
        assert!(out.contains("Downstream"));
    }

    #[tokio::test]
    async fn mcp_tool_call_find_path_returns_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            src.join("main.rs"),
            "use crate::sub;\nfn main() { sub::run(); }\n",
        )
        .expect("main");
        fs::write(src.join("sub.rs"), "pub fn run() {}\n").expect("sub");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_find_path",
            &json!({
                "path": dir.path().to_string_lossy(),
                "source": "src_main_rs",
                "target": "src_sub_rs"
            }),
            ".",
            &cache,
        )
        .await
        .expect("path");

        assert!(out.contains("# Path from src_main_rs to src_sub_rs"));
        assert!(out.contains("src_main_rs -> src_sub_rs"));
    }

    #[tokio::test]
    async fn mcp_tool_call_architecture_index_from_dsl_returns_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("repo.sruja"),
            r#"
MySystem = system "My System" {
  description "Test system"

  Api = container "API" {
    technology "Rust"
    description "HTTP API"
  }

  Db = database "DB" {
    technology "PostgreSQL"
    description "Data store"
  }
}

MySystem.Api -> MySystem.Db "SQL"
"#,
        )
        .expect("repo.sruja");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_list_architecture_index",
            &json!({ "path": dir.path().to_string_lossy(), "max_tokens": 2000 }),
            ".",
            &cache,
        )
        .await
        .expect("index");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            parsed.get("schema_version").and_then(|v| v.as_str()),
            Some("architecture_index/v1")
        );
        assert!(parsed.get("elements").and_then(|v| v.as_array()).is_some());
        assert!(parsed.get("estimated_tokens").is_some());
        assert_eq!(
            parsed.get("next_suggested_tool").and_then(|v| v.as_str()),
            Some("sruja_get_topology")
        );
    }

    #[tokio::test]
    async fn mcp_tool_call_topology_from_dsl_returns_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("repo.sruja"),
            r#"
MySystem = system "My System" {
  description "Test system"

  Api = container "API" {
    technology "Rust"
    description "HTTP API"
  }

  Db = database "DB" {
    technology "PostgreSQL"
    description "Data store"
  }
}

MySystem.Api -> MySystem.Db "SQL"
"#,
        )
        .expect("repo.sruja");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_topology",
            &json!({ "path": dir.path().to_string_lossy(), "id": "MySystem.Api", "depth": 1 }),
            ".",
            &cache,
        )
        .await
        .expect("topology");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            parsed.get("schema_version").and_then(|v| v.as_str()),
            Some("topology/v1")
        );
        assert!(parsed.get("upstream").is_some());
        assert!(parsed.get("downstream").is_some());
        assert_eq!(
            parsed.get("next_suggested_tool").and_then(|v| v.as_str()),
            Some("sruja_get_elements")
        );
        assert!(parsed
            .get("element_ids")
            .and_then(|v| v.as_array())
            .is_some());
    }

    #[tokio::test]
    async fn mcp_tool_call_get_elements_from_dsl_returns_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("repo.sruja"),
            r#"
MySystem = system "My System" {
  description "Test system"

  Api = container "API" {
    technology "Rust"
    description "HTTP API"
  }
}
"#,
        )
        .expect("repo.sruja");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_elements",
            &json!({ "path": dir.path().to_string_lossy(), "ids": ["MySystem.Api"] }),
            ".",
            &cache,
        )
        .await
        .expect("elements");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            parsed.get("schema_version").and_then(|v| v.as_str()),
            Some("elements/v1")
        );
        assert!(parsed.get("elements").and_then(|v| v.as_array()).is_some());
        assert_eq!(
            parsed.get("next_suggested_tool").and_then(|v| v.as_str()),
            Some("sruja_get_task_context")
        );
    }

    #[tokio::test]
    async fn mcp_tool_call_topology_element_ids_include_neighbors() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("repo.sruja"),
            r#"
MySystem = system "My System" {
  description "Test system"
  Api = container "API" { technology "Rust" description "HTTP API" }
  Db = database "DB" { technology "PostgreSQL" description "Data store" }
}
MySystem.Api -> MySystem.Db "SQL"
"#,
        )
        .expect("repo.sruja");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_topology",
            &json!({ "path": dir.path().to_string_lossy(), "id": "MySystem.Api", "depth": 1 }),
            ".",
            &cache,
        )
        .await
        .expect("topology");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        let ids: Vec<String> = parsed
            .get("element_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        assert!(ids.iter().any(|id| id == "MySystem.Api"));
        assert!(ids.iter().any(|id| id == "MySystem.Db"));
    }

    #[tokio::test]
    async fn mcp_tool_call_topology_resolves_short_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("repo.sruja"),
            r#"
MySystem = system "My System" {
  description "Test"
  Api = container "API" { technology "Rust" description "API" }
}
"#,
        )
        .expect("repo.sruja");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_topology",
            &json!({ "path": dir.path().to_string_lossy(), "id": "Api", "depth": 1 }),
            ".",
            &cache,
        )
        .await
        .expect("topology");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            parsed.get("target").and_then(|v| v.as_str()),
            Some("MySystem.Api")
        );
    }

    #[tokio::test]
    async fn mcp_tool_call_architecture_index_truncates_when_budget_low() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut dsl = String::new();
        dsl.push_str("App = system \"App\" {\n  description \"Many elements\"\n");
        for i in 0..80 {
            dsl.push_str(&format!(
                "  S{i} = container \"S{i}\" {{ technology \"Go\" description \"Service {i}\" }}\n"
            ));
        }
        dsl.push_str("}\n");
        fs::write(dir.path().join("repo.sruja"), dsl).expect("repo.sruja");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_list_architecture_index",
            &json!({ "path": dir.path().to_string_lossy(), "max_tokens": 400 }),
            ".",
            &cache,
        )
        .await
        .expect("index");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            parsed.get("truncated").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[tokio::test]
    async fn mcp_tool_call_get_diagnostic_full_reads_vfs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let uri = crate::commands::diagnostic_vfs::write_vfs_diagnostic(
            dir.path(),
            "sample.txt",
            "full diagnostic body\n",
        )
        .expect("write");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_diagnostic_full",
            &json!({
                "path": dir.path().to_string_lossy(),
                "uri": uri
            }),
            ".",
            &cache,
        )
        .await
        .expect("diagnostic full");

        assert_eq!(out.trim(), "full diagnostic body");
    }

    #[tokio::test]
    async fn mcp_tool_call_architecture_index_scan_fallback_without_dsl() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("lib.rs"), "pub fn hello() {}\n").expect("write");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_list_architecture_index",
            &json!({ "path": dir.path().to_string_lossy() }),
            ".",
            &cache,
        )
        .await
        .expect("index");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            parsed.pointer("/source/kind").and_then(|v| v.as_str()),
            Some("scan")
        );
    }

    #[tokio::test]
    async fn mcp_tool_call_uses_default_root_when_path_is_omitted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("main.rs"), "fn main() { println!(\"hello\"); }\n").expect("write");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_repomap",
            &json!({}),
            &dir.path().to_string_lossy(),
            &cache,
        )
        .await
        .expect("repomap");

        assert!(out.contains("# Repository Map"));
    }

    #[tokio::test]
    async fn mcp_tool_call_query_graph_returns_grounded_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("main.rs"), "mod sub;\nfn main() {}\n").expect("main");
        fs::write(src.join("sub.rs"), "pub fn run() {}\n").expect("sub");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_query_graph",
            &json!({
                "path": dir.path().to_string_lossy(),
                "query": "main sub module",
                "enrich": false
            }),
            ".",
            &cache,
        )
        .await
        .expect("query graph");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON output");
        assert_eq!(
            parsed.get("query").and_then(|v| v.as_str()),
            Some("main sub module")
        );
        assert!(parsed.get("matched_nodes").is_some());
        assert!(parsed.get("relationships").is_some());
    }

    #[tokio::test]
    async fn mcp_tool_call_explain_element_returns_grounded_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("main.rs"), "mod sub;\nfn main() {}\n").expect("main");
        fs::write(src.join("sub.rs"), "pub fn run() {}\n").expect("sub");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_explain_element",
            &json!({
                "path": dir.path().to_string_lossy(),
                "id": "src_sub_rs",
                "enrich": false
            }),
            ".",
            &cache,
        )
        .await
        .expect("explain element");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON output");
        assert_eq!(
            parsed.pointer("/element/id").and_then(|v| v.as_str()),
            Some("src_sub_rs")
        );
        assert!(parsed.get("neighbors").is_some());
        assert!(parsed.get("notes").is_some());
    }
}
