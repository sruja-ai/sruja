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
    } else {
        EdgeKind::Calls
    }
}
