//! Extractor for inter-service dependency signals from source code.
//!
//! Looks for environment-variable-style config patterns that suggest
//! external service dependencies (e.g. `PAYMENT_SERVICE_URL`,
//! `ORDER_HOST`, `USER_API_BASE`).

use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

const SOURCE_EXTENSIONS: &[&str] = &[
    "rs",
    "go",
    "ts",
    "js",
    "tsx",
    "jsx",
    "py",
    "java",
    "kt",
    "rb",
    "cs",
    "scala",
    "yaml",
    "yml",
    "toml",
    "env",
    "properties",
    "cfg",
    "ini",
    "conf",
];

const DEPENDENCY_SUFFIXES: &[&str] = &["_URL", "_SERVICE", "_HOST", "_API", "_ENDPOINT", "_ADDR"];

const NOISE_PREFIXES: &[&str] = &[
    "BASE",
    "APP",
    "LOCAL",
    "CURRENT",
    "WEB",
    "SERVER",
    "MY",
    "THIS",
    "API",
    "PROXY",
    "TARGET",
    "HOME",
    "ROOT",
    "SELF",
    "INTERNAL",
    "DEFAULT",
    "MAIN",
    "PRIMARY",
    "FRONTEND",
    "BACKEND",
    "DATABASE",
    "DB",
    "CACHE",
    "REDIS",
    "POSTGRES",
    "MYSQL",
    "MONGO",
    "RABBIT",
    "KAFKA",
    "ELASTIC",
    "GRAFANA",
    "PROMETHEUS",
];

#[derive(Default)]
pub struct DependencyExtractor;

impl DependencyExtractor {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn is_source_code(ext: &str) -> bool {
        SOURCE_EXTENSIONS.contains(&ext)
    }

    pub(crate) fn strip_suffix(token: &str) -> Option<String> {
        for suffix in DEPENDENCY_SUFFIXES {
            if let Some(stripped) = token.strip_suffix(suffix) {
                return Some(stripped.to_string());
            }
        }
        None
    }

    pub(crate) fn is_noise_prefix(token: &str) -> bool {
        NOISE_PREFIXES.contains(&token)
    }

    pub(crate) fn is_config_token(token: &str) -> bool {
        token.len() > 3
            && token
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    }

    pub(crate) fn has_dependency_signal(content: &str) -> bool {
        DEPENDENCY_SUFFIXES
            .iter()
            .any(|suffix| content.contains(suffix))
    }
}

impl Extractor for DependencyExtractor {
    fn name(&self) -> &'static str {
        "dependency"
    }

    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError> {
        let ext = ctx.extension().to_lowercase();
        if !Self::is_source_code(&ext) {
            return Ok(Vec::new());
        }

        let content = match ctx.content() {
            Some(c) => c,
            None => return Ok(Vec::new()),
        };

        if !Self::has_dependency_signal(content) {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for line in content.lines() {
            let tokens: Vec<&str> = line
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .filter(|t| !t.is_empty())
                .collect();

            for token in tokens {
                if !Self::is_config_token(token) {
                    continue;
                }

                if let Some(service_name_upper) = Self::strip_suffix(token) {
                    if Self::is_noise_prefix(&service_name_upper) {
                        continue;
                    }

                    let service_name = service_name_upper.to_lowercase().replace('_', "-");
                    if service_name.len() <= 2 || seen.contains(&service_name) {
                        continue;
                    }
                    seen.insert(service_name.clone());

                    results.push(DiscoveredSource {
                        binding: SourceBinding {
                            kind: SourceKind::Custom("dependency_signal".to_string()),
                            path: ctx.relative_path().to_string(),
                            description: Some(format!(
                                "Dependency signal: {service_name} (from {token})"
                            )),
                        },
                        suggested_element: Some(service_name),
                        confidence: 0.3,
                    });
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_source_code() {
        assert!(DependencyExtractor::is_source_code("rs"));
        assert!(DependencyExtractor::is_source_code("go"));
        assert!(DependencyExtractor::is_source_code("ts"));
        assert!(DependencyExtractor::is_source_code("py"));
        assert!(DependencyExtractor::is_source_code("yaml"));
        assert!(DependencyExtractor::is_source_code("env"));
    }

    #[test]
    fn test_is_not_source_code() {
        assert!(!DependencyExtractor::is_source_code("txt"));
        assert!(!DependencyExtractor::is_source_code("md"));
        assert!(!DependencyExtractor::is_source_code("png"));
    }

    #[test]
    fn test_is_noise_prefix() {
        assert!(DependencyExtractor::is_noise_prefix("BASE"));
        assert!(DependencyExtractor::is_noise_prefix("DATABASE"));
        assert!(DependencyExtractor::is_noise_prefix("REDIS"));
        assert!(!DependencyExtractor::is_noise_prefix("PAYMENT"));
        assert!(!DependencyExtractor::is_noise_prefix("USER"));
    }

    #[test]
    fn test_is_config_token() {
        assert!(DependencyExtractor::is_config_token("PAYMENT_SERVICE_URL"));
        assert!(DependencyExtractor::is_config_token("USER_HOST_8080"));
        assert!(DependencyExtractor::is_config_token("API_V2"));
        assert!(!DependencyExtractor::is_config_token("lowercase"));
        assert!(!DependencyExtractor::is_config_token("API"));
        assert!(!DependencyExtractor::is_config_token("AB"));
    }

    #[test]
    fn test_has_dependency_signal() {
        assert!(DependencyExtractor::has_dependency_signal(
            "PAYMENT_SERVICE_URL=https://api"
        ));
        assert!(DependencyExtractor::has_dependency_signal(
            "USER_HOST=localhost"
        ));
        assert!(DependencyExtractor::has_dependency_signal(
            "BASE_URL=https://example.com"
        ));
    }

    #[test]
    fn test_strip_suffix() {
        assert_eq!(
            DependencyExtractor::strip_suffix("PAYMENT_SERVICE_URL"),
            Some("PAYMENT_SERVICE".to_string())
        );
        assert_eq!(
            DependencyExtractor::strip_suffix("USER_HOST"),
            Some("USER".to_string())
        );
        assert_eq!(
            DependencyExtractor::strip_suffix("BASE_URL"),
            Some("BASE".to_string())
        );
    }
}
