//! Extractor for Helm chart files.
//!
//! Detects `Chart.yaml` as the primary chart definition and
//! `values.yaml` for configuration.

use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

#[derive(Default)]
pub struct HelmExtractor;

impl HelmExtractor {
    pub fn new() -> Self {
        Self
    }

    fn extract_chart_name(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("name:") {
                let name = rest.trim().trim_matches('"').trim_matches('\'');
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    fn extract_chart_description(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("description:") {
                let desc = rest.trim().trim_matches('"').trim_matches('\'');
                if !desc.is_empty() {
                    return Some(desc.to_string());
                }
            }
        }
        None
    }
}

impl Extractor for HelmExtractor {
    fn name(&self) -> &'static str {
        "helm"
    }

    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError> {
        let name = ctx.file_name_lower();

        let is_chart = name == "chart.yaml" || name == "chart.yml";
        let is_values = name == "values.yaml" || name == "values.yml";

        if !is_chart && !is_values {
            return Ok(Vec::new());
        }

        let content = match ctx.content() {
            Some(c) => c,
            None => return Ok(Vec::new()),
        };

        if is_chart {
            if !content.contains("apiVersion:") || !content.contains("name:") {
                return Ok(Vec::new());
            }

            let chart_name = Self::extract_chart_name(content);
            let desc = Self::extract_chart_description(content)
                .or_else(|| chart_name.as_ref().map(|n| format!("Helm chart: {n}")))
                .unwrap_or_else(|| "Helm chart".to_string());

            return Ok(vec![DiscoveredSource {
                binding: SourceBinding {
                    kind: SourceKind::Helm,
                    path: ctx.relative_path().to_string(),
                    description: Some(desc),
                },
                suggested_element: chart_name,
                confidence: 0.85,
            }]);
        }

        // values.yaml — only count it if a Chart.yaml is in the same or parent directory
        let parent = ctx.path.parent();
        let has_chart_yaml = parent
            .map(|p| p.join("Chart.yaml").exists() || p.join("Chart.yml").exists())
            .unwrap_or(false);

        let has_chart_in_parent = parent
            .and_then(|p| p.parent())
            .map(|p| p.join("Chart.yaml").exists() || p.join("Chart.yml").exists())
            .unwrap_or(false);

        if !has_chart_yaml && !has_chart_in_parent {
            return Ok(Vec::new());
        }

        let suggested_element = ctx.parent_dir_name().map(|s| s.to_string());

        Ok(vec![DiscoveredSource {
            binding: SourceBinding {
                kind: SourceKind::Helm,
                path: ctx.relative_path().to_string(),
                description: Some("Helm values configuration".to_string()),
            },
            suggested_element,
            confidence: 0.6,
        }])
    }
}
