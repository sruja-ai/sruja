//! Query interface for Knowledge Graph
//!
//! Evidence is produced deterministically from graph data (no LLM). Templates
//! format nodes, edges, and decisions for consistent CLI output.

use crate::*;

mod formatters;
mod types;
mod methods;

pub use types::*;
use formatters::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_node(
        id: &str,
        kind: NodeKind,
        label: &str,
        tech: Option<&str>,
    ) -> ArchitectureNode {
        let mut node = ArchitectureNode {
            id: id.to_string(),
            kind,
            label: label.to_string(),
            ..ArchitectureNode::default()
        };
        if let Some(t) = tech {
            node.set_technology(Some(t.to_string()));
        }
        node
    }

    fn create_test_graph() -> KnowledgeGraph {
        let mut graph = KnowledgeGraph::new();

        graph
            .add_node(make_test_node(
                "api",
                NodeKind::new(NodeKind::SERVICE),
                "API Service",
                Some("Node.js"),
            ))
            .unwrap();

        graph
            .add_node(make_test_node(
                "db",
                NodeKind::new(NodeKind::DATABASE),
                "PostgreSQL",
                Some("PostgreSQL"),
            ))
            .unwrap();

        graph
    }

    #[test]
    fn test_query_what_services() {
        let graph = create_test_graph();
        let result = graph.query("what services do we have?").unwrap();
        assert!(result.answer.contains("service"));
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_query_why_technology() {
        let graph = create_test_graph();
        let result = graph.query("why are we using Node.js?").unwrap();
        assert!(result.answer.to_lowercase().contains("node") || result.confidence > 0.0);
    }

    #[test]
    fn test_query_what_databases() {
        let graph = create_test_graph();
        let result = graph.query("what databases do we have?").unwrap();
        assert!(result.answer.to_lowercase().contains("database"));
    }

    #[test]
    fn test_query_how_connected() {
        let graph = create_test_graph();
        let result = graph.query("how are components connected?").unwrap();
        assert!(!result.answer.is_empty());
    }

    #[test]
    fn test_query_decisions() {
        let graph = create_test_graph();
        let result = graph.query("show me all decisions").unwrap();
        assert!(result.answer.contains("decisions"));
    }

    #[test]
    fn test_query_generic() {
        let graph = create_test_graph();
        let result = graph.query("tell me about the architecture").unwrap();
        assert!(result.answer.contains("components"));
    }

    #[test]
    fn test_query_why_reasoned_returns_steps() {
        let mut graph = create_test_graph();
        graph
            .add_edge(ArchitectureEdge {
                id: "api-to-db".to_string(),
                source: "api".to_string(),
                target: "db".to_string(),
                kind: EdgeKind::new(EdgeKind::CALLS),
                label: Some("SQL".to_string()),
                description: None,
                source_ref: SourceReference::manual(),
            })
            .unwrap();

        let result = graph.query_why_reasoned("api", 3).unwrap();
        assert_eq!(result.question, "api");
        assert_eq!(result.target_id, "api");
        assert!(!result.steps.is_empty());
        let step = &result.steps[0];
        assert_eq!(step.direction, "downstream");
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_query_why_reasoned_orphan_node() {
        let graph = create_test_graph();
        let result = graph.query_why_reasoned("db", 2).unwrap();
        assert_eq!(result.target_id, "db");
        assert!(result.steps.is_empty());
        assert!(result.final_answer.contains("orphan"));
    }

    #[test]
    fn test_format_decision_evidence_short() {
        let decision = Decision {
            id: "adr-001".to_string(),
            title: "Use PostgreSQL".to_string(),
            status: DecisionStatus::Accepted,
            decision: "We chose PostgreSQL for its reliability.".to_string(),
            context: String::new(),
            consequences: String::new(),
            alternatives: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ratified_at: None,
            author: None,
            source: SourceReference::manual(),
            affects: vec![],
        };
        let evidence = format_decision_evidence(&decision);
        assert!(evidence.contains("Use PostgreSQL"));
    }

    #[test]
    fn test_format_decision_evidence_long() {
        let long_decision = "x".repeat(300);
        let decision = Decision {
            id: "adr-002".to_string(),
            title: "Long Decision".to_string(),
            status: DecisionStatus::Proposed,
            decision: long_decision.clone(),
            context: String::new(),
            consequences: String::new(),
            alternatives: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ratified_at: None,
            author: None,
            source: SourceReference::manual(),
            affects: vec![],
        };
        let evidence = format_decision_evidence(&decision);
        assert!(evidence.contains("..."));
        assert!(evidence.len() < long_decision.len() + 50);
    }

    #[test]
    fn test_format_node_evidence() {
        let node = make_test_node(
            "svc",
            NodeKind::new(NodeKind::SERVICE),
            "My Service",
            Some("Rust"),
        );
        let evidence = format_node_evidence(&node, None);
        assert!(evidence.contains("My Service"));
        assert!(evidence.contains("Rust"));
    }

    #[test]
    fn test_format_node_evidence_no_tech() {
        let node = make_test_node(
            "svc",
            NodeKind::new(NodeKind::SERVICE),
            "No Tech Service",
            None,
        );
        let evidence = format_node_evidence(&node, None);
        assert!(evidence.contains("(not set)"));
    }

    #[test]
    fn test_format_edge_evidence_with_label() {
        let evidence = format_edge_evidence(
            "Source",
            &EdgeKind::new(EdgeKind::CALLS),
            "Target",
            Some("HTTP"),
        );
        assert!(evidence.contains("Source"));
        assert!(evidence.contains("Target"));
        assert!(evidence.contains("HTTP"));
    }

    #[test]
    fn test_format_edge_evidence_without_label() {
        let evidence = format_edge_evidence("A", &EdgeKind::new(EdgeKind::READS_FROM), "B", None);
        assert!(evidence.contains("A"));
        assert!(evidence.contains("B"));
    }

    #[test]
    fn test_query_result_serialization() {
        let result = QueryResult {
            question: "test?".to_string(),
            answer: "answer".to_string(),
            evidence: vec![],
            confidence: 0.5,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test?"));
    }

    #[test]
    fn test_evidence_kind_variants() {
        let kinds = vec![
            EvidenceKind::Decision,
            EvidenceKind::Policy,
            EvidenceKind::Requirement,
            EvidenceKind::Node,
            EvidenceKind::Edge,
        ];
        for kind in kinds {
            let evidence = Evidence {
                kind: kind.clone(),
                reference: "ref".to_string(),
                excerpt: "excerpt".to_string(),
            };
            let json = serde_json::to_string(&evidence).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_extract_tech_patterns() {
        let graph = create_test_graph();
        let patterns = graph.extract_tech_patterns("why use postgresql and redis?");
        assert!(patterns.contains(&"postgresql".to_string()));
        assert!(patterns.contains(&"redis".to_string()));
    }

    #[test]
    fn test_find_policy_violations_empty() {
        let graph = create_test_graph();
        let violations = graph.find_policy_violations();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_query_why_entity() {
        let mut graph = create_test_graph();
        graph
            .add_edge(ArchitectureEdge {
                id: "api_to_db".to_string(),
                source: "api".to_string(),
                target: "db".to_string(),
                kind: EdgeKind::new(EdgeKind::CALLS),
                label: Some("SQL".to_string()),
                description: None,
                source_ref: SourceReference::manual(),
            })
            .unwrap();

        let result = graph.query("why api?").unwrap();
        assert!(result.answer.contains("API Service"));
        assert!(result.answer.contains("depends on"));
        assert_eq!(result.evidence.len(), 1);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_query_what_entity() {
        let graph = create_test_graph();
        let result = graph.query("what is api?").unwrap();
        assert!(result.answer.contains("API Service"));
        assert!(result.answer.contains("service"));
        assert_eq!(result.evidence.len(), 1);
    }

    #[test]
    fn test_query_connections() {
        let mut graph = create_test_graph();
        graph
            .add_edge(ArchitectureEdge {
                id: "api_to_db".to_string(),
                source: "api".to_string(),
                target: "db".to_string(),
                kind: EdgeKind::new(EdgeKind::CALLS),
                label: Some("SQL".to_string()),
                description: None,
                source_ref: SourceReference::manual(),
            })
            .unwrap();

        let result = graph.query("how does api connect?").unwrap();
        assert!(result.answer.contains("connects to"));
        assert_eq!(result.evidence.len(), 1);
    }

    #[test]
    fn test_query_describe() {
        let graph = create_test_graph();
        let result = graph.query("tell me about api").unwrap();
        assert!(result.answer.contains("API Service"));
        assert!(result.answer.contains("service"));
    }

    #[test]
    fn test_query_why_llmguided_basic() {
        let mut graph = create_test_graph();
        graph
            .add_edge(ArchitectureEdge {
                id: "api_to_db".to_string(),
                source: "api".to_string(),
                target: "db".to_string(),
                kind: EdgeKind::new(EdgeKind::CALLS),
                label: Some("SQL".to_string()),
                description: None,
                source_ref: SourceReference::manual(),
            })
            .unwrap();

        let result = graph.query_why_llmguided("api", 3).unwrap();
        assert_eq!(result.question, "api");
        assert_eq!(result.target_id, "api");
        assert_eq!(result.target_label, "API Service");
        assert!(!result.steps.is_empty());
        assert!(result.confidence > 0.0);
        let step = &result.steps[0];
        assert!(["upstream", "downstream"].contains(&step.direction.as_str()));
    }

    #[test]
    fn test_query_why_llmguided_multi_hop() {
        let mut graph = KnowledgeGraph::new();
        graph
            .add_node(make_test_node(
                "frontend",
                NodeKind::new(NodeKind::FRONTEND),
                "Web Frontend",
                Some("React"),
            ))
            .unwrap();
        graph
            .add_node(make_test_node(
                "api",
                NodeKind::new(NodeKind::SERVICE),
                "API Service",
                Some("Node.js"),
            ))
            .unwrap();
        graph
            .add_node(make_test_node(
                "db",
                NodeKind::new(NodeKind::DATABASE),
                "PostgreSQL",
                Some("PostgreSQL"),
            ))
            .unwrap();
        graph
            .add_edge(ArchitectureEdge {
                id: "fe_to_api".to_string(),
                source: "frontend".to_string(),
                target: "api".to_string(),
                kind: EdgeKind::new(EdgeKind::CALLS),
                label: Some("HTTPS".to_string()),
                description: None,
                source_ref: SourceReference::manual(),
            })
            .unwrap();
        graph
            .add_edge(ArchitectureEdge {
                id: "api_to_db".to_string(),
                source: "api".to_string(),
                target: "db".to_string(),
                kind: EdgeKind::new(EdgeKind::CALLS),
                label: Some("SQL".to_string()),
                description: None,
                source_ref: SourceReference::manual(),
            })
            .unwrap();

        let result = graph.query_why_llmguided("frontend", 3).unwrap();
        assert_eq!(result.target_id, "frontend");
        assert!(!result.steps.is_empty());
        let step_labels: Vec<&str> = result.steps.iter().map(|s| s.node_label.as_str()).collect();
        assert!(step_labels.contains(&"API Service") || step_labels.contains(&"Web Frontend"));
    }

    #[test]
    fn test_query_why_llmguided_branching_paths() {
        let mut graph = KnowledgeGraph::new();
        graph
            .add_node(make_test_node(
                "api",
                NodeKind::new(NodeKind::SERVICE),
                "API Service",
                Some("Node.js"),
            ))
            .unwrap();
        graph
            .add_node(make_test_node(
                "db",
                NodeKind::new(NodeKind::DATABASE),
                "PostgreSQL",
                Some("PostgreSQL"),
            ))
            .unwrap();
        graph
            .add_node(make_test_node(
                "cache",
                NodeKind::new(NodeKind::DATABASE),
                "Redis Cache",
                Some("Redis"),
            ))
            .unwrap();
        graph
            .add_edge(ArchitectureEdge {
                id: "api_to_db".to_string(),
                source: "api".to_string(),
                target: "db".to_string(),
                kind: EdgeKind::new(EdgeKind::CALLS),
                label: Some("SQL".to_string()),
                description: None,
                source_ref: SourceReference::manual(),
            })
            .unwrap();
        graph
            .add_edge(ArchitectureEdge {
                id: "api_to_cache".to_string(),
                source: "api".to_string(),
                target: "cache".to_string(),
                kind: EdgeKind::new(EdgeKind::CALLS),
                label: Some("GET/SET".to_string()),
                description: None,
                source_ref: SourceReference::manual(),
            })
            .unwrap();

        let result = graph.query_why_llmguided("api", 2).unwrap();
        assert_eq!(result.target_id, "api");
        let downstream_labels: Vec<&str> = result
            .steps
            .iter()
            .filter(|s| s.direction == "downstream")
            .map(|s| s.node_label.as_str())
            .collect();
        assert!(
            downstream_labels.contains(&"PostgreSQL") || downstream_labels.contains(&"Redis Cache")
        );
    }

    #[test]
    fn test_query_why_llmguided_orphan_node() {
        let graph = create_test_graph();
        let result = graph.query_why_llmguided("db", 2).unwrap();
        assert_eq!(result.target_id, "db");
        assert!(result.steps.is_empty());
        assert!(result.summary.contains("isolated") || result.summary.contains("no relevant"));
        assert_eq!(result.confidence, 0.3);
    }

    #[test]
    fn test_query_why_llmguided_upstream_and_downstream() {
        let mut graph = KnowledgeGraph::new();
        graph
            .add_node(make_test_node(
                "upstream_svc",
                NodeKind::new(NodeKind::SERVICE),
                "Auth Service",
                Some("Go"),
            ))
            .unwrap();
        graph
            .add_node(make_test_node(
                "api",
                NodeKind::new(NodeKind::SERVICE),
                "API Service",
                Some("Node.js"),
            ))
            .unwrap();
        graph
            .add_node(make_test_node(
                "db",
                NodeKind::new(NodeKind::DATABASE),
                "PostgreSQL",
                Some("PostgreSQL"),
            ))
            .unwrap();
        graph
            .add_edge(ArchitectureEdge {
                id: "auth_to_api".to_string(),
                source: "upstream_svc".to_string(),
                target: "api".to_string(),
                kind: EdgeKind::new(EdgeKind::CALLS),
                label: Some("gRPC".to_string()),
                description: None,
                source_ref: SourceReference::manual(),
            })
            .unwrap();
        graph
            .add_edge(ArchitectureEdge {
                id: "api_to_db".to_string(),
                source: "api".to_string(),
                target: "db".to_string(),
                kind: EdgeKind::new(EdgeKind::CALLS),
                label: Some("SQL".to_string()),
                description: None,
                source_ref: SourceReference::manual(),
            })
            .unwrap();

        let result = graph.query_why_llmguided("api", 2).unwrap();
        assert_eq!(result.target_id, "api");
        let directions: std::collections::HashSet<&str> =
            result.steps.iter().map(|s| s.direction.as_str()).collect();
        assert!(directions.contains("upstream") || directions.contains("downstream"));
    }
}
