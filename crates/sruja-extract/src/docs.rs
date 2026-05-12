//! Extractor for documentation files (Markdown, AsciiDoc, reStructuredText).

use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

pub struct DocExtractor;

impl Default for DocExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DocExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_doc_file(name: &str) -> bool {
        name.ends_with(".md")
            || name.ends_with(".adoc")
            || name.ends_with(".rst")
            || (name.ends_with(".txt") && Self::is_doc_name(name))
    }

    fn is_doc_name(name: &str) -> bool {
        let upper = name.to_uppercase();
        upper.starts_with("README")
            || upper.starts_with("CHANGELOG")
            || upper.starts_with("CONTRIBUTING")
            || upper.starts_with("LICENSE")
            || upper.starts_with("AUTHORS")
            || upper.starts_with("ARCHITECTURE")
            || upper.starts_with("ADR")
    }

    fn source_kind(name: &str) -> SourceKind {
        if name.to_lowercase().starts_with("readme") {
            SourceKind::Readme
        } else {
            SourceKind::Docs
        }
    }

    fn extract_title(content: &str) -> Option<String> {
        for line in content.lines().take(10) {
            let trimmed = line.trim();
            if let Some(title) = trimmed.strip_prefix("# ") {
                return Some(title.trim().to_string());
            }
            if let Some(title) = trimmed.strip_prefix("= ") {
                return Some(title.trim().to_string());
            }
        }
        None
    }
}

impl Extractor for DocExtractor {
    fn name(&self) -> &'static str {
        "docs"
    }

    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError> {
        let name = ctx.file_name_lower();
        if !Self::is_doc_file(&name) {
            return Ok(Vec::new());
        }

        let description = ctx
            .content()
            .and_then(Self::extract_title)
            .unwrap_or_else(|| "Discovered documentation".to_string());

        let suggested_element = ctx.parent_dir_name().map(|s| s.to_string());

        let confidence = if name.starts_with("readme") {
            0.7
        } else if Self::is_doc_name(&name) {
            0.65
        } else {
            0.5
        };

        Ok(vec![DiscoveredSource {
            binding: SourceBinding {
                kind: Self::source_kind(&name),
                path: ctx.relative_path().to_string(),
                description: Some(description),
            },
            suggested_element,
            confidence,
        }])
    }
}
