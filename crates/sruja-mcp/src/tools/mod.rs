//! MCP Tools for Sruja

mod analyze;
mod architecture;
mod scan;

use serde::{Deserialize, Serialize};
use sruja_graph::KnowledgeGraph;
use std::path::{Path, PathBuf};

pub(crate) fn validate_path(path_str: &str) -> Result<PathBuf, String> {
    let path = Path::new(path_str);

    if path_str.contains("..") {
        return Err(format!("Path traversal detected: '{}'", path_str));
    }

    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Path does not exist or is inaccessible: {}", e))?;

    let cwd = std::env::current_dir()
        .map_err(|e| format!("Cannot determine current directory: {}", e))?;
    let temp = std::env::temp_dir();

    if canonical.starts_with(&cwd) || canonical.starts_with(&temp) {
        return Ok(canonical);
    }

    Ok(canonical)
}

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
    SemanticAnalyze {
        repo_path: String,
    },
    Complexity {
        repo_path: String,
        treewidth: bool,
        scc: bool,
        centrality: bool,
        coupling: bool,
    },
    RunAnalyze {
        repo_path: String,
        traces_path: Option<String>,
        intent_path: Option<String>,
    },
    IntentCheck {
        repo_path: String,
        intent_path: Option<String>,
    },
    DetectDriftWithBaseline {
        repo_path: String,
        architecture_path: String,
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
            SrujaTool::SemanticAnalyze { .. } => "semantic_analyze",
            SrujaTool::Complexity { .. } => "complexity",
            SrujaTool::RunAnalyze { .. } => "run_analyze",
            SrujaTool::IntentCheck { .. } => "intent_check",
            SrujaTool::DetectDriftWithBaseline { .. } => "detect_drift_with_baseline",
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
            SrujaTool::SemanticAnalyze { .. } => {
                "Analyze semantic coupling, bounded contexts, and vocabulary leakage"
            }
            SrujaTool::Complexity { .. } => {
                "Analyze structural complexity (treewidth, SCC, centrality, coupling)"
            }
            SrujaTool::RunAnalyze { .. } => {
                "Full four-layer analysis (structural, semantic, intent, optional runtime)"
            }
            SrujaTool::IntentCheck { .. } => {
                "Compare declared architectural intent vs actual implementation"
            }
            SrujaTool::DetectDriftWithBaseline { .. } => {
                "Compare scanned repo against a .sruja architecture baseline"
            }
        }
    }

    pub fn execute(&self, graph: &KnowledgeGraph) -> ToolResponse {
        if let Some(r) = architecture::execute_architecture(self, graph) {
            return r;
        }
        if let Some(r) = scan::execute_scan(self, graph, validate_path) {
            return r;
        }
        if let Some(r) = analyze::execute_analyze(self, graph, validate_path) {
            return r;
        }
        unreachable!("every tool variant is handled by one of the modules")
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
        SrujaTool::SemanticAnalyze {
            repo_path: String::new(),
        },
        SrujaTool::Complexity {
            repo_path: String::new(),
            treewidth: false,
            scc: false,
            centrality: false,
            coupling: false,
        },
        SrujaTool::RunAnalyze {
            repo_path: String::new(),
            traces_path: None,
            intent_path: None,
        },
        SrujaTool::IntentCheck {
            repo_path: String::new(),
            intent_path: None,
        },
        SrujaTool::DetectDriftWithBaseline {
            repo_path: String::new(),
            architecture_path: String::new(),
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
        let cwd = std::env::current_dir().expect("current dir");
        let temp_dir = tempfile::tempdir_in(cwd).expect("Failed to create temp dir");

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
        assert!(response.result.get("repo").is_some());
        assert!(response.result.get("health_score").is_some());
        assert!(response.result.get("inventory").is_some());

        let inventory = response.result.get("inventory").unwrap();
        assert!(inventory.get("modules").is_some());
        assert!(inventory.get("services").is_some());
        assert!(inventory.get("databases").is_some());
    }

    #[test]
    fn test_semantic_analyze_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::SemanticAnalyze {
            repo_path: repo.path().to_str().unwrap().to_string(),
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "semantic_analyze");
        assert!(
            response.success,
            "SemanticAnalyze should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());
        assert!(response.result.get("component_count").is_some());
        assert!(response.result.get("health_score").is_some());
    }

    #[test]
    fn test_complexity_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::Complexity {
            repo_path: repo.path().to_str().unwrap().to_string(),
            treewidth: true,
            scc: true,
            centrality: false,
            coupling: false,
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "complexity");
        assert!(
            response.success,
            "Complexity should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());
        assert!(response.result.get("total_nodes").is_some());
        assert!(response.result.get("total_edges").is_some());
    }

    #[test]
    fn test_run_analyze_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::RunAnalyze {
            repo_path: repo.path().to_str().unwrap().to_string(),
            traces_path: None,
            intent_path: None,
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "run_analyze");
        assert!(
            response.success,
            "RunAnalyze should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());
        assert!(response.result.get("structural").is_some());
        assert!(response.result.get("semantic").is_some());
        assert!(response.result.get("overall_health").is_some());
    }

    #[test]
    fn test_intent_check_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::IntentCheck {
            repo_path: repo.path().to_str().unwrap().to_string(),
            intent_path: None,
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "intent_check");
        assert!(
            response.success,
            "IntentCheck should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());
        let has_error = response.result.get("error").is_some();
        let has_drift_score = response.result.get("drift_score").is_some();
        assert!(
            has_error || has_drift_score,
            "result should have error or drift_score"
        );
    }

    #[test]
    fn test_detect_drift_with_baseline_tool() {
        let repo = create_test_repo();
        let sruja_path = repo.path().join("arch.sruja");
        let minimal_sruja = r#"
api = container "API" {
  technology "Node.js"
  description "HTTP API"
}
"#;
        fs::write(&sruja_path, minimal_sruja).expect("write arch.sruja");

        let tool = SrujaTool::DetectDriftWithBaseline {
            repo_path: repo.path().to_str().unwrap().to_string(),
            architecture_path: sruja_path.to_str().unwrap().to_string(),
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "detect_drift_with_baseline");
        assert!(
            response.success,
            "DetectDriftWithBaseline should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());
        assert!(
            response.result.get("node_diff").is_some()
                || response.result.get("violations").is_some(),
            "result should have node_diff or violations"
        );
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
        assert!(
            tool_names.contains(&"semantic_analyze"),
            "list_tools should include semantic_analyze"
        );
        assert!(
            tool_names.contains(&"complexity"),
            "list_tools should include complexity"
        );
        assert!(
            tool_names.contains(&"run_analyze"),
            "list_tools should include run_analyze"
        );
        assert!(
            tool_names.contains(&"intent_check"),
            "list_tools should include intent_check"
        );
        assert!(
            tool_names.contains(&"detect_drift_with_baseline"),
            "list_tools should include detect_drift_with_baseline"
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

    #[test]
    fn test_path_validation_rejects_traversal() {
        assert!(
            validate_path("../../../etc/passwd").is_err(),
            "Should reject path traversal"
        );
        assert!(
            validate_path("foo/../bar").is_err(),
            "Should reject path traversal in middle"
        );
        assert!(
            validate_path("/nonexistent/path").is_err(),
            "Should reject nonexistent path"
        );
    }

    #[test]
    fn test_path_validation_accepts_valid() {
        let cwd = std::env::current_dir().expect("Need cwd");
        assert!(
            validate_path(cwd.to_str().unwrap()).is_ok(),
            "Should accept current directory"
        );
    }
}
