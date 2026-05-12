//! Extractor for project configuration / manifest files.
//!
//! Detects `package.json`, `Cargo.toml`, `go.mod`, `pyproject.toml`,
//! `pom.xml`, `build.gradle`, and other package manifests as
//! architectural signals about the technology stack.

use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

const CONFIG_FILES: &[(&str, &str)] = &[
    ("package.json", "Node.js"),
    ("cargo.toml", "Rust"),
    ("go.mod", "Go"),
    ("pyproject.toml", "Python"),
    ("setup.py", "Python"),
    ("setup.cfg", "Python"),
    ("requirements.txt", "Python"),
    ("pom.xml", "Java/Maven"),
    ("build.gradle", "Java/Gradle"),
    ("build.gradle.kts", "Kotlin/Gradle"),
    ("gemfile", "Ruby"),
    ("mix.exs", "Elixir"),
    ("project.clj", "Clojure"),
    ("pubspec.yaml", "Dart/Flutter"),
    ("composer.json", "PHP"),
    ("stack.yaml", "Haskell"),
    ("deno.json", "Deno"),
    ("deno.jsonc", "Deno"),
    ("bunfig.toml", "Bun"),
    ("shard.yml", "Crystal"),
    ("vcpkg.json", "C++/vcpkg"),
    ("conanfile.txt", "C++/Conan"),
    ("cmakelists.txt", "C/C++/CMake"),
    ("makefile", "Make"),
];

#[derive(Default)]
pub struct ConfigExtractor;

impl ConfigExtractor {
    pub fn new() -> Self {
        Self
    }

    fn detect_config(name: &str) -> Option<&'static str> {
        let lower = name.to_lowercase();
        CONFIG_FILES
            .iter()
            .find(|(f, _)| *f == lower)
            .map(|(_, tech)| *tech)
    }

    fn extract_name_from_package_json(content: &str) -> Option<String> {
        let needle = "\"name\"";
        let idx = content.find(needle)?;
        let after = &content[idx + needle.len()..];
        let after = after.trim_start();
        let after = after.strip_prefix(':')?;
        let after = after.trim_start();
        let after = after.strip_prefix('"')?;
        let end = after.find('"')?;
        let name = &after[..end];
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }

    fn extract_name_from_cargo_toml(content: &str) -> Option<String> {
        let mut in_package = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[package]" {
                in_package = true;
                continue;
            }
            if trimmed.starts_with('[') {
                in_package = false;
                continue;
            }
            if in_package {
                if let Some(rest) = trimmed.strip_prefix("name") {
                    let rest = rest.trim();
                    if let Some(rest) = rest.strip_prefix('=') {
                        let name = rest.trim().trim_matches('"').trim_matches('\'');
                        if !name.is_empty() {
                            return Some(name.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_name_from_go_mod(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("module ") {
                let module = rest.trim();
                let short = module.rsplit('/').next().unwrap_or(module);
                return Some(short.to_string());
            }
        }
        None
    }
}

impl Extractor for ConfigExtractor {
    fn name(&self) -> &'static str {
        "config"
    }

    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError> {
        let file_name = ctx.file_name();
        let technology = match Self::detect_config(file_name) {
            Some(t) => t,
            None => return Ok(Vec::new()),
        };

        let lower = file_name.to_lowercase();
        let project_name = ctx.content().and_then(|content| {
            if lower == "package.json" {
                Self::extract_name_from_package_json(content)
            } else if lower == "cargo.toml" {
                Self::extract_name_from_cargo_toml(content)
            } else if lower == "go.mod" {
                Self::extract_name_from_go_mod(content)
            } else {
                None
            }
        });

        let suggested_element = project_name
            .clone()
            .or_else(|| ctx.parent_dir_name().map(|s| s.to_string()));

        let description = project_name
            .map(|n| format!("{technology} project: {n}"))
            .unwrap_or_else(|| format!("{technology} project manifest"));

        Ok(vec![DiscoveredSource {
            binding: SourceBinding {
                kind: SourceKind::Config,
                path: ctx.relative_path().to_string(),
                description: Some(description),
            },
            suggested_element,
            confidence: 0.6,
        }])
    }
}
