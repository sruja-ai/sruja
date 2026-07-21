use super::super::types::*;
use super::grounding::{
    assemble_sources_and_hydration, build_focus_elements, build_grounding_trace,
    estimate_risk, expand_neighbors_and_impact, load_baseline_elements, resolve_focus,
    semantic_candidates_from_scan, GroundingTraceInputs,
};
use crate::commands::CliError;
use sruja_scan::Graph;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct TaskSelectors<'a> {
    pub file: Option<&'a str>,
    pub element_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub base_ref: Option<&'a str>,
    pub head_ref: Option<&'a str>,
    pub depth: Option<usize>,
}

pub fn build_task_context(
    graph: &Graph,
    repo_root: &str,
    selectors: TaskSelectors<'_>,
    max_tokens: usize,
) -> Result<TaskContext, CliError> {
    let repo_path = Path::new(repo_root);
    let baseline = load_baseline_elements(repo_path);

    let (focus_ids, selection_reason, truth_status, confidence) =
        resolve_focus(graph, repo_root, selectors, &baseline)?;

    let focus_elements = build_focus_elements(graph, &focus_ids, &baseline, &selection_reason);
    let (neighbors, impacted) =
        expand_neighbors_and_impact(graph, &focus_ids, selectors.depth.unwrap_or(1));
    let (source_bindings, hydrated_files) = assemble_sources_and_hydration(
        graph, repo_root, &focus_ids, &neighbors, &baseline, max_tokens,
    )?;

    let semantic_candidates = if selectors.query.is_some() {
        semantic_candidates_from_scan(graph, selectors.query.unwrap_or_default(), 10)
    } else {
        Vec::new()
    };

    let risk = estimate_risk(graph, &focus_ids, &neighbors, &baseline);

    let grounding_trace = build_grounding_trace(GroundingTraceInputs {
        selectors,
        max_tokens,
        selection_reason: &selection_reason,
        focus_ids: &focus_ids,
        semantic_candidates: &semantic_candidates,
        neighbors: &neighbors,
        impacted: &impacted,
        source_bindings: &source_bindings,
        hydrated_files: &hydrated_files,
    });

    let estimated_used_tokens: usize = hydrated_files
        .iter()
        .map(|f| TokenBudget::estimate_tokens(&f.content))
        .sum();
    let estimated_remaining_tokens = max_tokens.saturating_sub(estimated_used_tokens);
    let truncated_files = hydrated_files.iter().filter(|f| f.truncated).count();

    Ok(TaskContext {
        run_id: None,
        schema_version: "task_context/v1".to_string(),
        context_budget: Some(TaskContextBudget {
            max_tokens,
            estimated_used_tokens,
            estimated_remaining_tokens,
            hydrated_files_included: hydrated_files.len(),
            hydrated_files_truncated: truncated_files,
        }),
        selection_reason,
        grounding_trace,
        focus_elements,
        impacted_systems: impacted.systems,
        impacted_containers: impacted.containers,
        impacted_components: impacted.components,
        neighbors,
        source_bindings,
        hydrated_files,
        risk,
        truth_status,
        confidence,
        semantic_candidates,
    })
}

const CACHE_FRIENDLY_WHEN_SUGGESTING: &[&str] = &[
    "Respect layer boundaries — check imports before suggesting",
    "Use existing patterns in the codebase",
    "If adding a new dependency, verify it does not violate boundaries",
    "Run `sruja drift -r .` after changes to verify architecture health",
    "Prefer MCP progressive disclosure: list_architecture_index → get_topology → get_elements before pasting full architecture",
];

pub fn build_cache_friendly_task_export(
    repo_root: &str,
    arch: &ArchitectureContext,
    volatile: TaskContext,
) -> CacheFriendlyTaskContextExport {
    CacheFriendlyTaskContextExport {
        schema_version: "task_context_cache_friendly/v1".to_string(),
        invariant: ContextInvariantBlock {
            schema_version: "context_invariant/v1".to_string(),
            repo: repo_root.to_string(),
            summary: arch.summary.clone(),
            layers: arch.layers.clone(),
            boundaries: arch.boundaries.clone(),
            forbidden_patterns: arch.forbidden_patterns.clone(),
            active_decisions: arch.active_decisions.clone(),
            retrieval_ladder: vec![
                "sruja_list_architecture_index".to_string(),
                "sruja_get_topology".to_string(),
                "sruja_get_elements".to_string(),
            ],
            when_suggesting_code: CACHE_FRIENDLY_WHEN_SUGGESTING
                .iter()
                .map(|s| s.to_string())
                .collect(),
        },
        tools: ContextToolsBlock {
            schema_version: "context_tools/v1".to_string(),
            mcp_retrieval_ladder: vec![
                ContextToolHint {
                    name: "sruja_list_architecture_index".to_string(),
                    layer: "index".to_string(),
                    description: "Compact element list with validation signals (cycles, policies)."
                        .to_string(),
                },
                ContextToolHint {
                    name: "sruja_get_topology".to_string(),
                    layer: "topology".to_string(),
                    description: "Upstream/downstream neighbors for one element id.".to_string(),
                },
                ContextToolHint {
                    name: "sruja_get_elements".to_string(),
                    layer: "detail".to_string(),
                    description: "Batch element detail for ids from index/topology.".to_string(),
                },
                ContextToolHint {
                    name: "sruja_get_task_context".to_string(),
                    layer: "task".to_string(),
                    description: "Task-scoped hydration after ladder orientation.".to_string(),
                },
            ],
        },
        volatile,
    }
}

#[cfg(test)]
mod cache_friendly_tests {
    use super::*;
    use crate::commands::context::types::{BoundaryRule, ContextSummary, LayerInfo};

    #[test]
    fn cache_friendly_export_orders_invariant_before_volatile() {
        let arch = ArchitectureContext {
            repo: ".".to_string(),
            summary: ContextSummary {
                total_crates: None,
                total_modules: 1,
                total_services: 0,
                total_databases: 0,
                total_external_apis: 0,
            },
            layers: vec![LayerInfo {
                name: "models".to_string(),
                modules: 1,
                can_depend_on: vec![],
            }],
            boundaries: vec![BoundaryRule {
                from: "ui".to_string(),
                to: "data".to_string(),
                allowed: false,
                reason: "test".to_string(),
            }],
            forbidden_patterns: vec!["no direct db".to_string()],
            active_decisions: vec![],
            focus: None,
            system_context: None,
            max_tokens: 1000,
        };
        let volatile = TaskContext {
            run_id: Some("run-1".to_string()),
            schema_version: "task_context/v1".to_string(),
            context_budget: None,
            selection_reason: SelectionReason {
                primary: "test".to_string(),
                resolution_path: vec![],
                details: None,
            },
            grounding_trace: vec![],
            focus_elements: vec![],
            impacted_systems: vec![],
            impacted_containers: vec![],
            impacted_components: vec![],
            neighbors: vec![],
            source_bindings: vec![],
            hydrated_files: vec![],
            risk: TaskRisk::Low,
            truth_status: TaskTruthStatus::Unknown,
            confidence: TaskConfidence::Medium,
            semantic_candidates: vec![],
        };

        let export = build_cache_friendly_task_export(".", &arch, volatile);
        assert_eq!(
            export.schema_version,
            "task_context_cache_friendly/v1".to_string()
        );
        assert_eq!(
            export.invariant.schema_version,
            "context_invariant/v1".to_string()
        );
        assert!(!export.tools.mcp_retrieval_ladder.is_empty());
        assert_eq!(export.volatile.run_id.as_deref(), Some("run-1"));
    }
}
