//! Language detection from file extensions and content.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    TypeScript,
    JavaScript,
    Python,
    Go,
    Rust,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::TypeScript => write!(f, "TypeScript"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::Python => write!(f, "Python"),
            Language::Go => write!(f, "Go"),
            Language::Rust => write!(f, "Rust"),
        }
    }
}

pub fn detect_language(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    match ext.as_str() {
        "ts" | "tsx" => Some(Language::TypeScript),
        "js" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
        "py" => Some(Language::Python),
        "go" => Some(Language::Go),
        "rs" => Some(Language::Rust),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn is_source_file(path: &Path) -> bool {
    detect_language(path).is_some()
}
