//! Language detection from file extensions and content.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    TypeScript,
    JavaScript,
    Python,
    Go,
    Rust,
    Java,
    CSharp,
    Ruby,
    Php,
    Kotlin,
    Scala,
    C,
    Cpp,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::TypeScript => write!(f, "TypeScript"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::Python => write!(f, "Python"),
            Language::Go => write!(f, "Go"),
            Language::Rust => write!(f, "Rust"),
            Language::Java => write!(f, "Java"),
            Language::CSharp => write!(f, "C#"),
            Language::Ruby => write!(f, "Ruby"),
            Language::Php => write!(f, "PHP"),
            Language::Kotlin => write!(f, "Kotlin"),
            Language::Scala => write!(f, "Scala"),
            Language::C => write!(f, "C"),
            Language::Cpp => write!(f, "C++"),
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
        "java" => Some(Language::Java),
        "cs" => Some(Language::CSharp),
        "rb" => Some(Language::Ruby),
        "php" => Some(Language::Php),
        "kt" | "kts" => Some(Language::Kotlin),
        "scala" | "sc" => Some(Language::Scala),
        "c" | "h" => Some(Language::C),
        "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Some(Language::Cpp),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn is_source_file(path: &Path) -> bool {
    detect_language(path).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_language_rust() {
        assert_eq!(
            detect_language(Path::new("src/lib.rs")),
            Some(Language::Rust)
        );
    }

    #[test]
    fn detect_language_typescript_and_js() {
        assert_eq!(
            detect_language(Path::new("app.ts")),
            Some(Language::TypeScript)
        );
        assert_eq!(
            detect_language(Path::new("app.tsx")),
            Some(Language::TypeScript)
        );
        assert_eq!(
            detect_language(Path::new("index.mjs")),
            Some(Language::JavaScript)
        );
        assert_eq!(
            detect_language(Path::new("index.cjs")),
            Some(Language::JavaScript)
        );
    }

    #[test]
    fn detect_language_go_python_java() {
        assert_eq!(detect_language(Path::new("main.go")), Some(Language::Go));
        assert_eq!(detect_language(Path::new("script.py")), Some(Language::Python));
        assert_eq!(
            detect_language(Path::new("Main.java")),
            Some(Language::Java)
        );
    }

    #[test]
    fn detect_language_unknown_returns_none() {
        assert_eq!(detect_language(Path::new("file.txt")), None);
        assert_eq!(detect_language(Path::new("README")), None);
        assert_eq!(detect_language(Path::new("noext")), None);
    }

    #[test]
    fn is_source_file_true_for_known_extensions() {
        assert!(is_source_file(Path::new("lib.rs")));
        assert!(is_source_file(Path::new("main.go")));
    }

    #[test]
    fn is_source_file_false_for_unknown() {
        assert!(!is_source_file(Path::new("data.json")));
    }
}
