use std::path::PathBuf;

use async_trait::async_trait;
use serde_json::{json, Value};

use super::{opt_usize, str_param, Tool, ToolError};

// ---------------------------------------------------------------------------
// Shell
// ---------------------------------------------------------------------------

/// Run an allowlisted shell command.
pub struct Shell {
    root: PathBuf,
    allowed: Vec<String>,
}

impl Shell {
    /// Create with a custom executable allowlist.
    pub fn with_allowlist(root: impl Into<PathBuf>, allowed: Vec<String>) -> Self {
        Self {
            root: root.into(),
            allowed,
        }
    }

    fn is_allowed(&self, exe: &str) -> bool {
        self.allowed.iter().any(|a| a == exe)
    }
}

#[async_trait]
impl Tool for Shell {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "Run an allowlisted command (cargo, git, npm, etc). Use AFTER making changes to verify they work. Timeout default: 300s."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Executable name" },
                "args": { "type": "array", "items": { "type": "string" } },
                "timeout_ms": { "type": "integer", "description": "Max runtime in ms (default: 300000). Use 300000 for cargo commands." }
            },
            "required": ["command"]
        })
    }

    fn is_mutating(&self) -> bool {
        true
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let command = str_param(&params, "command")?;
        if !self.is_allowed(&command) {
            return Err(ToolError::Execution(format!(
                "'{command}' is not in the allowlist: [{}]",
                self.allowed.join(", ")
            )));
        }

        let args: Vec<String> = params
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| a.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let timeout_ms = opt_usize(&params, "timeout_ms").unwrap_or(300_000);
        let mut cmd = tokio::process::Command::new(&command);
        cmd.args(&args).current_dir(&self.root);

        let output = tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms as u64),
            cmd.output(),
        )
        .await
        .map_err(|_| ToolError::Execution(format!("timed out after {timeout_ms}ms")))?
        .map_err(|e| ToolError::Execution(format!("spawn failed: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);

        Ok(format!(
            "exit: {code}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
        ))
    }
}
