//! Extractor for Dockerfiles.
//!
//! Detects `Dockerfile`, `Dockerfile.*` variants, and `*.dockerfile`.
//! Extracts the base image and exposed ports as architectural context.

use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

#[derive(Default)]
pub struct DockerfileExtractor;

impl DockerfileExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_dockerfile(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower == "dockerfile"
            || lower.starts_with("dockerfile.")
            || lower.ends_with(".dockerfile")
            || lower == "containerfile"
    }

    fn extract_base_image(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed
                .strip_prefix("FROM ")
                .or_else(|| trimmed.strip_prefix("FROM\t"))
            {
                let image = rest.split_ascii_whitespace().next().unwrap_or(rest);
                return Some(image.to_string());
            }
        }
        None
    }

    fn extract_exposed_ports(content: &str) -> Vec<String> {
        let mut ports = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed
                .strip_prefix("EXPOSE ")
                .or_else(|| trimmed.strip_prefix("EXPOSE\t"))
            {
                for port in rest.split_whitespace() {
                    let port = port.split('/').next().unwrap_or(port);
                    if port.chars().all(|c| c.is_ascii_digit()) {
                        ports.push(port.to_string());
                    }
                }
            }
        }
        ports
    }
}

impl Extractor for DockerfileExtractor {
    fn name(&self) -> &'static str {
        "dockerfile"
    }

    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError> {
        if !Self::is_dockerfile(ctx.file_name()) {
            return Ok(Vec::new());
        }

        let content = match ctx.content() {
            Some(c) => c,
            None => return Ok(Vec::new()),
        };

        if !content.contains("FROM ") && !content.contains("FROM\t") {
            return Ok(Vec::new());
        }

        let base_image = Self::extract_base_image(content);
        let ports = Self::extract_exposed_ports(content);

        let mut desc_parts = vec!["Dockerfile".to_string()];
        if let Some(ref img) = base_image {
            desc_parts.push(format!("base: {img}"));
        }
        if !ports.is_empty() {
            desc_parts.push(format!("ports: {}", ports.join(", ")));
        }

        let suggested_element = ctx.parent_dir_name().map(|s| s.to_string());

        Ok(vec![DiscoveredSource {
            binding: SourceBinding {
                kind: SourceKind::Dockerfile,
                path: ctx.relative_path().to_string(),
                description: Some(desc_parts.join(" | ")),
            },
            suggested_element,
            confidence: 0.85,
        }])
    }
}
