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

pub struct DependencyExtractor;

impl Default for DependencyExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_source_code(ext: &str) -> bool {
        SOURCE_EXTENSIONS.contains(&ext)
    }

    fn strip_suffix(token: &str) -> Option<String> {
        for suffix in DEPENDENCY_SUFFIXES {
            if let Some(stripped) = token.strip_suffix(suffix) {
                return Some(stripped.to_string());
            }
        }
        None
    }

    fn is_config_token(token: &str) -> bool {
        token.len() > 3
            && token
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
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

        let has_signal = DEPENDENCY_SUFFIXES
            .iter()
            .any(|suffix| content.contains(suffix));
        if !has_signal {
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
                    if NOISE_PREFIXES.contains(&service_name_upper.as_str()) {
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
