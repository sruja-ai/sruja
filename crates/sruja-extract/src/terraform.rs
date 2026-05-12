//! Extractor for Terraform / OpenTofu configuration files.
//!
//! Detects `.tf` files and extracts resource and module declarations
//! as architectural signals.

use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

pub struct TerraformExtractor;

impl Default for TerraformExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl TerraformExtractor {
    pub fn new() -> Self {
        Self
    }

    fn parse_resources(content: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some((kind, name)) = Self::parse_block_header(trimmed) {
                results.push((kind, name));
            }
        }
        results
    }

    fn parse_block_header(line: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = line
            .split(|c: char| c == '"' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .collect();

        match parts.as_slice() {
            ["resource", type_name, resource_name, ..] => {
                Some((format!("resource.{type_name}"), resource_name.to_string()))
            }
            ["module", module_name, ..] => Some(("module".to_string(), module_name.to_string())),
            ["data", type_name, data_name, ..] => {
                Some((format!("data.{type_name}"), data_name.to_string()))
            }
            _ => None,
        }
    }
}

impl Extractor for TerraformExtractor {
    fn name(&self) -> &'static str {
        "terraform"
    }

    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError> {
        if ctx.extension().to_lowercase() != "tf" {
            return Ok(Vec::new());
        }

        let content = match ctx.content() {
            Some(c) => c,
            None => return Ok(Vec::new()),
        };

        let resources = Self::parse_resources(content);
        if resources.is_empty() {
            return Ok(Vec::new());
        }

        Ok(resources
            .into_iter()
            .map(|(kind, name)| {
                let is_infra_resource = kind.starts_with("resource.");
                DiscoveredSource {
                    binding: SourceBinding {
                        kind: SourceKind::Terraform,
                        path: ctx.relative_path().to_string(),
                        description: Some(format!("Terraform {kind}: {name}")),
                    },
                    suggested_element: if is_infra_resource { Some(name) } else { None },
                    confidence: if is_infra_resource { 0.7 } else { 0.5 },
                }
            })
            .collect())
    }
}
