//! Extractor for documentation files (Markdown, AsciiDoc, reStructuredText).

use crate::utils::yaml;
use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

#[derive(Default)]
pub struct DocExtractor;

impl DocExtractor {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn is_doc_file(name: &str) -> bool {
        name.ends_with(".md")
            || name.ends_with(".adoc")
            || name.ends_with(".rst")
            || (name.ends_with(".txt") && Self::is_doc_name(name))
    }

    pub(crate) fn is_doc_name(name: &str) -> bool {
        let upper = name.to_uppercase();
        upper.starts_with("README")
            || upper.starts_with("CHANGELOG")
            || upper.starts_with("CONTRIBUTING")
            || upper.starts_with("LICENSE")
            || upper.starts_with("AUTHORS")
            || upper.starts_with("ARCHITECTURE")
            || upper.starts_with("ADR")
    }

    pub(crate) fn source_kind(name: &str) -> SourceKind {
        if name.to_lowercase().starts_with("readme") {
            SourceKind::Readme
        } else {
            SourceKind::Docs
        }
    }

    pub(crate) fn extract_title(content: &str) -> Option<String> {
        for line in content.lines().take(20) {
            let trimmed = line.trim();
            if let Some(title) = trimmed.strip_prefix("# ") {
                return Some(title.trim().to_string());
            }
            if let Some(title) = trimmed.strip_prefix("= ") {
                return Some(title.trim().to_string());
            }
        }
        yaml::extract_title_from_yaml(content, &["title:", "\"title\":", "'title':"])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_doc_file_markdown() {
        assert!(DocExtractor::is_doc_file("README.md"));
        assert!(DocExtractor::is_doc_file("guide.adoc"));
        assert!(DocExtractor::is_doc_file("doc.rst"));
    }

    #[test]
    fn test_is_doc_file_txt_docs() {
        assert!(DocExtractor::is_doc_file("README.txt"));
        assert!(DocExtractor::is_doc_file("CHANGELOG.txt"));
    }

    #[test]
    fn test_is_not_doc_file() {
        assert!(!DocExtractor::is_doc_file("config.yaml"));
        assert!(!DocExtractor::is_doc_file("data.txt"));
    }

    #[test]
    fn test_is_doc_name() {
        assert!(DocExtractor::is_doc_name("README.md"));
        assert!(DocExtractor::is_doc_name("CHANGELOG.md"));
        assert!(DocExtractor::is_doc_name("CONTRIBUTING.md"));
        assert!(DocExtractor::is_doc_name("LICENSE.txt"));
        assert!(DocExtractor::is_doc_name("ARCHITECTURE.md"));
    }

    #[test]
    fn test_source_kind() {
        assert_eq!(DocExtractor::source_kind("README.md"), SourceKind::Readme);
        assert_eq!(DocExtractor::source_kind("CHANGELOG.md"), SourceKind::Docs);
        assert_eq!(DocExtractor::source_kind("guide.md"), SourceKind::Docs);
    }

    #[test]
    fn test_extract_title_markdown() {
        assert_eq!(
            DocExtractor::extract_title("# My Title\nContent"),
            Some("My Title".to_string())
        );
    }

    #[test]
    fn test_extract_title_asciidoc() {
        assert_eq!(
            DocExtractor::extract_title("= AsciiDoc Title\nContent"),
            Some("AsciiDoc Title".to_string())
        );
    }

    #[test]
    fn test_extract_title_yaml() {
        assert_eq!(
            DocExtractor::extract_title("title: My Title\ninfo:\n  version: 1.0"),
            Some("My Title".to_string())
        );
    }

    #[test]
    fn test_extract_title_no_title() {
        assert_eq!(DocExtractor::extract_title("No title here"), None);
    }
}
