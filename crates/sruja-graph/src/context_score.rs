//! Context Score: quantifies how well-equipped an AI agent is to work on a codebase.
//!
//! The Context Score (0–100) measures five dimensions of context quality:
//! - Architecture Coverage: % of scanned modules mapped in architecture truth
//! - Decision Completeness: ADRs/decisions linked to architecture elements
//! - Evidence Freshness: how recently the context was refreshed
//! - Relationship Density: how well-connected the architecture graph is
//! - External Context: how much non-code context (docs, ADRs, contracts) is available
//!
//! This is the "Lighthouse score for context engineering."

use crate::{DecisionStatus, KnowledgeGraph};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// Weight configuration for context score dimensions.
const WEIGHT_ARCHITECTURE_COVERAGE: f64 = 0.25;
const WEIGHT_DECISION_COMPLETENESS: f64 = 0.20;
const WEIGHT_EVIDENCE_FRESHNESS: f64 = 0.15;
const WEIGHT_RELATIONSHIP_DENSITY: f64 = 0.20;
const WEIGHT_EXTERNAL_CONTEXT: f64 = 0.20;

/// The computed context score and its breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextScore {
    /// Overall score 0–100.
    pub score: u8,
    /// Architecture coverage dimension (0.0–1.0).
    pub architecture_coverage: DimensionScore,
    /// Decision completeness dimension (0.0–1.0).
    pub decision_completeness: DimensionScore,
    /// Evidence freshness dimension (0.0–1.0).
    pub evidence_freshness: DimensionScore,
    /// Relationship density dimension (0.0–1.0).
    pub relationship_density: DimensionScore,
    /// External context availability dimension (0.0–1.0).
    pub external_context: DimensionScore,
    /// Actionable quick wins to improve the score.
    pub quick_wins: Vec<QuickWin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    pub name: String,
    pub value: f64,
    pub max: f64,
    pub detail: String,
}

impl DimensionScore {
    pub fn pct(&self) -> f64 {
        if self.max <= 0.0 {
            1.0
        } else {
            (self.value / self.max).min(1.0)
        }
    }

    pub fn pct_u8(&self) -> u8 {
        (self.pct() * 100.0).round().min(100.0) as u8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickWin {
    pub action: String,
    pub impact_points: u8,
}

/// External context summary found in `.sruja/context/` directory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExternalContextSummary {
    /// Number of external context files found.
    pub file_count: usize,
    /// Categories of external context found.
    pub categories: Vec<String>,
    /// Files that reference architecture element IDs.
    pub linked_elements: usize,
    /// Total word count across all external context.
    pub total_words: usize,
}

/// Scan the `.sruja/context/` directory for external context files.
///
/// External context files can be:
/// - Markdown files (`.md`) — ADRs, design docs, runbooks, meeting notes
/// - YAML files (`.yaml`, `.yml`) — API contracts, config references
/// - JSON files (`.json`) — structured context (e.g., exported from Jira/Linear)
/// - Text files (`.txt`) — plain notes, decision logs
///
/// Files can optionally include a YAML front-matter header to link to elements:
/// ```text
/// ---
/// elements: [Auth.Handler, Database.Users]
/// category: adr
/// ---
/// # ADR-007: JWT tokens over session cookies
/// ...
/// ```
pub fn scan_external_context(repo_path: &Path) -> ExternalContextSummary {
    let context_dir = repo_path.join(".sruja").join("context");
    if !context_dir.exists() || !context_dir.is_dir() {
        return ExternalContextSummary::default();
    }

    let mut summary = ExternalContextSummary::default();
    let mut categories: HashSet<String> = HashSet::new();
    let mut linked_count = 0usize;
    let mut total_words = 0usize;

    if let Ok(entries) = std::fs::read_dir(&context_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            match ext.as_str() {
                "md" | "yaml" | "yml" | "json" | "txt" => {
                    summary.file_count += 1;

                    // Detect category from filename or front-matter
                    let name = path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    let category = detect_context_category(&name, &ext);
                    categories.insert(category);

                    // Read content for word count and element linking
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        total_words += content.split_whitespace().count();

                        // Check for front-matter element links
                        if has_element_links(&content) {
                            linked_count += 1;
                        }
                    }
                }
                _ => {} // Skip unsupported formats
            }
        }
    }

    summary.categories = categories.into_iter().collect();
    summary.categories.sort();
    summary.linked_elements = linked_count;
    summary.total_words = total_words;

    summary
}

/// Detect category from filename conventions.
pub fn detect_context_category(name: &str, ext: &str) -> String {
    if name.starts_with("adr") || name.contains("decision") {
        "adr".to_string()
    } else if name.contains("runbook") || name.contains("playbook") || name.contains("sop") {
        "runbook".to_string()
    } else if name.contains("api") || name.contains("contract") || name.contains("openapi") {
        "api-contract".to_string()
    } else if name.contains("onboard") || name.contains("getting-started") {
        "onboarding".to_string()
    } else if name.contains("design") || name.contains("rfc") || name.contains("proposal") {
        "design-doc".to_string()
    } else if ext == "yaml" || ext == "yml" {
        "config".to_string()
    } else {
        "note".to_string()
    }
}

/// Check if content has YAML front-matter with `elements:` field.
fn has_element_links(content: &str) -> bool {
    if !content.starts_with("---") {
        return false;
    }
    // Find the closing ---
    if let Some(end) = content[3..].find("---") {
        let front_matter = &content[3..3 + end];
        return front_matter.contains("elements:");
    }
    false
}

/// Compute the full context score.
///
/// Arguments:
/// - `graph`: The knowledge graph (from .sruja architecture + scan merge).
/// - `scanned_module_count`: Total modules found by code scanner.
/// - `repo_path`: Path to repo root (for external context scan).
/// - `context_age_hours`: Hours since last context refresh (0 = just refreshed).
pub fn compute_context_score(
    graph: &KnowledgeGraph,
    scanned_module_count: usize,
    repo_path: &Path,
    context_age_hours: u64,
) -> ContextScore {
    let mut quick_wins: Vec<QuickWin> = Vec::new();

    // --- 1. Architecture Coverage ---
    let arch_node_count = graph.nodes.len();
    let coverage_value = arch_node_count as f64;
    let coverage_max = scanned_module_count.max(arch_node_count).max(1) as f64;
    let arch_coverage = DimensionScore {
        name: "Architecture Coverage".to_string(),
        value: coverage_value,
        max: coverage_max,
        detail: format!(
            "{} of {} modules mapped in architecture",
            arch_node_count, scanned_module_count
        ),
    };

    let unmapped = scanned_module_count.saturating_sub(arch_node_count);
    if unmapped > 0 {
        let impact = (unmapped as u8).min(20);
        quick_wins.push(QuickWin {
            action: format!(
                "Map {} unmapped module{} in your .sruja file",
                unmapped,
                if unmapped == 1 { "" } else { "s" }
            ),
            impact_points: impact,
        });
    }

    // --- 2. Decision Completeness ---
    let total_decisions = graph.decisions.len();
    let accepted_decisions = graph
        .decisions
        .values()
        .filter(|d| d.status == DecisionStatus::Accepted)
        .count();
    let nodes_with_decisions: HashSet<&str> = graph
        .decisions
        .values()
        .flat_map(|d| d.affects.iter().map(|a| a.as_str()))
        .collect();
    // A "good" ratio: at least 1 decision per 5 architectural nodes
    let target_decisions = (arch_node_count / 5).max(1);
    let decision_value = total_decisions as f64;
    let decision_max = target_decisions as f64;
    let decision_completeness = DimensionScore {
        name: "Decision Completeness".to_string(),
        value: decision_value.min(decision_max),
        max: decision_max,
        detail: format!(
            "{} decisions ({} accepted), {} nodes have linked decisions",
            total_decisions,
            accepted_decisions,
            nodes_with_decisions.len()
        ),
    };

    if total_decisions == 0 {
        quick_wins.push(QuickWin {
            action: "Add ADR files to .sruja/context/ (e.g., adr-001-database-choice.md)"
                .to_string(),
            impact_points: 15,
        });
    } else if accepted_decisions < total_decisions {
        let pending = total_decisions - accepted_decisions;
        quick_wins.push(QuickWin {
            action: format!(
                "Accept {} pending decision{}",
                pending,
                if pending == 1 { "" } else { "s" }
            ),
            impact_points: 5,
        });
    }

    // --- 3. Evidence Freshness ---
    let freshness_value = if context_age_hours == 0 {
        1.0
    } else if context_age_hours <= 24 {
        0.9
    } else if context_age_hours <= 72 {
        0.6
    } else if context_age_hours <= 168 {
        0.3
    } else {
        0.1
    };
    let evidence_freshness = DimensionScore {
        name: "Evidence Freshness".to_string(),
        value: freshness_value,
        max: 1.0,
        detail: if context_age_hours == 0 {
            "Context just refreshed".to_string()
        } else {
            format!("Last refreshed {}h ago", context_age_hours)
        },
    };

    if context_age_hours > 24 {
        quick_wins.push(QuickWin {
            action: "Run 'sruja sync -r .' to refresh evidence".to_string(),
            impact_points: 10,
        });
    }

    // --- 4. Relationship Density ---
    let edge_count = graph.edges.len();
    // A healthy graph has roughly 1.5–2x edges per node
    let target_edges = ((arch_node_count as f64) * 1.5).ceil() as usize;
    let density_value = edge_count as f64;
    let density_max = target_edges.max(1) as f64;
    let relationship_density = DimensionScore {
        name: "Relationship Density".to_string(),
        value: density_value.min(density_max),
        max: density_max,
        detail: format!(
            "{} relationships across {} nodes (target: {})",
            edge_count, arch_node_count, target_edges
        ),
    };

    if edge_count < target_edges && arch_node_count > 1 {
        let missing = target_edges.saturating_sub(edge_count);
        quick_wins.push(QuickWin {
            action: format!(
                "Add ~{} more relationship{} between components",
                missing,
                if missing == 1 { "" } else { "s" }
            ),
            impact_points: (missing as u8).min(10),
        });
    }

    // --- 5. External Context ---
    let ext_ctx = scan_external_context(repo_path);
    // Scoring: each category adds value, linked elements add bonus
    let ext_value = (ext_ctx.file_count as f64 * 0.3)
        + (ext_ctx.categories.len() as f64 * 0.2)
        + (ext_ctx.linked_elements as f64 * 0.5);
    // Target: at least 3 files across 2+ categories with some linked elements
    let ext_max = 3.0;
    let external_context = DimensionScore {
        name: "External Context".to_string(),
        value: ext_value.min(ext_max),
        max: ext_max,
        detail: if ext_ctx.file_count == 0 {
            "No external context found. Add docs to .sruja/context/".to_string()
        } else {
            format!(
                "{} files across {} categories ({} linked to elements, ~{} words)",
                ext_ctx.file_count,
                ext_ctx.categories.len(),
                ext_ctx.linked_elements,
                ext_ctx.total_words
            )
        },
    };

    if ext_ctx.file_count == 0 {
        quick_wins.push(QuickWin {
            action: "Create .sruja/context/ and add ADRs, design docs, or API contracts"
                .to_string(),
            impact_points: 15,
        });
    } else if ext_ctx.linked_elements == 0 {
        quick_wins.push(QuickWin {
            action: "Add YAML front-matter with 'elements:' to link docs to architecture nodes"
                .to_string(),
            impact_points: 8,
        });
    }

    // --- Compute weighted score ---
    let raw_score = (arch_coverage.pct() * WEIGHT_ARCHITECTURE_COVERAGE
        + decision_completeness.pct() * WEIGHT_DECISION_COMPLETENESS
        + evidence_freshness.pct() * WEIGHT_EVIDENCE_FRESHNESS
        + relationship_density.pct() * WEIGHT_RELATIONSHIP_DENSITY
        + external_context.pct() * WEIGHT_EXTERNAL_CONTEXT)
        * 100.0;

    let score = raw_score.round().clamp(0.0, 100.0) as u8;

    // Sort quick wins by impact
    quick_wins.sort_by_key(|b| std::cmp::Reverse(b.impact_points));
    quick_wins.truncate(5);

    ContextScore {
        score,
        architecture_coverage: arch_coverage,
        decision_completeness,
        evidence_freshness,
        relationship_density,
        external_context,
        quick_wins,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArchitectureNode, NodeKind};

    fn test_graph_with_nodes(n: usize) -> KnowledgeGraph {
        let mut g = KnowledgeGraph::new();
        for i in 0..n {
            g.add_node(ArchitectureNode {
                id: format!("node_{}", i),
                kind: NodeKind::Service,
                label: format!("Service {}", i),
                ..ArchitectureNode::default()
            })
            .unwrap();
        }
        g
    }

    #[test]
    fn empty_graph_scores_low() {
        let g = KnowledgeGraph::new();
        let tmp = tempfile::tempdir().unwrap();
        let score = compute_context_score(&g, 10, tmp.path(), 0);
        assert!(
            score.score < 50,
            "Empty graph should score low: {}",
            score.score
        );
    }

    #[test]
    fn full_graph_scores_high() {
        let g = test_graph_with_nodes(10);
        let tmp = tempfile::tempdir().unwrap();
        // Create context dir with a file
        let ctx_dir = tmp.path().join(".sruja").join("context");
        std::fs::create_dir_all(&ctx_dir).unwrap();
        std::fs::write(
            ctx_dir.join("adr-001.md"),
            "---\nelements: [node_0]\ncategory: adr\n---\n# Use PostgreSQL\nWe chose PostgreSQL.",
        )
        .unwrap();

        let score = compute_context_score(&g, 10, tmp.path(), 0);
        assert!(
            score.score >= 40,
            "Well-populated graph should score >= 40: {}",
            score.score
        );
    }

    #[test]
    fn stale_evidence_reduces_score() {
        let g = test_graph_with_nodes(5);
        let tmp = tempfile::tempdir().unwrap();
        let fresh = compute_context_score(&g, 5, tmp.path(), 0);
        let stale = compute_context_score(&g, 5, tmp.path(), 200);
        assert!(
            fresh.score > stale.score,
            "Fresh ({}) should beat stale ({})",
            fresh.score,
            stale.score
        );
    }

    #[test]
    fn quick_wins_are_actionable() {
        let g = KnowledgeGraph::new();
        let tmp = tempfile::tempdir().unwrap();
        let score = compute_context_score(&g, 10, tmp.path(), 48);
        assert!(!score.quick_wins.is_empty(), "Should have quick wins");
        for qw in &score.quick_wins {
            assert!(!qw.action.is_empty());
            assert!(qw.impact_points > 0);
        }
    }

    #[test]
    fn external_context_detection_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let summary = scan_external_context(tmp.path());
        assert_eq!(summary.file_count, 0);
    }

    #[test]
    fn external_context_detection_with_files() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx_dir = tmp.path().join(".sruja").join("context");
        std::fs::create_dir_all(&ctx_dir).unwrap();
        std::fs::write(ctx_dir.join("adr-001.md"), "# ADR 001\nSome decision.").unwrap();
        std::fs::write(ctx_dir.join("api-contract.yaml"), "openapi: 3.0.0").unwrap();
        std::fs::write(
            ctx_dir.join("runbook-deploy.md"),
            "# Deploy runbook\nSteps...",
        )
        .unwrap();

        let summary = scan_external_context(tmp.path());
        assert_eq!(summary.file_count, 3);
        assert!(summary.categories.contains(&"adr".to_string()));
        assert!(summary.categories.contains(&"api-contract".to_string()));
        assert!(summary.categories.contains(&"runbook".to_string()));
    }

    #[test]
    fn front_matter_element_linking() {
        assert!(has_element_links(
            "---\nelements: [Auth.Handler]\n---\n# Content"
        ));
        assert!(!has_element_links("# No front matter"));
        assert!(!has_element_links("---\ntitle: test\n---\n# No elements"));
    }
}
