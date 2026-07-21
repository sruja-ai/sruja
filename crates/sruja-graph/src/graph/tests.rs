use crate::learning::{ExperimentOutcome, LearningKind};
use crate::*;
use super::{GraphMetadata, GraphStats};

fn test_node(id: &str) -> ArchitectureNode {
    ArchitectureNode {
        id: id.to_string(),
        kind: NodeKind::new(NodeKind::SERVICE),
        label: id.to_string(),
        ..ArchitectureNode::default()
    }
}

#[test]
fn test_add_node() {
    let mut graph = KnowledgeGraph::new();
    let node = test_node("api");
    graph.add_node(node).unwrap();
    assert!(graph.get_node("api").is_some());
}

#[test]
fn test_duplicate_node() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("api")).unwrap();
    let result = graph.add_node(test_node("api"));
    assert!(result.is_err());
}

#[test]
fn test_add_edge() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("api")).unwrap();
    graph.add_node(test_node("db")).unwrap();

    let edge = ArchitectureEdge {
        id: "edge1".to_string(),
        source: "api".to_string(),
        target: "db".to_string(),
        kind: EdgeKind::new(EdgeKind::DEPENDS_ON),
        label: None,
        description: None,
        source_ref: SourceReference::manual(),
    };

    graph.add_edge(edge).unwrap();
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_edge_to_nonexistent_node() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("api")).unwrap();

    let edge = ArchitectureEdge {
        id: "edge1".to_string(),
        source: "api".to_string(),
        target: "nonexistent".to_string(),
        kind: EdgeKind::new(EdgeKind::DEPENDS_ON),
        label: None,
        description: None,
        source_ref: SourceReference::manual(),
    };

    let result = graph.add_edge(edge);
    assert!(result.is_err());
}

fn test_edge(id: &str, source: &str, target: &str, kind: EdgeKind) -> ArchitectureEdge {
    ArchitectureEdge {
        id: id.to_string(),
        source: source.to_string(),
        target: target.to_string(),
        kind,
        label: None,
        description: None,
        source_ref: SourceReference::manual(),
    }
}

#[test]
fn test_remove_node_removes_edges() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("api")).unwrap();
    graph.add_node(test_node("db")).unwrap();
    graph
        .add_edge(test_edge(
            "e1",
            "api",
            "db",
            EdgeKind::new(EdgeKind::DEPENDS_ON),
        ))
        .unwrap();

    let removed = graph.remove_node("api").unwrap();
    assert_eq!(removed.id, "api");
    assert!(graph.get_node("api").is_none());
    assert!(graph.edges.is_empty());
}

#[test]
fn test_remove_nonexistent_node_errors() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("api")).unwrap();
    let result = graph.remove_node("missing");
    assert!(result.is_err());
}

#[test]
fn test_merge_node_inserts_or_updates() {
    let mut graph = KnowledgeGraph::new();
    let n1 = test_node("api");
    graph.merge_node(n1);
    assert_eq!(graph.nodes.len(), 1);

    let mut n2 = test_node("api");
    n2.label = "API v2".to_string();
    graph.merge_node(n2);
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.get_node("api").unwrap().label, "API v2");
}

#[test]
fn test_merge_edge_skips_if_nodes_missing() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("api")).unwrap();
    graph.merge_edge(test_edge(
        "e1",
        "api",
        "nonexistent",
        EdgeKind::new(EdgeKind::CALLS),
    ));
    assert!(graph.edges.is_empty());
}

#[test]
fn test_merge_edge_adds_when_nodes_exist() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("api")).unwrap();
    graph.add_node(test_node("db")).unwrap();
    graph.merge_edge(test_edge(
        "e1",
        "api",
        "db",
        EdgeKind::new(EdgeKind::READS_FROM),
    ));
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_stats_counts_correctly() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("a")).unwrap();
    graph.add_node(test_node("b")).unwrap();
    graph
        .add_edge(test_edge("e1", "a", "b", EdgeKind::new(EdgeKind::CALLS)))
        .unwrap();

    let stats = graph.stats();
    assert_eq!(stats.total_nodes, 2);
    assert_eq!(stats.total_edges, 1);
}

#[test]
fn test_with_name_sets_metadata() {
    let graph = KnowledgeGraph::with_name("My Architecture");
    assert_eq!(graph.metadata.name, "My Architecture");
}

#[test]
fn test_find_nodes_by_kind() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("api")).unwrap();
    let mut db_node = test_node("db");
    db_node.kind = NodeKind::new(NodeKind::DATABASE);
    graph.add_node(db_node).unwrap();

    let services = graph.find_nodes_by_kind(NodeKind::new(NodeKind::SERVICE));
    let dbs = graph.find_nodes_by_kind(NodeKind::new(NodeKind::DATABASE));
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].id, "api");
    assert_eq!(dbs.len(), 1);
    assert_eq!(dbs[0].id, "db");
}

#[test]
fn test_get_edges_from_and_to() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("a")).unwrap();
    graph.add_node(test_node("b")).unwrap();
    graph.add_node(test_node("c")).unwrap();
    graph
        .add_edge(test_edge("e1", "a", "b", EdgeKind::new(EdgeKind::CALLS)))
        .unwrap();
    graph
        .add_edge(test_edge("e2", "a", "c", EdgeKind::new(EdgeKind::CALLS)))
        .unwrap();

    let from_a = graph.get_edges_from("a");
    let to_b = graph.get_edges_to("b");
    assert_eq!(from_a.len(), 2);
    assert_eq!(to_b.len(), 1);
    assert_eq!(to_b[0].source, "a");
}

#[test]
fn test_to_json_and_from_json() {
    let mut graph = KnowledgeGraph::with_name("TestGraph");
    graph.add_node(test_node("svc")).unwrap();

    let json = graph.to_json().unwrap();
    let restored = KnowledgeGraph::from_json(&json).unwrap();

    assert_eq!(restored.metadata.name, "TestGraph");
    assert!(restored.get_node("svc").is_some());
}

#[test]
fn test_add_decision() {
    let mut graph = KnowledgeGraph::new();
    let decision = Decision {
        id: "ADR-001".to_string(),
        title: "Use PostgreSQL".to_string(),
        status: DecisionStatus::Proposed,
        context: "Need reliable database".to_string(),
        decision: "Use PostgreSQL for primary datastore".to_string(),
        consequences: "Team needs training".to_string(),
        alternatives: vec!["MySQL".to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        ratified_at: None,
        author: Some("Team".to_string()),
        source: SourceReference::manual(),
        affects: vec!["db".to_string()],
    };

    graph.add_decision(decision).unwrap();
    assert!(graph.get_decision("ADR-001").is_some());
}

#[test]
fn test_accept_decision() {
    let mut graph = KnowledgeGraph::new();
    let decision = Decision {
        id: "ADR-001".to_string(),
        title: "Test".to_string(),
        status: DecisionStatus::Proposed,
        context: String::new(),
        decision: String::new(),
        consequences: String::new(),
        alternatives: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        ratified_at: None,
        author: None,
        source: SourceReference::manual(),
        affects: vec![],
    };
    graph.add_decision(decision).unwrap();

    graph.accept_decision("ADR-001").unwrap();
    let accepted = graph.get_decision("ADR-001").unwrap();
    assert_eq!(accepted.status, DecisionStatus::Accepted);
    assert!(accepted.ratified_at.is_some());
}

#[test]
fn test_accept_nonexistent_decision_errors() {
    let mut graph = KnowledgeGraph::new();
    let result = graph.accept_decision("missing");
    assert!(result.is_err());
}

#[test]
fn test_get_decisions_for_node() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("api")).unwrap();
    graph.add_node(test_node("db")).unwrap();

    let d1 = Decision {
        id: "ADR-1".to_string(),
        title: "API Decision".to_string(),
        status: DecisionStatus::Accepted,
        context: String::new(),
        decision: String::new(),
        consequences: String::new(),
        alternatives: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        ratified_at: None,
        author: None,
        source: SourceReference::manual(),
        affects: vec!["api".to_string()],
    };
    let d2 = Decision {
        id: "ADR-2".to_string(),
        title: "DB Decision".to_string(),
        status: DecisionStatus::Accepted,
        context: String::new(),
        decision: String::new(),
        consequences: String::new(),
        alternatives: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        ratified_at: None,
        author: None,
        source: SourceReference::manual(),
        affects: vec!["db".to_string()],
    };

    graph.add_decision(d1).unwrap();
    graph.add_decision(d2).unwrap();

    let api_decisions = graph.get_decisions_for_node("api");
    assert_eq!(api_decisions.len(), 1);
    assert_eq!(api_decisions[0].id, "ADR-1");
}

#[test]
fn test_add_policy() {
    let mut graph = KnowledgeGraph::new();
    let policy = Policy {
        id: "POL-001".to_string(),
        name: "Security Policy".to_string(),
        description: "Enforce security standards".to_string(),
        rules: vec![],
        severity: crate::PolicySeverity::Error,
        source: SourceReference::manual(),
    };

    graph.add_policy(policy).unwrap();
    assert!(graph.get_policy("POL-001").is_some());
}

#[test]
fn test_add_requirement() {
    let mut graph = KnowledgeGraph::new();
    let req = Requirement {
        id: "REQ-001".to_string(),
        title: "Login Feature".to_string(),
        description: "Users must be able to login".to_string(),
        priority: crate::RequirementPriority::Must,
        source: SourceReference::manual(),
        satisfied_by: vec![],
    };

    graph.add_requirement(req).unwrap();
    assert!(graph.get_requirement("REQ-001").is_some());
}

#[test]
fn test_find_nodes_by_technology() {
    let mut graph = KnowledgeGraph::new();
    let mut node1 = test_node("api");
    node1.set_technology(Some("Rust".to_string()));
    let mut node2 = test_node("web");
    node2.set_technology(Some("TypeScript".to_string()));

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();

    let rust_nodes = graph.find_nodes_by_technology("rust");
    assert_eq!(rust_nodes.len(), 1);
    assert_eq!(rust_nodes[0].id, "api");
}

#[test]
fn test_find_nodes_by_technology_case_insensitive() {
    let mut graph = KnowledgeGraph::new();
    let mut node = test_node("api");
    node.set_technology(Some("Rust".to_string()));
    graph.add_node(node).unwrap();

    let rust_nodes = graph.find_nodes_by_technology("RUST");
    assert_eq!(rust_nodes.len(), 1);
}

#[test]
fn test_remove_edge_by_id() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("a")).unwrap();
    graph.add_node(test_node("b")).unwrap();
    graph
        .add_edge(test_edge("e1", "a", "b", EdgeKind::new(EdgeKind::CALLS)))
        .unwrap();

    let removed = graph.remove_edge("e1");
    assert!(removed.is_some());
    assert!(graph.edges.is_empty());
}

#[test]
fn test_remove_nonexistent_edge_returns_none() {
    let mut graph = KnowledgeGraph::new();
    let result = graph.remove_edge("missing");
    assert!(result.is_none());
}

#[test]
fn test_get_node_mut() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("api")).unwrap();

    let node = graph.get_node_mut("api").unwrap();
    node.label = "Updated API".to_string();

    assert_eq!(graph.get_node("api").unwrap().label, "Updated API");
}

#[test]
fn test_get_node_mut_nonexistent() {
    let mut graph = KnowledgeGraph::new();
    assert!(graph.get_node_mut("missing").is_none());
}

#[test]
fn test_default_metadata() {
    let meta = GraphMetadata::default();
    assert_eq!(meta.name, "Architecture Graph");
    assert_eq!(meta.version, "1.0.0");
}

#[test]
fn test_touch_updates_timestamp() {
    let mut graph = KnowledgeGraph::new();
    let before = graph.metadata.updated_at;
    std::thread::sleep(std::time::Duration::from_millis(1));
    graph.touch();
    assert!(graph.metadata.updated_at > before);
}

#[test]
fn test_stats_empty_graph() {
    let graph = KnowledgeGraph::new();
    let stats = graph.stats();
    assert_eq!(stats.total_nodes, 0);
    assert_eq!(stats.total_edges, 0);
    assert_eq!(stats.total_decisions, 0);
}

#[test]
fn test_stats_with_accepted_decisions() {
    let mut graph = KnowledgeGraph::new();
    let d1 = Decision {
        id: "ADR-1".to_string(),
        title: "Test".to_string(),
        status: DecisionStatus::Accepted,
        context: String::new(),
        decision: String::new(),
        consequences: String::new(),
        alternatives: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        ratified_at: None,
        author: None,
        source: SourceReference::manual(),
        affects: vec![],
    };
    let d2 = Decision {
        id: "ADR-2".to_string(),
        title: "Test".to_string(),
        status: DecisionStatus::Proposed,
        context: String::new(),
        decision: String::new(),
        consequences: String::new(),
        alternatives: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        ratified_at: None,
        author: None,
        source: SourceReference::manual(),
        affects: vec![],
    };
    graph.add_decision(d1).unwrap();
    graph.add_decision(d2).unwrap();

    let stats = graph.stats();
    assert_eq!(stats.accepted_decisions, 1);
    assert_eq!(stats.proposed_decisions, 1);
}

#[test]
fn test_merge_edge_skips_duplicate() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("a")).unwrap();
    graph.add_node(test_node("b")).unwrap();
    graph.merge_edge(test_edge("e1", "a", "b", EdgeKind::new(EdgeKind::CALLS)));
    graph.merge_edge(test_edge("e2", "a", "b", EdgeKind::new(EdgeKind::CALLS)));
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_merge_edge_allows_different_kinds() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("a")).unwrap();
    graph.add_node(test_node("b")).unwrap();
    graph.merge_edge(test_edge("e1", "a", "b", EdgeKind::new(EdgeKind::CALLS)));
    graph.merge_edge(test_edge(
        "e2",
        "a",
        "b",
        EdgeKind::new(EdgeKind::DEPENDS_ON),
    ));
    assert_eq!(graph.edges.len(), 2);
}

#[test]
fn test_empty_graph_json() {
    let graph = KnowledgeGraph::new();
    let json = graph.to_json().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.get("nodes").unwrap().as_object().unwrap().is_empty());
}

fn test_learning(id: &str, affected: Vec<&str>) -> LearningEntry {
    LearningEntry {
        id: id.to_string(),
        kind: Some(LearningKind::Guardrail),
        timestamp: Utc::now(),
        run_id: None,
        repo: None,
        selector: None,
        context: "test context".to_string(),
        hypothesis: "test hypothesis".to_string(),
        outcome: ExperimentOutcome::Failed,
        reason: None,
        guardrail_advice: "don't do X".to_string(),
        affected_elements: affected.into_iter().map(String::from).collect(),
        evidence_refs: vec![],
        confidence: Some("high".to_string()),
        tags: Vec::new(),
        hitl_kind: None,
        related_ids: Vec::new(),
        retrieval_count: 0,
        task_success_after: 0,
        task_total_after: 0,
        category: None,
        signals_match: Vec::new(),
        constraints: None,
        validation: Vec::new(),
        blast_radius: None,
    }
}

#[test]
fn test_add_learning() {
    let mut graph = KnowledgeGraph::new();
    let learning = test_learning("L1", vec!["api"]);
    graph.add_learning(learning);
    assert!(graph.get_learning("L1").is_some());
}

#[test]
fn test_get_learnings_for_node() {
    let mut graph = KnowledgeGraph::new();
    graph.add_learning(test_learning("L1", vec!["api", "db"]));
    graph.add_learning(test_learning("L2", vec!["db"]));
    graph.add_learning(test_learning("L3", vec!["web"]));

    let api_learnings = graph.get_learnings_for_node("api");
    assert_eq!(api_learnings.len(), 1);
    assert_eq!(api_learnings[0].id, "L1");

    let db_learnings = graph.get_learnings_for_node("db");
    assert_eq!(db_learnings.len(), 2);
}

#[test]
fn test_get_learnings_for_cluster() {
    let mut graph = KnowledgeGraph::new();
    graph.add_learning(test_learning("L1", vec!["api"]));
    graph.add_learning(test_learning("L2", vec!["db"]));
    graph.add_learning(test_learning("L3", vec!["web"]));

    let cluster = vec!["api".to_string(), "db".to_string()];
    let learnings = graph.get_learnings_for_cluster(&cluster);
    assert_eq!(learnings.len(), 2);
}

#[test]
fn test_get_learning_neighbors() {
    let mut graph = KnowledgeGraph::new();
    let mut l1 = test_learning("L1", vec!["api"]);
    l1.related_ids = vec!["L2".to_string()];
    graph.add_learning(l1);
    graph.add_learning(test_learning("L2", vec!["db"]));
    graph.add_learning(test_learning("L3", vec!["web"]));

    let neighbors = graph.get_learning_neighbors("L1");
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].id, "L2");
}

#[test]
fn test_get_decisions_for_blast_radius() {
    let mut graph = KnowledgeGraph::new();
    graph.add_node(test_node("api")).unwrap();
    graph.add_node(test_node("db")).unwrap();
    graph.add_node(test_node("cache")).unwrap();
    graph
        .add_edge(test_edge("e1", "api", "db", EdgeKind::new(EdgeKind::CALLS)))
        .unwrap();

    let d1 = Decision {
        id: "ADR-1".to_string(),
        title: "API Decision".to_string(),
        status: DecisionStatus::Accepted,
        context: String::new(),
        decision: String::new(),
        consequences: String::new(),
        alternatives: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        ratified_at: None,
        author: None,
        source: SourceReference::manual(),
        affects: vec!["api".to_string()],
    };
    let d2 = Decision {
        id: "ADR-2".to_string(),
        title: "DB Decision".to_string(),
        status: DecisionStatus::Accepted,
        context: String::new(),
        decision: String::new(),
        consequences: String::new(),
        alternatives: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        ratified_at: None,
        author: None,
        source: SourceReference::manual(),
        affects: vec!["db".to_string()],
    };
    let d3 = Decision {
        id: "ADR-3".to_string(),
        title: "Cache Decision".to_string(),
        status: DecisionStatus::Accepted,
        context: String::new(),
        decision: String::new(),
        consequences: String::new(),
        alternatives: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        ratified_at: None,
        author: None,
        source: SourceReference::manual(),
        affects: vec!["cache".to_string()],
    };
    graph.add_decision(d1).unwrap();
    graph.add_decision(d2).unwrap();
    graph.add_decision(d3).unwrap();

    // Blast radius of "api" should include "api" and "db" (downstream), but not "cache"
    let decisions = graph.get_decisions_for_blast_radius("api");
    assert_eq!(decisions.len(), 2);
    let ids: Vec<&str> = decisions.iter().map(|d| d.id.as_str()).collect();
    assert!(ids.contains(&"ADR-1"));
    assert!(ids.contains(&"ADR-2"));
    assert!(!ids.contains(&"ADR-3"));
}

#[test]
fn test_stats_includes_learnings() {
    let mut graph = KnowledgeGraph::new();
    graph.add_learning(test_learning("L1", vec!["api"]));
    graph.add_learning(test_learning("L2", vec!["db"]));

    let stats = graph.stats();
    assert_eq!(stats.total_learnings, 2);
}

#[test]
fn test_learning_roundtrip_json() {
    let mut graph = KnowledgeGraph::new();
    graph.add_learning(test_learning("L1", vec!["api"]));

    let json = graph.to_json().unwrap();
    let restored = KnowledgeGraph::from_json(&json).unwrap();
    assert!(restored.get_learning("L1").is_some());
    assert_eq!(restored.learnings.len(), 1);
}

fn test_event(kind: &str, elements: Vec<&str>) -> ContextEventSummary {
    ContextEventSummary {
        timestamp: Utc::now(),
        kind: kind.to_string(),
        elements: elements.into_iter().map(String::from).collect(),
        outcome: "ok".to_string(),
        summary: None,
    }
}

#[test]
fn test_set_recent_events() {
    let mut graph = KnowledgeGraph::new();
    let events = vec![
        test_event("drift_detected", vec!["api"]),
        test_event("intent_check", vec!["db"]),
    ];
    graph.set_recent_events(events);
    assert_eq!(graph.recent_events.len(), 2);
}

#[test]
fn test_get_events_for_node() {
    let mut graph = KnowledgeGraph::new();
    graph.set_recent_events(vec![
        test_event("drift_detected", vec!["api", "db"]),
        test_event("intent_check", vec!["db"]),
        test_event("sync", vec!["web"]),
    ]);

    let api_events = graph.get_events_for_node("api");
    assert_eq!(api_events.len(), 1);

    let db_events = graph.get_events_for_node("db");
    assert_eq!(db_events.len(), 2);
}

#[test]
fn test_get_events_for_cluster() {
    let mut graph = KnowledgeGraph::new();
    graph.set_recent_events(vec![
        test_event("drift_detected", vec!["api"]),
        test_event("intent_check", vec!["db"]),
        test_event("sync", vec!["web"]),
    ]);

    let cluster = vec!["api".to_string(), "db".to_string()];
    let events = graph.get_events_for_cluster(&cluster);
    assert_eq!(events.len(), 2);
}

#[test]
fn test_events_roundtrip_json() {
    let mut graph = KnowledgeGraph::new();
    graph.set_recent_events(vec![test_event("drift", vec!["api"])]);

    let json = graph.to_json().unwrap();
    let restored = KnowledgeGraph::from_json(&json).unwrap();
    assert_eq!(restored.recent_events.len(), 1);
    assert_eq!(restored.recent_events[0].kind, "drift");
}
