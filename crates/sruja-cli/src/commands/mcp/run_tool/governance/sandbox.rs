use serde_json::Value;

use super::super::finish;
use crate::commands::CliError;

pub(crate) async fn handle(
    arguments: &Value,
    repo: &str,
) -> Result<Option<String>, CliError> {
    let action = arguments
        .get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::validation("Missing action"))?;

    let name = arguments.get("name").and_then(|v| v.as_str());
    let sruja_dir = std::path::Path::new(&repo).join(".sruja");
    let sandbox_dir = sruja_dir.join("sandboxes");
    std::fs::create_dir_all(&sandbox_dir)?;

    match action {
        "create" => {
            let name =
                name.ok_or_else(|| CliError::validation("Missing name for create"))?;
            let target = sandbox_dir.join(name);
            if target.exists() {
                return Err(CliError::validation(format!(
                    "Sandbox '{}' already exists",
                    name
                )));
            }

            let output = std::process::Command::new("git")
                .args([
                    "worktree",
                    "add",
                    "-b",
                    &format!("sruja-sandbox/{}", name),
                    target.to_str().ok_or_else(|| {
                        CliError::validation("Target path is not valid UTF-8")
                    })?,
                ])
                .current_dir(repo)
                .output()?;

            if !output.status.success() {
                return Err(CliError::validation(format!(
                    "Git worktree failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
            finish(Ok(format!("✅ Created isolated sandbox at {}. Run your tools and evaluations against this path.", target.display())))
        }
        "discard" => {
            let name =
                name.ok_or_else(|| CliError::validation("Missing name for discard"))?;
            let target = sandbox_dir.join(name);

            if !target.exists() {
                return Err(CliError::validation(format!(
                    "Sandbox '{}' not found",
                    name
                )));
            }

            std::process::Command::new("git")
                .args([
                    "worktree",
                    "remove",
                    "--force",
                    target.to_str().ok_or_else(|| {
                        CliError::validation("Target path is not valid UTF-8")
                    })?,
                ])
                .current_dir(repo)
                .output()?;

            std::process::Command::new("git")
                .args(["branch", "-D", &format!("sruja-sandbox/{}", name)])
                .current_dir(repo)
                .output()?;

            finish(Ok(format!("🗑️ Discarded sandbox '{}'.", name)))
        }
        "commit" => {
            let name =
                name.ok_or_else(|| CliError::validation("Missing name for commit"))?;
            let target = sandbox_dir.join(name);

            if !target.exists() {
                return Err(CliError::validation(format!(
                    "Sandbox '{}' not found",
                    name
                )));
            }

            std::process::Command::new("git")
                .args(["add", "-A"])
                .current_dir(&target)
                .output()?;

            std::process::Command::new("git")
                .args(["commit", "-m", &format!("Sruja Sandbox: {}", name)])
                .current_dir(&target)
                .output()?;

            finish(Ok(format!("✅ Sandbox '{}' successfully committed to branch 'sruja-sandbox/{}'. A human can now merge this into the main branch.", name, name)))
        }
        "list" => {
            if let Ok(entries) = std::fs::read_dir(&sandbox_dir) {
                let mut sandboxes = Vec::new();
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        sandboxes
                            .push(format!("- {}", entry.file_name().to_string_lossy()));
                    }
                }
                if sandboxes.is_empty() {
                    finish(Ok("No active sandboxes.".to_string()))
                } else {
                    finish(Ok(format!("Active Sandboxes:\n{}", sandboxes.join("\n"))))
                }
            } else {
                finish(Ok("No active sandboxes.".to_string()))
            }
        }
        _ => Err(CliError::validation(format!(
            "Invalid sandbox action: {}",
            action
        ))),
    }
}
