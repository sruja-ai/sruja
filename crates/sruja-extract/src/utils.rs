//! Shared utilities for extractors.
//!
//! Common parsing functions, title extraction, and YAML content detection
//! used across multiple extractors.

pub mod yaml;

pub use yaml::{extract_title_from_yaml, has_markers};

const YAML_TITLE_PREFIXES: &[&str] = &["title:", "\"title\":", "'title':"];

pub fn extract_title(content: &str) -> Option<String> {
    extract_title_from_yaml(content, YAML_TITLE_PREFIXES)
}

pub fn has_yaml_content(content: &str, markers: &[&str]) -> bool {
    yaml::has_markers(content, markers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_from_yaml_simple() {
        assert_eq!(
            extract_title("title: My API\ninfo:\n  version: 1.0"),
            Some("My API".to_string())
        );
    }

    #[test]
    fn test_extract_title_quoted() {
        assert_eq!(
            extract_title("title: \"Quoted Title\"\ninfo:\n  version: 1.0"),
            Some("Quoted Title".to_string())
        );
    }

    #[test]
    fn test_extract_title_single_quoted() {
        assert_eq!(
            extract_title("title: 'Single Quoted'\ninfo:\n  version: 1.0"),
            Some("Single Quoted".to_string())
        );
    }

    #[test]
    fn test_extract_title_no_title() {
        assert_eq!(extract_title("openapi: 3.0.0\ninfo:\n  version: 1.0"), None);
    }

    #[test]
    fn test_extract_title_empty() {
        assert_eq!(extract_title(""), None);
    }

    #[test]
    fn test_has_yaml_content_openapi() {
        assert!(has_yaml_content(
            "openapi: 3.0.0\ninfo:\n  title: Test",
            &["openapi:", "\"openapi\":"]
        ));
    }

    #[test]
    fn test_has_yaml_content_swagger() {
        assert!(has_yaml_content(
            "swagger: 2.0\ninfo:\n  title: Test",
            &["swagger:", "\"swagger\":"]
        ));
    }

    #[test]
    fn test_has_yaml_content_negative() {
        assert!(!has_yaml_content(
            "name: value\nversion: 1.0",
            &["openapi:", "\"openapi\":"]
        ));
    }
}
