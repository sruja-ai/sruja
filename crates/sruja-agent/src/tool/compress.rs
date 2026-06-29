//! Tool for retrieving the original uncompressed text of a compressed message.
//!
//! Registered alongside the agent's other tools when compression is enabled.
//! The model calls it with a CCR handle (from the `[compressed:...]` marker)
//! and gets back the full verbatim original.

use std::sync::Arc;

use serde_json::json;
use sruja_compress::{CcrHandle, CcrStore};

use crate::tool::{Tool, ToolError};

/// Retrieve the original uncompressed content for a CCR handle.
///
/// The agent calls this when it needs the full text of a message that was
/// compressed by [`CompressingClient`](crate::llm::CompressingClient).
pub struct CompressRestoreTool {
    ccr: Arc<dyn CcrStore>,
}

impl CompressRestoreTool {
    /// Create a restore tool sharing the same CCR store as the compressing client.
    pub fn new(ccr: Arc<dyn CcrStore>) -> Self {
        Self { ccr }
    }
}

#[async_trait::async_trait]
impl Tool for CompressRestoreTool {
    fn name(&self) -> &str {
        "compress_restore"
    }

    fn description(&self) -> &str {
        "Retrieve the original uncompressed text for a message that was compressed. \
         Pass the handle string from the [compressed:...] marker to get back the \
         full verbatim content. Use this when you need details that were dropped \
         during context compression."
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "handle": {
                    "type": "string",
                    "description": "The handle from the [compressed:...] marker prefix"
                }
            },
            "required": ["handle"]
        })
    }

    async fn call(&self, params: serde_json::Value) -> Result<String, ToolError> {
        let handle_str = params
            .get("handle")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParams("missing 'handle' parameter".into()))?;

        match self.ccr.get(&CcrHandle(handle_str.to_string())) {
            Ok(Some(original)) => Ok(original),
            Ok(None) => Ok(format!(
                "(no content found for handle: {handle_str} — it may have expired)"
            )),
            Err(e) => Err(ToolError::Execution(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn retrieves_original() {
        let ccr = Arc::new(sruja_compress::InMemoryCcrStore::default()) as Arc<dyn CcrStore>;
        let original = "this is the original large content that was compressed";
        let handle = ccr.put(original).unwrap();

        let tool = CompressRestoreTool::new(ccr);
        let result = tool
            .call(json!({ "handle": handle.as_str() }))
            .await
            .unwrap();
        assert_eq!(result, original);
    }

    #[tokio::test]
    async fn unknown_handle_returns_message() {
        let ccr = Arc::new(sruja_compress::InMemoryCcrStore::default()) as Arc<dyn CcrStore>;
        let tool = CompressRestoreTool::new(ccr);
        let result = tool.call(json!({ "handle": "nonexistent" })).await.unwrap();
        assert!(result.contains("no content found"));
    }

    #[tokio::test]
    async fn missing_handle_param_errors() {
        let ccr = Arc::new(sruja_compress::InMemoryCcrStore::default()) as Arc<dyn CcrStore>;
        let tool = CompressRestoreTool::new(ccr);
        let result = tool.call(json!({})).await;
        assert!(result.is_err());
    }
}
