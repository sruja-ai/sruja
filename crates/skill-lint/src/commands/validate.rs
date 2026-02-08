use crate::checker::SkillChecker;
use crate::error::Result;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub async fn run(schema_path: PathBuf, skills_path: PathBuf) -> Result<()> {
    println!("{}", "Validating skill files against schema:".bold());
    println!("  Schema: {}", schema_path.display().to_string().cyan());
    println!("  Skills: {}", skills_path.display().to_string().cyan());
    println!();

    let schema_content = fs::read_to_string(&schema_path)?;
    let checker = SkillChecker::new(&schema_content)?;

    let mut total_files = 0;
    let mut valid_files = 0;
    let mut invalid_files = 0;
    let mut errors: Vec<(String, Vec<String>)> = Vec::new();

    for entry in WalkDir::new(&skills_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        total_files += 1;
        let display_path = path.display().to_string();

        match checker.check_file(path) {
            Ok(diagnostics) => {
                if diagnostics.is_empty() {
                    valid_files += 1;
                    println!("{} {}", "✓".green(), display_path);
                } else {
                    invalid_files += 1;
                    let error_messages: Vec<String> = diagnostics
                        .iter()
                        .map(|d| {
                            let line_info = d.line.map(|l| format!(":{}", l)).unwrap_or_default();
                            format!(
                                "  {}: {}{}",
                                d.level.to_string().red(),
                                d.message,
                                line_info
                            )
                        })
                        .collect();
                    errors.push((display_path.clone(), error_messages));
                    println!("{} {}", "✗".red(), display_path);
                }
            }
            Err(e) => {
                invalid_files += 1;
                errors.push((display_path.clone(), vec![format!("  Error: {}", e)]));
                println!("{} {}", "✗".red(), display_path);
            }
        }
    }

    println!();
    println!("{}", "=".repeat(50));
    println!("{}", "Validation Summary:".bold());
    println!("  Total files: {}", total_files.to_string().white());
    println!("  {}: {}", "Valid".green(), valid_files.to_string().green());
    println!("  {}: {}", "Invalid".red(), invalid_files.to_string().red());

    if !errors.is_empty() {
        println!();
        println!("{}", "Errors:".red().bold());
        for (path, error_msgs) in errors {
            println!("\n{}:", path.yellow());
            for msg in error_msgs {
                println!("{}", msg);
            }
        }
        Err(crate::error::SkillLintError::SchemaValidation(format!(
            "{} invalid file(s) found",
            invalid_files
        )))
    } else {
        Ok(())
    }
}
