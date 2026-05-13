//! Extractor for service aliases from Docker Compose files.
//!
//! Handles `docker-compose.yaml`, `docker-compose.yml`, `compose.yaml`,
//! `compose.yml`, and override variants like `docker-compose.prod.yaml`.

use crate::utils::yaml;
use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

#[derive(Default)]
pub struct AliasExtractor;

impl AliasExtractor {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn is_docker_compose(name: &str) -> bool {
        let n = name.to_lowercase();
        n == "docker-compose.yaml"
            || n == "docker-compose.yml"
            || n == "compose.yaml"
            || n == "compose.yml"
            || (n.starts_with("docker-compose.") && (n.ends_with(".yaml") || n.ends_with(".yml")))
            || (n.starts_with("compose.") && (n.ends_with(".yaml") || n.ends_with(".yml")))
    }

    pub(crate) fn parse_compose_services(content: &str) -> Vec<String> {
        yaml::parse_yaml_services(content)
    }

    pub(crate) fn is_override(name: &str) -> bool {
        name.contains('.') && name.split('.').count() > 2
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
        let is_override = Self::is_override(&name);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_docker_compose_standard_names() {
        assert!(AliasExtractor::is_docker_compose("docker-compose.yaml"));
        assert!(AliasExtractor::is_docker_compose("docker-compose.yml"));
        assert!(AliasExtractor::is_docker_compose("compose.yaml"));
        assert!(AliasExtractor::is_docker_compose("compose.yml"));
    }

    #[test]
    fn test_is_docker_compose_override_files() {
        assert!(AliasExtractor::is_docker_compose(
            "docker-compose.prod.yaml"
        ));
        assert!(AliasExtractor::is_docker_compose("docker-compose.dev.yml"));
        assert!(AliasExtractor::is_docker_compose("compose.override.yaml"));
    }

    #[test]
    fn test_is_docker_compose_rejects_non_compose() {
        assert!(!AliasExtractor::is_docker_compose("config.yaml"));
        assert!(!AliasExtractor::is_docker_compose("docker.yaml"));
        assert!(!AliasExtractor::is_docker_compose("docker-compose.txt"));
    }

    #[test]
    fn test_parse_compose_services_basic() {
        let content = "services:\n  api:\n    image: api:v1\n  worker:\n    image: worker:v1";
        let services = AliasExtractor::parse_compose_services(content);
        assert_eq!(services, vec!["api", "worker"]);
    }

    #[test]
    fn test_parse_compose_services_empty() {
        let content = "version: 3\nnetworks:\n  api:";
        let services = AliasExtractor::parse_compose_services(content);
        assert!(services.is_empty());
    }

    #[test]
    fn test_parse_compose_services_with_comments() {
        let content = "services:\n  # comment\n  api:\n    image: api:v1";
        let services = AliasExtractor::parse_compose_services(content);
        assert_eq!(services, vec!["api"]);
    }

    #[test]
    fn test_parse_compose_services_nested_keys() {
        let content =
            "services:\n  api:\n    build:\n      context: ./api\n    ports:\n      - 8080:8080";
        let services = AliasExtractor::parse_compose_services(content);
        assert_eq!(services, vec!["api"]);
    }

    #[test]
    fn test_is_override_true() {
        assert!(AliasExtractor::is_override("docker-compose.prod.yaml"));
        assert!(AliasExtractor::is_override("compose.override.yml"));
    }

    #[test]
    fn test_is_override_false() {
        assert!(!AliasExtractor::is_override("docker-compose.yaml"));
        assert!(!AliasExtractor::is_override("compose.yml"));
    }
}
