//! Language detection from file extensions and content.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
        assert_eq!(
            detect_language(Path::new("script.py")),
            Some(Language::Python)
        );
        assert_eq!(
            detect_language(Path::new("Main.java")),
            Some(Language::Java)
        );
    }

    #[test]
    fn detect_language_csharp_ruby_php() {
        assert_eq!(
            detect_language(Path::new("Program.cs")),
            Some(Language::CSharp)
        );
        assert_eq!(detect_language(Path::new("app.rb")), Some(Language::Ruby));
        assert_eq!(detect_language(Path::new("index.php")), Some(Language::Php));
    }

    #[test]
    fn detect_language_kotlin_scala() {
        assert_eq!(
            detect_language(Path::new("Main.kt")),
            Some(Language::Kotlin)
        );
        assert_eq!(
            detect_language(Path::new("build.gradle.kts")),
            Some(Language::Kotlin)
        );
        assert_eq!(
            detect_language(Path::new("App.scala")),
            Some(Language::Scala)
        );
        assert_eq!(
            detect_language(Path::new("Script.sc")),
            Some(Language::Scala)
        );
    }

    #[test]
    fn detect_language_c_cpp() {
        assert_eq!(detect_language(Path::new("main.c")), Some(Language::C));
        assert_eq!(detect_language(Path::new("header.h")), Some(Language::C));
        assert_eq!(detect_language(Path::new("main.cpp")), Some(Language::Cpp));
        assert_eq!(detect_language(Path::new("impl.cc")), Some(Language::Cpp));
        assert_eq!(
            detect_language(Path::new("header.hpp")),
            Some(Language::Cpp)
        );
        assert_eq!(
            detect_language(Path::new("header.hxx")),
            Some(Language::Cpp)
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

    #[test]
    fn language_display() {
        assert_eq!(Language::TypeScript.to_string(), "TypeScript");
        assert_eq!(Language::JavaScript.to_string(), "JavaScript");
        assert_eq!(Language::Python.to_string(), "Python");
        assert_eq!(Language::Go.to_string(), "Go");
        assert_eq!(Language::Rust.to_string(), "Rust");
        assert_eq!(Language::Java.to_string(), "Java");
        assert_eq!(Language::CSharp.to_string(), "C#");
        assert_eq!(Language::Ruby.to_string(), "Ruby");
        assert_eq!(Language::Php.to_string(), "PHP");
        assert_eq!(Language::Kotlin.to_string(), "Kotlin");
        assert_eq!(Language::Scala.to_string(), "Scala");
        assert_eq!(Language::C.to_string(), "C");
        assert_eq!(Language::Cpp.to_string(), "C++");
    }
}
