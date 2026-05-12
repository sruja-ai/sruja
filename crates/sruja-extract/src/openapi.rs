//! Extractor for OpenAPI / Swagger specifications.

use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

pub struct OpenApiExtractor;

impl Default for OpenApiExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenApiExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_candidate(name: &str, ext: &str) -> bool {
        if !matches!(ext, "yaml" | "yml" | "json") {
            return false;
        }
        name.contains("openapi")
            || name.contains("swagger")
            || name.contains("api-spec")
            || name.contains("api_spec")
    }

    fn has_openapi_content(content: &str) -> bool {
        content.contains("openapi:")
            || content.contains("\"openapi\":")
            || content.contains("swagger:")
            || content.contains("\"swagger\":")
    }

    fn extract_title(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("title:") {
                return Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
            }
            if let Some(rest) = trimmed.strip_prefix("\"title\":") {
                return Some(rest.trim().trim_matches('"').trim_matches(',').to_string());
            }
        }
        None
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
