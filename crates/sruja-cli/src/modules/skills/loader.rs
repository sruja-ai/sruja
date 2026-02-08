//! Skill loading with filtering (simplified version)
//!
//! Simple file-based skill discovery without complex metadata parsing.

use crate::modules::skills::filter::{Category, Level, OutputFormat, SkillFilter};
use std::fs;
use std::path::Path;

/// Load filtered skills from rules directory
pub fn load_filtered_skills(skills_path: &Path, filter: &SkillFilter) -> Result<String, String> {
    let mut skills = Vec::new();

    // Find all rule files
    let rules_path = skills_path.join("rules");

    if !rules_path.exists() {
        // Try beginner/intermediate/advanced subdirectories
        for level_dir in [Level::Beginner, Level::Intermediate, Level::Advanced] {
            let level_path = match filter.levels.as_ref() {
                Some(levels) if levels.contains(&level_dir) => {
                    skills_path.join(level_dir.to_string()).join("rules")
                }
                _ => skills_path.join(level_dir.to_string()).join("rules"),
            };

            if level_path.exists() {
                load_rules_from_dir(&level_path, filter, &mut skills)?;
            }
        }
    } else {
        load_rules_from_dir(&rules_path, filter, &mut skills)?;
    }

    if skills.is_empty() {
        return Ok(format!(
            "# No skills found matching criteria\n\nPath: {:?}",
            skills_path
        ));
    }

    let skills = if let Some(limit) = filter.limit {
        skills.into_iter().take(limit).collect()
    } else {
        skills
    };

    match filter.output_format {
        OutputFormat::Json => Ok(format_skills_json(&skills)),
        OutputFormat::Markdown => Ok(format_skills_markdown(&skills)),
        OutputFormat::Concise => Ok(format_skills_concise(&skills)),
    }
}

fn load_rules_from_dir(
    dir_path: &Path,
    _filter: &SkillFilter,
    skills: &mut Vec<SimpleSkill>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory {:?}: {}", dir_path, e))?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        skills.push(SimpleSkill {
            id: stem.to_string(),
            path: path.to_string_lossy().to_string(),
        });
    }

    skills.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(())
}

#[derive(Debug, Clone)]
struct SimpleSkill {
    id: String,
    path: String,
}

fn format_skills_markdown(skills: &[SimpleSkill]) -> String {
    let mut output = String::new();

    output.push_str("# Rust Skills\n\n");
    output.push_str(&format!("**Total:** {} rules\n\n", skills.len()));

    for skill in skills {
        output.push_str(&format!("- [{}]({}.md)\n", skill.id, skill.id));
    }

    output
}

fn format_skills_json(skills: &[SimpleSkill]) -> String {
    let json_skills: Vec<serde_json::Value> = skills
        .iter()
        .map(|s| serde_json::json!({ "id": s.id, "path": s.path }))
        .collect();

    serde_json::to_string_pretty(&serde_json::json!({
        "total": skills.len(),
        "skills": json_skills
    }))
    .unwrap_or_default()
}

fn format_skills_concise(skills: &[SimpleSkill]) -> String {
    skills
        .iter()
        .map(|s| format!("{} - {}", s.id, s.path))
        .collect::<Vec<_>>()
        .join("\n")
}
