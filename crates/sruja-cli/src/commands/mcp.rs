use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use std::path::Path;

use super::CliError;

const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

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

struct McpServer {
    initialized: bool,
    client_ready: bool,
    default_repo: String,
    graph_cache: std::sync::Arc<tokio::sync::Mutex<HashMap<String, sruja_scan::Graph>>>,
}

impl McpServer {
    fn new(default_repo: String) -> Self {
        Self {
            initialized: false,
            client_ready: false,
            default_repo,
            graph_cache: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
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
                self.initialized = true;
                Some(self.handle_initialize(id, message.get("params")))
            }
            "notifications/initialized" => {
                self.client_ready = true;
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
                    "tools": { "listChanged": false }
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
                "tools": tool_definitions()
            }
        })
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

        match run_tool(name, &args, &self.default_repo, &self.graph_cache).await {
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
            "name": "sruja_get_architecture_context",
            "title": "Sruja Architecture Context",
            "description": "Export high-level architecture context and project rules. Provide a file or element_id to get a localized, task-scoped context map.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
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
                    "max_tokens": { "type": "integer", "description": "Maximum tokens for the hydrated context (default: 20000)" }
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
            "description": "Get high-fidelity architectural context for a specific task. Supports selection by element ID, file path, git diff (base/head refs), or search query. Returns focus elements, neighbors, impact analysis, and hydrated source code.",
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
                    "max_tokens": { "type": "integer", "description": "Maximum tokens for hydrated source code (default: 10000)" }
                }
            }
        }),
        json!({
            "name": "sruja_semantic_search",
            "title": "Sruja Semantic Search",
            "description": "Search for architectural components using natural language (semantic similarity).",
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
            "name": "sruja_get_focus_briefing",
            "title": "Sruja Focus Briefing",
            "description": "Get a task-scoped architectural briefing for a specific file or element. Includes blast radius, linked decisions, and AI instructions.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "file": { "type": "string", "description": "File path to focus on" },
                    "element_id": { "type": "string", "description": "Element ID to focus on" },
                    "format": { "type": "string", "description": "Output format: text (default) or json" }
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

    match name {
        "sruja_get_repomap" => {
            let repomap = super::discover::discover_repomap(&repo, 100, 5000)?;
            Ok(repomap)
        }
        "sruja_get_architecture_context" => {
            let file = arguments.get("file").and_then(|v| v.as_str()).map(String::from);
            let element_id = arguments.get("element_id").and_then(|v| v.as_str()).map(String::from);
            let intent = arguments.get("intent").and_then(|v| v.as_str()).map(String::from);
            let content = super::context::context_string(
                &repo,
                "markdown",
                super::context::ContextRequest {
                    file: file.as_deref(),
                    element_id: element_id.as_deref(),
                    query: None,
                    base_ref: None,
                    head_ref: None,
                    intent: intent.as_deref(),
                    depth: 2,
                    max_tokens: 10000,
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
                    file: None,
                    element_id: None,
                    query: None,
                    base_ref: None,
                    head_ref: None,
                    intent: None,
                    depth: 1,
                    max_tokens: 3000,
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
        "sruja_get_entrypoints" => {
            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let mut entrypoints = Vec::new();

            let mut has_incoming = HashMap::new();
            for edge in &graph.edges {
                *has_incoming.entry(edge.target.as_str()).or_insert(0) += 1;
            }

            for node in &graph.nodes {
                let is_high_level = matches!(
                    node.kind,
                    sruja_scan::NodeKind::Service
                        | sruja_scan::NodeKind::ExternalApi
                        | sruja_scan::NodeKind::System
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
                    node.kind,
                    sruja_scan::NodeKind::Database | sruja_scan::NodeKind::Queue
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
            Ok(serde_json::to_string_pretty(&ctx)?)
        }
        "sruja_get_state_machine" => {
            let element_id = arguments.get("element_id").and_then(|v| v.as_str()).ok_or_else(|| CliError::validation("Missing element_id"))?;
            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let node = graph.nodes.iter().find(|n| n.id == element_id).ok_or_else(|| CliError::validation(format!("Element {} not found", element_id)))?;
            
            if node.state_machines.is_empty() {
                return Ok(format!("No state machines found for element {}.", element_id));
            }
            
            Ok(serde_json::to_string_pretty(&node.state_machines)?)
        }
        "sruja_get_contract" => {
            let element_id = arguments.get("element_id").and_then(|v| v.as_str()).ok_or_else(|| CliError::validation("Missing element_id"))?;
            let contract_name = arguments.get("contract_name").and_then(|v| v.as_str());
            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let node = graph.nodes.iter().find(|n| n.id == element_id).ok_or_else(|| CliError::validation(format!("Element {} not found", element_id)))?;
            
            if node.contracts.is_empty() {
                return Ok(format!("No contracts found for element {}.", element_id));
            }
            
            if let Some(name) = contract_name {
                let contract = node.contracts.iter().find(|c| c.name == name).ok_or_else(|| CliError::validation(format!("Contract {} not found on element {}", name, element_id)))?;
                Ok(serde_json::to_string_pretty(contract)?)
            } else {
                Ok(serde_json::to_string_pretty(&node.contracts)?)
            }
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
                        out.push_str(&format!(
                            "### {} - {}\n",
                            inc.id,
                            inc.title
                        ));
                        if let Some(s) = &inc.severity {
                            out.push_str(&format!("- **Severity**: {}\n", s));
                        }
                        if let Some(d) = &inc.date {
                            out.push_str(&format!("- **Date**: {}\n", d));
                        }
                        if !inc.affected.is_empty() {
                            out.push_str("- **Affected**: ");
                            out.push_str(&inc.affected.join(", "));
                            out.push_str("\n");
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
                        out.push_str("\n");
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
            let description = arguments.get("description").and_then(|v| v.as_str()).unwrap_or("");
            
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
            if let Some(rels) = arguments.get("add_relationships").and_then(|v| v.as_array()) {
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

            super::propose_create(&repo, description, add_elements, add_relationships, remove_elements).await?;
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

            get_hydrated_context(&repo, id, max_tokens, graph_cache).await
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
        "sruja_get_context_score" => {
            let format = arguments.get("format").and_then(|v| v.as_str()).unwrap_or("text");
            let kg = crate::graph_store::load_or_build_graph(Path::new(&repo))?;
            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            let age_hours = crate::utils::context::context_age_hours(Path::new(&repo));
            let score = sruja_graph::compute_context_score(&kg, graph.nodes.len(), Path::new(&repo), age_hours);
            
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
        "sruja_get_focus_briefing" => {
            let file = arguments.get("file").and_then(|v| v.as_str());
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());
            
            let kg = crate::graph_store::load_or_build_graph(Path::new(&repo))?;
            let graph = get_or_scan_graph(graph_cache, &repo).await?;
            
            let target_id = super::focus::resolve_target(&kg, file, element_id)?;
            let briefing = super::focus::build_focus_briefing(&kg, &target_id, Path::new(&repo), graph.nodes.len());
            
            Ok(serde_json::to_string_pretty(&briefing)?)
        }
        _ => Err(CliError::validation(format!("Unknown tool: {name}"))),
    }
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
        return Err(CliError::validation(
            "Invalid string: contains newline",
        ));
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
}
