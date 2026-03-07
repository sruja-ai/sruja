//! Language-specific parsers.

pub mod c;
pub mod cpp;
pub mod csharp;
pub mod go;
pub mod java;
pub mod kotlin;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod scala;
pub mod typescript;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
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

use std::path::Path;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParsedFile {
    pub name: String,
    pub path: String,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Definition {
    pub name: String,
    pub kind: DefinitionKind,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionKind {
    Function,
    Class,
    Interface,
    Struct,
    Enum,
    Constant,
    #[allow(dead_code)]
    Variable,
}

#[allow(dead_code)]
pub trait SourceLanguage {
    fn parse(path: &Path, content: &str) -> Option<ParsedFile>;
}
