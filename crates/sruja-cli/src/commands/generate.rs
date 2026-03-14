//! Generate a prompt that combines the Sruja architecture skill and repo context
//! so any LLM can produce architecture.sruja without Cursor CLI. See docs/SKILLS_WITHOUT_CURSOR_CLI.md.

use std::path::Path;

use super::CliError;
use crate::commands::discover::discover_context_string;

const PROMPT_INSTRUCTION: &str = r#"You are an architecture discovery agent. Follow the rules and context below. Your reply must be only valid Sruja DSL (no markdown fences, no extra commentary before or after the DSL). If you are uncertain about boundaries or externals, append a line "// Open questions: ..." at the end. The user will run `sruja lint` on the output; fix any errors until it passes.

---
SKILL (follow these rules):
"#;

const PROMPT_AFTER_SKILL: &str = r#"
---
REPO CONTEXT (use this to tailor the architecture; derive 2–5 questions if needed, then produce the .sruja):
"#;

/// Resolve path to the skill file: --skill-path, SRUJA_SKILL_PATH, or defaults.
fn resolve_skill_path(skill_path: Option<&str>) -> Option<std::path::PathBuf> {
    if let Some(p) = skill_path {
        let path = Path::new(p);
        if path.exists() {
            return Some(path.to_path_buf());
        }
        return None;
    }
    if let Ok(p) = std::env::var("SRUJA_SKILL_PATH") {
        let path = Path::new(&p);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }
    // Defaults: ./SKILL.md, then ./skills/sruja-architecture/SKILL.md
    for default in ["./SKILL.md", "./skills/sruja-architecture/SKILL.md"] {
        let path = Path::new(default);
        if path.exists() {
            return Some(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
        }
    }
    None
}

/// Generate a prompt file containing skill + repo context for use with any LLM.
pub fn generate_prompt(
    repo: &str,
    skill_path: Option<&str>,
    output_path: Option<&str>,
) -> Result<(), CliError> {
    let skill_path_buf = resolve_skill_path(skill_path).ok_or_else(|| {
        CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Skill file not found. Set --skill-path or SRUJA_SKILL_PATH to the path to sruja-architecture/SKILL.md (e.g. /path/to/sruja/skills/sruja-architecture/SKILL.md), or put SKILL.md in the current directory.",
        ))
    })?;

    let skill_content = std::fs::read_to_string(&skill_path_buf).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!(
                "Failed to read skill file {}: {}",
                skill_path_buf.display(),
                e
            ),
        ))
    })?;

    let context = discover_context_string(repo)?;

    let prompt = format!(
        "{}{}{}{}",
        PROMPT_INSTRUCTION, skill_content, PROMPT_AFTER_SKILL, context
    );

    if let Some(out) = output_path {
        std::fs::write(out, &prompt).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write prompt to {}: {}", out, e),
            ))
        })?;
        println!("Wrote prompt to {}. Use it with any LLM; save the model output as architecture.sruja then run: sruja lint architecture.sruja", out);
    } else {
        println!("{}", prompt);
    }

    Ok(())
}
