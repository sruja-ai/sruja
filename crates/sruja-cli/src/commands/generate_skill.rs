//! Generate a prompt for AI to extract procedural knowledge and create a project skill.
//!
//! This command collects sruja evidence (classification, context, graph) and formats
//! it into a prompt that AI agents can use to extract procedural knowledge and generate
//! a project-specific skill.

use std::fs;
use std::path::Path;

use super::CliError;

/// Options for skill prompt generation.
#[derive(Debug, Clone)]
pub struct GenerateSkillPromptOptions<'a> {
    pub repo: &'a str,
    pub output: Option<&'a str>,
}

const SKILL_GENERATION_PROMPT: &str = r#"You are a skill extraction agent. Your task is to analyze the repository evidence below and extract procedural knowledge into a project-specific skill.

## What to Extract

Extract **procedural knowledge** (how to do things), NOT factual data (static counts, lists).

### 1. Workflows (SKILL.md)
- How to add a new component/module/service
- How to validate architecture changes
- How to run drift detection
- How to handle cross-cutting concerns (auth, logging, etc.)
- How to onboard new developers

### 2. Patterns (rules/common-patterns.md)
- Naming conventions used in this project
- Module organization patterns
- Error handling patterns
- Testing patterns
- Dependency injection patterns

### 3. Anti-patterns (rules/anti-patterns.md)
- What NOT to do (from forbidden_patterns and observed patterns)
- Common mistakes to avoid
- Layer violations to watch for

### 4. Project-specific rules (rules/*.md)
- How to add a new crate/module (if multi-module)
- How to add a new API endpoint
- How to add a new database migration
- How to handle configuration

## Output Format

Generate a skill directory structure:

```
skills/project/
  SKILL.md              # Main entry point with workflows
  rules/
    add-component.md    # How to add components
    validate-changes.md # How to validate
    run-drift.md        # How to run drift detection
    common-patterns.md  # Common patterns
    anti-patterns.md    # Anti-patterns to avoid
```

## SKILL.md Format

```markdown
---
name: project-architecture
description: >
  Project-specific architecture workflows and patterns.
  Teaches AI editors how to work with this codebase's architecture.
license: Apache-2.0
---

# Project Architecture Skill

[Procedural workflows extracted from evidence]

## Workflows

### Adding a New Component
[Step-by-step procedure based on actual project structure]

### Validating Changes
[Validation workflow based on project tools]

## Progressive Discovery

| Task | Load only |
|------|-----------|
| Add component | `rules/add-component.md` |
| Validate changes | `rules/validate-changes.md` |
| ... | ... |

## Quick Start

[How to use this skill]
```

## Rules Format

Each rule file should follow this structure:

```markdown
# rule-name

## Why It Matters
[Why this procedure exists]

## When to Apply
[When to use this procedure]

## Correct Approach
[Step-by-step procedure]

## Incorrect Approach
[What NOT to do]

## Summary
[One-line summary]
```

## Important Guidelines

1. **Extract from evidence, not templates** - Use the actual project structure, tools, and patterns
2. **Be specific** - Reference actual commands, file paths, and patterns from the project
3. **Include examples** - Show real code snippets and commands
4. **Focus on procedures** - How to do things, not what things are
5. **Keep it concise** - Each rule should be focused and actionable

---

## Repository Evidence

"#;

/// Generate a prompt for AI to extract procedural knowledge.
pub fn generate_skill_prompt(options: GenerateSkillPromptOptions<'_>) -> Result<(), CliError> {
    let repo_path = Path::new(options.repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", options.repo),
        )));
    }

    let mut evidence = String::new();

    // Collect classification evidence
    let classification_path = repo_path.join(".sruja").join("classification.json");
    if classification_path.exists() {
        let content = fs::read_to_string(&classification_path)?;
        evidence.push_str("## Classification\n\n```json\n");
        evidence.push_str(&content);
        evidence.push_str("\n```\n\n");
    }

    // Collect context evidence
    let context_path = repo_path.join(".sruja").join("context.json");
    if context_path.exists() {
        let content = fs::read_to_string(&context_path)?;
        evidence.push_str("## Context\n\n```json\n");
        evidence.push_str(&content);
        evidence.push_str("\n```\n\n");
    }

    // Collect AGENTS.md if exists
    let agents_path = repo_path.join("AGENTS.md");
    if agents_path.exists() {
        let content = fs::read_to_string(&agents_path)?;
        evidence.push_str("## AGENTS.md\n\n```markdown\n");
        evidence.push_str(&content);
        evidence.push_str("\n```\n\n");
    }

    // Collect .cursorrules if exists
    let cursorrules_path = repo_path.join(".cursorrules");
    if cursorrules_path.exists() {
        let content = fs::read_to_string(&cursorrules_path)?;
        evidence.push_str("## .cursorrules\n\n```\n");
        evidence.push_str(&content);
        evidence.push_str("\n```\n\n");
    }

    // Collect repo.sruja if exists
    let repo_sruja_path = repo_path.join("repo.sruja");
    if repo_sruja_path.exists() {
        let content = fs::read_to_string(&repo_sruja_path)?;
        evidence.push_str("## repo.sruja\n\n```sruja\n");
        evidence.push_str(&content);
        evidence.push_str("\n```\n\n");
    }

    if evidence.is_empty() {
        return Err(CliError::validation(
            "No sruja evidence found. Run `sruja sync -r .` and `sruja classify -r .` first.".to_string(),
        ));
    }

    let prompt = format!("{}{}", SKILL_GENERATION_PROMPT, evidence);

    match options.output {
        Some(path) => {
            fs::write(path, &prompt)?;
            eprintln!("Wrote skill generation prompt to: {}", path);
            eprintln!("Feed this to an AI agent to generate the skill.");
        }
        None => {
            println!("{}", prompt);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_generate_skill_prompt_no_evidence() {
        let dir = tempdir().unwrap();
        let options = GenerateSkillPromptOptions {
            repo: dir.path().to_str().unwrap(),
            output: None,
        };

        let result = generate_skill_prompt(options);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("No sruja evidence found"));
    }

    #[test]
    fn test_generate_skill_prompt_with_classification() {
        let dir = tempdir().unwrap();
        let sruja_dir = dir.path().join(".sruja");
        fs::create_dir_all(&sruja_dir).unwrap();

        let classification = r#"{
            "schema_version": "classification/v1",
            "project_type": "rust-workspace",
            "summary": { "crates": 5, "source_files": 50 },
            "layers": [],
            "boundaries": [],
            "forbidden_patterns": []
        }"#;
        fs::write(sruja_dir.join("classification.json"), classification).unwrap();

        let output_path = dir.path().join("prompt.md");
        let options = GenerateSkillPromptOptions {
            repo: dir.path().to_str().unwrap(),
            output: Some(output_path.to_str().unwrap()),
        };

        let result = generate_skill_prompt(options);
        assert!(result.is_ok());

        let prompt = fs::read_to_string(&output_path).unwrap();
        assert!(prompt.contains("skill extraction agent"));
        assert!(prompt.contains("rust-workspace"));
    }

    #[test]
    fn test_generate_skill_prompt_with_context() {
        let dir = tempdir().unwrap();
        let sruja_dir = dir.path().join(".sruja");
        fs::create_dir_all(&sruja_dir).unwrap();

        let context = r#"{
            "total_modules": 10,
            "total_services": 2,
            "total_databases": 1
        }"#;
        fs::write(sruja_dir.join("context.json"), context).unwrap();

        let options = GenerateSkillPromptOptions {
            repo: dir.path().to_str().unwrap(),
            output: None,
        };

        let result = generate_skill_prompt(options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_skill_prompt_repo_not_found() {
        let options = GenerateSkillPromptOptions {
            repo: "/nonexistent/path",
            output: None,
        };

        let result = generate_skill_prompt(options);
        assert!(result.is_err());
    }
}
