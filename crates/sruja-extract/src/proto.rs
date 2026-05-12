//! Extractor for Protocol Buffer (`.proto`) schema files.
//!
//! Detects service and message definitions in `.proto` files as
//! architectural signals for gRPC-based communication.

use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

#[derive(Default)]
pub struct ProtoExtractor;

impl ProtoExtractor {
    pub fn new() -> Self {
        Self
    }

    fn parse_services(content: &str) -> Vec<String> {
        let mut services = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("service ") {
                let name = rest
                    .split(|c: char| c == '{' || c.is_whitespace())
                    .next()
                    .unwrap_or("")
                    .trim();
                if !name.is_empty() {
                    services.push(name.to_string());
                }
            }
        }
        services
    }

    fn extract_package(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("package ") {
                let pkg = rest.trim_end_matches(';').trim();
                if !pkg.is_empty() {
                    return Some(pkg.to_string());
                }
            }
        }
        None
    }
}

impl Extractor for ProtoExtractor {
    fn name(&self) -> &'static str {
        "proto"
    }

    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError> {
        if ctx.extension().to_lowercase() != "proto" {
            return Ok(Vec::new());
        }

        let content = match ctx.content() {
            Some(c) => c,
            None => return Ok(Vec::new()),
        };

        if !content.contains("syntax") {
            return Ok(Vec::new());
        }

        let services = Self::parse_services(content);
        let package = Self::extract_package(content);

        if services.is_empty() {
            let desc = package
                .as_ref()
                .map(|p| format!("Protobuf schema: {p}"))
                .unwrap_or_else(|| "Protobuf schema".to_string());

            return Ok(vec![DiscoveredSource {
                binding: SourceBinding {
                    kind: SourceKind::Proto,
                    path: ctx.relative_path().to_string(),
                    description: Some(desc),
                },
                suggested_element: ctx.parent_dir_name().map(|s| s.to_string()),
                confidence: 0.5,
            }]);
        }

        Ok(services
            .into_iter()
            .map(|svc| {
                let desc = package
                    .as_ref()
                    .map(|p| format!("gRPC service: {p}.{svc}"))
                    .unwrap_or_else(|| format!("gRPC service: {svc}"));

                DiscoveredSource {
                    binding: SourceBinding {
                        kind: SourceKind::Proto,
                        path: ctx.relative_path().to_string(),
                        description: Some(desc),
                    },
                    suggested_element: Some(svc),
                    confidence: 0.8,
                }
            })
            .collect())
    }
}
