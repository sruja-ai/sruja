//! Extractor for GraphQL schema files (`.graphql`, `.gql`).
//!
//! Detects schema definitions and extracts type names as architectural
//! signals for API surface discovery.

use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

#[derive(Default)]
pub struct GraphqlExtractor;

impl GraphqlExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_graphql_file(ext: &str) -> bool {
        matches!(ext, "graphql" | "gql")
    }

    fn is_schema(content: &str) -> bool {
        content.contains("type Query")
            || content.contains("type Mutation")
            || content.contains("type Subscription")
            || content.contains("schema {")
            || content.contains("schema{")
            || content.contains("extend type")
            || content.contains("input ")
            || content.contains("enum ")
            || content.contains("interface ")
    }

    fn extract_root_types(content: &str) -> Vec<String> {
        let mut types = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            for prefix in &["type ", "extend type ", "interface "] {
                if let Some(rest) = trimmed.strip_prefix(prefix) {
                    let name = rest
                        .split(|c: char| c == '{' || c == '@' || c.is_whitespace())
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !name.is_empty()
                        && name
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false)
                    {
                        types.push(name.to_string());
                    }
                }
            }
        }
        types
    }
}

impl Extractor for GraphqlExtractor {
    fn name(&self) -> &'static str {
        "graphql"
    }

    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError> {
        let ext = ctx.extension().to_lowercase();
        if !Self::is_graphql_file(&ext) {
            return Ok(Vec::new());
        }

        let content = match ctx.content() {
            Some(c) => c,
            None => return Ok(Vec::new()),
        };

        if !Self::is_schema(content) {
            return Ok(Vec::new());
        }

        let types = Self::extract_root_types(content);

        let desc = if types.is_empty() {
            "GraphQL schema".to_string()
        } else {
            let preview: Vec<&str> = types.iter().map(|s| s.as_str()).take(5).collect();
            let suffix = if types.len() > 5 {
                format!(" (+{} more)", types.len() - 5)
            } else {
                String::new()
            };
            format!("GraphQL schema: {}{suffix}", preview.join(", "))
        };

        let suggested_element = ctx.parent_dir_name().map(|s| s.to_string());

        Ok(vec![DiscoveredSource {
            binding: SourceBinding {
                kind: SourceKind::GraphQL,
                path: ctx.relative_path().to_string(),
                description: Some(desc),
            },
            suggested_element,
            confidence: 0.75,
        }])
    }
}
