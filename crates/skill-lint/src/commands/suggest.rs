use crate::context::{ContextAnalyzer, FileContext, ProjectContext};
use crate::error::Result;
use colored::Colorize;
use std::path::{Path, PathBuf};

pub async fn run(
    path: PathBuf,
    project_path: Option<PathBuf>,
    file_path: Option<PathBuf>,
    top_rules: bool,
    limit: usize,
    _db_path: Option<PathBuf>,
) -> Result<()> {
    println!("{}", "Dynamic Rule Suggestions:".bold());
    println!("  Skills path: {}", path.display().to_string().cyan());
    if let Some(ref proj) = project_path {
        println!("  Project path: {}", proj.display().to_string().cyan());
    }
    if let Some(ref file) = file_path {
        println!("  File path: {}", file.display().to_string().cyan());
    }
    println!();

    let mut analyzer = ContextAnalyzer::new();

    if top_rules {
        return show_top_rules(&mut analyzer, limit).await;
    }

    let proj_path = project_path.unwrap_or_else(|| {
        let current_dir = std::env::current_dir().unwrap_or(PathBuf::from("."));
        current_dir
    });

    let project_context = analyzer.analyze_project(&proj_path)?;
    println!("\n{}", "Project Context:".bold());
    println!("  Language: {}", project_context.language.cyan());
    println!("  Tech Stack: {}", format!("{:?}", project_context.tech_stack).cyan());
    println!("  Frameworks: {}", format!("{:?}", project_context.frameworks).cyan());
    println!("  Patterns: {}", format!("{:?}", project_context.patterns).cyan());
    println!("  Async: {}", bool_str(project_context.async_usage));
    println!("  WASM: {}", bool_str(project_context.wasm_usage));
    println!("  Embedded: {}", bool_str(project_context.embedded_usage));

    let file_context = if let Some(ref fp) = file_path {
        if fp.exists() {
            let fc = analyzer.analyze_file(fp)?;
            println!("\n{}", "File Context:".bold());
            println!("  Language: {}", fc.language.cyan());
            println!("  Imports: {}", format!("{:?}", fc.imports).cyan());
            println!("  Async: {}", bool_str(fc.has_async));
            println!("  Unsafe: {}", bool_str(fc.has_unsafe));
            println!("  Extern Crate: {}", bool_str(fc.has_extern_crate));
            println!("  Macros: {}", bool_str(fc.has_macros));
            Some(fc)
        } else {
            println!("\n{}", "Warning:".yellow().bold());
            println!("  File not found: {}", fp.display().to_string());
            None
        }
    } else {
        None
    };

    let suggestions = analyzer.suggest_rules(&project_context, file_context.as_ref())?;

    println!("\n{}", "=".repeat(60));
    println!("{}", "Suggested Rules:".bold());
    println!("{}", "=".repeat(60));

    if suggestions.is_empty() {
        println!("\n{}", "No suggestions available for this context.".yellow());
        return Ok(());
    }

    for (i, suggestion) in suggestions.iter().enumerate() {
        println!("\n{} {}", format!("{}. ", i + 1).bold(), suggestion.rule_id.cyan());
        let relevance_str = format!("{:.2}", suggestion.relevance_score);
        println!("{}", format!("  Relevance: {}%", relevance_str).yellow());
        println!("{}", format!("  Priority: {}", suggestion.priority).green());
        println!("{}", format!("  Confidence: {}", suggestion.confidence).green());
        println!("\n  {}", "Reasoning:".bold());
        for reason in suggestion.reasoning.iter() {
            println!("    • {}", reason.dimmed().to_string());
        }
    }

    Ok(())
}

async fn show_top_rules(analyzer: &mut ContextAnalyzer, limit: usize) -> Result<()> {
    let suggestions = analyzer.get_top_rules(limit);

    println!("\n{}", "=".repeat(60));
    println!("{}", format!("Top {} Rules by Usage:", limit).bold());
    println!("{}", "=".repeat(60));

    for (i, suggestion) in suggestions.iter().enumerate() {
        println!("\n{} {}", format!("{}. ", i + 1).bold(), suggestion.rule_id.cyan());
        let relevance_str = format!("{:.2}", suggestion.relevance_score);
        println!("{}", format!("  Relevance: {}", relevance_str).yellow());
        println!("{}", format!("  Priority: {}", suggestion.priority).green());
        println!("{}", format!("  Confidence: {}", suggestion.confidence).green());
        println!("\n  {}", "Reasoning:".bold());
        for reason in suggestion.reasoning.iter() {
            println!("    • {}", reason.dimmed());
        }
    }

    Ok(())
}

fn bool_str(value: bool) -> String {
    if value {
        "Yes".green().to_string()
    } else {
        "No".red().to_string()
    }
}
