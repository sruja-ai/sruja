use crate::error::Result;
use colored::Colorize;
use regex::Regex;
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

struct XrefCheckResult {
    file: String,
    line: Option<usize>,
    reference: String,
    status: XrefStatus,
}

#[derive(Debug)]
enum XrefStatus {
    Ok,
    FileNotFound,
}

pub async fn run(path: PathBuf) -> Result<()> {
    println!("{}", "Checking cross-references in skill files:".bold());
    println!("  Path: {}", path.display().to_string().cyan());
    println!();

    let mut rule_files: HashMap<String, String> = HashMap::new();
    let mut results: Vec<XrefCheckResult> = Vec::new();
    let mut total_refs = 0;
    let mut broken_refs = 0;

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

        if let Some(rule_id) = extract_rule_id(&display_path, &content) {
            rule_files.insert(rule_id, display_path.clone());
        }
    }

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

        results.extend(check_xrefs_in_metadata(
            &content,
            &display_path,
            &rule_files,
        ));
        results.extend(check_xrefs_in_body(
            &content,
            &display_path,
            file_path.parent().unwrap_or(Path::new("")),
        ));
    }

    for result in &results {
        total_refs += 1;
        if !matches!(result.status, XrefStatus::Ok) {
            broken_refs += 1;
        }
    }

    println!("{}", "=".repeat(50));
    println!("{}", "Cross-Reference Check Summary:".bold());
    println!("  Total references: {}", total_refs.to_string().white());
    println!(
        "  {}: {}",
        "Valid".green(),
        (total_refs - broken_refs).to_string().green()
    );
    println!("  {}: {}", "Broken".red(), broken_refs.to_string().red());

    if broken_refs > 0 {
        println!();
        println!("{}", "Broken Cross-References:".red().bold());

        for result in &results {
            if !matches!(result.status, XrefStatus::Ok) {
                let line_info = result.line.map(|l| format!(":{}", l)).unwrap_or_default();
                let status_msg = match &result.status {
                    XrefStatus::FileNotFound => "Referenced file not found".to_string(),
                    XrefStatus::Ok => unreachable!(),
                };

                println!("\n{}{}", result.file.yellow(), line_info);
                println!(
                    "  {} {} - {}",
                    "✗".red(),
                    result.reference.cyan(),
                    status_msg
                );
            }
        }

        Err(crate::error::SkillLintError::XrefCheck(format!(
            "{} broken cross-reference(s) found",
            broken_refs
        )))
    } else {
        Ok(())
    }
}

fn extract_rule_id(file_path: &str, content: &str) -> Option<String> {
    if let Some(frontmatter) = extract_frontmatter(content) {
        if let Ok(yaml) = serde_yaml::from_str::<Value>(&frontmatter) {
            if let Some(_metadata) = yaml.get("metadata") {
                let path = PathBuf::from(file_path);
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                return Some(file_name.to_string());
            }
        }
    }
    None
}

fn extract_frontmatter(content: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }

    content
        .lines()
        .skip(1)
        .take_while(|line| *line != "---")
        .collect::<Vec<_>>()
        .join("\n")
        .into()
}

fn check_xrefs_in_metadata(
    content: &str,
    file_path: &str,
    rule_files: &HashMap<String, String>,
) -> Vec<XrefCheckResult> {
    let mut results = Vec::new();

    if let Some(frontmatter) = extract_frontmatter(content) {
        if let Ok(yaml) = serde_yaml::from_str::<Value>(&frontmatter) {
            if let Some(_metadata) = yaml.get("metadata") {
                if let Some(related) = _metadata.get("related_rules") {
                    if let Some(related_array) = related.as_sequence() {
                        for item in related_array {
                            if let Some(rule_id) = item.as_str() {
                                let status = if rule_files.contains_key(rule_id) {
                                    XrefStatus::Ok
                                } else {
                                    XrefStatus::FileNotFound
                                };

                                results.push(XrefCheckResult {
                                    file: file_path.to_string(),
                                    line: None,
                                    reference: rule_id.to_string(),
                                    status,
                                });
                            }
                        }
                    }
                }

                if let Some(alternatives) = _metadata.get("alternatives") {
                    if let Some(alt_array) = alternatives.as_sequence() {
                        for item in alt_array {
                            if let Some(rule_id) = item.as_str() {
                                let cleaned_id = rule_id.trim().trim_matches('`');
                                let status = if rule_files.contains_key(cleaned_id) {
                                    XrefStatus::Ok
                                } else {
                                    XrefStatus::FileNotFound
                                };

                                results.push(XrefCheckResult {
                                    file: file_path.to_string(),
                                    line: None,
                                    reference: cleaned_id.to_string(),
                                    status,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    results
}

fn check_xrefs_in_body(content: &str, file_path: &str, base_path: &Path) -> Vec<XrefCheckResult> {
    let mut results = Vec::new();

    let link_regex = Regex::new(r"\[`([^\]]+)`\]\(([^\)]+)\)").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        for caps in link_regex.captures_iter(line) {
            if let Some(link_target) = caps.get(2) {
                let target = link_target.as_str();

                if target.starts_with('#') {
                    continue;
                }

                let resolved_path = base_path.join(target);
                let status = if resolved_path.exists() {
                    XrefStatus::Ok
                } else {
                    XrefStatus::FileNotFound
                };

                results.push(XrefCheckResult {
                    file: file_path.to_string(),
                    line: Some(line_num + 1),
                    reference: target.to_string(),
                    status,
                });
            }
        }
    }

    results
}
