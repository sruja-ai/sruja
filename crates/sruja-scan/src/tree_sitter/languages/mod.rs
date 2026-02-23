//! Language-specific parsers.

pub mod go;
pub mod python;
pub mod rust;
pub mod typescript;

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
