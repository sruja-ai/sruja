//! Convert DSL program to scan graph for comparison.

use sruja_language::traversal::collect_elements;
use sruja_language::{ElementKind, Program};
use sruja_scan::{Edge, EdgeEvidence, EdgeKind, Graph, Node, NodeKind};
use std::collections::HashMap;

/// Convert a DSL Program to sruja_scan::Graph for comparison with scanned architecture.
pub fn program_to_graph(program: &Program) -> Graph {
    let (elements, relations) = collect_elements(program);
    let mut nodes = Vec::with_capacity(elements.len());
    let mut edges = Vec::with_capacity(relations.len());

    for (fqn, elem) in &elements {
        let a = &elem.assignment;
        let kind = element_kind_to_node_kind(&a.kind);
        let label = a.title.as_deref().unwrap_or(&a.name).to_string();
        let technology = a.body.as_ref().and_then(|b| b.technology.as_ref()).cloned();

        nodes.push(Node {
            id: fqn.clone(),
            kind,
            label,
            technology,
            path: None,
            metadata: HashMap::new(),
        });
    }

    for rel in &relations {
        let from_id = rel.from.as_string();
        let to_id = rel.to.as_string();
        if elements.contains_key(&from_id) && elements.contains_key(&to_id) {
            let kind = relation_label_to_edge_kind(rel.label.as_deref().unwrap_or("calls"));
            edges.push(Edge {
                source: from_id,
                target: to_id,
                kind,
                evidence: vec![EdgeEvidence {
                    rule: "dsl".to_string(),
                    file: None,
                    line: None,
                    detail: rel.label.clone(),
                }],
            });
        }
    }

    Graph {
        metadata: HashMap::new(),
        nodes,
        edges,
    }
}

fn element_kind_to_node_kind(kind: &ElementKind) -> NodeKind {
    match kind {
        ElementKind::Database | ElementKind::DataStore => NodeKind::Database,
        ElementKind::ExternalSystem => NodeKind::ExternalApi,
        ElementKind::Person
        | ElementKind::Role
        | ElementKind::System
        | ElementKind::Container
        | ElementKind::Component
        | ElementKind::Queue
        | ElementKind::Policy
        | ElementKind::Requirement
        | ElementKind::Adr
        | ElementKind::Flow
        | ElementKind::Scenario
        | ElementKind::Story
        | ElementKind::Custom(_) => NodeKind::Module,
    }
}

fn relation_label_to_edge_kind(label: &str) -> EdgeKind {
    let lower = label.to_lowercase();
    if lower.contains("read") || lower == "reads" {
        EdgeKind::ReadsFrom
    } else if lower.contains("write") || lower == "writes" {
        EdgeKind::WritesTo
    } else if lower.contains("depend") {
        EdgeKind::DependsOn
    } else if lower.contains("publish") {
        EdgeKind::PublishesTo
    } else if lower.contains("subscrib") {
        EdgeKind::SubscribesTo
    } else if lower.contains("own") {
        EdgeKind::Owns
    } else if lower.contains("contain") {
        EdgeKind::Contains
    } else if lower.contains("use") {
        EdgeKind::Uses
    } else {
        EdgeKind::Calls
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_language::Parser;

    fn parse_dsl(input: &str) -> Program {
        Parser::new("test.sruja".to_string())
            .parse(input)
            .expect("parse failed")
    }

    #[test]
    fn program_to_graph_empty_program_produces_empty_graph() {
        let program = Program::default();
        let graph = program_to_graph(&program);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn program_to_graph_writes_label_maps_to_writes_to() {
        let program = parse_dsl(
            r#"
A = system "Service A"
DB = database "Database"
A -> DB "writes to"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].kind, EdgeKind::WritesTo);
    }

    #[test]
    fn program_to_graph_depends_label_maps_to_depends_on() {
        let program = parse_dsl(
            r#"
A = system "Service A"
B = system "Service B"
A -> B "depends on"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].kind, EdgeKind::DependsOn);
    }

    #[test]
    fn program_to_graph_reads_label_maps_to_reads_from() {
        let program = parse_dsl(
            r#"
A = system "A"
DB = database "DB"
A -> DB "reads"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].kind, EdgeKind::ReadsFrom);
    }

    #[test]
    fn program_to_graph_publishes_label_maps_to_publishes_to() {
        let program = parse_dsl(
            r#"
A = system "A"
B = system "B"
A -> B "publishes"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].kind, EdgeKind::PublishesTo);
    }

    #[test]
    fn program_to_graph_subscribes_label_maps_to_subscribes_to() {
        let program = parse_dsl(
            r#"
A = system "A"
B = system "B"
A -> B "subscribes"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].kind, EdgeKind::SubscribesTo);
    }

    #[test]
    fn program_to_graph_owns_label_maps_to_owns() {
        let program = parse_dsl(
            r#"
A = system "A"
DB = database "DB"
A -> DB "owns"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].kind, EdgeKind::Owns);
    }

    #[test]
    fn program_to_graph_contains_label_maps_to_contains() {
        let program = parse_dsl(
            r#"
A = system "A"
B = system "B"
A -> B "contains"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].kind, EdgeKind::Contains);
    }

    #[test]
    fn program_to_graph_uses_label_maps_to_uses() {
        let program = parse_dsl(
            r#"
A = system "A"
B = system "B"
A -> B "uses"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].kind, EdgeKind::Uses);
    }

    #[test]
    fn program_to_graph_unknown_label_defaults_to_calls() {
        let program = parse_dsl(
            r#"
A = system "A"
B = system "B"
A -> B "HTTPS"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].kind, EdgeKind::Calls);
    }

    #[test]
    fn program_to_graph_ignores_relations_to_unknown_elements() {
        let program = parse_dsl(
            r#"
A = system "A"
A -> Missing "calls"
"#,
        );
        let graph = program_to_graph(&program);
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn program_to_graph_edge_includes_dsl_evidence_and_label_detail() {
        let program = parse_dsl(
            r#"
A = system "A"
B = system "B"
A -> B "writes"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.edges.len(), 1);
        let edge = &graph.edges[0];
        assert_eq!(edge.evidence.len(), 1);
        assert_eq!(edge.evidence[0].rule, "dsl");
        assert_eq!(edge.evidence[0].detail, Some("writes".to_string()));
    }

    #[test]
    fn program_to_graph_maps_database_kind_to_database_node_kind() {
        let program = parse_dsl(
            r#"
S = system "S" {
  description "S"
  DB = database "DB" { description "db" }
}
"#,
        );
        let graph = program_to_graph(&program);
        let db = graph.nodes.iter().find(|n| n.id == "S.DB").expect("db");
        assert_eq!(db.kind, NodeKind::Database);
    }

    #[test]
    fn program_to_graph_single_node() {
        let program = parse_dsl(r#"S = system "My System" {}"#);
        let graph = program_to_graph(&program);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id, "S");
        assert_eq!(graph.nodes[0].kind, NodeKind::Module);
        assert!(graph.edges.is_empty());
    }
}
