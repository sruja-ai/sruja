use crate::error::Result;
use colored::Colorize;
use std::path::PathBuf;
use walkdir::WalkDir;

struct FileFormatResult {
    file: String,
    was_formatted: bool,
    issues: Vec<String>,
}

pub async fn run(path: PathBuf, check_only: bool) -> Result<()> {
    println!("{}", "Formatting skill files:".bold());
    println!("  Path: {}", path.display().to_string().cyan());
    if check_only {
        println!("  Mode: Check only (no changes)");
    } else {
        println!("  Mode: Format and fix");
    }
    println!();

    let mut results: Vec<FileFormatResult> = Vec::new();
    let mut total_files = 0;
    let mut formatted_files = 0;
    let mut total_issues = 0;

    for entry in WalkDir::new(&path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();

        if file_path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        total_files += 1;
        let content = std::fs::read_to_string(file_path)?;
        let display_path = file_path.display().to_string();

        let format_result = format_content(&content);

        total_issues += format_result.issues.len();

        if format_result.was_formatted {
            formatted_files += 1;
            results.push(FileFormatResult {
                file: display_path.clone(),
                was_formatted: format_result.was_formatted,
                issues: format_result.issues,
            });

            if !check_only {
                std::fs::write(file_path, &format_result.formatted_content)?;
            }
        }
    }

    println!("{}", "=".repeat(50));
    println!("{}", "Format Summary:".bold());
    println!("  Total files: {}", total_files.to_string().white());
    println!(
        "  {}: {}",
        "Formatted".yellow(),
        formatted_files.to_string().yellow()
    );
    println!(
        "  {}: {}",
        "Already correct".green(),
        (total_files - formatted_files).to_string().green()
    );
    println!(
        "  {}: {}",
        "Issues found".cyan(),
        total_issues.to_string().cyan()
    );

    if check_only && formatted_files > 0 {
        println!();
        println!("{}", "Files that need formatting:".yellow().bold());

        for result in &results {
            if result.was_formatted {
                println!("\n{}", result.file.cyan());
                for issue in &result.issues {
                    println!("  - {}", issue.dimmed());
                }
            }
        }

        Err(crate::error::SkillLintError::FormatCheck(format!(
            "{} file(s) need formatting",
            formatted_files
        )))
    } else if !check_only {
        if formatted_files > 0 {
            println!();
            println!("{}", "Formatted files:".green());
            for result in &results {
                if result.was_formatted {
                    println!("  {}", result.file.green());
                }
            }
        }
        Ok(())
    } else {
        Ok(())
    }
}

struct ContentFormatResult {
    formatted_content: String,
    was_formatted: bool,
    issues: Vec<String>,
}

fn format_content(content: &str) -> ContentFormatResult {
    let mut formatted = content.to_string();
    let mut issues = Vec::new();
    let mut was_formatted = false;

    if let Some(frontmatter_start) = formatted.find("---") {
        if let Some(frontmatter_end) = formatted[frontmatter_start + 3..].find("---") {
            let frontmatter_end = frontmatter_start + 3 + frontmatter_end + 3;

            let frontmatter = &formatted[frontmatter_start..frontmatter_end];
            let formatted_frontmatter = format_frontmatter(frontmatter);

            if formatted_frontmatter != frontmatter {
                formatted.replace_range(frontmatter_start..frontmatter_end, &formatted_frontmatter);
                was_formatted = true;
                issues.push("Formatted YAML frontmatter".to_string());
            }

            let body = &formatted[frontmatter_end..];
            let formatted_body = format_body(body);

            if formatted_body != body {
                formatted.replace_range(frontmatter_end.., &formatted_body);
                was_formatted = true;
                issues.push("Formatted body content".to_string());
            }
        }
    } else {
        let formatted_body = format_body(&formatted);
        if formatted_body != formatted {
            formatted = formatted_body;
            was_formatted = true;
            issues.push("Formatted body content".to_string());
        }
    }

    formatted = normalize_line_endings(&formatted);

    let normalized = formatted.replace("\n\n\n", "\n\n");
    if normalized != formatted {
        formatted = normalized;
        was_formatted = true;
        issues.push("Removed excessive blank lines".to_string());
    }

    ContentFormatResult {
        formatted_content: formatted,
        was_formatted,
        issues,
    }
}

fn format_frontmatter(frontmatter: &str) -> String {
    let lines: Vec<&str> = frontmatter.lines().collect();

    if lines.len() < 2 || !lines[0].starts_with("---") || !lines[lines.len() - 1].starts_with("---")
    {
        return frontmatter.to_string();
    }

    let mut formatted: Vec<String> = vec![lines[0].trim_end().to_string()];
    let mut _indentation = None;

    for (_i, line) in lines.iter().enumerate().skip(1).take(lines.len() - 2) {
        let trimmed = line.trim_end();

        if trimmed.is_empty() {
            formatted.push(String::new());
            continue;
        }

        if let Some(_indent) = detect_indentation(trimmed) {
            if _indentation.is_none() {
                _indentation = Some(_indent);
            }
        }

        formatted.push(trimmed.to_string());
    }

    formatted.push(lines[lines.len() - 1].trim_end().to_string());

    formatted.join("\n")
}

fn format_body(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut formatted: Vec<&str> = Vec::new();
    let mut in_code_block = false;
    let mut _code_block_start = "---";

    for line in &lines {
        let trimmed = line.trim_end();

        if trimmed.starts_with("```") {
            if in_code_block {
                in_code_block = false;
                formatted.push(trimmed);
            } else {
                in_code_block = true;
                _code_block_start = trimmed;
                formatted.push(trimmed);
            }
            continue;
        }

        if in_code_block {
            formatted.push(trimmed);
            continue;
        }

        if trimmed.is_empty() {
            if formatted.last().is_some_and(|l| !l.is_empty()) {
                formatted.push("");
            }
        } else {
            formatted.push(trimmed);
        }
    }

    while formatted.last().is_some_and(|l| l.is_empty()) {
        formatted.pop();
    }

    formatted.join("\n")
}

fn detect_indentation(line: &str) -> Option<usize> {
    line.find(|c: char| !c.is_whitespace())
}

fn normalize_line_endings(content: &str) -> String {
    content.replace("\r\n", "\n").replace('\r', "\n")
}
