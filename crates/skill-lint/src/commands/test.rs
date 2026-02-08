use crate::error::Result;
use colored::Colorize;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use std::path::PathBuf;
use walkdir::WalkDir;

struct CodeTestResult {
    file: String,
    line: Option<usize>,
    code_type: String,
    status: CodeTestStatus,
}

#[derive(Debug)]
enum CodeTestStatus {
    Ok,
    ParseError(String),
    InvalidSyntax(String),
}

pub async fn run(path: PathBuf, generate_code: bool) -> Result<()> {
    println!("{}", "Testing code examples in skill files:".bold());
    println!("  Path: {}", path.display().to_string().cyan());
    if generate_code {
        println!("  Mode: Generate test code");
    } else {
        println!("  Mode: Syntax check only");
    }
    println!();

    let mut results: Vec<CodeTestResult> = Vec::new();
    let mut total_examples = 0;
    let mut failed_examples = 0;

    for entry in WalkDir::new(&path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();

        if file_path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let content = std::fs::read_to_string(file_path)?;
        let display_path = file_path.display().to_string();

        let mut line_num = 1;
        let parser = Parser::new(&content);
        let mut in_code_block = false;
        let mut code_block_type = String::new();
        let mut code_block_start = 0;
        let mut code_block_lines: Vec<String> = Vec::new();

        for event in parser {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_block_start = line_num;
                    code_block_type = match kind {
                        CodeBlockKind::Fenced(lang) => lang.to_string(),
                        CodeBlockKind::Indented => "text".to_string(),
                    };
                }
                Event::End(TagEnd::CodeBlock) => {
                    if in_code_block {
                        in_code_block = false;

                        if code_block_type == "rust" || code_block_type == "rs" {
                            total_examples += 1;
                            let code = code_block_lines.join("\n");
                            let status = test_rust_code(&code);

                            if !matches!(status, CodeTestStatus::Ok) {
                                failed_examples += 1;
                            }

                            results.push(CodeTestResult {
                                file: display_path.clone(),
                                line: Some(code_block_start),
                                code_type: code_block_type.clone(),
                                status,
                            });
                        }

                        code_block_lines.clear();
                        code_block_type.clear();
                    }
                }
                Event::Text(text) => {
                    if in_code_block {
                        code_block_lines.push(text.to_string());
                    } else {
                        line_num += text.lines().count();
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if !in_code_block {
                        line_num += 1;
                    }
                }
                _ => {}
            }
        }
    }

    println!("{}", "=".repeat(50));
    println!("{}", "Code Test Summary:".bold());
    println!("  Total examples: {}", total_examples.to_string().white());
    println!(
        "  {}: {}",
        "Passed".green(),
        (total_examples - failed_examples).to_string().green()
    );
    println!(
        "  {}: {}",
        "Failed".red(),
        failed_examples.to_string().red()
    );

    if failed_examples > 0 {
        println!();
        println!("{}", "Failed Code Examples:".red().bold());

        for result in &results {
            if !matches!(result.status, CodeTestStatus::Ok) {
                let line_info = result.line.map(|l| format!(":{}", l)).unwrap_or_default();
                let status_msg = match &result.status {
                    CodeTestStatus::ParseError(msg) => format!("Parse error: {}", msg),
                    CodeTestStatus::InvalidSyntax(msg) => format!("Syntax error: {}", msg),
                    CodeTestStatus::Ok => unreachable!(),
                };

                println!("\n{}{}", result.file.yellow(), line_info);
                println!(
                    "  {} {} ({})",
                    "✗".red(),
                    "code block".cyan(),
                    result.code_type
                );
                println!("  {} {}", "Error:".red(), status_msg);
            }
        }

        Err(crate::error::SkillLintError::CodeTest(format!(
            "{} code example(s) failed",
            failed_examples
        )))
    } else {
        if generate_code {
            println!();
            println!("{}", "Generated test code feature:".yellow().italic());
            println!("  Code generation is not yet implemented.");
            println!("  Currently only syntax checking is available.");
        }
        Ok(())
    }
}

fn test_rust_code(code: &str) -> CodeTestStatus {
    let mut cleaned_code = clean_rust_code(code);

    if is_incomplete_snippet(&cleaned_code) {
        return CodeTestStatus::Ok;
    }

    cleaned_code = add_main_wrapper(&cleaned_code);

    match syn::parse_file(&cleaned_code) {
        Ok(_) => CodeTestStatus::Ok,
        Err(e) => {
            let error_msg = format_error(&e);
            CodeTestStatus::InvalidSyntax(error_msg)
        }
    }
}

fn clean_rust_code(code: &str) -> String {
    let mut lines: Vec<String> = Vec::new();

    for line in code.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("# ") || trimmed.starts_with("```") {
            continue;
        }

        if trimmed.starts_with("```") {
            continue;
        }

        lines.push(line.to_string());
    }

    lines.join("\n")
}

fn is_incomplete_snippet(code: &str) -> bool {
    code.contains("...") || code.contains("// ...") || code.lines().count() < 3
}

fn add_main_wrapper(code: &str) -> String {
    if code.contains("fn main(") {
        return code.to_string();
    }

    if code.contains("pub fn") && code.contains("{") {
        return code.to_string();
    }

    format!(
        "fn main() {{\n{}}}\n",
        code.lines()
            .map(|line| format!("    {}", line))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn format_error(error: &syn::Error) -> String {
    let error_str = error.to_string();

    let simplified = error_str
        .split('\n')
        .next()
        .unwrap_or(&error_str)
        .to_string();

    let trimmed = simplified.trim_start_matches("error: ").to_string();

    if trimmed.len() > 100 {
        format!("{}...", &trimmed[..100])
    } else {
        trimmed
    }
}
