//! Architecture knowledge tools: get_architecture, get_decision, get_decisions, get_policy_conflicts, query, get_component, add_decision.

use sruja_graph::KnowledgeGraph;

use crate::tools::{SrujaTool, ToolResponse};

pub(super) fn execute_architecture(
    tool: &SrujaTool,
    graph: &KnowledgeGraph,
) -> Option<ToolResponse> {
    match tool {
        SrujaTool::GetArchitecture => {
            let summary = crate::ArchitectureSummary::from(graph);
            Some(ToolResponse::success(
                "get_architecture",
                serde_json::to_value(summary).unwrap_or_default(),
            ))
        }

        SrujaTool::GetDecisions => {
            let decisions: Vec<_> = graph
                .decisions
                .values()
                .map(crate::DecisionResponse::from)
                .collect();
            Some(ToolResponse::success(
                "get_decisions",
                serde_json::to_value(decisions).unwrap_or_default(),
            ))
        }

        SrujaTool::GetDecision { id } => match graph.get_decision(id) {
            Some(d) => {
                let response = crate::DecisionResponse::from(d);
                Some(ToolResponse::success(
                    "get_decision",
                    serde_json::to_value(response).unwrap_or_default(),
                ))
            }
            None => Some(ToolResponse::error(
                "get_decision",
                format!("Decision {} not found", id),
            )),
        },

        SrujaTool::GetPolicyConflicts => {
            let violations = graph.find_policy_violations();
            let responses: Vec<_> = violations
                .into_iter()
                .map(crate::PolicyViolationResponse::from)
                .collect();
            Some(ToolResponse::success(
                "get_policy_conflicts",
                serde_json::to_value(responses).unwrap_or_default(),
            ))
        }

        SrujaTool::Query { question } => match graph.query(question) {
            Ok(result) => {
                let response = crate::QueryResponse::from(result);
                Some(ToolResponse::success(
                    "query",
                    serde_json::to_value(response).unwrap_or_default(),
                ))
            }
            Err(e) => Some(ToolResponse::error("query", e.to_string())),
        },

        SrujaTool::GetComponent { id } => match graph.get_node(id) {
            Some(node) => Some(ToolResponse::success(
                "get_component",
                serde_json::to_value(node).unwrap_or_default(),
            )),
            None => Some(ToolResponse::error(
                "get_component",
                format!("Component {} not found", id),
            )),
        },

        SrujaTool::AddDecision { .. } => Some(ToolResponse::error(
            "add_decision",
            "Use the chat API to add decisions",
        )),

        _ => None,
    }
}
