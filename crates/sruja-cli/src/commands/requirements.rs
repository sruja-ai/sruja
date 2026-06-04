//! Requirements command: list and filter requirements from .sruja files.

use serde::Serialize;
use std::path::Path;

use crate::commands::CliError;

#[derive(Debug, Serialize)]
pub struct RequirementListItem {
    pub id: String,
    pub title: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub affects: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scenarios: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub adrs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

fn parse_requirements_from_repo(repo_path: &Path) -> Result<Vec<RequirementListItem>, CliError> {
    let resolved =
        crate::utils::architecture_path::resolve_architecture_path(repo_path);
    let arch_path = match resolved {
        Some(p) => p,
        None => return Ok(Vec::new()),
    };
    let content = std::fs::read_to_string(&arch_path).map_err(CliError::Io)?;
    let program = sruja_language::Parser::new(arch_path.to_string_lossy().as_ref())
        .parse(&content)
        .map_err(|e| CliError::validation(format!("Parse error: {:?}", e)))?;

    Ok(program
        .items
        .iter()
        .filter_map(|item| {
            if let sruja_language::TopLevelItem::Requirement(req) = item {
                Some(RequirementListItem {
                    id: req.id.clone(),
                    title: req.title.clone(),
                    r#type: req.r#type.clone(),
                    priority: req.priority.clone(),
                    status: req.status.clone(),
                    affects: req.affects.clone(),
                    scenarios: req.scenarios.clone(),
                    adrs: req.adrs.clone(),
                    source: req.source.clone(),
                })
            } else {
                None
            }
        })
        .collect())
}

pub async fn requirements_list(
    repo: &str,
    format: &str,
    priority: Option<&str>,
    status: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let mut items = parse_requirements_from_repo(repo_path)?;

    if let Some(p) = priority {
        items.retain(|r| {
            r.priority
                .as_deref()
                .map_or(false, |rp| rp.eq_ignore_ascii_case(p))
        });
    }
    if let Some(s) = status {
        items.retain(|r| {
            r.status
                .as_deref()
                .map_or(false, |rs| rs.eq_ignore_ascii_case(s))
        });
    }

    if items.is_empty() {
        if format == "json" {
            println!("[]");
        } else {
            println!("No requirements found.");
        }
        return Ok(());
    }

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&items)?);
    } else {
        for it in &items {
            let priority_str = it
                .priority
                .as_deref()
                .map(|p| format!(" [{}]", p))
                .unwrap_or_default();
            let status_str = it
                .status
                .as_deref()
                .map(|s| format!(" ({})", s))
                .unwrap_or_default();
            println!(
                "{} {}{}{} — {}",
                it.id, it.r#type, priority_str, status_str, it.title
            );
            if !it.affects.is_empty() {
                println!("  affects: {}", it.affects.join(", "));
            }
            if !it.scenarios.is_empty() {
                println!("  scenarios: {}", it.scenarios.join(", "));
            }
            if !it.adrs.is_empty() {
                println!("  adrs: {}", it.adrs.join(", "));
            }
            if let Some(source) = &it.source {
                println!("  source: {}", source);
            }
        }
    }
    Ok(())
}
