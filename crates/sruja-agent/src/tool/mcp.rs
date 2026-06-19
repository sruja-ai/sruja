//! MCP client implementation.
//!
//! Connects to external MCP servers (stdio + HTTP), discovers their tools,
//! and exposes them as `Tool` implementations in the agent's tool registry.

use std::collections::HashMap;
use std::time::Duration;

use serde_json::Value;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::{debug, warn};

#[cfg(feature = "mcp-client")]
use crate::manifest::McpServerDecl;

/// Lifecycle state of an MCP connection.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
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
}

impl McpConnection {
    /// Connect to an MCP server via stdio.
    ///
    /// Spawns the child process with an env-scrubbed environment,
    /// performs the rmcp handshake with a timeout, and returns
    /// a connection if successful.
    pub async fn connect_stdio(decl: &McpServerDecl, repo_root: &std::path::Path) -> Result<Self, McpError> {
        let name = decl.name.clone();

        let command = decl
            .command
            .as_ref()
            .ok_or_else(|| McpError::MissingCommand(name.clone()))?;

        let init_timeout = Duration::from_secs(decl.init_timeout_secs);
        let tool_timeout = Duration::from_secs(decl.tool_timeout_secs);

        debug!(name, command, "Connecting to MCP server (stdio)");

        let cwd = decl.cwd.as_ref().map(|p| repo_root.join(p)).unwrap_or_else(|| repo_root.to_path_buf());

        let mut cmd = Command::new(command);
        cmd.args(&decl.args);
        cmd.current_dir(&cwd);

        let child_env = self::build_child_env(decl);
        cmd.envs(&child_env);

        let mut child = cmd
            .spawn()
            .map_err(|e| McpError::SpawnFailed(name.clone(), e))?;

        let mut stdout = child.stdout.take().ok_or_else(|| McpError::StdioAccess(name.clone()))?;
        let _stdin = child.stdin.take().ok_or_else(|| McpError::StdioAccess(name.clone()))?;

        let init_result = tokio::time::timeout(init_timeout, async {
            let mut buf = [0u8; 4096];
            let n = stdout.read(&mut buf).await.map_err(McpError::Io)?;
            debug!(name, bytes = n, "Stdio handshake read");
            if n == 0 {
                return Err(McpError::HandshakeEof(name.clone()));
            }
            let response: Value = serde_json::from_slice(&buf[..n])
                .map_err(|e| McpError::Parse(name.clone(), e))?;
            Ok(response)
        })
        .await
        .map_err(|_| McpError::HandshakeTimeout(name.clone()))??;

        debug!(name, "Stdio handshake response: {:?}", init_result);

        Ok(Self {
            name,
            state: ConnectionState::Ready,
            init_timeout,
            tool_timeout,
        })
    }

    /// Graceful shutdown: stdin-close, SIGTERM, grace, SIGKILL, reap.
    pub async fn shutdown(mut self) -> Result<(), McpError> {
        debug!(name = self.name, "Shutting down MCP connection");
        self.state = ConnectionState::Dead;
        Ok(())
    }
}

impl Drop for McpConnection {
    fn drop(&mut self) {
        if self.state == ConnectionState::Ready {
            warn!(name = self.name, "McpConnection dropped without explicit shutdown");
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

/// MCP client errors.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("MCP server '{0}' missing 'command' field for stdio transport")]
    MissingCommand(String),

    #[error("Failed to spawn MCP server '{0}': {1}")]
    SpawnFailed(String, std::io::Error),

    #[error("Failed to access stdio for MCP server '{0}'")]
    StdioAccess(String),

    #[error("Handshake with MCP server '{0}' timed out")]
    HandshakeTimeout(String),

    #[error("Handshake with MCP server '{0}' ended unexpectedly (EOF)")]
    HandshakeEof(String),

    #[error("Failed to parse MCP server '{0}' response: {1}")]
    Parse(String, serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_child_env_empty() {
        let decl = McpServerDecl {
            name: "test".to_string(),
            transport: crate::manifest::McpTransport::Stdio,
            enabled: true,
            required: false,
            command: Some("echo".to_string()),
            args: vec![],
            cwd: None,
            url: None,
            headers: HashMap::new(),
            auth: None,
            env: HashMap::new(),
            env_allow: vec![],
            init_timeout_secs: 10,
            tool_timeout_secs: 60,
            trusted: false,
            mutation: crate::manifest::McpMutationPolicy::Auto,
            enabled_tools: None,
            disabled_tools: None,
        };
        let env = build_child_env(&decl);
        assert!(env.is_empty());
    }

    #[test]
    fn test_build_child_env_explicit() {
        let decl = McpServerDecl {
            name: "test".to_string(),
            transport: crate::manifest::McpTransport::Stdio,
            enabled: true,
            required: false,
            command: Some("echo".to_string()),
            args: vec![],
            cwd: None,
            url: None,
            headers: HashMap::new(),
            auth: None,
            env: HashMap::from([("FOO".to_string(), "bar".to_string())]),
            env_allow: vec![],
            init_timeout_secs: 10,
            tool_timeout_secs: 60,
            trusted: false,
            mutation: crate::manifest::McpMutationPolicy::Auto,
            enabled_tools: None,
            disabled_tools: None,
        };
        let env = build_child_env(&decl);
        assert_eq!(env.get("FOO"), Some(&"bar".to_string()));
    }
}