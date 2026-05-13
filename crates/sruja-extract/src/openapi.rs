//! Extractor for OpenAPI / Swagger specifications.

use crate::utils::yaml;
use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

#[derive(Default)]
pub struct OpenApiExtractor;

impl OpenApiExtractor {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn is_candidate(name: &str, ext: &str) -> bool {
        if !matches!(ext, "yaml" | "yml" | "json") {
            return false;
        }
        name.contains("openapi")
            || name.contains("swagger")
            || name.contains("api-spec")
            || name.contains("api_spec")
    }

    fn has_openapi_content(content: &str) -> bool {
        yaml::has_markers(
            content,
            &["openapi:", "\"openapi\":", "swagger:", "\"swagger\":"],
        )
    }

    pub(crate) fn extract_title(content: &str) -> Option<String> {
        yaml::extract_title_from_yaml(content, &["title:", "\"title\":", "'title':"])
    }
}

impl Extractor for OpenApiExtractor {
    fn name(&self) -> &'static str {
        "openapi"
    }

    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError> {
        let name = ctx.file_name_lower();
        let ext = ctx.extension().to_lowercase();

        let is_match = if Self::is_candidate(&name, &ext) {
            true
        } else if matches!(ext.as_str(), "yaml" | "yml" | "json") {
            ctx.content()
                .map(Self::has_openapi_content)
                .unwrap_or(false)
        } else {
            false
        };

        if !is_match {
            return Ok(Vec::new());
        }

        let description = ctx
            .content()
            .and_then(Self::extract_title)
            .map(|t| format!("OpenAPI: {t}"))
            .unwrap_or_else(|| "Discovered OpenAPI specification".to_string());

        let suggested_element = ctx.parent_dir_name().map(|s| s.to_string());

        Ok(vec![DiscoveredSource {
            binding: SourceBinding {
                kind: SourceKind::OpenApi,
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
        assert!(OpenApiExtractor::is_candidate("openapi.yaml", "yaml"));
        assert!(OpenApiExtractor::is_candidate("swagger.yaml", "yaml"));
        assert!(OpenApiExtractor::is_candidate("api-spec.yaml", "yaml"));
        assert!(OpenApiExtractor::is_candidate("api_spec.json", "json"));
    }

    #[test]
    fn test_is_candidate_rejects_invalid_ext() {
        assert!(!OpenApiExtractor::is_candidate("openapi.yaml", "txt"));
        assert!(!OpenApiExtractor::is_candidate("openapi.yaml", "xml"));
    }

    #[test]
    fn test_is_candidate_rejects_unrelated_names() {
        assert!(!OpenApiExtractor::is_candidate("config.yaml", "yaml"));
        assert!(!OpenApiExtractor::is_candidate("data.json", "json"));
    }

    #[test]
    fn test_extract_title_simple() {
        assert_eq!(
            OpenApiExtractor::extract_title("title: My API\ninfo:\n  version: 1.0"),
            Some("My API".to_string())
        );
    }

    #[test]
    fn test_extract_title_quoted() {
        assert_eq!(
            OpenApiExtractor::extract_title("title: \"Quoted Title\""),
            Some("Quoted Title".to_string())
        );
    }

    #[test]
    fn test_extract_title_no_title() {
        assert_eq!(OpenApiExtractor::extract_title("openapi: 3.0.0"), None);
    }

    #[test]
    fn test_extract_title_empty() {
        assert_eq!(OpenApiExtractor::extract_title(""), None);
    }

    #[test]
    fn test_has_openapi_content_openapi() {
        assert!(OpenApiExtractor::has_openapi_content(
            "openapi: 3.0.0\ninfo:\n  title: Test"
        ));
    }

    #[test]
    fn test_has_openapi_content_swagger() {
        assert!(OpenApiExtractor::has_openapi_content(
            "swagger: 2.0\ninfo:\n  title: Test"
        ));
    }

    #[test]
    fn test_has_openapi_content_negative() {
        assert!(!OpenApiExtractor::has_openapi_content(
            "name: value\nversion: 1.0"
        ));
    }
}
