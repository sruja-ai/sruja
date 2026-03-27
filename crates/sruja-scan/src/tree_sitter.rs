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
use std::path::{Path, PathBuf};

use crate::graph::{Edge, EdgeEvidence, EdgeKind, Graph, Node, NodeKind};
use crate::ScanError;
use rayon::prelude::*;

use self::detector::Language;
use crate::scan_scope::should_exclude_with_config;

pub use detector::detect_language;
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
        }
    }
}

pub fn scan_with_tree_sitter(repo_root: &Path, config: &ScanConfig) -> Result<Graph, ScanError> {
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

    let go_module_path: Option<String> = std::fs::read_to_string(repo_root.join("go.mod"))
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|l| l.starts_with("module "))
                .map(|l| l.trim_start_matches("module ").trim().to_string())
        });

    // 1. Walk and collect eligible (path, content, language).
    let (collected, diagnostics): (Vec<(PathBuf, String, Language)>, ScanDiagnostics) = {
        let walker = build_walker(repo_root, config);
        let mut out = Vec::new();
        let mut diagnostics = ScanDiagnostics::default();
        for entry in walker {
            let entry = entry.map_err(|e| ScanError::Walk(e.to_string()))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(language) = detect_language(path) {
                diagnostics.language_files_seen += 1;
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        if content.len() <= config.max_file_size {
                            diagnostics.collected_files += 1;
                            out.push((path.to_path_buf(), content, language));
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
        (out, diagnostics)
    };

    // 2. Build file_path_to_id from paths so resolve_import_improved works in merge.
    let file_path_to_id: HashMap<String, String> = collected
        .iter()
        .filter_map(|(path, _, _)| {
            let file_id = file_to_id(repo_root, path);
            path.canonicalize().ok().and_then(|canon_path| {
                canon_path
                    .strip_prefix(&repo_canon)
                    .ok()
                    .map(|rel| (rel.to_string_lossy().to_string(), file_id))
            })
        })
        .collect();

    // 3. Parallel parse; keep content for merge (infer_node_kind needs it).
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

    let mut parsed: Vec<(PathBuf, String, Language, ParsedFile)> = Vec::new();
    let mut diagnostics = diagnostics;
    for outcome in outcomes {
        match outcome {
            ParseOutcome::Ok(path, content, language, parsed_file) => {
                parsed.push((path, content, language, parsed_file));
            }
            ParseOutcome::Err(rel) => {
                diagnostics.parse_failed += 1;
                push_example(&mut diagnostics.parse_failed_examples, rel);
            }
        }
    }

    // 4. Single-threaded merge: same logic as before.
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut module_nodes: HashMap<String, Node> = HashMap::new();
    let mut file_imports: HashMap<String, Vec<String>> = HashMap::new();
    let mut module_imports: HashMap<String, Vec<String>> = HashMap::new();

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
                    kind: NodeKind::Module,
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
                },
            );
        }

        let node = Node {
            id: file_id.clone(),
            kind: infer_node_kind(parsed, path, content.as_str()),
            label: parsed.name.clone(),
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
                kind: NodeKind::Module,
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

    for (source_module, target_modules) in &module_imports {
        let unique_targets: std::collections::HashSet<_> = target_modules.iter().cloned().collect();
        for target_module in unique_targets {
            let source_id = format!("module:{}", source_module);
            let target_id = format!("module:{}", target_module);
            edges.push(Edge {
                source: source_id,
                target: target_id,
                kind: EdgeKind::Calls,
                evidence: vec![EdgeEvidence {
                    rule: "module_imports".to_string(),
                    file: None,
                    line: None,
                    detail: Some(format!("module imports from {}", target_module)),
                }],
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
        confidence: None,
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

fn infer_node_kind(parsed: &ParsedFile, path: &Path, content: &str) -> NodeKind {
    let path_str = path.to_string_lossy().to_lowercase();
    let name_lower = parsed.name.to_lowercase();
    let content_lower = content.to_lowercase();

    // Early exit: detect CLI tools and exclude from service detection
    if is_cli_tool(&path_str, &content_lower) {
        return NodeKind::Module;
    }

    // Check for deployment configs (highest confidence)
    if is_deployment_config(&path_str) {
        return NodeKind::Service;
    }

    // Check for Go cmd/*/main.go pattern (high confidence for Go services)
    if is_go_service_entry_point(&path_str, &name_lower, &content_lower) {
        return NodeKind::Service;
    }

    // Check for Java Spring Boot services
    if is_java_service_entry_point(&content_lower, &path_str) {
        return NodeKind::Service;
    }

    // Check for TypeScript/JavaScript server entry points
    if is_js_service_entry_point(&content_lower, &path_str) {
        return NodeKind::Service;
    }

    // Check for Python Flask/Django/FastAPI services
    if is_python_service_entry_point(&content_lower, &path_str) {
        return NodeKind::Service;
    }

    // Check for framework-specific server code (high confidence)
    if has_framework_server_patterns(&content_lower) {
        return NodeKind::Service;
    }

    // Check for main files with server initialization
    if is_main_server_file(parsed, &path_str, &content_lower) {
        return NodeKind::Service;
    }

    // Path/Name heuristics (conservative - require additional signals)
    // Check for Next.js specific patterns
    if is_nextjs_route(&path_str, content) {
        return NodeKind::ExternalApi;
    }
    if is_nextjs_component(&path_str, content) {
        return NodeKind::Module;
    }

    // Check for database/repository patterns (e.g. Prisma, Mongoose)
    if is_database_file(&path_str, &content_lower) {
        return NodeKind::Database;
    }

    if is_service_directory(&path_str, &content_lower) {
        return NodeKind::Service;
    }

    if is_database_directory(&path_str, &name_lower) {
        return NodeKind::Database;
    }

    if is_external_api_directory(&path_str, &name_lower) {
        return NodeKind::ExternalApi;
    }

    NodeKind::Module
}

fn is_go_service_entry_point(path_str: &str, name_lower: &str, content_lower: &str) -> bool {
    if !path_str.ends_with(".go") {
        return false;
    }

    if path_str.contains("/cmd/") && name_lower == "main" {
        let http_indicators = [
            "http.", "grpc.", "serve", "listen", "router", "mux", "handler",
        ];
        return http_indicators.iter().any(|i| content_lower.contains(i));
    }

    false
}

fn is_java_service_entry_point(content_lower: &str, path_str: &str) -> bool {
    if !path_str.ends_with(".java") {
        return false;
    }

    let spring_patterns = [
        "@springbootapplication",
        "@restcontroller",
        "@controller",
        "@service",
        "extends springbootservletinitializer",
        "@enablewebmvc",
    ];

    if spring_patterns.iter().any(|p| content_lower.contains(p)) {
        return true;
    }

    if content_lower.contains("public static void main") {
        let server_indicators = ["springapplication.run", "server", "tomcat", "jetty"];
        return server_indicators.iter().any(|i| content_lower.contains(i));
    }

    false
}

fn is_js_service_entry_point(content_lower: &str, path_str: &str) -> bool {
    let is_js_file = path_str.ends_with(".js")
        || path_str.ends_with(".ts")
        || path_str.ends_with(".mjs")
        || path_str.ends_with(".cjs");

    if !is_js_file {
        return false;
    }

    let server_patterns = [
        "express()",
        "fastify(",
        "fastify({",
        "nestfactory.create",
        "http.createserver",
        "https.createserver",
        "app.listen(",
        "server.listen(",
        "koa()",
        "hapi.server",
        "@nestjs/core",
    ];

    server_patterns.iter().any(|p| content_lower.contains(p))
}

fn is_python_service_entry_point(content_lower: &str, path_str: &str) -> bool {
    if !path_str.ends_with(".py") {
        return false;
    }

    let web_framework_patterns = [
        "flask(__name__)",
        "fastapi(",
        "apirouter",
        "django",
        "tornado.web.application",
        "aiohttp.web",
        "sanic(",
        "starlette(",
        "quart(",
        "bottle.run",
    ];

    web_framework_patterns
        .iter()
        .any(|p| content_lower.contains(p))
}

fn is_deployment_config(path_str: &str) -> bool {
    (path_str.contains("deployment.") || path_str.contains("docker-compose."))
        && (path_str.ends_with(".yaml") || path_str.ends_with(".yml"))
        || path_str.ends_with("dockerfile")
        || path_str.contains("serverless.yml")
}

fn has_framework_server_patterns(content_lower: &str) -> bool {
    let patterns = [
        "http.listenandserve",
        "http.listenandservertls",
        "gin.run(",
        "echo.start(",
        "fiber.listen(",
        "app.listen(",
        "server.listen(",
        "express()",
        "app.run(",
        "uvicorn.run(",
        "actix_web::httpserver",
        "warp::serve",
        "rocket::ignite",
        "@springbootapplication",
        "grpc.newserver",
        "grpc.server",
    ];

    patterns.iter().any(|p| content_lower.contains(p))
}

fn is_main_server_file(parsed: &ParsedFile, _path_str: &str, content_lower: &str) -> bool {
    if parsed.name.to_lowercase() != "main" {
        return false;
    }

    let server_indicators = [
        "http.listenandserve",
        "listenandserve",
        "app.listen",
        "server.listen",
        "gin.run",
        "http.server",
        "grpc.server",
    ];

    server_indicators
        .iter()
        .any(|indicator| content_lower.contains(indicator))
}

fn is_database_file(path_str: &str, content_lower: &str) -> bool {
    // Check for Prisma schema
    if path_str.ends_with(".prisma") {
        return true;
    }

    // Check for ORM/DB patterns in content (Prisma, Mongoose, Sequelize, TypeORM)
    let db_patterns = [
        "prisma.client",
        "@prisma/client",
        "mongoose.model",
        "mongoose.schema",
        "@module sequelize",
        "sequelize.define",
        "@entity",                 // TypeORM
        "@primarygeneratedcolumn", // TypeORM
        "drizzle-orm",
    ];

    if db_patterns.iter().any(|p| content_lower.contains(p)) {
        return true;
    }

    false
}

fn is_cli_tool(path_str: &str, content_lower: &str) -> bool {
    // CLI framework patterns
    let cli_patterns = [
        "cobra.",
        "urfave/cli",
        "urfave/cli",
        "cli.app",
        "commander",
        "yargs",
        "argparse",
        "click.command",
        "cobra.command",
        "flag.parse",
    ];

    // Check for CLI indicators in content
    let has_cli_framework = cli_patterns.iter().any(|p| content_lower.contains(p));

    // Check for CLI-specific paths
    let is_cli_path = path_str.contains("/cmd/") && !path_str.contains("/cmd/server")
        || path_str.contains("/cli/")
        || path_str.ends_with("/main.go")
            && (content_lower.contains("flag.") || content_lower.contains("args"));

    has_cli_framework || is_cli_path
}

fn is_external_api_directory(path_str: &str, name_lower: &str) -> bool {
    path_str.contains("/external/")
        || path_str.contains("/thirdparty/")
        || path_str.contains("/vendor/")
        || name_lower.ends_with("client")
        || name_lower.ends_with("gateway")
}

fn is_nextjs_route(path_str: &str, _content: &str) -> bool {
    if path_str.contains("/app/") && path_str.ends_with("/route.ts") {
        return true;
    }
    if path_str.contains("/pages/api/") {
        return true;
    }
    false
}

fn is_nextjs_component(path_str: &str, _content: &str) -> bool {
    let special_files = [
        "page.tsx",
        "layout.tsx",
        "loading.tsx",
        "error.tsx",
        "not-found.tsx",
    ];
    if path_str.contains("/app/") && special_files.iter().any(|f| path_str.ends_with(f)) {
        return true;
    }
    if path_str.contains("/pages/") && !path_str.contains("/pages/api/") {
        return true;
    }
    false
}
fn is_service_directory(path_str: &str, content_lower: &str) -> bool {
    let patterns = ["/cmd/server", "/internal/server", "/pkg/server"];

    // High-confidence server paths
    if patterns.iter().any(|p| path_str.contains(p)) {
        return true;
    }

    // Lower confidence paths - require additional HTTP/API indicators
    let http_patterns = [
        "http.",
        "grpc.",
        "rest",
        "api",
        "router",
        "mux",
        "handler",
        "controller",
        "endpoint",
    ];

    let has_http_indicator = http_patterns.iter().any(|p| content_lower.contains(p));

    let service_paths = [
        "/api/",
        "/apis/",
        "/controller",
        "/controllers/",
        "/handler",
        "/handlers/",
        "/routes/",
    ];

    service_paths.iter().any(|p| path_str.contains(p)) && has_http_indicator
}

fn is_database_directory(path_str: &str, name_lower: &str) -> bool {
    path_str.contains("/db/")
        || path_str.contains("/database/")
        || path_str.contains("/dao/")
        || (path_str.contains("/repository/") && !path_str.contains("/repositories/"))
        || path_str.contains("/model/")
        || path_str.contains("/entity/")
        || (name_lower.ends_with("repository") && !name_lower.ends_with("services"))
        || name_lower.ends_with("dao")
        || name_lower.ends_with("schema")
}

/// Common extensions to try when resolving extensionless imports (TypeScript/JavaScript).
const RESOLVE_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"];

fn extract_module_from_id(id: &str) -> String {
    id.rsplit('_').nth(1).unwrap_or(id).to_string()
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
}
