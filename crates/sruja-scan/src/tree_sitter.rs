//! Tree-sitter based code parsing for architecture extraction.
//!
//! This module parses source code files using Tree-sitter grammars to extract:
//! - Module/package structure from file paths
//! - Import statements (dependencies)
//! - Export statements (public interfaces)
//! - Function and class definitions (components)

mod detector;
mod languages;

use std::collections::HashMap;
use std::path::Path;

use crate::graph::{Edge, EdgeEvidence, EdgeKind, Graph, Node, NodeKind};
use crate::ScanError;

use self::detector::Language;
use self::languages::ParsedFile;

pub use detector::detect_language;

pub struct ScanConfig {
    pub include_tests: bool,
    pub include_node_modules: bool,
    pub max_file_size: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            include_tests: false,
            include_node_modules: false,
            max_file_size: 500 * 1024,
        }
    }
}

pub fn scan_with_tree_sitter(repo_root: &Path, config: &ScanConfig) -> Result<Graph, ScanError> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut module_nodes: HashMap<String, Node> = HashMap::new();
    let mut file_imports: HashMap<String, Vec<String>> = HashMap::new();

    let walker = build_walker(repo_root, config);

    for entry in walker {
        let entry = entry.map_err(|e| ScanError::Walk(e.to_string()))?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Some(language) = detect_language(path) {
            if let Ok(content) = std::fs::read_to_string(path) {
                if content.len() > config.max_file_size {
                    continue;
                }

                match parse_file(path, &content, language) {
                    Some(parsed) => {
                        let file_id = file_to_id(repo_root, path);

                        let parent_module = path
                            .parent()
                            .and_then(|p| p.strip_prefix(repo_root).ok())
                            .map(|p| p.to_string_lossy().replace(['/', '\\'], "_"))
                            .unwrap_or_else(|| "root".to_string());

                        let module_id = format!("module:{}", parent_module);
                        if !module_nodes.contains_key(&module_id) {
                            module_nodes.insert(
                                module_id.clone(),
                                Node {
                                    id: module_id.clone(),
                                    kind: NodeKind::Module,
                                    label: parent_module.clone(),
                                    technology: Some(language.to_string()),
                                    path: Some(parent_module.clone()),
                                    metadata: HashMap::new(),
                                },
                            );
                        }

                        let node = Node {
                            id: file_id.clone(),
                            kind: infer_node_kind(&parsed, path),
                            label: parsed.name.clone(),
                            technology: Some(language.to_string()),
                            path: Some(path.to_string_lossy().to_string()),
                            metadata: HashMap::new(),
                        };
                        nodes.push(node);

                        edges.push(Edge {
                            source: module_id.clone(),
                            target: file_id.clone(),
                            kind: EdgeKind::Calls,
                            evidence: vec![EdgeEvidence {
                                rule: "contains".to_string(),
                                file: Some(path.to_string_lossy().to_string()),
                                line: None,
                                detail: Some("module contains this file".to_string()),
                            }],
                        });

                        for import in &parsed.imports {
                            let target_id = resolve_import(repo_root, path, import);
                            file_imports
                                .entry(file_id.clone())
                                .or_default()
                                .push(target_id);
                        }

                        for export in &parsed.exports {
                            let export_node_id = format!("{}#{}", file_id, export);
                            nodes.push(Node {
                                id: export_node_id.clone(),
                                kind: NodeKind::Module,
                                label: export.clone(),
                                technology: Some(language.to_string()),
                                path: Some(path.to_string_lossy().to_string()),
                                metadata: HashMap::new(),
                            });
                            edges.push(Edge {
                                source: file_id.clone(),
                                target: export_node_id,
                                kind: EdgeKind::Calls,
                                evidence: vec![EdgeEvidence {
                                    rule: "exports".to_string(),
                                    file: Some(path.to_string_lossy().to_string()),
                                    line: None,
                                    detail: Some(format!("exports {}", export)),
                                }],
                            });
                        }
                    }
                    None => {
                        log::warn!("Parse failed: {} ({})", path.display(), language);
                    }
                }
            }
        }
    }

    for (_module_id, node) in module_nodes {
        nodes.push(node);
    }

    for (source, targets) in &file_imports {
        for target in targets {
            edges.push(Edge {
                source: source.clone(),
                target: target.clone(),
                kind: EdgeKind::Calls,
                evidence: vec![EdgeEvidence {
                    rule: "imports".to_string(),
                    file: None,
                    line: None,
                    detail: Some(format!("imports from {}", target)),
                }],
            });
        }
    }

    Ok(Graph {
        metadata: HashMap::new(),
        nodes,
        edges,
    })
}

fn build_walker(repo_root: &Path, config: &ScanConfig) -> ignore::Walk {
    let mut builder = ignore::WalkBuilder::new(repo_root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true);

    if !config.include_node_modules {
        builder.filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "node_modules" && name != "target" && name != "dist" && name != "build"
        });
    }

    if !config.include_tests {
        builder.filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.contains("test") && !name.contains("spec") && name != "__tests__"
        });
    }

    builder.build()
}

fn file_to_id(repo_root: &Path, file_path: &Path) -> String {
    file_path
        .strip_prefix(repo_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .replace(['/', '\\', '.'], "_")
        .trim_start_matches('_')
        .to_string()
}

fn parse_file(path: &Path, content: &str, language: Language) -> Option<ParsedFile> {
    match language {
        Language::TypeScript | Language::JavaScript => languages::typescript::parse(path, content),
        Language::Python => languages::python::parse(path, content),
        Language::Go => languages::go::parse(path, content),
        Language::Rust => languages::rust::parse(path, content),
        Language::Java => languages::java::parse(path, content),
        Language::CSharp => languages::csharp::parse(path, content),
        Language::Ruby => languages::ruby::parse(path, content),
        Language::Php => languages::php::parse(path, content),
        Language::Kotlin => languages::kotlin::parse(path, content),
        Language::Scala => languages::scala::parse(path, content),
        Language::C => languages::c::parse(path, content),
        Language::Cpp => languages::cpp::parse(path, content),
    }
}

fn infer_node_kind(parsed: &ParsedFile, path: &Path) -> NodeKind {
    let path_str = path.to_string_lossy().to_lowercase();
    let name_lower = parsed.name.to_lowercase();

    if path_str.contains("/api/")
        || path_str.contains("/controller")
        || path_str.contains("/handler")
        || path_str.contains("/routes/")
        || name_lower.contains("api")
        || name_lower.contains("controller")
        || name_lower.contains("handler")
        || name_lower.contains("routes")
    {
        return NodeKind::Service;
    }

    if path_str.contains("/db/")
        || path_str.contains("/database/")
        || path_str.contains("/repository/")
        || path_str.contains("/dao/")
        || path_str.contains("/model/")
        || path_str.contains("/entity/")
        || name_lower.contains("repository")
        || name_lower.contains("model")
        || name_lower.contains("entity")
        || name_lower.contains("db")
        || name_lower.contains("schema")
    {
        return NodeKind::Database;
    }

    if path_str.contains("/external/")
        || path_str.contains("/thirdparty/")
        || path_str.contains("/vendor/")
        || name_lower.contains("client")
        || name_lower.contains("gateway")
    {
        return NodeKind::ExternalApi;
    }

    NodeKind::Module
}

/// Common extensions to try when resolving extensionless imports (TypeScript/JavaScript).
const RESOLVE_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"];

fn resolve_import(repo_root: &Path, from_file: &Path, import_path: &str) -> String {
    if import_path.starts_with('.') || import_path.starts_with('/') {
        let from_dir = from_file.parent().unwrap_or(repo_root);
        let base = from_dir.join(import_path);

        // Canonicalize repo_root for consistent strip_prefix (handles /var vs /private/var on macOS)
        let repo_canon = match repo_root.canonicalize() {
            Ok(p) => p,
            Err(_) => repo_root.to_path_buf(),
        };

        let to_id = |p: &Path| {
            p.strip_prefix(&repo_canon)
                .ok()
                .map(|s| s.to_string_lossy().replace(['/', '\\', '.'], "_"))
        };

        // Try exact path first
        if let Ok(resolved) = base.canonicalize() {
            if let Some(id) = to_id(&resolved) {
                return id;
            }
        }

        // Try with common extensions (e.g. './a' -> './a.ts')
        for ext in RESOLVE_EXTENSIONS {
            let candidate = base.with_extension(ext.trim_start_matches('.'));
            if candidate.exists() {
                if let Ok(resolved) = candidate.canonicalize() {
                    if let Some(id) = to_id(&resolved) {
                        return id;
                    }
                }
            }
        }

        // Try directory index (e.g. './foo' -> './foo/index.ts')
        const INDEX_FILES: &[&str] = &[
            "index.ts",
            "index.tsx",
            "index.js",
            "index.jsx",
            "index.mjs",
        ];
        for index_file in INDEX_FILES {
            let candidate = base.join(index_file);
            if candidate.exists() {
                if let Ok(resolved) = candidate.canonicalize() {
                    if let Some(id) = to_id(&resolved) {
                        return id;
                    }
                }
            }
        }
    }

    import_path
        .replace(['/', '@', '-', '.'], "_")
        .trim_start_matches('_')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let config = ScanConfig::default();
        let graph = scan_with_tree_sitter(dir.path(), &config).unwrap();
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn test_scan_typescript_file() {
        let dir = tempfile::tempdir().unwrap();
        let ts_file = dir.path().join("service.ts");
        std::fs::write(
            &ts_file,
            r#"
import { db } from './database';
import { User } from './models';

export class UserService {
    getUser(id: string) {
        return db.find(id);
    }
}
"#,
        )
        .unwrap();

        let config = ScanConfig::default();
        let graph = scan_with_tree_sitter(dir.path(), &config).unwrap();

        assert!(!graph.nodes.is_empty(), "Should have at least one node");
    }
}
