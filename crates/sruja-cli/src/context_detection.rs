//! Repository context detection for better LLM prompts.
//!
//! Detects primary language, framework, domain, and architecture style
//! to improve LLM response quality and avoid incorrect assumptions.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;

use sruja_scan::{Graph, NodeKind};

/// Repository context for better LLM prompts
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RepoContext {
    pub name: String,
    pub primary_language: String,
    pub languages: Vec<(String, usize)>,
    pub framework: Option<String>,
    pub domain: Option<String>,
    pub component_count: usize,
    pub is_monolith: bool,
    pub is_microservices: bool,
}

/// Map file extensions to language names
fn extension_to_language(ext: &str) -> Option<&'static str> {
    match ext.to_lowercase().as_str() {
        "py" => Some("Python"),
        "js" => Some("JavaScript"),
        "ts" | "tsx" => Some("TypeScript"),
        "go" => Some("Go"),
        "java" => Some("Java"),
        "kt" | "kts" => Some("Kotlin"),
        "rs" => Some("Rust"),
        "rb" => Some("Ruby"),
        "cs" => Some("C#"),
        "cpp" | "cc" | "cxx" => Some("C++"),
        "c" => Some("C"),
        "php" => Some("PHP"),
        "swift" => Some("Swift"),
        "scala" | "sc" => Some("Scala"),
        "ex" | "exs" => Some("Elixir"),
        "erl" => Some("Erlang"),
        "hs" => Some("Haskell"),
        "clj" | "cljs" => Some("Clojure"),
        _ => None,
    }
}

/// Detect primary language from file extensions.
///
/// When the repo has a Rust workspace root (`Cargo.toml` at root), only counts
/// under `crates/` and `src/` so the main workspace language wins over ancillary
/// code (e.g. a VS Code extension in `extension/`).
pub fn detect_languages(repo_path: &Path) -> Vec<(String, usize)> {
    let mut lang_counts: HashMap<String, usize> = HashMap::new();
    let rust_workspace_root = repo_path.join("Cargo.toml").exists();

    fn count_files(
        dir: &Path,
        counts: &mut HashMap<String, usize>,
        rust_workspace_root: bool,
        repo_path: &Path,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();

                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.')
                        || name == "node_modules"
                        || name == "target"
                        || name == "build"
                        || name == "dist"
                        || name == "vendor"
                        || name == "venv"
                        || name == "env"
                        || name == "__pycache__"
                        || name == ".git"
                    {
                        continue;
                    }
                }

                if path.is_dir() {
                    // For Rust workspaces, only descend into main source trees so
                    // primary language is Rust, not e.g. TypeScript in extension/.
                    if rust_workspace_root && dir == repo_path {
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if name != "crates" && name != "src" {
                            continue;
                        }
                    }
                    count_files(&path, counts, rust_workspace_root, repo_path);
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if let Some(lang) = extension_to_language(ext) {
                        *counts.entry(lang.to_string()).or_default() += 1;
                    }
                }
            }
        }
    }

    count_files(
        repo_path,
        &mut lang_counts,
        rust_workspace_root,
        repo_path,
    );

    let mut languages: Vec<_> = lang_counts.into_iter().collect();
    languages.sort_by(|a, b| b.1.cmp(&a.1));
    languages
}

/// Detect framework from common files
pub fn detect_framework(repo_path: &Path, primary_language: &str) -> Option<String> {
    match primary_language {
        "Python" => detect_python_framework(repo_path),
        "JavaScript" | "TypeScript" => detect_js_framework(repo_path),
        "Go" => detect_go_framework(repo_path),
        "Java" | "Kotlin" => detect_java_framework(repo_path),
        "Ruby" => detect_ruby_framework(repo_path),
        "Rust" => detect_rust_framework(repo_path),
        "PHP" => detect_php_framework(repo_path),
        _ => None,
    }
}

fn detect_python_framework(repo_path: &Path) -> Option<String> {
    // Django indicators
    if repo_path.join("manage.py").exists() {
        return Some("Django".to_string());
    }

    // Check for settings.py in common Django locations
    for pattern in &["settings.py", "config/settings.py", "app/settings.py"] {
        if repo_path.join(pattern).exists() {
            return Some("Django".to_string());
        }
    }

    // Check requirements.txt or pyproject.toml for framework hints
    if let Ok(content) = std::fs::read_to_string(repo_path.join("requirements.txt")) {
        let content_lower = content.to_lowercase();
        if content_lower.contains("flask") {
            return Some("Flask".to_string());
        }
        if content_lower.contains("fastapi") {
            return Some("FastAPI".to_string());
        }
        if content_lower.contains("django") {
            return Some("Django".to_string());
        }
    }

    if let Ok(content) = std::fs::read_to_string(repo_path.join("pyproject.toml")) {
        let content_lower = content.to_lowercase();
        if content_lower.contains("flask") {
            return Some("Flask".to_string());
        }
        if content_lower.contains("fastapi") {
            return Some("FastAPI".to_string());
        }
        if content_lower.contains("django") {
            return Some("Django".to_string());
        }
    }

    None
}

fn detect_js_framework(repo_path: &Path) -> Option<String> {
    let package_json_path = repo_path.join("package.json");

    if let Ok(content) = std::fs::read_to_string(&package_json_path) {
        let content_lower = content.to_lowercase();

        // Check dependencies for framework indicators
        if content_lower.contains("\"express\"") || content_lower.contains("'express'") {
            return Some("Express".to_string());
        }
        if content_lower.contains("\"next\"") || content_lower.contains("'next'") {
            return Some("Next.js".to_string());
        }
        if content_lower.contains("\"nestjs\"")
            || content_lower.contains("'nestjs'")
            || content_lower.contains("\"@nestjs/core\"")
        {
            return Some("NestJS".to_string());
        }
        if content_lower.contains("\"react\"") && !content_lower.contains("\"next\"") {
            return Some("React".to_string());
        }
        if content_lower.contains("\"vue\"") {
            return Some("Vue".to_string());
        }
        if content_lower.contains("\"angular\"") || content_lower.contains("\"@angular/core\"") {
            return Some("Angular".to_string());
        }
        if content_lower.contains("\"svelte\"") {
            return Some("Svelte".to_string());
        }
        if content_lower.contains("\"fastify\"") {
            return Some("Fastify".to_string());
        }
        if content_lower.contains("\"hapi\"") || content_lower.contains("\"@hapi/hapi\"") {
            return Some("Hapi".to_string());
        }
        if content_lower.contains("\"koa\"") {
            return Some("Koa".to_string());
        }
        if content_lower.contains("\"remix\"") {
            return Some("Remix".to_string());
        }
        if content_lower.contains("\"nuxt\"") {
            return Some("Nuxt".to_string());
        }
    }

    // Check for Next.js config
    if repo_path.join("next.config.js").exists() || repo_path.join("next.config.mjs").exists() {
        return Some("Next.js".to_string());
    }

    // Check for Nuxt config
    if repo_path.join("nuxt.config.js").exists() || repo_path.join("nuxt.config.ts").exists() {
        return Some("Nuxt".to_string());
    }

    None
}

fn detect_go_framework(repo_path: &Path) -> Option<String> {
    if let Ok(content) = std::fs::read_to_string(repo_path.join("go.mod")) {
        let content_lower = content.to_lowercase();

        if content_lower.contains("github.com/gin-gonic/gin") {
            return Some("Gin".to_string());
        }
        if content_lower.contains("github.com/labstack/echo") {
            return Some("Echo".to_string());
        }
        if content_lower.contains("github.com/gofiber/fiber") {
            return Some("Fiber".to_string());
        }
        if content_lower.contains("github.com/go-chi/chi") {
            return Some("Chi".to_string());
        }
        if content_lower.contains("github.com/gorilla/mux") {
            return Some("Gorilla Mux".to_string());
        }
        if content_lower.contains("github.com/beego/beego") {
            return Some("Beego".to_string());
        }
    }

    None
}

fn detect_java_framework(repo_path: &Path) -> Option<String> {
    // Check pom.xml for Maven
    if let Ok(content) = std::fs::read_to_string(repo_path.join("pom.xml")) {
        let content_lower = content.to_lowercase();

        if content_lower.contains("spring-boot")
            || content_lower.contains("org.springframework.boot")
        {
            return Some("Spring Boot".to_string());
        }
        if content_lower.contains("quarkus") {
            return Some("Quarkus".to_string());
        }
        if content_lower.contains("micronaut") {
            return Some("Micronaut".to_string());
        }
    }

    // Check build.gradle for Gradle
    for gradle_file in &["build.gradle", "build.gradle.kts"] {
        if let Ok(content) = std::fs::read_to_string(repo_path.join(gradle_file)) {
            let content_lower = content.to_lowercase();

            if content_lower.contains("spring-boot")
                || content_lower.contains("org.springframework.boot")
            {
                return Some("Spring Boot".to_string());
            }
            if content_lower.contains("quarkus") {
                return Some("Quarkus".to_string());
            }
            if content_lower.contains("micronaut") {
                return Some("Micronaut".to_string());
            }
        }
    }

    None
}

fn detect_ruby_framework(repo_path: &Path) -> Option<String> {
    if repo_path.join("config/application.rb").exists() {
        // Check for Rails
        if repo_path.join("bin/rails").exists() || repo_path.join("script/rails").exists() {
            return Some("Rails".to_string());
        }
    }

    if let Ok(content) = std::fs::read_to_string(repo_path.join("Gemfile")) {
        let content_lower = content.to_lowercase();

        if content_lower.contains("rails") {
            return Some("Rails".to_string());
        }
        if content_lower.contains("sinatra") {
            return Some("Sinatra".to_string());
        }
        if content_lower.contains("hanami") {
            return Some("Hanami".to_string());
        }
    }

    None
}

fn detect_rust_framework(repo_path: &Path) -> Option<String> {
    if let Ok(content) = std::fs::read_to_string(repo_path.join("Cargo.toml")) {
        let content_lower = content.to_lowercase();

        if content_lower.contains("actix-web") || content_lower.contains("actix") {
            return Some("Actix".to_string());
        }
        if content_lower.contains("rocket") {
            return Some("Rocket".to_string());
        }
        if content_lower.contains("warp") {
            return Some("Warp".to_string());
        }
        if content_lower.contains("axum") {
            return Some("Axum".to_string());
        }
        if content_lower.contains("tide") {
            return Some("Tide".to_string());
        }
        if content_lower.contains("poem") {
            return Some("Poem".to_string());
        }
    }

    None
}

fn detect_php_framework(repo_path: &Path) -> Option<String> {
    // Laravel
    if repo_path.join("artisan").exists() {
        return Some("Laravel".to_string());
    }

    // Symfony
    if repo_path.join("symfony.lock").exists() || repo_path.join("bin/console").exists() {
        return Some("Symfony".to_string());
    }

    // Check composer.json
    if let Ok(content) = std::fs::read_to_string(repo_path.join("composer.json")) {
        let content_lower = content.to_lowercase();

        if content_lower.contains("laravel") {
            return Some("Laravel".to_string());
        }
        if content_lower.contains("symfony") {
            return Some("Symfony".to_string());
        }
        if content_lower.contains("slim/slim") {
            return Some("Slim".to_string());
        }
        if content_lower.contains("lumen") {
            return Some("Lumen".to_string());
        }
    }

    None
}

/// Infer domain from repo name and structure
pub fn infer_domain(repo_path: &Path, name: &str) -> Option<String> {
    let name_lower = name.to_lowercase();

    // Check repo name patterns
    if name_lower.ends_with("-api") || name_lower.ends_with("_api") || name_lower.contains("api-") {
        return Some("API/Service".to_string());
    }
    if name_lower.contains("-service") || name_lower.contains("_service") {
        return Some("Service".to_string());
    }
    if name_lower.contains("-shop")
        || name_lower.contains("-store")
        || name_lower.contains("-commerce")
        || name_lower.contains("ecommerce")
        || name_lower.contains("store")
        || name_lower.contains("shop")
    {
        return Some("E-commerce".to_string());
    }
    if name_lower.contains("-demo")
        || name_lower.starts_with("demo-")
        || name_lower.starts_with("example")
        || name_lower.contains("sample")
    {
        return Some("Demo/Example".to_string());
    }
    if name_lower.contains("microservice") {
        return Some("Microservices".to_string());
    }
    if name_lower.contains("blog") || name_lower.contains("cms") {
        return Some("Content Management".to_string());
    }
    if name_lower.contains("dashboard") || name_lower.contains("admin") {
        return Some("Admin/Dashboard".to_string());
    }
    if name_lower.contains("mobile") || name_lower.contains("app") {
        return Some("Mobile App".to_string());
    }
    if name_lower.contains("cli") || name_lower.contains("tool") {
        return Some("CLI Tool".to_string());
    }
    if name_lower.contains("sdk") || name_lower.contains("library") || name_lower.contains("lib") {
        return Some("Library/SDK".to_string());
    }

    // Check structure for additional hints
    let has_docker_compose = repo_path.join("docker-compose.yml").exists()
        || repo_path.join("docker-compose.yaml").exists();
    let has_k8s = repo_path.join("k8s").exists()
        || repo_path.join("kubernetes").exists()
        || repo_path.join("helm").exists();

    // Count Dockerfiles
    let dockerfile_count = count_dockerfiles(repo_path);

    // Require stronger signals to avoid labeling e.g. a Rust workspace with
    // a few Dockerfiles as "Microservices".
    if dockerfile_count > 5 || (has_docker_compose && dockerfile_count > 2) {
        return Some("Microservices".to_string());
    }

    if has_k8s {
        return Some("Production System".to_string());
    }

    None
}

/// Directories we skip when counting Dockerfiles for domain inference,
/// so evaluation/local-artifacts, benchmarks, and docs don't pollute the count.
const DOCKERFILE_COUNT_SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    "dist",
    "build",
    ".git",
    "vendor",
    "venv",
    "__pycache__",
    "evaluation",
    "benchmark",
    "bench",
    "perf",
    "docs",
    "documentation",
    "fixtures",
    "__mocks__",
    "test_data",
];

fn count_dockerfiles(path: &Path) -> usize {
    let mut count = 0;

    fn count_recursive(dir: &Path, cnt: &mut usize) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();

                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') {
                        continue;
                    }
                    if DOCKERFILE_COUNT_SKIP_DIRS.contains(&name) {
                        continue;
                    }
                    if path.is_dir() {
                        count_recursive(&path, cnt);
                    } else if name == "Dockerfile" || name.starts_with("Dockerfile.") {
                        *cnt += 1;
                    }
                }
            }
        }
    }

    count_recursive(path, &mut count);
    count
}

/// Detect if architecture is monolith or microservices
pub fn detect_architecture_style(graph: &Graph) -> (bool, bool) {
    let service_count = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Service)
        .count();

    let is_monolith = service_count <= 1;
    let is_microservices = service_count > 3;

    (is_monolith, is_microservices)
}

/// Build complete repository context
pub fn build_repo_context(repo_path: &Path, graph: &Graph) -> RepoContext {
    let name = repo_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let languages = detect_languages(repo_path);
    let primary_language = languages
        .first()
        .map(|(lang, _)| lang.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let framework = detect_framework(repo_path, &primary_language);
    let domain = infer_domain(repo_path, &name);
    let (is_monolith, is_microservices) = detect_architecture_style(graph);

    RepoContext {
        name,
        primary_language,
        languages,
        framework,
        domain,
        component_count: graph.nodes.len(),
        is_monolith,
        is_microservices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo(files: &[(&str, &str)]) -> TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        for (path, content) in files {
            let full_path = temp_dir.path().join(path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(full_path, content).unwrap();
        }
        temp_dir
    }

    #[test]
    fn test_detect_python_language() {
        let temp = create_test_repo(&[
            ("main.py", "# Python"),
            ("app.py", "# Python"),
            ("test.js", "// JavaScript"),
        ]);

        let languages = detect_languages(temp.path());
        assert_eq!(languages[0].0, "Python");
        assert_eq!(languages[0].1, 2);
    }

    #[test]
    fn test_detect_django_framework() {
        let temp = create_test_repo(&[("manage.py", ""), ("myapp/settings.py", "")]);

        let framework = detect_framework(temp.path(), "Python");
        assert_eq!(framework, Some("Django".to_string()));
    }

    #[test]
    fn test_detect_express_framework() {
        let temp = create_test_repo(&[
            ("package.json", r#"{"dependencies": {"express": "4.18.0"}}"#),
            ("index.js", ""),
        ]);

        let framework = detect_framework(temp.path(), "JavaScript");
        assert_eq!(framework, Some("Express".to_string()));
    }

    #[test]
    fn test_detect_nextjs_framework() {
        let temp = create_test_repo(&[
            (
                "package.json",
                r#"{"dependencies": {"next": "14.0.0", "react": "18.0.0"}}"#,
            ),
            ("next.config.js", ""),
        ]);

        let framework = detect_framework(temp.path(), "JavaScript");
        assert_eq!(framework, Some("Next.js".to_string()));
    }

    #[test]
    fn test_infer_ecommerce_domain() {
        let temp = create_test_repo(&[("shop.py", "")]);

        let domain = infer_domain(temp.path(), "my-shop");
        assert_eq!(domain, Some("E-commerce".to_string()));
    }

    #[test]
    fn test_infer_api_domain() {
        let temp = create_test_repo(&[]);

        let domain = infer_domain(temp.path(), "user-api");
        assert_eq!(domain, Some("API/Service".to_string()));
    }

    #[test]
    fn test_detect_spring_boot_framework() {
        let temp = create_test_repo(&[(
            "pom.xml",
            r#"<project><dependencies><dependency><groupId>org.springframework.boot</groupId></dependency></dependencies></project>"#,
        )]);

        let framework = detect_framework(temp.path(), "Java");
        assert_eq!(framework, Some("Spring Boot".to_string()));
    }

    #[test]
    fn test_detect_go_gin_framework() {
        let temp = create_test_repo(&[
            (
                "go.mod",
                r#"module myapp
require github.com/gin-gonic/gin v1.9.0"#,
            ),
            ("main.go", ""),
        ]);

        let framework = detect_framework(temp.path(), "Go");
        assert_eq!(framework, Some("Gin".to_string()));
    }

    #[test]
    fn test_detect_rust_actix_framework() {
        let temp = create_test_repo(&[
            (
                "Cargo.toml",
                r#"[dependencies]
actix-web = "4.0""#,
            ),
            ("src/main.rs", ""),
        ]);

        let framework = detect_framework(temp.path(), "Rust");
        assert_eq!(framework, Some("Actix".to_string()));
    }

    #[test]
    fn test_detect_laravel_framework() {
        let temp = create_test_repo(&[
            ("artisan", "#!/usr/bin/env php"),
            ("composer.json", r#"{"name": "laravel/laravel"}"#),
        ]);

        let framework = detect_framework(temp.path(), "PHP");
        assert_eq!(framework, Some("Laravel".to_string()));
    }

    #[test]
    fn test_detect_rails_framework() {
        let temp = create_test_repo(&[
            ("Gemfile", "gem 'rails', '~> 7.0'"),
            ("config/application.rb", ""),
        ]);

        let framework = detect_framework(temp.path(), "Ruby");
        assert_eq!(framework, Some("Rails".to_string()));
    }
}
