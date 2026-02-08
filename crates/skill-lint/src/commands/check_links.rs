use crate::error::Result;
use colored::Colorize;
use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;

struct LinkCheckResult {
    file: String,
    link: String,
    line: Option<usize>,
    is_external: bool,
    status: LinkStatus,
}

#[derive(Debug, Clone)]
enum LinkStatus {
    Ok,
    NotFound,
    InvalidUrl,
    Timeout,
    OtherError(String),
}

pub async fn run(path: PathBuf) -> Result<()> {
    println!("{}", "Checking links in skill files:".bold());
    println!("  Path: {}", path.display().to_string().cyan());
    println!();

    let http_client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("skill-lint/0.1.0")
        .build()?;

    let mut results: Vec<LinkCheckResult> = Vec::new();
    let mut total_links = 0;
    let mut broken_links = 0;

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

        let current_line = 1;
        let link_checks = check_links_in_content(&content, &display_path, current_line);

        for check in link_checks {
            total_links += 1;
            let status = if check.is_external {
                check_external_link(&http_client, &check.link).await
            } else {
                check_relative_link(file_path.parent().unwrap_or(Path::new("")), &check.link)
            };

            results.push(LinkCheckResult {
                file: display_path.clone(),
                link: check.link.clone(),
                line: check.line,
                is_external: check.is_external,
                status: status.clone(),
            });

            if !matches!(status, LinkStatus::Ok) {
                broken_links += 1;
            }
        }
    }

    println!("{}", "=".repeat(50));
    println!("{}", "Link Check Summary:".bold());
    println!("  Total links: {}", total_links.to_string().white());
    println!(
        "  {}: {}",
        "Valid".green(),
        (total_links - broken_links).to_string().green()
    );
    println!("  {}: {}", "Broken".red(), broken_links.to_string().red());

    if broken_links > 0 {
        println!();
        println!("{}", "Broken Links:".red().bold());

        for result in &results {
            if !matches!(result.status, LinkStatus::Ok) {
                let line_info = result.line.map(|l| format!(":{}", l)).unwrap_or_default();
                let status_msg = match &result.status {
                    LinkStatus::NotFound => "Not found".to_string(),
                    LinkStatus::InvalidUrl => "Invalid URL".to_string(),
                    LinkStatus::Timeout => "Request timeout".to_string(),
                    LinkStatus::OtherError(msg) => msg.clone(),
                    LinkStatus::Ok => unreachable!(),
                };

                println!(
                    "\n{}{} {}",
                    result.file.yellow(),
                    line_info,
                    format!(
                        "({})",
                        if result.is_external {
                            "external"
                        } else {
                            "internal"
                        }
                    )
                    .dimmed()
                );
                println!("  {} {}", "✗".red(), result.link.cyan());
                println!("  {} {}", "Reason:".red(), status_msg);
            }
        }

        Err(crate::error::SkillLintError::LinkCheck(format!(
            "{} broken link(s) found",
            broken_links
        )))
    } else {
        Ok(())
    }
}

struct PendingLinkCheck {
    link: String,
    line: Option<usize>,
    is_external: bool,
}

fn check_links_in_content(
    content: &str,
    _file: &str,
    mut current_line: usize,
) -> Vec<PendingLinkCheck> {
    let mut checks = Vec::new();
    let parser = Parser::new(content);

    let url_regex = Regex::new(r"^https?://").unwrap();

    for event in parser {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                let link = dest_url.to_string();
                let is_external = url_regex.is_match(&link);

                checks.push(PendingLinkCheck {
                    link,
                    line: Some(current_line),
                    is_external,
                });
            }
            Event::Text(text) => {
                current_line += text.lines().count();
            }
            Event::SoftBreak | Event::HardBreak => {
                current_line += 1;
            }
            _ => {}
        }
    }

    checks
}

async fn check_external_link(client: &Client, url: &str) -> LinkStatus {
    match url.parse::<reqwest::Url>() {
        Err(_) => LinkStatus::InvalidUrl,
        Ok(parsed_url) => {
            if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
                return LinkStatus::Ok;
            }

            match client.head(url).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() || status.is_redirection() {
                        LinkStatus::Ok
                    } else {
                        LinkStatus::NotFound
                    }
                }
                Err(e) => {
                    if e.is_timeout() {
                        LinkStatus::Timeout
                    } else if e.is_connect() {
                        LinkStatus::NotFound
                    } else {
                        LinkStatus::OtherError(e.to_string())
                    }
                }
            }
        }
    }
}

fn check_relative_link(base_path: &Path, link: &str) -> LinkStatus {
    if link.starts_with('#') {
        return LinkStatus::Ok;
    }

    let resolved_path = base_path.join(link);
    if resolved_path.exists() {
        LinkStatus::Ok
    } else {
        LinkStatus::NotFound
    }
}
