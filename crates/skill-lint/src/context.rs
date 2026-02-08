use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub language: String,
    pub frameworks: Vec<String>,
    pub patterns: Vec<String>,
    pub tech_stack: Vec<String>,
    pub async_usage: bool,
    pub wasm_usage: bool,
    pub embedded_usage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSuggestion {
    pub rule_id: String,
    pub relevance_score: f32,
    pub priority: String,
    pub confidence: String,
    pub reasoning: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    pub path: PathBuf,
    pub language: String,
    pub imports: Vec<String>,
    pub has_async: bool,
    pub has_extern_crate: bool,
    pub has_unsafe: bool,
    pub has_macros: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleUsageStats {
    pub rule_id: String,
    pub count: u32,
    pub last_used: i64,
    pub avg_score: f32,
}

pub struct ContextAnalyzer {
    pub usage_stats: HashMap<String, RuleUsageStats>,
}

impl ContextAnalyzer {
    pub fn new() -> Self {
        Self {
            usage_stats: HashMap::new(),
        }
    }

    pub fn analyze_project(&self, path: &Path) -> Result<ProjectContext> {
        let mut frameworks = Vec::new();
        let mut patterns = Vec::new();
        let mut tech_stack = Vec::new();
        let language = "rust".to_string();

        let cargo_toml = path.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("tokio") || content.contains("async-std") {
                    tech_stack.push("async".to_string());
                }
                if content.contains("serde") {
                    frameworks.push("serde".to_string());
                }
                if content.contains("wasm-bindgen") {
                    tech_stack.push("wasm".to_string());
                }
                if content.contains("clap") {
                    frameworks.push("cli".to_string());
                }
                if content.contains("axum") || content.contains("actix") {
                    frameworks.push("web".to_string());
                    tech_stack.push("web-server".to_string());
                }
            }
        }

        for entry in walkdir::WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let file_path = entry.path();

            if file_path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    if content.contains("#[async]") {
                        patterns.push("async".to_string());
                    }
                    if content.contains("unsafe") {
                        patterns.push("unsafe".to_string());
                    }
                    if content.contains("macro_rules!") {
                        patterns.push("macros".to_string());
                    }
                    if content.contains("extern crate") {
                        patterns.push("extern".to_string());
                    }
                }
            }
        }

        patterns.sort();
        patterns.dedup();

        let async_usage = tech_stack.iter().any(|t| t.contains("async"));
        let wasm_usage = tech_stack.iter().any(|t| t.contains("wasm"));
        let embedded_usage = tech_stack.iter().any(|t| t.contains("embedded"));

        Ok(ProjectContext {
            language,
            frameworks,
            patterns,
            tech_stack,
            async_usage,
            wasm_usage,
            embedded_usage,
        })
    }

    pub fn analyze_file(&self, path: &Path) -> Result<FileContext> {
        let content = std::fs::read_to_string(path)?;

        let imports = extract_imports(&content);
        let has_async = content.contains("#[async]") || content.contains("async fn");
        let has_extern_crate = content.contains("extern crate");
        let has_unsafe = content.contains("unsafe ");
        let has_macros = content.contains("macro_rules!");

        Ok(FileContext {
            path: path.to_path_buf(),
            language: "rust".to_string(),
            imports,
            has_async,
            has_extern_crate,
            has_unsafe,
            has_macros,
        })
    }

    pub fn record_rule_usage(&mut self, rule_id: &str) {
        let stats = self
            .usage_stats
            .entry(rule_id.to_string())
            .or_insert_with(|| RuleUsageStats {
                rule_id: rule_id.to_string(),
                count: 0,
                last_used: chrono::Utc::now().timestamp(),
                avg_score: 0.5,
            });

        stats.count += 1;
        stats.last_used = chrono::Utc::now().timestamp();
    }

    pub fn suggest_rules(
        &mut self,
        project_context: &ProjectContext,
        file_context: Option<&FileContext>,
    ) -> Result<Vec<RuleSuggestion>> {
        let mut suggestions = Vec::new();

        let base_rules = get_all_applicable_rules(project_context);
        let mut scored_rules: HashMap<String, f32> = HashMap::new();

        for rule in &base_rules {
            let mut score = calculate_base_score(rule, project_context);

            if let Some(file) = file_context {
                score += adjust_for_file_context(rule, file);
            }

            if let Some(stats) = self.usage_stats.get(rule) {
                score += stats.avg_score * 0.3;
                if stats.count > 5 {
                    score -= 0.2;
                }
            }

            scored_rules.insert(rule.clone(), score);
        }

        let mut rule_scores: Vec<(String, f32)> = scored_rules.into_iter().collect();

        rule_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        for (rule_id, score) in rule_scores.iter().take(10) {
            suggestions.push(RuleSuggestion {
                rule_id: rule_id.clone(),
                relevance_score: *score,
                priority: determine_priority(*score),
                confidence: determine_confidence(*score),
                reasoning: generate_reasoning(rule_id, project_context, file_context),
            });
        }

        Ok(suggestions)
    }

    pub fn get_all_stats(&self) -> Vec<RuleUsageStats> {
        self.usage_stats.values().cloned().collect()
    }

    pub fn get_top_rules(&self, limit: usize) -> Vec<RuleSuggestion> {
        let mut stats: Vec<_> = self.usage_stats.values().cloned().collect();
        stats.sort_by(|a, b| {
            b.count
                .cmp(&a.count)
                .then_with(|| b.last_used.cmp(&a.last_used))
        });
        stats.truncate(limit);

        let suggestions: Vec<RuleSuggestion> = stats
            .into_iter()
            .map(|stats| RuleSuggestion {
                rule_id: stats.rule_id.clone(),
                relevance_score: stats.avg_score,
                priority: determine_priority(stats.avg_score),
                confidence: determine_confidence(stats.avg_score),
                reasoning: vec![
                    format!("Used {} times", stats.count),
                    format!("Last used: {}", format_timestamp(stats.last_used)),
                    format!("Average score: {:.2}", stats.avg_score),
                ],
            })
            .collect();

        suggestions
    }
}

fn extract_imports(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    let import_regex = regex::Regex::new(r"use\s+([a-zA-Z0-9_:]+)").unwrap();

    for caps in import_regex.captures_iter(content) {
        if let Some(import) = caps.get(1) {
            imports.push(import.as_str().to_string());
        }
    }

    imports.sort();
    imports.dedup();
    imports
}

fn get_all_applicable_rules(context: &ProjectContext) -> Vec<String> {
    let mut rules = Vec::new();

    if context.async_usage {
        rules.extend(vec![
            "async-no-lock-await".to_string(),
            "async-spawn-blocking".to_string(),
            "async-cancellation-token".to_string(),
        ]);
    }

    if context.wasm_usage {
        rules.extend(vec![
            "mem-smallvec".to_string(),
            "perf-iter-lazy".to_string(),
        ]);
    }

    if context.embedded_usage {
        rules.extend(vec![
            "mem-arrayvec".to_string(),
            "mem-assert-type-size".to_string(),
            "opt-inline-always-rare".to_string(),
        ]);
    }

    if context.frameworks.contains(&"web".to_string()) {
        rules.extend(vec![
            "async-bounded-channel".to_string(),
            "async-mpsc-queue".to_string(),
        ]);
    }

    if context.patterns.contains(&"unsafe".to_string()) {
        rules.extend(vec![
            "own-borrow-over-clone".to_string(),
            "own-slice-over-vec".to_string(),
        ]);
    }

    if context.patterns.contains(&"async".to_string()) {
        rules.extend(vec![
            "mem-reuse-collections".to_string(),
            "perf-drain-reuse".to_string(),
        ]);
    }

    if context.patterns.contains(&"extern".to_string()) {
        rules.extend(vec![
            "type-newtype-validated".to_string(),
            "type-newtype-ids".to_string(),
        ]);
    }

    rules.sort();
    rules.dedup();
    rules
}

fn calculate_base_score(rule_id: &str, context: &ProjectContext) -> f32 {
    let mut score = 0.5;

    match rule_id {
        r if r.starts_with("async-") => {
            score += if context.async_usage { 0.4 } else { -0.2 };
        }
        r if r.starts_with("mem-") => score += 0.2,
        r if r.starts_with("perf-") => score += 0.2,
        r if r.starts_with("own-") => score += 0.2,
        r if r.starts_with("err-") => score += 0.3,
        _ => {}
    }

    for framework in &context.frameworks {
        if matches_framework(rule_id, framework) {
            score += 0.3;
        }
    }

    (score as f32).clamp(0.0_f32, 1.0_f32)
}

fn adjust_for_file_context(rule_id: &str, file: &FileContext) -> f32 {
    let mut adjustment = 0.0;

    if file.has_async && rule_id.starts_with("async-") {
        adjustment += 0.2;
    }

    if file.has_unsafe && rule_id.starts_with("own-") {
        adjustment += 0.2;
    }

    if file.has_extern_crate && rule_id.starts_with("type-") {
        adjustment += 0.1;
    }

    (adjustment as f32).clamp(-0.2_f32, 0.3_f32)
}

fn matches_framework(rule_id: &str, framework: &str) -> bool {
    match framework {
        "web" => rule_id.starts_with("async-") || rule_id.starts_with("perf-"),
        "cli" => rule_id.starts_with("err-") || rule_id.starts_with("api-"),
        "serde" => rule_id.starts_with("api-") || rule_id.starts_with("type-"),
        "async" => rule_id.starts_with("async-"),
        _ => false,
    }
}

fn determine_priority(score: f32) -> String {
    if score >= 0.7 {
        "high".to_string()
    } else if score >= 0.5 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

fn determine_confidence(score: f32) -> String {
    if score >= 0.8 {
        "very high".to_string()
    } else if score >= 0.6 {
        "high".to_string()
    } else if score >= 0.4 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

fn generate_reasoning(
    rule_id: &str,
    project_context: &ProjectContext,
    file_context: Option<&FileContext>,
) -> Vec<String> {
    let mut reasoning = Vec::new();

    if rule_id.starts_with("async-") && project_context.async_usage {
        reasoning.push("Project uses async/await patterns".to_string());
    }

    if rule_id.starts_with("mem-") && (project_context.wasm_usage || project_context.embedded_usage)
    {
        reasoning.push("Memory optimization important for constrained environments".to_string());
    }

    if rule_id.starts_with("err-") {
        reasoning.push("Error handling rules apply to all Rust projects".to_string());
    }

    if rule_id.starts_with("perf-") {
        reasoning.push("Performance optimization guidelines".to_string());
    }

    if let Some(file) = file_context {
        if file.has_async && rule_id.starts_with("async-") {
            reasoning.push("File contains async code".to_string());
        }
        if file.has_unsafe && rule_id.starts_with("own-") {
            reasoning.push("File uses unsafe code, ownership rules apply".to_string());
        }
    }

    if reasoning.is_empty() {
        reasoning.push("General Rust best practice rule".to_string());
    }

    reasoning
}

fn format_timestamp(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;

    if diff < 3600 {
        format!("{} minutes ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else if diff < 604800 {
        format!("{} days ago", diff / 86400)
    } else {
        format!("{} weeks ago", diff / 604800)
    }
}
