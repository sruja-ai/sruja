use crate::commands::CliError;
use std::fs;
use std::path::Path;

const AGENTS_MD_MARKER: &str = "## Sruja";
const AGENTS_MD_SECTION: &str = r#"## Sruja

This project has a Sruja architecture model at repo.sruja.

Rules:
- Before making architectural changes, run `sruja scan -r .` or check `.sruja/context.json` if available to understand the system structure.
- After modifying code files that affect architecture, suggest running `sruja sync -r .` to keep the model current.
- When answering architecture or codebase questions, prefer using `sruja why "<question>" -r .` to search the knowledge graph.
- If asked to propose an architectural change, use `sruja propose` workflow.
"#;

const CLAUDE_MD_MARKER: &str = "## Sruja";
const CLAUDE_MD_SECTION: &str = r#"## Sruja

This project has a Sruja architecture model at repo.sruja.

Rules:
- Before making architectural changes, run `sruja scan -r .` or check `.sruja/context.json` if available to understand the system structure.
- After modifying code files that affect architecture, suggest running `sruja sync -r .` to keep the model current.
- When answering architecture or codebase questions, prefer using `sruja why "<question>" -r .` to search the knowledge graph.
- If asked to propose an architectural change, use `sruja propose` workflow.
"#;

pub async fn agent_install(repo: &str, platform: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::validation("Repository path does not exist"));
    }

    match platform.to_lowercase().as_str() {
        "codex" | "opencode" | "aider" | "claw" | "droid" | "trae" | "trae-cn" | "kiro"
        | "hermes" | "gemini" => {
            install_agents_md(repo_path, platform)?;
        }
        "claude" => {
            install_claude_md(repo_path)?;
        }
        _ => {
            return Err(CliError::validation(format!(
                "Unsupported platform: {}",
                platform
            )));
        }
    }

    Ok(())
}

pub async fn agent_uninstall(repo: &str, platform: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::validation("Repository path does not exist"));
    }

    match platform.to_lowercase().as_str() {
        "codex" | "opencode" | "aider" | "claw" | "droid" | "trae" | "trae-cn" | "kiro"
        | "hermes" | "gemini" => {
            uninstall_agents_md(repo_path)?;
        }
        "claude" => {
            uninstall_claude_md(repo_path)?;
        }
        _ => {
            return Err(CliError::validation(format!(
                "Unsupported platform: {}",
                platform
            )));
        }
    }

    Ok(())
}

fn install_agents_md(repo_path: &Path, platform: &str) -> Result<(), CliError> {
    let target = repo_path.join("AGENTS.md");

    if target.exists() {
        let content = fs::read_to_string(&target)?;
        if content.contains(AGENTS_MD_MARKER) {
            println!("Sruja already configured in AGENTS.md");
        } else {
            let new_content = format!("{}\n\n{}", content.trim_end(), AGENTS_MD_SECTION);
            fs::write(&target, new_content)?;
            println!("Sruja section written to {}", target.display());
        }
    } else {
        fs::write(&target, AGENTS_MD_SECTION)?;
        println!("Sruja section written to {}", target.display());
    }

    println!(
        "\n{} will now check the knowledge graph before answering",
        platform
    );
    println!("codebase questions and rebuild it after code changes.");

    Ok(())
}

fn uninstall_agents_md(repo_path: &Path) -> Result<(), CliError> {
    let target = repo_path.join("AGENTS.md");

    if !target.exists() {
        println!("No AGENTS.md found in current directory - nothing to do");
        return Ok(());
    }

    let content = fs::read_to_string(&target)?;
    if !content.contains(AGENTS_MD_MARKER) {
        println!("Sruja section not found in AGENTS.md - nothing to do");
        return Ok(());
    }

    // A simple replacement to remove the section
    // In a real implementation this would need to handle the end of the section robustly
    let mut new_content = String::new();
    let mut in_section = false;

    for line in content.lines() {
        if line.starts_with(AGENTS_MD_MARKER) {
            in_section = true;
            continue;
        }

        if in_section && line.starts_with("## ") {
            in_section = false;
        }

        if !in_section {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    fs::write(&target, new_content.trim_end())?;
    println!("Sruja section removed from {}", target.display());

    Ok(())
}

fn install_claude_md(repo_path: &Path) -> Result<(), CliError> {
    let target = repo_path.join("CLAUDE.md");

    if target.exists() {
        let content = fs::read_to_string(&target)?;
        if content.contains(CLAUDE_MD_MARKER) {
            println!("Sruja already configured in CLAUDE.md");
        } else {
            let new_content = format!("{}\n\n{}", content.trim_end(), CLAUDE_MD_SECTION);
            fs::write(&target, new_content)?;
            println!("Sruja section written to {}", target.display());
        }
    } else {
        fs::write(&target, CLAUDE_MD_SECTION)?;
        println!("Sruja section written to {}", target.display());
    }

    println!("\nClaude Code will now check the knowledge graph before answering");
    println!("codebase questions and rebuild it after code changes.");

    Ok(())
}

fn uninstall_claude_md(repo_path: &Path) -> Result<(), CliError> {
    let target = repo_path.join("CLAUDE.md");

    if !target.exists() {
        println!("No CLAUDE.md found in current directory - nothing to do");
        return Ok(());
    }

    let content = fs::read_to_string(&target)?;
    if !content.contains(CLAUDE_MD_MARKER) {
        println!("Sruja section not found in CLAUDE.md - nothing to do");
        return Ok(());
    }

    let mut new_content = String::new();
    let mut in_section = false;

    for line in content.lines() {
        if line.starts_with(CLAUDE_MD_MARKER) {
            in_section = true;
            continue;
        }

        if in_section && line.starts_with("## ") {
            in_section = false;
        }

        if !in_section {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    fs::write(&target, new_content.trim_end())?;
    println!("Sruja section removed from {}", target.display());

    Ok(())
}
