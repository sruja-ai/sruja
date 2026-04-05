//! ADR (Architecture Decision Record) Parser
//!
//! Parses ADR files in various formats (Markdown, MADR, Nygard) and extracts
//! structural implications for architecture comparison.

use crate::IntentError;
use chrono::{DateTime, Utc};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

pub struct AdrParser {
    status_patterns: Vec<(Regex, AdrStatus)>,
}

#[derive(Debug, Clone)]
pub struct ParsedAdr {
    pub path: PathBuf,
    pub number: Option<u32>,
    pub title: String,
    pub status: AdrStatus,
    pub date: Option<DateTime<Utc>>,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    pub implications: Vec<StructuralImplication>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded { by: Option<u32> },
    Rejected,
    Draft,
}

#[derive(Debug, Clone)]
pub struct StructuralImplication {
    pub component: Option<String>,
    pub boundary_change: Option<BoundaryChange>,
    pub new_policy: Option<String>,
    pub constraint: Option<String>,
    pub raw_text: String,
}

#[derive(Debug, Clone)]
pub struct BoundaryChange {
    pub component: String,
    pub change_type: BoundaryChangeType,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryChangeType {
    Added,
    Removed,
    Expanded,
    Contracted,
    Split,
    Merged,
}

impl Default for AdrParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AdrParser {
    pub fn new() -> Self {
        let status_patterns = vec![
            (
                Regex::new(r"(?i)^superseded(?:\s+by\s+(\d+))?").unwrap(),
                AdrStatus::Superseded { by: None },
            ),
            (Regex::new(r"(?i)^accepted").unwrap(), AdrStatus::Accepted),
            (Regex::new(r"(?i)^rejected").unwrap(), AdrStatus::Rejected),
            (
                Regex::new(r"(?i)^deprecated").unwrap(),
                AdrStatus::Deprecated,
            ),
            (Regex::new(r"(?i)^proposed").unwrap(), AdrStatus::Proposed),
            (Regex::new(r"(?i)^draft").unwrap(), AdrStatus::Draft),
        ];

        Self { status_patterns }
    }

    pub fn parse_dir(&self, dir: &Path) -> Result<Vec<ParsedAdr>, IntentError> {
        let mut adrs = Vec::new();

        if !dir.exists() {
            return Ok(adrs);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path
                .extension()
                .is_some_and(|ext| ext == "md" || ext == "markdown")
            {
                if let Ok(adr) = self.parse_file(&path) {
                    adrs.push(adr);
                }
            }
        }

        adrs.sort_by(|a, b| a.number.unwrap_or(9999).cmp(&b.number.unwrap_or(9999)));

        Ok(adrs)
    }

    pub fn parse_file(&self, path: &Path) -> Result<ParsedAdr, IntentError> {
        let content = fs::read_to_string(path)?;

        let number = self.extract_number(path, &content);
        let title = self.extract_title(&content);
        let status = self.extract_status(&content);
        let date = self.extract_date(&content);
        let (context, decision, consequences) = self.extract_sections(&content);
        let tags = self.extract_tags(&content);
        let implications = self.extract_implications(&content);

        Ok(ParsedAdr {
            path: path.to_path_buf(),
            number,
            title,
            status,
            date,
            context,
            decision,
            consequences,
            implications,
            tags,
        })
    }

    fn extract_number(&self, path: &Path, content: &str) -> Option<u32> {
        let filename = path.file_stem()?.to_string_lossy();

        let re = Regex::new(r"^(\d{4})").ok()?;
        if let Some(caps) = re.captures(&filename) {
            return caps[1].parse().ok();
        }

        let re = Regex::new(r"(?i)ADR[-_]?(\d+)").ok()?;
        if let Some(caps) = re.captures(content) {
            return caps[1].parse().ok();
        }

        None
    }

    fn extract_title(&self, content: &str) -> String {
        let adr_prefix_re = Regex::new(r"(?i)^ADR[-_]?\d+[-_:]\s*").ok();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                let title = trimmed.trim_start_matches('#').trim();
                let title = adr_prefix_re
                    .as_ref()
                    .and_then(|re| {
                        let replaced = re.replace(title, "");
                        if replaced.is_empty() {
                            None
                        } else {
                            Some(replaced.into_owned())
                        }
                    })
                    .unwrap_or_else(|| title.to_string());
                return title.trim().to_string();
            }
        }
        "Untitled ADR".to_string()
    }

    fn extract_status(&self, content: &str) -> AdrStatus {
        let status_re = Regex::new(r"(?i)(?:^|\n)\s*[-*]?\s*Status\s*:\s*(.+?)(?:\n|$)").unwrap();

        if let Some(caps) = status_re.captures(content) {
            let status_str = caps[1].trim();

            for (pattern, status) in &self.status_patterns {
                if let Some(caps) = pattern.captures(status_str) {
                    if let AdrStatus::Superseded { .. } = status {
                        let by = caps.get(1).and_then(|m| m.as_str().parse().ok());
                        return AdrStatus::Superseded { by };
                    }
                    return *status;
                }
            }
        }

        AdrStatus::Proposed
    }

    fn extract_date(&self, content: &str) -> Option<DateTime<Utc>> {
        let date_re =
            Regex::new(r"(?i)(?:^|\n)\s*[-*]?\s*Date\s*:\s*(\d{4}[-/]\d{2}[-/]\d{2})").unwrap();

        if let Some(caps) = date_re.captures(content) {
            let date_str = caps[1].replace('/', "-");
            if let Ok(dt) = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                return Some(dt.and_hms_opt(0, 0, 0)?.and_utc());
            }
        }

        None
    }

    fn extract_sections(&self, content: &str) -> (String, String, String) {
        let mut context = String::new();
        let mut decision = String::new();
        let mut consequences = String::new();

        let mut current_section: Option<&str> = None;
        let mut section_content = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("## ") || trimmed.starts_with("# ") {
                if let Some(section) = current_section {
                    match section {
                        "context" => context = section_content.clone(),
                        "decision" => decision = section_content.clone(),
                        "consequences" => consequences = section_content.clone(),
                        _ => {}
                    }
                }
                section_content.clear();

                let heading = trimmed.trim_start_matches('#').trim().to_lowercase();
                current_section = if heading.contains("context") {
                    Some("context")
                } else if heading.contains("decision") {
                    Some("decision")
                } else if heading.contains("consequence") {
                    Some("consequences")
                } else {
                    None
                };
            } else if current_section.is_some() {
                if !section_content.is_empty() {
                    section_content.push('\n');
                }
                section_content.push_str(line);
            }
        }

        if let Some(section) = current_section {
            match section {
                "context" => context = section_content,
                "decision" => decision = section_content,
                "consequences" => consequences = section_content,
                _ => {}
            }
        }

        (
            context.trim().to_string(),
            decision.trim().to_string(),
            consequences.trim().to_string(),
        )
    }

    fn extract_tags(&self, content: &str) -> Vec<String> {
        let mut tags = Vec::new();

        let tag_re = Regex::new(r"(?i)Tags\s*:\s*(.+)").unwrap();
        if let Some(caps) = tag_re.captures(content) {
            for tag in caps[1].split(',') {
                let tag = tag.trim().to_string();
                if !tag.is_empty() {
                    tags.push(tag);
                }
            }
        }

        let hash_re = Regex::new(r"#(\w+)").unwrap();
        for caps in hash_re.captures_iter(content) {
            let tag = caps[1].to_string();
            if !tags.contains(&tag) {
                tags.push(tag);
            }
        }

        tags
    }

    fn extract_implications(&self, content: &str) -> Vec<StructuralImplication> {
        let mut implications = Vec::new();

        let patterns = vec![
            (Regex::new(r"(?i)(?:introduce|add|create)\s+(?:a\s+)?(?:new\s+)?(\w+)\s+(?:component|service|module)").unwrap(), "added"),
            (Regex::new(r"(?i)(?:remove|delete|deprecate)\s+(?:the\s+)?(\w+)\s+(?:component|service|module)").unwrap(), "removed"),
            (Regex::new(r"(?i)(?:split|refactor)\s+(\w+)\s+into\s+(\w+)\s+and\s+(\w+)").unwrap(), "split"),
            (Regex::new(r"(?i)(?:merge|combine)\s+(\w+)\s+and\s+(\w+)").unwrap(), "merged"),
            (Regex::new(r"(?i)(?:boundary|interface|api)\s+(?:between|for)\s+(\w+)\s+and\s+(\w+)").unwrap(), "boundary"),
        ];

        for line in content.lines() {
            for (pattern, _) in &patterns {
                if pattern.is_match(line) {
                    implications.push(StructuralImplication {
                        component: None,
                        boundary_change: None,
                        new_policy: None,
                        constraint: None,
                        raw_text: line.trim().to_string(),
                    });
                    break;
                }
            }
        }

        let policy_re = Regex::new(r"(?i)(?:policy|rule|constraint):\s*(.+)").unwrap();
        for caps in policy_re.captures_iter(content) {
            implications.push(StructuralImplication {
                component: None,
                boundary_change: None,
                new_policy: Some(caps[1].trim().to_string()),
                constraint: None,
                raw_text: caps[0].to_string(),
            });
        }

        implications
    }
}

impl std::fmt::Display for AdrStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdrStatus::Proposed => write!(f, "Proposed"),
            AdrStatus::Accepted => write!(f, "Accepted"),
            AdrStatus::Deprecated => write!(f, "Deprecated"),
            AdrStatus::Superseded { by } => {
                if let Some(n) = by {
                    write!(f, "Superseded by ADR-{:04}", n)
                } else {
                    write!(f, "Superseded")
                }
            }
            AdrStatus::Rejected => write!(f, "Rejected"),
            AdrStatus::Draft => write!(f, "Draft"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_adr_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_parse_basic_adr() {
        let content = r#"
# ADR-0001: Use PostgreSQL for Primary Database

- Status: Accepted
- Date: 2024-01-15

## Context

We need a relational database for our primary data storage.

## Decision

We will use PostgreSQL as our primary database.

## Consequences

- Need to set up replication
- Team needs PostgreSQL expertise
"#;
        let file = create_adr_file(content);
        let parser = AdrParser::new();
        let adr = parser.parse_file(file.path()).unwrap();

        assert_eq!(adr.title, "Use PostgreSQL for Primary Database");
        assert_eq!(adr.status, AdrStatus::Accepted);
        assert!(adr.context.contains("relational database"));
        assert!(adr.decision.contains("PostgreSQL"));
    }

    #[test]
    fn test_parse_superseded() {
        let content = r#"
# ADR-0002: Use MySQL Instead

Status: Superseded by 0003

## Decision

Use MySQL.
"#;
        let file = create_adr_file(content);
        let parser = AdrParser::new();
        let adr = parser.parse_file(file.path()).unwrap();

        assert_eq!(adr.status, AdrStatus::Superseded { by: Some(3) });
    }

    #[test]
    fn test_extract_title() {
        let parser = AdrParser::new();
        assert_eq!(parser.extract_title("# ADR-0001: My Title"), "My Title");
        assert_eq!(parser.extract_title("# 0001-my-title"), "0001-my-title");
    }

    #[test]
    fn test_parse_empty_file() {
        let file = create_adr_file("");
        let parser = AdrParser::new();
        let adr = parser.parse_file(file.path()).unwrap();
        assert_eq!(adr.title, "Untitled ADR");
        assert_eq!(adr.status, AdrStatus::Proposed);
    }

    #[test]
    fn test_extract_implications() {
        let parser = AdrParser::new();
        
        let content = r#"
            We will introduce a new Payment service to handle transactions.
            We should remove the old Legacy module.
            Policy: Payments must be secure
        "#;
        
        let implications = parser.extract_implications(content);
        
        // Let's just find the policy one explicitly to make it robust against other matches
        let policy_impl = implications.iter().find(|i| i.new_policy.is_some()).unwrap();
        assert_eq!(policy_impl.new_policy.as_deref(), Some("Payments must be secure"));
    }

    #[test]
    fn test_extract_tags() {
        let parser = AdrParser::new();
        let content = r#"
            Tags: architecture, database,  security 
            #rust #fast
        "#;
        let tags = parser.extract_tags(content);
        assert_eq!(tags.len(), 5);
        assert!(tags.contains(&"architecture".to_string()));
        assert!(tags.contains(&"database".to_string()));
        assert!(tags.contains(&"security".to_string()));
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"fast".to_string()));
    }

    #[test]
    fn test_adr_status_display() {
        assert_eq!(format!("{}", AdrStatus::Proposed), "Proposed");
        assert_eq!(format!("{}", AdrStatus::Accepted), "Accepted");
        assert_eq!(format!("{}", AdrStatus::Deprecated), "Deprecated");
        assert_eq!(format!("{}", AdrStatus::Rejected), "Rejected");
        assert_eq!(format!("{}", AdrStatus::Draft), "Draft");
        assert_eq!(format!("{}", AdrStatus::Superseded { by: Some(5) }), "Superseded by ADR-0005");
        assert_eq!(format!("{}", AdrStatus::Superseded { by: None }), "Superseded");
    }
}
