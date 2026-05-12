//! Extractor for AsyncAPI specification files.
//!
//! Detects AsyncAPI specs in YAML/JSON format, identifying event-driven
//! architectural components (channels, messages, servers).

use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

pub struct AsyncApiExtractor;

impl Default for AsyncApiExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncApiExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_candidate(name: &str, ext: &str) -> bool {
        if !matches!(ext, "yaml" | "yml" | "json") {
            return false;
        }
        name.contains("asyncapi") || name.contains("async-api") || name.contains("async_api")
    }

    fn has_asyncapi_content(content: &str) -> bool {
        content.contains("asyncapi:")
            || content.contains("\"asyncapi\":")
            || content.contains("'asyncapi':")
    }

    fn extract_title(content: &str) -> Option<String> {
        let mut in_info = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "info:" {
                in_info = true;
                continue;
            }
            if in_info && trimmed.starts_with("title:") {
                return Some(
                    trimmed["title:".len()..]
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                );
            }
            if in_info && !trimmed.is_empty() {
                let indent = line.len() - line.trim_start().len();
                if indent == 0 {
                    in_info = false;
                }
            }
        }
        None
    }
}

impl Extractor for AsyncApiExtractor {
    fn name(&self) -> &'static str {
        "asyncapi"
    }

    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError> {
        let name = ctx.file_name_lower();
        let ext = ctx.extension().to_lowercase();

        let is_match = if Self::is_candidate(&name, &ext) {
            true
        } else if matches!(ext.as_str(), "yaml" | "yml" | "json") {
            ctx.content()
                .map(Self::has_asyncapi_content)
                .unwrap_or(false)
        } else {
            false
        };

        if !is_match {
            return Ok(Vec::new());
        }

        let title = ctx.content().and_then(Self::extract_title);
        let description = title
            .as_ref()
            .map(|t| format!("AsyncAPI: {t}"))
            .unwrap_or_else(|| "Discovered AsyncAPI specification".to_string());

        let suggested_element = ctx.parent_dir_name().map(|s| s.to_string());

        Ok(vec![DiscoveredSource {
            binding: SourceBinding {
                kind: SourceKind::AsyncApi,
                path: ctx.relative_path().to_string(),
                description: Some(description),
            },
            suggested_element,
            confidence: 0.8,
        }])
    }
}
