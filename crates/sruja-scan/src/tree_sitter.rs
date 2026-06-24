//! Tree-sitter based code parsing for architecture extraction.
//!
//! This module parses source code files using Tree-sitter grammars to extract:
//! - Module/package structure from file paths
//! - Import statements (dependencies)
//! - Export statements (public interfaces)
//! - Function and class definitions (components)

mod classifier;
mod detector;
mod languages;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::graph::{Edge, EdgeEvidence, EdgeKind, Graph, Node, NodeKind};
use crate::scan_scope::should_exclude_with_config;
use crate::ScanError;
use rayon::prelude::*;

pub use detector::{detect_language, Language};
pub use languages::{Definition, DefinitionKind, ParsedFile};

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

#[tracing::instrument(skip(repo_root, config))]
pub fn scan_with_tree_sitter(repo_root: &Path, config: &ScanConfig) -> Result<Graph, ScanError> {
    tracing::info!("Scanning with tree-sitter: {:?}", repo_root);
    let repo_canon = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());

    #[derive(Default)]
    struct ScanDiagnostics {
        language_files_seen: usize,
        collected_files: usize,
        read_failed: usize,
        skipped_large: usize,
        parse_failed: usize,
        read_failed_examples: Vec<String>,
        skipped_large_examples: Vec<String>,
        parse_failed_examples: Vec<String>,
    }

    fn push_example(examples: &mut Vec<String>, s: String) {
        if examples.len() < 3 {
            examples.push(s);
        }
    }

    let go_mod_path = repo_root.join("go.mod");
    let go_module_path: Option<String> = if crate::is_safe_path(&go_mod_path, &repo_canon) {
        std::fs::read_to_string(&go_mod_path)
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|l| l.starts_with("module "))
                    .map(|l| l.trim_start_matches("module ").trim().to_string())
            })
    } else {
        None
    };

    let mut manifest = if config.incremental {
        crate::manifest::ScanManifest::load(repo_root).unwrap_or_default()
    } else {
        crate::manifest::ScanManifest::new()
    };

    let mut ast_cache = if config.incremental {
        crate::ast_cache::AstCache::load(repo_root).unwrap_or_default()
    } else {
        crate::ast_cache::AstCache::new()
    };

    let mut visited_paths = std::collections::HashSet::new();
    let mut parsed: Vec<(PathBuf, String, Language, ParsedFile)> = Vec::new();
    let mut collected = Vec::new();
    let mut diagnostics = ScanDiagnostics::default();

    // 1. Walk and collect eligible (path, content, language).
    {
        let walker = build_walker(repo_root, config);
        for entry in walker {
            let entry = entry.map_err(|e| ScanError::Walk(e.to_string()))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if !crate::is_safe_path(path, &repo_canon) {
                continue;
            }
            if let Some(language) = detect_language(path) {
                diagnostics.language_files_seen += 1;
                let path_key = rel_path(repo_root, &repo_canon, path);
                visited_paths.insert(path_key.clone());

                let metadata_res = std::fs::metadata(path);
                let size = metadata_res.as_ref().map(|m| m.len()).unwrap_or(0);

                let mut use_cache = false;
                if config.incremental {
                    if let Some(entry) = manifest.entries.get(&path_key) {
                        if entry.size_bytes == size {
                            if let Ok(current_hash) = crate::manifest::ScanManifest::hash_file(path)
                            {
                                if entry.blake3_hash == current_hash {
                                    if let Some((cached_lang, cached_pf)) =
                                        ast_cache.files.get(&path_key)
                                    {
                                        if let Ok(content) = std::fs::read_to_string(path) {
                                            diagnostics.collected_files += 1;
                                            parsed.push((
                                                path.to_path_buf(),
                                                content,
                                                *cached_lang,
                                                cached_pf.clone(),
                                            ));
                                            use_cache = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if !use_cache {
                    match std::fs::read_to_string(path) {
                        Ok(content) => {
                            if content.len() <= config.max_file_size {
                                diagnostics.collected_files += 1;
                                collected.push((path.to_path_buf(), content, language));
                            } else {
                                diagnostics.skipped_large += 1;
                                push_example(
                                    &mut diagnostics.skipped_large_examples,
                                    rel_path(repo_root, &repo_canon, path),
                                );
                            }
                        }
                        Err(_) => {
                            diagnostics.read_failed += 1;
                            push_example(
                                &mut diagnostics.read_failed_examples,
                                rel_path(repo_root, &repo_canon, path),
                            );
                        }
                    }
                }
            }
        }
    }

    // 2. Parallel parse; keep content for merge (infer_node_kind needs it).
    enum ParseOutcome {
        Ok(PathBuf, String, Language, ParsedFile),
        Err(String),
    }

    let outcomes: Vec<ParseOutcome> = collected
        .par_iter()
        .map(|(path, content, language)| {
            parse_file(path.as_path(), content, *language).map_or_else(
                || ParseOutcome::Err(rel_path(repo_root, &repo_canon, path.as_path())),
                |p| ParseOutcome::Ok(path.clone(), content.clone(), *language, p),
            )
        })
        .collect();

    for outcome in outcomes {
        match outcome {
            ParseOutcome::Ok(path, content, language, parsed_file) => {
                let path_key = rel_path(repo_root, &repo_canon, &path);

                if config.incremental {
                    if let Ok(hash) = crate::manifest::ScanManifest::hash_file(&path) {
                        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                        manifest.entries.insert(
                            path_key.clone(),
                            crate::manifest::ManifestEntry {
                                size_bytes: size,
                                blake3_hash: hash,
                            },
                        );
                        ast_cache
                            .files
                            .insert(path_key.clone(), (language, parsed_file.clone()));
                    }
                }

                parsed.push((path, content, language, parsed_file));
            }
            ParseOutcome::Err(rel) => {
                diagnostics.parse_failed += 1;
                push_example(&mut diagnostics.parse_failed_examples, rel);
            }
        }
    }

    if config.incremental {
        let mut deleted_keys = Vec::new();
        for key in manifest.entries.keys() {
            if !visited_paths.contains(key) {
                deleted_keys.push(key.clone());
            }
        }
        for key in deleted_keys {
            manifest.entries.remove(&key);
            ast_cache.files.remove(&key);
        }

        let _ = manifest.save(repo_root);
        let _ = ast_cache.save(repo_root);
    }

    // 3. Build file_path_to_id from paths so resolve_import_improved works in merge.
    let file_path_to_id: HashMap<String, String> = parsed
        .iter()
        .filter_map(|(path, _, _, _)| {
            let file_id = file_to_id(repo_root, path);
            path.canonicalize().ok().and_then(|canon_path| {
                canon_path
                    .strip_prefix(&repo_canon)
                    .ok()
                    .map(|rel| (rel.to_string_lossy().to_string(), file_id))
            })
        })
        .collect();

    // 4. Single-threaded merge: same logic as before.
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut module_nodes: HashMap<String, Node> = HashMap::new();
    let mut file_imports: HashMap<String, Vec<String>> = HashMap::new();
    let mut module_imports: HashMap<String, Vec<String>> = HashMap::new();

    let engine = if let Some(ref config_path) = config.classification_rules_path {
        classifier::ClassificationEngine::load_from_file(config_path).unwrap_or_else(|e| {
            tracing::warn!(
                "Failed to load classification rules from {:?}: {}. Using default rules.",
                config_path,
                e
            );
            classifier::ClassificationEngine::default()
        })
    } else {
        classifier::ClassificationEngine::default()
    };
    for (path, content, language, parsed) in &parsed {
        let file_id = file_to_id(repo_root, path);

        let parent_module = path
            .parent()
            .and_then(|p| p.strip_prefix(repo_root).ok())
            .map(|p| p.to_string_lossy().replace(['/', '\\'], "_"))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "root".to_string());

        let module_id = format!("module:{}", parent_module);
        if !module_nodes.contains_key(&module_id) {
            module_nodes.insert(
                module_id.clone(),
                Node {
                    id: module_id.clone(),
                    kind: NodeKind::new(NodeKind::MODULE),
                    label: parent_module.clone(),
                    technology: Some(language.to_string()),
                    path: Some(parent_module.clone()),
                    metadata: HashMap::new(),
                    canonical_id: None,
                    aliases: Vec::new(),
                    owner: None,
                    domain: None,
                    criticality: None,
                    sources: Vec::new(),
                    confidence: None,
                    ..Default::default()
                },
            );
        }

        let (kind, classification_metadata) =
            infer_node_kind(parsed, path, content.as_str(), &engine);
        let confidence = classification_metadata
            .get("classification.confidence")
            .and_then(|c| c.parse::<u8>().ok());

        let node = Node {
            id: file_id.clone(),
            kind,
            label: parsed.name.clone(),
            technology: Some(language.to_string()),
            path: Some(path.to_string_lossy().to_string()),
            metadata: classification_metadata,
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence,
            ..Default::default()
        };
        nodes.push(node);

        let extracted_comments = extract_comments_from_content(content);
        for comment in extracted_comments {
            let note_id = format!("note:{}:line{}", file_id, comment.line);
            nodes.push(Node {
                id: note_id.clone(),
                kind: NodeKind::new("Note"),
                label: format!("{}: {}", comment.keyword, comment.text),
                technology: Some(language.to_string()),
                path: Some(path.to_string_lossy().to_string()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("line".to_string(), comment.line.to_string());
                    meta.insert("keyword".to_string(), comment.keyword);
                    meta
                },
                ..Default::default()
            });
            edges.push(Edge {
                source: note_id,
                target: file_id.clone(),
                kind: EdgeKind::new("explains"),
                evidence: vec![EdgeEvidence {
                    rule: "extracted_comment".to_string(),
                    file: Some(path.to_string_lossy().to_string()),
                    line: Some(comment.line as u32),
                    detail: Some(format!("explains line {}", comment.line)),
                }],
                confidence: Default::default(),
            });
        }

        edges.push(Edge {
            source: module_id.clone(),
            target: file_id.clone(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            evidence: vec![EdgeEvidence {
                rule: "contains".to_string(),
                file: Some(path.to_string_lossy().to_string()),
                line: None,
                detail: Some("module contains this file".to_string()),
            }],
            confidence: Default::default(),
        });

        for import in &parsed.imports {
            let target_id = resolve_import_improved(
                repo_root,
                &repo_canon,
                path,
                import,
                &file_path_to_id,
                go_module_path.as_deref(),
                *language,
            );
            file_imports
                .entry(file_id.clone())
                .or_default()
                .push(target_id.clone());

            let target_module = extract_module_from_id(&target_id);
            if target_module != parent_module {
                module_imports
                    .entry(parent_module.clone())
                    .or_default()
                    .push(target_module);
            }
        }

        for export in &parsed.exports {
            let export_node_id = format!("{}#{}", file_id, export);
            nodes.push(Node {
                id: export_node_id.clone(),
                kind: NodeKind::new(NodeKind::MODULE),
                label: export.clone(),
                technology: Some(language.to_string()),
                path: Some(path.to_string_lossy().to_string()),
                metadata: HashMap::new(),
                canonical_id: None,
                aliases: Vec::new(),
                owner: None,
                domain: None,
                criticality: None,
                sources: Vec::new(),
                confidence: None,
                ..Default::default()
            });
            edges.push(Edge {
                source: file_id.clone(),
                target: export_node_id,
                kind: EdgeKind::new(EdgeKind::CALLS),
                evidence: vec![EdgeEvidence {
                    rule: "exports".to_string(),
                    file: Some(path.to_string_lossy().to_string()),
                    line: None,
                    detail: Some(format!("exports {}", export)),
                }],
                confidence: Default::default(),
            });
        }

        let mut definitions_sorted = parsed.definitions.clone();
        definitions_sorted.sort_by_key(|d| d.line);

        for (i, def) in definitions_sorted.iter().enumerate() {
            let symbol_kind = definition_kind_to_symbol_kind(def.kind);
            let end_line = if i + 1 < definitions_sorted.len() {
                definitions_sorted[i + 1].line.saturating_sub(1).max(def.line)
            } else {
                def.line + 60
            };

            let symbol_node_id = format!("{}:{}", file_id, def.name);
            let mut metadata = HashMap::new();
            metadata.insert("line".to_string(), def.line.to_string());
            metadata.insert("end_line".to_string(), end_line.to_string());
            metadata.insert("symbol_kind".to_string(), symbol_kind.to_string());
            metadata.insert("language".to_string(), language.to_string());

            nodes.push(Node {
                id: symbol_node_id.clone(),
                kind: NodeKind::new(symbol_kind),
                label: def.name.clone(),
                technology: Some(language.to_string()),
                path: Some(path.to_string_lossy().to_string()),
                metadata,
                canonical_id: None,
                aliases: Vec::new(),
                owner: None,
                domain: None,
                criticality: None,
                sources: Vec::new(),
                confidence: None,
                ..Default::default()
            });

            edges.push(Edge {
                source: file_id.clone(),
                target: symbol_node_id.clone(),
                kind: EdgeKind::new("defines"),
                evidence: vec![EdgeEvidence {
                    rule: "defines_symbol".to_string(),
                    file: Some(path.to_string_lossy().to_string()),
                    line: Some(def.line as u32),
                    detail: Some(format!("defines {}", def.name)),
                }],
                confidence: Default::default(),
            });
        }
    }

    for (_module_id, node) in module_nodes {
        nodes.push(node);
    }

    let symbols_by_file: HashMap<String, Vec<(String, String)>> = {
        let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
        for node in &nodes {
            if let Some(idx) = node.id.rfind(':') {
                let file_part = &node.id[..idx];
                map.entry(file_part.to_string())
                    .or_default()
                    .push((node.id.clone(), node.label.clone()));
            }
        }
        map
    };

    for (source, targets) in &file_imports {
        for target in targets {
            edges.push(Edge {
                source: source.clone(),
                target: target.clone(),
                kind: EdgeKind::new(EdgeKind::CALLS),
                evidence: vec![EdgeEvidence {
                    rule: "imports".to_string(),
                    file: None,
                    line: None,
                    detail: Some(format!("imports from {}", target)),
                }],
                confidence: Default::default(),
            });

            if let Some(symbols) = symbols_by_file.get(target) {
                for (symbol_id, label) in symbols {
                    edges.push(Edge {
                        source: source.clone(),
                        target: symbol_id.clone(),
                        kind: EdgeKind::new(EdgeKind::CALLS),
                        evidence: vec![EdgeEvidence {
                            rule: "imports_symbol".to_string(),
                            file: None,
                            line: None,
                            detail: Some(format!("imports symbol {}", label)),
                        }],
                        confidence: Default::default(),
                    });
                }
            }
        }
    }

    for (source_module, target_modules) in &module_imports {
        let unique_targets: std::collections::HashSet<_> = target_modules.iter().cloned().collect();
        for target_module in unique_targets {
            let source_id = format!("module:{}", source_module);
            let target_id = format!("module:{}", target_module);
            edges.push(Edge {
                source: source_id,
                target: target_id,
                kind: EdgeKind::new(EdgeKind::CALLS),
                evidence: vec![EdgeEvidence {
                    rule: "module_imports".to_string(),
                    file: None,
                    line: None,
                    detail: Some(format!("module imports from {}", target_module)),
                }],
                confidence: Default::default(),
            });
        }
    }

    let mut graph = Graph {
        metadata: {
            let mut metadata: HashMap<String, String> = HashMap::new();
            metadata.insert(
                "scan.language_files_seen".to_string(),
                diagnostics.language_files_seen.to_string(),
            );
            metadata.insert(
                "scan.collected_files".to_string(),
                diagnostics.collected_files.to_string(),
            );
            metadata.insert("scan.parsed_files".to_string(), parsed.len().to_string());
            metadata.insert(
                "scan.read_failed".to_string(),
                diagnostics.read_failed.to_string(),
            );
            metadata.insert(
                "scan.skipped_large".to_string(),
                diagnostics.skipped_large.to_string(),
            );
            metadata.insert(
                "scan.parse_failed".to_string(),
                diagnostics.parse_failed.to_string(),
            );
            if !diagnostics.read_failed_examples.is_empty() {
                metadata.insert(
                    "scan.read_failed_examples".to_string(),
                    diagnostics.read_failed_examples.join(", "),
                );
            }
            if !diagnostics.skipped_large_examples.is_empty() {
                metadata.insert(
                    "scan.skipped_large_examples".to_string(),
                    diagnostics.skipped_large_examples.join(", "),
                );
            }
            if !diagnostics.parse_failed_examples.is_empty() {
                metadata.insert(
                    "scan.parse_failed_examples".to_string(),
                    diagnostics.parse_failed_examples.join(", "),
                );
            }
            metadata
        },
        nodes,
        edges,
        incidents: Vec::new(),
        confidence: None,
        auto_context: Default::default(),
    };
    graph.canonicalize();
    Ok(graph)
}

fn build_walker(repo_root: &Path, config: &ScanConfig) -> ignore::Walk {
    build_walker_internal(repo_root, config)
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
        !should_exclude_with_config(path, &config_clone)
    });

    builder.build()
}

fn rel_path(repo_root: &Path, repo_canon: &Path, path: &Path) -> String {
    let rel = path
        .strip_prefix(repo_canon)
        .or_else(|_| path.strip_prefix(repo_root))
        .unwrap_or(path);
    rel.to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string()
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

pub fn parse_file(path: &Path, content: &str, language: Language) -> Option<ParsedFile> {
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

#[cfg(test)]
mod scan_diagnostics_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn walker_honors_srujaignore() {
        let dir = tempdir().expect("tempdir");
        let repo_root = dir.path();
        std::fs::create_dir_all(repo_root.join("src")).expect("mkdir src");
        std::fs::create_dir_all(repo_root.join("ignored")).expect("mkdir ignored");
        std::fs::write(repo_root.join(".srujaignore"), "ignored\n").expect("write .srujaignore");
        std::fs::write(repo_root.join("src/main.rs"), "fn main() {}\n").expect("write keep");
        std::fs::write(repo_root.join("ignored/bad.rs"), "fn bad() {}\n").expect("write ignored");

        let config = ScanConfig::default();
        let walker = build_walker_internal(repo_root, &config);
        let mut files: Vec<String> = Vec::new();
        for entry in walker {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if path.is_file() {
                files.push(
                    path.strip_prefix(repo_root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }

        assert!(files.contains(&"src/main.rs".to_string()));
        assert!(!files.iter().any(|p| p.contains("ignored/bad.rs")));
    }

    #[test]
    fn scan_attaches_basic_diagnostics_metadata() {
        let dir = tempdir().expect("tempdir");
        let repo_root = dir.path();
        std::fs::create_dir_all(repo_root.join("src")).expect("mkdir src");
        std::fs::write(repo_root.join("src/main.rs"), "fn main() {}\n").expect("write file");

        let config = ScanConfig::default();
        let graph = scan_with_tree_sitter(repo_root, &config).expect("scan");

        assert!(graph.metadata.contains_key("scan.language_files_seen"));
        assert!(graph.metadata.contains_key("scan.collected_files"));
        assert!(graph.metadata.contains_key("scan.parsed_files"));
        assert!(graph.metadata.contains_key("scan.read_failed"));
        assert!(graph.metadata.contains_key("scan.skipped_large"));
        assert!(graph.metadata.contains_key("scan.parse_failed"));
    }
}

fn infer_node_kind(
    parsed: &ParsedFile,
    path: &Path,
    content: &str,
    engine: &classifier::ClassificationEngine,
) -> (NodeKind, std::collections::HashMap<String, String>) {
    let ctx = classifier::ClassificationContext {
        path_str: path.to_string_lossy().to_lowercase(),
        content_lower: content.to_lowercase(),
        parsed,
    };

    let (kind, confidence, signals) = engine.classify(&ctx);

    let mut metadata = std::collections::HashMap::new();
    metadata.insert(
        "classification.confidence".to_string(),
        confidence.to_string(),
    );

    if !signals.is_empty() {
        let signal_names: Vec<String> = signals.iter().map(|s| s.name.to_string()).collect();
        metadata.insert("classification.signals".to_string(), signal_names.join(","));
    }

    (kind, metadata)
}

/// Common extensions to try when resolving extensionless imports (TypeScript/JavaScript).
const RESOLVE_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"];

fn extract_module_from_id(id: &str) -> String {
    id.rsplit('_').nth(1).unwrap_or(id).to_string()
}

fn definition_kind_to_symbol_kind(kind: languages::DefinitionKind) -> &'static str {
    match kind {
        languages::DefinitionKind::Function => "function",
        languages::DefinitionKind::Class => "class",
        languages::DefinitionKind::Interface => "interface",
        languages::DefinitionKind::Struct => "struct",
        languages::DefinitionKind::Enum => "enum",
        languages::DefinitionKind::Constant => "constant",
        languages::DefinitionKind::Variable => "variable",
    }
}

fn resolve_import_improved(
    repo_root: &Path,
    repo_canon: &Path,
    from_file: &Path,
    import_path: &str,
    file_path_to_id: &HashMap<String, String>,
    go_module_path: Option<&str>,
    language: Language,
) -> String {
    if import_path.starts_with('.') || import_path.starts_with('/') {
        let from_dir = from_file.parent().unwrap_or(repo_root);
        let base = from_dir.join(import_path);

        let to_id = |p: &Path| {
            p.strip_prefix(repo_canon)
                .ok()
                .map(|s| s.to_string_lossy().replace(['/', '\\', '.'], "_"))
        };

        if let Ok(resolved) = base.canonicalize() {
            if let Some(id) = to_id(&resolved) {
                return id;
            }
        }

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

    if language == Language::Go {
        if let Some(module_path) = go_module_path {
            if import_path.starts_with(module_path) {
                let relative = import_path.strip_prefix(module_path).unwrap_or(import_path);
                let relative = relative.trim_start_matches('/');

                for ext in &[".go", ""] {
                    let candidate = relative.to_string() + ext;
                    if let Some(id) = file_path_to_id.get(&candidate) {
                        return id.clone();
                    }
                }

                for (path, id) in file_path_to_id {
                    if path.starts_with(relative) || path.contains(&format!("/{}/", relative)) {
                        return id.clone();
                    }
                }
            }
        }

        for (path, id) in file_path_to_id {
            let path_lower = path.to_lowercase();
            let import_lower = import_path.to_lowercase();
            let import_suffix = import_lower.rsplit('/').next().unwrap_or(&import_lower);
            let path_suffix = path_lower.rsplit('/').next().unwrap_or(&path_lower);
            let path_stem = path_suffix.trim_end_matches(".go");

            if import_suffix == path_stem || import_lower.ends_with(&format!("/{}", path_stem)) {
                return id.clone();
            }
        }
    }

    if language == Language::Java || language == Language::Kotlin || language == Language::Scala {
        let java_path = import_path.replace('.', "/");
        for (path, id) in file_path_to_id {
            let path_normalized = path.replace('\\', "/");
            if path_normalized.contains(&java_path)
                || java_path.contains(
                    path_normalized
                        .trim_end_matches(".java")
                        .trim_end_matches(".kt"),
                )
            {
                return id.clone();
            }
        }
    }

    if language == Language::Rust {
        let mut resolved = import_path.replace("::", "/");
        if resolved.starts_with("crate/") {
            resolved = resolved.replacen("crate/", "", 1);
        } else if resolved.starts_with("super/") {
            if let Some(parent) = from_file.parent().and_then(|p| p.parent()) {
                if let Ok(rel) = parent.strip_prefix(repo_root) {
                    resolved = format!(
                        "{}/{}",
                        rel.to_string_lossy(),
                        resolved.replacen("super/", "", 1)
                    );
                }
            }
        }

        for ext in &[".rs", ""] {
            let candidate = resolved.clone() + ext;
            if let Some(id) = file_path_to_id.get(&candidate) {
                return id.clone();
            }
            let src_candidate = format!("src/{}", candidate);
            if let Some(id) = file_path_to_id.get(&src_candidate) {
                return id.clone();
            }
        }
    }

    import_path
        .replace(['/', '@', '-', '.', ':'], "_")
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

    #[test]
    fn test_scan_java_files() {
        let dir = tempfile::tempdir().unwrap();
        let main_file = dir.path().join("src").join("Main.java");
        let helper_file = dir.path().join("src").join("util").join("Helper.java");
        std::fs::create_dir_all(helper_file.parent().unwrap()).unwrap();
        std::fs::write(
            &main_file,
            r#"
package com.example;

import com.example.util.Helper;

public class Main {
    public static void main(String[] args) {
        Helper.help();
    }
}
"#,
        )
        .unwrap();
        std::fs::write(
            &helper_file,
            r#"
package com.example.util;

public class Helper {
    public static int help() {
        return 1;
    }
}
"#,
        )
        .unwrap();

        let config = ScanConfig::default();
        let graph = scan_with_tree_sitter(dir.path(), &config).unwrap();

        assert!(!graph.nodes.is_empty(), "Should have nodes");
        assert!(
            graph.nodes.iter().any(|n| n.id.contains("Main_java")),
            "Should include Main.java node"
        );
    }

    #[test]
    fn test_incremental_scan() {
        let dir = tempfile::tempdir().unwrap();
        let ts_file = dir.path().join("service.ts");
        std::fs::write(
            &ts_file,
            r#"
import { db } from './database';
export class UserService {
    getUser(id: string) { return db.find(id); }
}
"#,
        )
        .unwrap();

        let config = ScanConfig {
            incremental: true,
            ..ScanConfig::default()
        };

        // First scan - builds cache
        let graph1 = scan_with_tree_sitter(dir.path(), &config).unwrap();
        assert!(!graph1.nodes.is_empty());
        assert!(dir
            .path()
            .join(".sruja")
            .join("scan_manifest.json")
            .exists());
        assert!(dir.path().join(".sruja").join("ast_cache.json").exists());

        // Second scan - unchanged, should hit cache
        let graph2 = scan_with_tree_sitter(dir.path(), &config).unwrap();
        assert_eq!(graph1.nodes.len(), graph2.nodes.len());

        // Modify file - should invalidate cache and update successfully
        std::fs::write(
            &ts_file,
            r#"
import { db } from './database';
import { logger } from './logger';
export class UserService {
    getUser(id: string) { logger.info(id); return db.find(id); }
}
"#,
        )
        .unwrap();

        let graph3 = scan_with_tree_sitter(dir.path(), &config).unwrap();
        assert!(!graph3.nodes.is_empty());
    }
}

struct ExtractedComment {
    text: String,
    line: usize,
    keyword: String,
}

fn extract_comments_from_content(content: &str) -> Vec<ExtractedComment> {
    let mut comments = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();
        if trimmed.starts_with("//")
            || trimmed.starts_with("#")
            || trimmed.starts_with("///")
            || trimmed.starts_with('*')
        {
            for keyword in &["NOTE:", "WHY:", "HACK:", "TODO:"] {
                if let Some(idx) = trimmed.find(keyword) {
                    let comment_text = trimmed[idx + keyword.len()..].trim().to_string();
                    if !comment_text.is_empty() {
                        comments.push(ExtractedComment {
                            text: comment_text,
                            line: line_num,
                            keyword: keyword.trim_end_matches(':').to_string(),
                        });
                    }
                }
            }
        }
    }
    comments
}
