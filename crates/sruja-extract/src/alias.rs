//! Extractor for service aliases from Docker Compose files.
//!
//! Handles `docker-compose.yaml`, `docker-compose.yml`, `compose.yaml`,
//! `compose.yml`, and override variants like `docker-compose.prod.yaml`.

use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

pub struct AliasExtractor;

impl Default for AliasExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl AliasExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_docker_compose(name: &str) -> bool {
        let n = name.to_lowercase();
        n == "docker-compose.yaml"
            || n == "docker-compose.yml"
            || n == "compose.yaml"
            || n == "compose.yml"
            || (n.starts_with("docker-compose.") && (n.ends_with(".yaml") || n.ends_with(".yml")))
            || (n.starts_with("compose.") && (n.ends_with(".yaml") || n.ends_with(".yml")))
    }

    fn parse_compose_services(content: &str) -> Vec<String> {
        let mut services = Vec::new();
        let mut in_services = false;

        for line in content.lines() {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with("services:") && indent == 0 {
                in_services = true;
                continue;
            }

            if in_services {
                if indent == 0 && !trimmed.is_empty() && !trimmed.starts_with('#') {
                    break;
                }

                if indent == 2 && trimmed.ends_with(':') && !trimmed.starts_with('#') {
                    let service_name = trimmed.trim_end_matches(':').trim().to_string();
                    if !service_name.is_empty() {
                        services.push(service_name);
                    }
                }
            }
        }

        services
    }
}

impl Extractor for AliasExtractor {
    fn name(&self) -> &'static str {
        "alias"
    }

    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError> {
        let name = ctx.file_name_lower();
        if !Self::is_docker_compose(&name) {
            return Ok(Vec::new());
        }

        let content = match ctx.content() {
            Some(c) => c,
            None => return Ok(Vec::new()),
        };

        let services = Self::parse_compose_services(content);
        let is_override = name.contains('.') && name.split('.').count() > 2;

        Ok(services
            .into_iter()
            .map(|service_name| DiscoveredSource {
                binding: SourceBinding {
                    kind: SourceKind::Custom("docker-compose".to_string()),
                    path: ctx.relative_path().to_string(),
                    description: Some(format!("Compose service: {service_name}")),
                },
                suggested_element: Some(service_name),
                confidence: if is_override { 0.7 } else { 0.9 },
            })
            .collect())
    }
}
