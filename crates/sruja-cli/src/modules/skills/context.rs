//! Project context analysis
//!
//! Analyzes Cargo.toml and source files to understand project context
//! for intelligent skill filtering.

use std::fs;
use std::path::Path;

/// Analyze project context from Cargo.toml and source files
pub fn analyze_project_context(path: &Path) -> ProjectContext {
    let mut context = ProjectContext::default();

    let cargo_toml = path.join("Cargo.toml");

    if cargo_toml.exists() {
        if let Ok(content) = fs::read_to_string(&cargo_toml) {
            context = analyze_cargo_toml(&content);
        }
    }

    let src_path = path.join("src");
    if src_path.exists() {
        context.complexity_score = calculate_complexity(&src_path);
    }

    context
}

fn analyze_cargo_toml(content: &str) -> ProjectContext {
    let mut context = ProjectContext::default();

    if let Some(deps_section) = extract_toml_section(content, "[dependencies]") {
        context.is_async = deps_section.contains("tokio") || deps_section.contains("async-std");
        context.web = deps_section.contains("actix-web")
            || deps_section.contains("axum")
            || deps_section.contains("warp")
            || deps_section.contains("rocket");
        context.embedded =
            deps_section.contains("embedded-hal") || deps_section.contains("cortex-m");
        context.wasm = deps_section.contains("wasm-bindgen") || deps_section.contains("wasm-pack");
        context.cli = deps_section.contains("clap") || deps_section.contains("argh");
        context.library = content.contains("[lib]") || content.contains("crate-type = [\"lib\"]");
    }

    context
}

fn extract_toml_section(content: &str, section_start: &str) -> Option<String> {
    content.find(section_start).and_then(|start| {
        content[start..]
            .lines()
            .skip_while(|line| line.starts_with('[') || line.is_empty())
            .take_while(|line| !line.starts_with('['))
            .collect::<Vec<_>>()
            .join("\n")
            .into()
    })
}

fn calculate_complexity(src_path: &Path) -> f32 {
    let mut complexity = 0.0;

    if let Ok(entries) = fs::read_dir(src_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let lines = content.lines().count();
                    let functions = content.matches("fn ").count();
                    let structs = content.matches("struct ").count();
                    let traits = content.matches("trait ").count();

                    complexity += (lines as f32 / 1000.0)
                        + (functions as f32 * 0.1)
                        + (structs as f32 * 0.05)
                        + (traits as f32 * 0.05);
                }
            }
        }
    }

    (complexity / 10.0).min(1.0)
}

/// Project context from analysis
#[derive(Debug, Clone, Default)]
pub struct ProjectContext {
    pub is_async: bool,
    pub web: bool,
    pub embedded: bool,
    pub wasm: bool,
    pub cli: bool,
    pub library: bool,
    pub complexity_score: f32,
}
