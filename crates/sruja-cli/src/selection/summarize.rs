//! Large component summarization using LLM.
//!
//! For components too large to include fully, generate LLM summaries.

use sruja_scan::Node;
use std::fs;
use std::path::Path;

pub const MAX_FULL_CONTENT_SIZE: usize = 5_000;
pub const SUMMARY_MAX_BULLETS: usize = 5;

#[derive(Debug, Clone)]
pub enum ComponentSummary {
    Full { content: String },
    Summarized { summary: String },
    Skipped { reason: String },
}

pub async fn summarize_large_component(
    node: &Node,
    repo_path: &Path,
    llm_enabled: bool,
) -> ComponentSummary {
    let file_path = match &node.path {
        Some(p) => repo_path.join(p),
        None => {
            return ComponentSummary::Skipped {
                reason: "No file path".to_string(),
            }
        }
    };

    if !file_path.exists() {
        return ComponentSummary::Skipped {
            reason: format!("File not found: {:?}", file_path),
        };
    }

    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            return ComponentSummary::Skipped {
                reason: format!("Read error: {}", e),
            }
        }
    };

    if content.len() <= MAX_FULL_CONTENT_SIZE {
        return ComponentSummary::Full { content };
    }

    if !llm_enabled {
        return ComponentSummary::Summarized {
            summary: format!(
                "[Large file: {} chars, {} lines. Enable LLM for summary.]",
                content.len(),
                content.lines().count()
            ),
        };
    }

    let summary = generate_llm_summary(&file_path, &content).await;
    ComponentSummary::Summarized { summary }
}

async fn generate_llm_summary(file_path: &Path, content: &str) -> String {
    let file_name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let preview = if content.len() > 3000 {
        let start = &content[..1500];
        let end = &content[content.len() - 1500..];
        format!("{}\n\n... [content truncated] ...\n\n{}", start, end)
    } else {
        content.to_string()
    };

    let prompt = format!(
        r#"Summarize this source file for architecture documentation.

File: {}
Size: {} characters

Content:
```
{}
```

Provide a concise summary with {} bullet points:
- Main purpose/responsibility
- Key exports (functions, types, classes)
- Important dependencies
- Architectural role

Return ONLY the bullet points, no preamble."#,
        file_name,
        content.len(),
        preview,
        SUMMARY_MAX_BULLETS
    );

    if let Ok(response) = crate::commands::llm::call_llm(
        "You are a code summarizer. Create concise architecture-focused summaries.",
        &prompt,
    )
    .await
    {
        response.trim().to_string()
    } else {
        format!(
            "[{} - {} chars, {} lines. LLM summary failed.]",
            file_name,
            content.len(),
            content.lines().count()
        )
    }
}

pub fn compute_summary_stats(summaries: &[ComponentSummary]) -> SummaryStats {
    let mut stats = SummaryStats::default();

    for summary in summaries {
        match summary {
            ComponentSummary::Full { content } => {
                stats.full_count += 1;
                stats.total_chars += content.len();
            }
            ComponentSummary::Summarized { summary } => {
                stats.summarized_count += 1;
                stats.total_chars += summary.len();
            }
            ComponentSummary::Skipped { .. } => {
                stats.skipped_count += 1;
            }
        }
    }

    stats
}

#[derive(Debug, Clone, Default)]
pub struct SummaryStats {
    pub full_count: usize,
    pub summarized_count: usize,
    pub skipped_count: usize,
    pub total_chars: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::NodeKind;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_small_file_full_content() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("small.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let node = Node {
            id: "test".into(),
            label: "small".into(),
            path: Some("small.rs".into()),
            kind: NodeKind::Module,
            technology: None,
            metadata: Default::default(),
        };

        let result = summarize_large_component(&node, temp.path(), false).await;

        match result {
            ComponentSummary::Full { content } => assert_eq!(content, "fn main() {}"),
            _ => panic!("Expected Full content"),
        }
    }

    #[tokio::test]
    async fn test_large_file_without_llm() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("large.rs");
        let large_content = "x".repeat(10_000);
        fs::write(&file_path, &large_content).unwrap();

        let node = Node {
            id: "test".into(),
            label: "large".into(),
            path: Some("large.rs".into()),
            kind: NodeKind::Module,
            technology: None,
            metadata: Default::default(),
        };

        let result = summarize_large_component(&node, temp.path(), false).await;

        match result {
            ComponentSummary::Summarized { summary } => {
                assert!(summary.contains("Large file"));
                assert!(summary.contains("chars"));
            }
            _ => panic!("Expected Summarized content"),
        }
    }
}
