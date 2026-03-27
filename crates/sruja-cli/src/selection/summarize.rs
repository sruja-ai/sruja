//! Large component summarization (truncation/placeholder).
//!
//! For components too large to include fully, use a size placeholder.

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

    let _ = llm_enabled; // unused; LLM removed
    ComponentSummary::Summarized {
        summary: format!(
            "[Large file: {} chars, {} lines.]",
            content.len(),
            content.lines().count()
        ),
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
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
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
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
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

    #[tokio::test]
    async fn test_no_path_returns_skipped() {
        let temp = TempDir::new().unwrap();
        let node = Node {
            id: "test".into(),
            label: "no-path".into(),
            path: None,
            kind: NodeKind::Module,
            technology: None,
            metadata: Default::default(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
        };
        let result = summarize_large_component(&node, temp.path(), false).await;
        match result {
            ComponentSummary::Skipped { reason } => assert!(reason.contains("No file path")),
            _ => panic!("Expected Skipped"),
        }
    }

    #[tokio::test]
    async fn test_file_not_found_returns_skipped() {
        let temp = TempDir::new().unwrap();
        let node = Node {
            id: "test".into(),
            label: "missing".into(),
            path: Some("nonexistent.rs".into()),
            kind: NodeKind::Module,
            technology: None,
            metadata: Default::default(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
        };
        let result = summarize_large_component(&node, temp.path(), false).await;
        match result {
            ComponentSummary::Skipped { reason } => assert!(reason.contains("File not found")),
            _ => panic!("Expected Skipped"),
        }
    }

    #[tokio::test]
    async fn test_boundary_at_max_full_content_size() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("exact.rs");
        let content = "x".repeat(MAX_FULL_CONTENT_SIZE);
        fs::write(&file_path, &content).unwrap();
        let node = Node {
            id: "test".into(),
            label: "exact".into(),
            path: Some("exact.rs".into()),
            kind: NodeKind::Module,
            technology: None,
            metadata: Default::default(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
        };
        let result = summarize_large_component(&node, temp.path(), false).await;
        match result {
            ComponentSummary::Full { content: c } => assert_eq!(c.len(), MAX_FULL_CONTENT_SIZE),
            _ => panic!("Expected Full at boundary"),
        }
    }

    #[test]
    fn test_compute_summary_stats_empty() {
        let stats = compute_summary_stats(&[]);
        assert_eq!(stats.full_count, 0);
        assert_eq!(stats.summarized_count, 0);
        assert_eq!(stats.skipped_count, 0);
        assert_eq!(stats.total_chars, 0);
    }

    #[test]
    fn test_compute_summary_stats_mixed() {
        let summaries = vec![
            ComponentSummary::Full {
                content: "abc".to_string(),
            },
            ComponentSummary::Summarized {
                summary: "x".to_string(),
            },
            ComponentSummary::Skipped {
                reason: "nope".to_string(),
            },
        ];
        let stats = compute_summary_stats(&summaries);
        assert_eq!(stats.full_count, 1);
        assert_eq!(stats.summarized_count, 1);
        assert_eq!(stats.skipped_count, 1);
        assert_eq!(stats.total_chars, 4);
    }
}
