//! Stub tree-sitter module for WebAssembly target.

use crate::Graph;
use crate::ScanError;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct ScanConfig {
    pub include_tests: bool,
    pub include_node_modules: bool,
    pub exclude_examples: bool,
    pub exclude_benches: bool,
    pub exclude_fixtures: bool,
    pub exclude_docs: bool,
    pub max_file_size: usize,
    pub classification_rules_path: Option<PathBuf>,
    pub incremental: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            include_tests: false,
            include_node_modules: false,
            exclude_examples: true,
            exclude_benches: true,
            exclude_fixtures: true,
            exclude_docs: true,
            max_file_size: 500 * 1024,
            classification_rules_path: None,
            incremental: false,
        }
    }
}

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
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedFile {
    pub name: String,
    pub path: String,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Definition {
    pub name: String,
    pub kind: DefinitionKind,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DefinitionKind {
    Function,
    Class,
    Interface,
    Struct,
    Enum,
    Constant,
    Variable,
}

pub fn detect_language(_path: &Path) -> Option<Language> {
    None
}

pub fn parse_file(_path: &Path, _content: &str, _language: Language) -> Option<ParsedFile> {
    None
}

pub fn scan_with_tree_sitter(_repo_root: &Path, _config: &ScanConfig) -> Result<Graph, ScanError> {
    Ok(Graph::new())
}

pub fn build_walker_internal(repo_root: &Path, config: &ScanConfig) -> ignore::Walk {
    let mut builder = ignore::WalkBuilder::new(repo_root);
    builder
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true);
    builder.add_custom_ignore_filename(".srujaignore");

    let config_clone = config.clone();
    builder.filter_entry(move |e| {
        let path = e.path();
        !crate::scan_scope::should_exclude_with_config(path, &config_clone)
    });

    builder.build()
}
