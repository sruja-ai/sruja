//! Skill suggestion system (simplified version)
//!
//! Provides intelligent rule suggestions based on project analysis.

use crate::modules::skills::context::{analyze_project_context, ProjectContext};
use crate::modules::skills::filter::Level;
use crate::modules::skills::filter::{OutputFormat, SkillFilter};
use crate::modules::skills::loader::load_filtered_skills;
use std::collections::HashSet;
use std::path::Path;

/// Suggest rules based on project context
pub fn suggest_rules(
    skills_path: &Path,
    project_path: &Path,
    count: usize,
    level: Option<Level>,
) -> Result<String, String> {
    let context = analyze_project_context(project_path);

    let mut filter = SkillFilter::new();
    filter.output_format = OutputFormat::Concise;
    filter.limit = Some(count);

    if let Some(lvl) = level {
        let mut levels = HashSet::new();
        levels.insert(lvl);
        filter.levels = Some(levels);
    }

    let skills_output = load_filtered_skills(skills_path, &filter)?;
    let analysis = format_analysis_summary(&context);

    Ok(format!("{}\n\n{}", analysis, skills_output))
}

fn format_analysis_summary(context: &ProjectContext) -> String {
    let mut summary = String::new();

    summary.push_str("# Project Analysis\n\n");

    let project_type = if context.web {
        "Web Application"
    } else if context.cli {
        "CLI Application"
    } else if context.embedded {
        "Embedded System"
    } else if context.wasm {
        "WebAssembly Target"
    } else if context.library {
        "Library"
    } else {
        "General Application"
    };

    summary.push_str(&format!("**Project Type:** {}\n", project_type));

    if context.is_async {
        summary.push_str("**Async:** Yes\n");
    }

    summary.push_str(&format!(
        "**Complexity Score:** {:.1}/1.0\n",
        context.complexity_score
    ));

    summary.push_str("\n**Relevant Context:**\n");

    if context.is_async {
        summary.push_str("- Async project (tokio/async-std)\n");
    }
    if context.web {
        summary.push_str("- Web framework detected\n");
    }
    if context.cli {
        summary.push_str("- CLI application\n");
    }
    if context.embedded {
        summary.push_str("- Embedded system\n");
    }
    if context.wasm {
        summary.push_str("- WebAssembly target\n");
    }
    if context.library {
        summary.push_str("- Library crate\n");
    }

    summary
}
