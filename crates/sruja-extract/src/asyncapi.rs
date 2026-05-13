//! Extractor for AsyncAPI specification files.
//!
//! Detects AsyncAPI specs in YAML/JSON format, identifying event-driven
//! architectural components (channels, messages, servers).

use crate::utils::yaml;
use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

#[derive(Default)]
pub struct AsyncApiExtractor;

impl AsyncApiExtractor {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn is_candidate(name: &str, ext: &str) -> bool {
        if !matches!(ext, "yaml" | "yml" | "json") {
            return false;
        }
        name.contains("asyncapi") || name.contains("async-api") || name.contains("async_api")
    }

    fn has_asyncapi_content(content: &str) -> bool {
        yaml::has_markers(content, &["asyncapi:", "\"asyncapi\":", "'asyncapi':"])
    }

    pub(crate) fn extract_title(content: &str) -> Option<String> {
        yaml::extract_title_from_yaml(content, &["title:", "\"title\":", "'title':"])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_candidate_yaml() {
        assert!(AsyncApiExtractor::is_candidate("asyncapi.yaml", "yaml"));
        assert!(AsyncApiExtractor::is_candidate("async-api.yaml", "yaml"));
        assert!(AsyncApiExtractor::is_candidate("async_api.json", "json"));
    }

    #[test]
    fn test_is_candidate_rejects_invalid_ext() {
        assert!(!AsyncApiExtractor::is_candidate("asyncapi.yaml", "txt"));
        assert!(!AsyncApiExtractor::is_candidate("asyncapi.yaml", "xml"));
    }

    #[test]
    fn test_is_candidate_rejects_unrelated_names() {
        assert!(!AsyncApiExtractor::is_candidate("events.yaml", "yaml"));
        assert!(!AsyncApiExtractor::is_candidate("messages.json", "json"));
    }

    #[test]
    fn test_extract_title_simple() {
        assert_eq!(
            AsyncApiExtractor::extract_title("title: Order Events\ninfo:\n  version: 1.0"),
            Some("Order Events".to_string())
        );
    }

    #[test]
    fn test_extract_title_quoted() {
        assert_eq!(
            AsyncApiExtractor::extract_title("title: \"Event Service\""),
            Some("Event Service".to_string())
        );
    }

    #[test]
    fn test_extract_title_no_title() {
        assert_eq!(AsyncApiExtractor::extract_title("asyncapi: 3.0.0"), None);
    }

    #[test]
    fn test_has_asyncapi_content() {
        assert!(AsyncApiExtractor::has_asyncapi_content(
            "asyncapi: 3.0.0\ninfo:\n  title: Test"
        ));
    }

    #[test]
    fn test_has_asyncapi_content_negative() {
        assert!(!AsyncApiExtractor::has_asyncapi_content(
            "name: value\nversion: 1.0"
        ));
    }
}
