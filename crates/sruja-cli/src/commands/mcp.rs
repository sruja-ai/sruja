use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};

use super::CliError;

const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

pub async fn mcp() -> Result<(), CliError> {
    let stdin = io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout);

    let mut server = McpServer::new();

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
}

impl McpServer {
    fn new() -> Self {
        Self {
            initialized: false,
            client_ready: false,
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

        match run_tool(name, &args).await {
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
            "description": "Export high-level architecture context and project rules for AI tooling.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
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
    ]
}

async fn run_tool(name: &str, arguments: &Value) -> Result<String, CliError> {
    let repo = arguments
        .get("path")
        .or_else(|| arguments.get("repo"))
        .and_then(|v| v.as_str())
        .unwrap_or(".")
        .to_string();

    match name {
        "sruja_get_repomap" => {
            let repomap = super::discover::discover_repomap(&repo, 100, 5000)?;
            Ok(repomap)
        }
        "sruja_get_architecture_context" => {
            let content =
                super::context::context_string(&repo, "markdown", None, None, 2, 10000).await?;
            Ok(content)
        }
        "sruja_explain_discovery" => {
            let format = arguments
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("text");
            match format {
                "json" => super::discover::discover_explanation_json(&repo),
                "text" => super::discover::discover_explanation_string(&repo),
                _ => Err(CliError::Validation(format!(
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
        "sruja_add_element" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::Validation("Missing id".into()))?;
            let kind = arguments
                .get("kind")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::Validation("Missing kind".into()))?;
            let title = arguments
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::Validation("Missing title".into()))?;
            let description = arguments.get("description").and_then(|v| v.as_str());
            let technology = arguments.get("technology").and_then(|v| v.as_str());

            add_element(&repo, id, kind, title, description, technology).await?;
            Ok(format!("Added {} {} to architecture", kind, id))
        }
        "sruja_add_relationship" => {
            let source = arguments
                .get("source")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::Validation("Missing source".into()))?;
            let target = arguments
                .get("target")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::Validation("Missing target".into()))?;
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
                        .map_err(|e| CliError::Validation(e.to_string()))?;
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
        _ => Err(CliError::Validation(format!("Unknown tool: {name}"))),
    }
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

fn validate_ident(value: &str, field: &str) -> Result<(), CliError> {
    if value.is_empty() {
        return Err(CliError::Validation(format!("Missing {}", field)));
    }
    if value.trim() != value {
        return Err(CliError::Validation(format!(
            "Invalid {}: leading/trailing whitespace",
            field
        )));
    }
    if value
        .chars()
        .any(|c| c.is_whitespace() || c == '"' || c == '{' || c == '}' || c == '\\')
    {
        return Err(CliError::Validation(format!(
            "Invalid {}: contains forbidden characters",
            field
        )));
    }
    Ok(())
}

fn escape_dsl_string(value: &str) -> Result<String, CliError> {
    if value.chars().any(|c| c == '\n' || c == '\r') {
        return Err(CliError::Validation(
            "Invalid string: contains newline".into(),
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
        let server = McpServer::new();
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
        let server = McpServer::new();
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

        let out = run_tool(
            "sruja_get_repomap",
            &json!({ "path": dir.path().to_string_lossy() }),
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

        let out = run_tool(
            "sruja_explain_discovery",
            &json!({ "path": dir.path().to_string_lossy() }),
        )
        .await
        .expect("discovery explanation");

        assert!(out.contains("# Sruja Discovery Explanation"));
        assert!(out.contains("Why Sruja Thinks That"));
    }
}
