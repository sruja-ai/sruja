//! MCP client implementation.
//!
//! Connects to external MCP servers (stdio + HTTP), discovers their tools,
//! and exposes them as `Tool` implementations in the agent's tool registry.

#[cfg(feature = "mcp-client")]
use std::collections::HashMap;
#[cfg(feature = "mcp-client")]
use std::sync::Arc;
#[cfg(feature = "mcp-client")]
use std::time::Duration;

#[cfg(feature = "mcp-client")]
use crate::manifest::McpServerDecl;
#[cfg(feature = "mcp-client")]
use crate::tool::{Tool, ToolError};
#[cfg(feature = "mcp-client")]
use async_trait::async_trait;
#[cfg(feature = "mcp-client")]
use rmcp::{
    model::{CallToolRequestParams, Tool as McpTool},
    transport::{StreamableHttpClientTransport, TokioChildProcess},
    ServiceExt,
};
#[cfg(feature = "mcp-client")]
use tokio::process::Command;
#[cfg(feature = "mcp-client")]
use tracing::{debug, warn};

/// Lifecycle state of an MCP connection.
#[derive(Debug, Clone, PartialEq)]
enum ConnectionState {
    Ready,
    Dead,
}

/// An MCP server connection (stdio only for now).
#[allow(dead_code)]
pub struct McpConnection {
    name: String,
    state: ConnectionState,
    init_timeout: Duration,
    tool_timeout: Duration,
    #[allow(dead_code)]
    server: rmcp::service::RunningService<rmcp::service::RoleClient, ()>,
}

impl McpConnection {
    /// Connect to an MCP server via stdio using rmcp.
    ///
    /// Spawns the child process with an env-scrubbed environment,
    /// performs the rmcp handshake with a timeout, and returns
    /// a connection if successful.
    pub async fn connect_stdio(
        decl: &McpServerDecl,
        repo_root: &std::path::Path,
    ) -> Result<Self, McpError> {
        let name = decl.name.clone();

        let command = decl
            .command
            .as_ref()
            .ok_or_else(|| McpError::MissingCommand(name.clone()))?;

        let init_timeout = Duration::from_secs(decl.init_timeout_secs);
        let tool_timeout = Duration::from_secs(decl.tool_timeout_secs);

        debug!(name, command, "Connecting to MCP server (stdio)");

        let cwd = decl
            .cwd
            .as_ref()
            .map(|p| repo_root.join(p))
            .unwrap_or_else(|| repo_root.to_path_buf());

        let mut cmd = Command::new(command);
        cmd.args(&decl.args);
        cmd.current_dir(&cwd);

        let child_env = self::build_child_env(decl);
        cmd.envs(&child_env);

        let transport =
            TokioChildProcess::new(cmd).map_err(|e| McpError::SpawnFailed(name.clone(), e))?;

        let server: rmcp::service::RunningService<rmcp::service::RoleClient, ()> =
            tokio::time::timeout(init_timeout, async {
                let server: Result<_, McpError> = ()
                    .serve(transport)
                    .await
                    .map_err(|e| McpError::HandshakeFailed(name.clone(), e.to_string()));
                server
            })
            .await
            .map_err(|_| McpError::HandshakeTimeout(name.clone()))??;

        debug!(name, "MCP server connected and initialized");

        Ok(Self {
            name,
            state: ConnectionState::Ready,
            init_timeout,
            tool_timeout,
            server,
        })
    }

    /// Connect to an MCP server via HTTP using rmcp's Streamable HTTP transport.
    ///
    /// Uses rmcp's built-in reqwest client (dual reqwest 0.12/0.13 accepted;
    /// they coexist as different majors). Applies init timeout.
    pub async fn connect_http(decl: &McpServerDecl) -> Result<Self, McpError> {
        let name = decl.name.clone();
        let url = decl
            .url
            .as_ref()
            .ok_or_else(|| McpError::MissingUrl(name.clone()))?;

        let init_timeout = Duration::from_secs(decl.init_timeout_secs);
        let tool_timeout = Duration::from_secs(decl.tool_timeout_secs);

        debug!(name, url, "Connecting to MCP server (HTTP)");

        let transport = StreamableHttpClientTransport::from_uri(url.clone());

        let server: rmcp::service::RunningService<rmcp::service::RoleClient, ()> =
            tokio::time::timeout(init_timeout, async {
                let server: Result<_, McpError> = ()
                    .serve(transport)
                    .await
                    .map_err(|e| McpError::HandshakeFailed(name.clone(), e.to_string()));
                server
            })
            .await
            .map_err(|_| McpError::HandshakeTimeout(name.clone()))??;

        debug!(name, "MCP HTTP server connected and initialized");

        Ok(Self {
            name,
            state: ConnectionState::Ready,
            init_timeout,
            tool_timeout,
            server,
        })
    }

    /// Connect to an MCP server using the transport declared in the manifest.
    pub async fn connect(
        decl: &McpServerDecl,
        repo_root: &std::path::Path,
    ) -> Result<Self, McpError> {
        use crate::manifest::McpTransport;
        match decl.transport {
            McpTransport::Stdio => Self::connect_stdio(decl, repo_root).await,
            McpTransport::Http => Self::connect_http(decl).await,
        }
    }

    /// List all tools from the MCP server.
    #[allow(dead_code)]
    pub async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        let tools = self
            .server
            .list_all_tools()
            .await
            .map_err(|e| McpError::ToolListFailed(self.name.clone(), e.to_string()))?;
        debug!(name = self.name, count = tools.len(), "Listed MCP tools");
        Ok(tools)
    }

    /// Call a tool on the MCP server.
    #[allow(dead_code)]
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: rmcp::model::JsonObject,
    ) -> Result<String, McpError> {
        let tool_name_owned = tool_name.to_string();
        let params = CallToolRequestParams::new(tool_name_owned).with_arguments(arguments);

        let result = tokio::time::timeout(self.tool_timeout, async {
            self.server.call_tool(params).await.map_err(|e| {
                McpError::ToolCallFailed(self.name.clone(), tool_name.to_string(), e.to_string())
            })
        })
        .await
        .map_err(|_| McpError::ToolCallTimeout(self.name.clone(), tool_name.to_string()))??;

        if result.is_error == Some(true) {
            let error_text = result
                .content
                .first()
                .and_then(|c| c.raw.as_text())
                .map(|t| t.text.as_str())
                .unwrap_or("(tool error with no text)");
            return Err(McpError::ToolError(
                self.name.clone(),
                tool_name.to_string(),
                error_text.to_string(),
            ));
        }

        let result_text = result
            .content
            .first()
            .and_then(|c| c.raw.as_text())
            .map(|t| t.text.as_str())
            .unwrap_or("(tool returned no text)");
        Ok(result_text.to_string())
    }

    /// Graceful shutdown with timeout: stdin-close → SIGTERM → grace → SIGKILL, reap.
    pub async fn shutdown(mut self) -> Result<(), McpError> {
        debug!(name = self.name, "Shutting down MCP connection");
        let _ = self
            .server
            .close_with_timeout(Duration::from_secs(5))
            .await
            .map_err(|e| warn!(name = self.name, "Close with timeout failed: {}", e));
        self.state = ConnectionState::Dead;
        Ok(())
    }
}

impl Drop for McpConnection {
    fn drop(&mut self) {
        if self.state == ConnectionState::Ready {
            warn!(
                name = self.name,
                "McpConnection dropped without explicit shutdown"
            );
        }
    }
}

/// Build the child process environment from the server declaration.
///
/// Security: child env = explicit `env` map + forwarded `env_allow` names only.
/// Never inherits the agent's environment.
fn build_child_env(decl: &McpServerDecl) -> HashMap<String, String> {
    let mut env = HashMap::new();

    for (k, v) in &decl.env {
        env.insert(k.clone(), v.clone());
    }

    for var_name in &decl.env_allow {
        if let Ok(val) = std::env::var(var_name) {
            env.insert(var_name.clone(), val);
        }
    }

    env
}

/// Bridge an MCP tool into Sruja's `Tool` registry.
///
/// Wraps a remote MCP tool behind the local `Tool` trait, namespacing it
/// as `mcp__<server>__<tool>` and applying mutation classification rules.
pub struct McpToolBridge {
    /// Namespaced tool name: `mcp__<server>__<tool>`
    name: String,
    /// Description from MCP tool
    description: String,
    /// JSON schema for parameters (converted from MCP input_schema)
    parameters: serde_json::Value,
    /// Whether this tool mutates state (mutation classification)
    is_mutating: bool,
    /// Original MCP tool name (for dispatching to the server)
    tool_name: String,
    /// Shared connection to the MCP server
    connection: Arc<McpConnection>,
}

impl McpToolBridge {
    /// Create a tool bridge from an MCP tool descriptor.
    ///
    /// Applies namespace `mcp__<server>__<tool>` and mutation classification:
    /// - If `trusted = false`: force mutating
    /// - If `trusted = true` + `readOnlyHint = true`: non-mutating
    /// - If `trusted = true` + `readOnlyHint = false/absent`: mutating
    /// - If `mutating = "readonly"`: override to non-mutating
    pub fn from_mcp_tool(
        mcp_tool: McpTool,
        server_name: &str,
        decl: &McpServerDecl,
        connection: Arc<McpConnection>,
    ) -> Self {
        let tool_name = mcp_tool.name.clone();
        let name = format!("mcp__{}__{}", server_name, tool_name);
        let description = mcp_tool
            .description
            .as_deref()
            .unwrap_or(&tool_name)
            .to_string();

        let input_schema: serde_json::Value = mcp_tool.input_schema.as_ref().clone().into();

        let is_mutating = classify_mutation(decl, &mcp_tool);

        Self {
            name,
            description,
            parameters: input_schema,
            is_mutating,
            tool_name: tool_name.to_string(),
            connection,
        }
    }
}

/// Classify whether an MCP tool is mutating based on policy.
///
/// R8: Remote-mutating sandbox: classify like `Shell` (mutating by default,
/// empty `affected_paths()`).
///
/// Rules:
/// - If `trusted = false`: force mutating (spec-aligned safety)
/// - If `trusted = true`:
///   - If `readOnlyHint = Some(true)`: non-mutating
///   - Otherwise: mutating
/// - If `mutating = "readonly"`: override to non-mutating
fn classify_mutation(decl: &McpServerDecl, mcp_tool: &McpTool) -> bool {
    use crate::manifest::McpMutationPolicy;

    let read_only_hint = mcp_tool.annotations.as_ref().and_then(|a| a.read_only_hint);

    let is_mutating_from_hint = match (decl.trusted, read_only_hint) {
        (false, _) => true,
        (true, Some(true)) => false,
        (true, Some(false) | None) => true,
    };

    match decl.mutation {
        McpMutationPolicy::Auto => is_mutating_from_hint,
        McpMutationPolicy::Readonly => false,
        McpMutationPolicy::Mutating => true,
    }
}

#[async_trait]
impl Tool for McpToolBridge {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters(&self) -> serde_json::Value {
        self.parameters.clone()
    }

    async fn call(&self, params: serde_json::Value) -> Result<String, ToolError> {
        let arguments: rmcp::model::JsonObject = params
            .as_object()
            .ok_or_else(|| {
                ToolError::InvalidParams("Expected object for MCP tool arguments".to_string())
            })?
            .clone();

        self.connection
            .call_tool(&self.tool_name, arguments)
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))
    }

    fn is_mutating(&self) -> bool {
        self.is_mutating
    }

    fn affected_paths(&self, _params: &serde_json::Value) -> Vec<String> {
        vec![]
    }
}

/// Manages MCP server connections and tool registration.
///
/// Holds `Arc<McpConnection>` for each server, provides tool discovery,
/// and handles graceful shutdown. Used by `AgentBuilder` to register
/// MCP tools before loop execution.
pub struct McpClientManager {
    repo_root: std::path::PathBuf,
    connections: Vec<Arc<McpConnection>>,
}

impl McpClientManager {
    /// Create a new manager from a loop manifest.
    ///
    /// Connects to all enabled MCP servers, lists their tools,
    /// and returns tool bridges for registration.
    ///
    /// Degrades gracefully: optional server failures log warnings,
    /// required server failures abort with an error.
    pub async fn from_manifest(
        manifest: &crate::manifest::LoopManifest,
        repo_root: impl Into<std::path::PathBuf>,
    ) -> Result<(Self, Vec<Box<dyn Tool>>), McpError> {
        let repo_root = repo_root.into();
        let mut connections = Vec::new();
        let mut tools = Vec::new();

        for decl in &manifest.mcp.servers {
            if !decl.enabled {
                debug!(name = decl.name, "MCP server disabled, skipping");
                continue;
            }

            let server_name = decl.name.clone();
            let result = McpConnection::connect(decl, &repo_root).await;

            match result {
                Ok(conn) => {
                    let conn_arc = Arc::new(conn);
                    connections.push(conn_arc.clone());

                    let mcp_tools = conn_arc.list_tools().await.map_err(|e| {
                        warn!(name = server_name, "Failed to list tools: {}", e);
                        e
                    })?;

                    for mcp_tool in &mcp_tools {
                        let bridge: Box<dyn Tool> = Box::new(McpToolBridge::from_mcp_tool(
                            mcp_tool.clone(),
                            &server_name,
                            decl,
                            conn_arc.clone(),
                        ));
                        tools.push(bridge);
                    }

                    debug!(
                        name = server_name,
                        count = mcp_tools.len(),
                        "MCP server tools registered"
                    );
                }
                Err(e) => {
                    if decl.required {
                        return Err(McpError::RequiredServerFailed(server_name, e.to_string()));
                    } else {
                        warn!(
                            name = server_name,
                            "MCP server connection failed (optional): {}", e
                        );
                    }
                }
            }
        }

        Ok((
            Self {
                repo_root,
                connections,
            },
            tools,
        ))
    }

    /// Get the repo root path.
    pub fn repo_root(&self) -> &std::path::Path {
        &self.repo_root
    }
}

impl Drop for McpClientManager {
    fn drop(&mut self) {
        let connections = std::mem::take(&mut self.connections);
        let repo_root = self.repo_root.clone();

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                for conn in connections {
                    if let Ok(conn) = Arc::try_unwrap(conn) {
                        let _ = conn.shutdown().await;
                    }
                }
                debug!(repo_root = %repo_root.display(), "MCP manager shutdown complete");
            });
        }
    }
}

/// MCP client errors.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("MCP server '{0}' missing 'command' field for stdio transport")]
    MissingCommand(String),

    #[error("MCP server '{0}' missing 'url' field for HTTP transport")]
    MissingUrl(String),

    #[error("Failed to spawn MCP server '{0}': {1}")]
    SpawnFailed(String, std::io::Error),

    #[error("Failed to access stdio for MCP server '{0}'")]
    StdioAccess(String),

    #[error("Handshake with MCP server '{0}' timed out")]
    HandshakeTimeout(String),

    #[error("Handshake with MCP server '{0}' failed: {1}")]
    HandshakeFailed(String, String),

    #[error("Handshake with MCP server '{0}' ended unexpectedly (EOF)")]
    HandshakeEof(String),

    #[error("Required MCP server '{0}' failed to connect: {1}")]
    RequiredServerFailed(String, String),

    #[error("Failed to list tools from MCP server '{0}': {1}")]
    ToolListFailed(String, String),

    #[error("Tool call to '{1}' on MCP server '{0}' timed out")]
    ToolCallTimeout(String, String),

    #[error("Tool call to '{1}' on MCP server '{0}' failed: {2}")]
    ToolCallFailed(String, String, String),

    #[error("Tool '{1}' on MCP server '{0}' returned an error: {2}")]
    ToolError(String, String, String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialize/deserialize error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_child_env_empty() {
        let decl = test_decl(
            "test",
            Some("echo"),
            false,
            crate::manifest::McpMutationPolicy::Auto,
        );
        let env = build_child_env(&decl);
        assert!(env.is_empty());
    }

    #[test]
    fn test_build_child_env_explicit() {
        let mut decl = test_decl(
            "test",
            Some("echo"),
            false,
            crate::manifest::McpMutationPolicy::Auto,
        );
        decl.env.insert("FOO".to_string(), "bar".to_string());
        let env = build_child_env(&decl);
        assert_eq!(env.get("FOO"), Some(&"bar".to_string()));
    }

    #[test]
    fn test_build_child_env_allowlist_forwards() {
        std::env::set_var("TEST_VAR_SENTINEL", "sentinel_value");
        let mut decl = test_decl(
            "test",
            Some("echo"),
            false,
            crate::manifest::McpMutationPolicy::Auto,
        );
        decl.env_allow.push("TEST_VAR_SENTINEL".to_string());
        let env = build_child_env(&decl);
        assert_eq!(
            env.get("TEST_VAR_SENTINEL"),
            Some(&"sentinel_value".to_string())
        );
    }

    #[tokio::test]
    async fn test_connection_missing_command() {
        let decl = test_decl(
            "test",
            None,
            false,
            crate::manifest::McpMutationPolicy::Auto,
        );
        let repo_root = std::path::Path::new(".");
        let result = McpConnection::connect_stdio(&decl, repo_root).await;
        assert!(matches!(result, Err(McpError::MissingCommand(_))));
    }

    #[tokio::test]
    async fn test_connection_http_missing_url() {
        let mut decl = test_decl(
            "test",
            None,
            false,
            crate::manifest::McpMutationPolicy::Auto,
        );
        decl.transport = crate::manifest::McpTransport::Http;
        let result = McpConnection::connect_http(&decl).await;
        assert!(matches!(result, Err(McpError::MissingUrl(_))));
    }

    fn test_decl(
        name: &str,
        command: Option<&str>,
        trusted: bool,
        mutation: crate::manifest::McpMutationPolicy,
    ) -> McpServerDecl {
        McpServerDecl {
            name: name.to_string(),
            transport: crate::manifest::McpTransport::Stdio,
            enabled: true,
            required: false,
            command: command.map(|s| s.to_string()),
            args: vec![],
            cwd: None,
            url: None,
            headers: HashMap::new(),
            auth: None,
            env: HashMap::new(),
            env_allow: vec![],
            init_timeout_secs: 10,
            tool_timeout_secs: 60,
            trusted,
            mutation,
            enabled_tools: None,
            disabled_tools: None,
        }
    }

    fn make_tool(name: &str, read_only_hint: Option<bool>) -> McpTool {
        let empty_schema = rmcp::model::JsonObject::new();
        let mut tool = McpTool::new(name.to_string(), "test tool", empty_schema);
        if let Some(ro) = read_only_hint {
            let mut ann = rmcp::model::ToolAnnotations::default();
            ann.read_only_hint = Some(ro);
            tool.annotations = Some(ann);
        }
        tool
    }

    #[test]
    fn test_classify_mutation_untrusted() {
        let decl = test_decl(
            "srv",
            Some("echo"),
            false,
            crate::manifest::McpMutationPolicy::Auto,
        );
        let tool = make_tool("safe_tool", Some(true));
        assert!(classify_mutation(&decl, &tool), "untrusted → mutating");
    }

    #[test]
    fn test_classify_mutation_trusted_readonly_hint() {
        let decl = test_decl(
            "srv",
            Some("echo"),
            true,
            crate::manifest::McpMutationPolicy::Auto,
        );
        let tool = make_tool("query", Some(true));
        assert!(
            !classify_mutation(&decl, &tool),
            "trusted + readOnlyHint=true → non-mutating"
        );
    }

    #[test]
    fn test_classify_mutation_trusted_mutating_hint() {
        let decl = test_decl(
            "srv",
            Some("echo"),
            true,
            crate::manifest::McpMutationPolicy::Auto,
        );
        let tool = make_tool("write", Some(false));
        assert!(
            classify_mutation(&decl, &tool),
            "trusted + readOnlyHint=false → mutating"
        );
    }

    #[test]
    fn test_classify_mutation_trusted_no_hint() {
        let decl = test_decl(
            "srv",
            Some("echo"),
            true,
            crate::manifest::McpMutationPolicy::Auto,
        );
        let tool = make_tool("unknown", None);
        assert!(
            classify_mutation(&decl, &tool),
            "trusted + no hint → mutating (conservative)"
        );
    }

    #[test]
    fn test_classify_mutation_override_readonly() {
        let decl = test_decl(
            "srv",
            Some("echo"),
            true,
            crate::manifest::McpMutationPolicy::Readonly,
        );
        let tool = make_tool("write", Some(false));
        assert!(
            !classify_mutation(&decl, &tool),
            "readonly policy overrides"
        );
    }

    #[test]
    fn test_classify_mutation_override_mutating() {
        let decl = test_decl(
            "srv",
            Some("echo"),
            true,
            crate::manifest::McpMutationPolicy::Mutating,
        );
        let tool = make_tool("read", Some(true));
        assert!(classify_mutation(&decl, &tool), "mutating policy overrides");
    }
}
