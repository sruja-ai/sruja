//! MCP Tools for Sruja

use serde::{Deserialize, Serialize};
use sruja_graph::KnowledgeGraph;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SrujaTool {
    GetArchitecture,
    GetDecision {
        id: String,
    },
    GetDecisions,
    GetPolicyConflicts,
    Query {
        question: String,
    },
    GetComponent {
        id: String,
    },
    AddDecision {
        title: String,
        decision: String,
        context: String,
    },
    ScanRepo {
        path: String,
    },
    DetectDrift {
        repo_path: String,
    },
    Quickstart {
        repo_path: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub tool: String,
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

impl ToolResponse {
    pub fn success(tool: impl Into<String>, result: serde_json::Value) -> Self {
        Self {
            tool: tool.into(),
            success: true,
            result,
            error: None,
        }
    }

    pub fn error(tool: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            success: false,
            result: serde_json::Value::Null,
            error: Some(error.into()),
        }
    }
}

impl SrujaTool {
    pub fn name(&self) -> &str {
        match self {
            SrujaTool::GetArchitecture => "get_architecture",
            SrujaTool::GetDecision { .. } => "get_decision",
            SrujaTool::GetDecisions => "get_decisions",
            SrujaTool::GetPolicyConflicts => "get_policy_conflicts",
            SrujaTool::Query { .. } => "query",
            SrujaTool::GetComponent { .. } => "get_component",
            SrujaTool::AddDecision { .. } => "add_decision",
            SrujaTool::ScanRepo { .. } => "scan_repo",
            SrujaTool::DetectDrift { .. } => "detect_drift",
            SrujaTool::Quickstart { .. } => "quickstart",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            SrujaTool::GetArchitecture => {
                "Get the architecture summary with all components and their relationships"
            }
            SrujaTool::GetDecision { .. } => "Get a specific architecture decision by ID",
            SrujaTool::GetDecisions => "List all architecture decisions",
            SrujaTool::GetPolicyConflicts => "Find policy violations in the current architecture",
            SrujaTool::Query { .. } => {
                "Query the architecture knowledge graph with natural language"
            }
            SrujaTool::GetComponent { .. } => "Get details about a specific component",
            SrujaTool::AddDecision { .. } => "Add a new architecture decision",
            SrujaTool::ScanRepo { .. } => {
                "Scan a repository and return the inferred architecture graph"
            }
            SrujaTool::DetectDrift { .. } => {
                "Detect architectural drift and health issues in a repository"
            }
            SrujaTool::Quickstart { .. } => {
                "Get immediate architecture insights (zero-key, deterministic)"
            }
        }
    }

    pub fn execute(&self, graph: &KnowledgeGraph) -> ToolResponse {
        match self {
            SrujaTool::GetArchitecture => {
                let summary = crate::ArchitectureSummary::from(graph);
                ToolResponse::success(
                    "get_architecture",
                    serde_json::to_value(summary).unwrap_or_default(),
                )
            }

            SrujaTool::GetDecisions => {
                let decisions: Vec<_> = graph
                    .decisions
                    .values()
                    .map(|d| crate::DecisionResponse::from(d))
                    .collect();
                ToolResponse::success(
                    "get_decisions",
                    serde_json::to_value(decisions).unwrap_or_default(),
                )
            }

            SrujaTool::GetDecision { id } => match graph.get_decision(id) {
                Some(d) => {
                    let response = crate::DecisionResponse::from(d);
                    ToolResponse::success(
                        "get_decision",
                        serde_json::to_value(response).unwrap_or_default(),
                    )
                }
                None => ToolResponse::error("get_decision", format!("Decision {} not found", id)),
            },

            SrujaTool::GetPolicyConflicts => {
                let violations = graph.find_policy_violations();
                let responses: Vec<_> = violations
                    .into_iter()
                    .map(crate::PolicyViolationResponse::from)
                    .collect();
                ToolResponse::success(
                    "get_policy_conflicts",
                    serde_json::to_value(responses).unwrap_or_default(),
                )
            }

            SrujaTool::Query { question } => match graph.query(question) {
                Ok(result) => {
                    let response = crate::QueryResponse::from(result);
                    ToolResponse::success(
                        "query",
                        serde_json::to_value(response).unwrap_or_default(),
                    )
                }
                Err(e) => ToolResponse::error("query", e.to_string()),
            },

            SrujaTool::GetComponent { id } => match graph.get_node(id) {
                Some(node) => ToolResponse::success(
                    "get_component",
                    serde_json::to_value(node).unwrap_or_default(),
                ),
                None => ToolResponse::error("get_component", format!("Component {} not found", id)),
            },

            SrujaTool::AddDecision { .. } => {
                ToolResponse::error("add_decision", "Use the chat API to add decisions")
            }

            SrujaTool::ScanRepo { path } => {
                use std::path::Path;
                match sruja_scan::scan_repo(Path::new(path)) {
                    Ok(scan_graph) => ToolResponse::success(
                        "scan_repo",
                        serde_json::to_value(scan_graph).unwrap_or_default(),
                    ),
                    Err(e) => ToolResponse::error("scan_repo", e.to_string()),
                }
            }

            SrujaTool::DetectDrift { repo_path } => {
                use std::path::Path;
                match sruja_scan::scan_repo(Path::new(repo_path)) {
                    Ok(scan_graph) => {
                        let drift_report = sruja_diff::detect_architectural_drift(&scan_graph);
                        ToolResponse::success(
                            "detect_drift",
                            serde_json::to_value(drift_report).unwrap_or_default(),
                        )
                    }
                    Err(e) => ToolResponse::error("detect_drift", e.to_string()),
                }
            }

            SrujaTool::Quickstart { repo_path } => {
                use std::path::Path;
                match sruja_scan::scan_repo(Path::new(repo_path)) {
                    Ok(scan_graph) => {
                        let drift_report = sruja_diff::detect_architectural_drift(&scan_graph);

                        // Create a quickstart result similar to CLI
                        let external_apis = scan_graph
                            .nodes
                            .iter()
                            .filter(|n| n.kind == sruja_scan::NodeKind::ExternalApi)
                            .count();

                        let result = serde_json::json!({
                            "repo": repo_path,
                            "health_score": drift_report.health_score,
                            "inventory": {
                                "modules": drift_report.total_modules,
                                "services": drift_report.total_services,
                                "databases": drift_report.total_databases,
                                "external_apis": external_apis,
                                "total_dependencies": drift_report.total_dependencies,
                            },
                            "violations_count": drift_report.violations.len(),
                            "suggestions_count": drift_report.suggestions.len(),
                        });

                        ToolResponse::success("quickstart", result)
                    }
                    Err(e) => ToolResponse::error("quickstart", e.to_string()),
                }
            }
        }
    }
}

pub fn list_tools() -> Vec<serde_json::Value> {
    let tools = [
        SrujaTool::GetArchitecture,
        SrujaTool::GetDecisions,
        SrujaTool::GetDecision { id: String::new() },
        SrujaTool::GetPolicyConflicts,
        SrujaTool::Query {
            question: String::new(),
        },
        SrujaTool::GetComponent { id: String::new() },
        SrujaTool::ScanRepo {
            path: String::new(),
        },
        SrujaTool::DetectDrift {
            repo_path: String::new(),
        },
        SrujaTool::Quickstart {
            repo_path: String::new(),
        },
    ];

    tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name(),
                "description": t.description(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_repo() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a simple TypeScript file
        fs::write(
            temp_dir.path().join("main.ts"),
            r#"
import { hello } from './hello';

export function main() {
    return hello();
}
"#,
        )
        .expect("Failed to write main.ts");

        fs::write(
            temp_dir.path().join("hello.ts"),
            r#"export function hello(): string { return 'world'; }"#,
        )
        .expect("Failed to write hello.ts");

        temp_dir
    }

    #[test]
    fn test_scan_repo_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::ScanRepo {
            path: repo.path().to_str().unwrap().to_string(),
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "scan_repo");
        assert!(
            response.success,
            "ScanRepo should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());

        // Verify the result contains nodes
        let nodes = response
            .result
            .get("nodes")
            .expect("Result should have nodes");
        assert!(nodes.is_array());
    }

    #[test]
    fn test_scan_repo_tool_with_invalid_path() {
        let tool = SrujaTool::ScanRepo {
            path: "/nonexistent/path/12345".to_string(),
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "scan_repo");
        assert!(!response.success, "ScanRepo with invalid path should fail");
        assert!(response.error.is_some());
    }

    #[test]
    fn test_detect_drift_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::DetectDrift {
            repo_path: repo.path().to_str().unwrap().to_string(),
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "detect_drift");
        assert!(
            response.success,
            "DetectDrift should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());

        // Verify the result contains drift report fields
        assert!(response.result.get("health_score").is_some());
        assert!(response.result.get("violations").is_some());
        assert!(response.result.get("suggestions").is_some());
    }

    #[test]
    fn test_quickstart_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::Quickstart {
            repo_path: repo.path().to_str().unwrap().to_string(),
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "quickstart");
        assert!(
            response.success,
            "Quickstart should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());

        // Verify the result contains quickstart fields
        assert!(response.result.get("repo").is_some());
        assert!(response.result.get("health_score").is_some());
        assert!(response.result.get("inventory").is_some());

        let inventory = response.result.get("inventory").unwrap();
        assert!(inventory.get("modules").is_some());
        assert!(inventory.get("services").is_some());
        assert!(inventory.get("databases").is_some());
    }

    #[test]
    fn test_list_tools_includes_new_tools() {
        let tools = list_tools();
        let tool_names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();

        assert!(
            tool_names.contains(&"scan_repo"),
            "list_tools should include scan_repo"
        );
        assert!(
            tool_names.contains(&"detect_drift"),
            "list_tools should include detect_drift"
        );
        assert!(
            tool_names.contains(&"quickstart"),
            "list_tools should include quickstart"
        );
    }

    #[test]
    fn test_tool_names() {
        assert_eq!(
            SrujaTool::ScanRepo {
                path: String::new()
            }
            .name(),
            "scan_repo"
        );
        assert_eq!(
            SrujaTool::DetectDrift {
                repo_path: String::new()
            }
            .name(),
            "detect_drift"
        );
        assert_eq!(
            SrujaTool::Quickstart {
                repo_path: String::new()
            }
            .name(),
            "quickstart"
        );
    }

    #[test]
    fn test_tool_descriptions() {
        let scan_tool = SrujaTool::ScanRepo {
            path: String::new(),
        };
        assert!(scan_tool.description().contains("Scan a repository"));

        let drift_tool = SrujaTool::DetectDrift {
            repo_path: String::new(),
        };
        assert!(drift_tool.description().contains("drift"));

        let quickstart_tool = SrujaTool::Quickstart {
            repo_path: String::new(),
        };
        assert!(quickstart_tool.description().contains("insights"));
    }
}
