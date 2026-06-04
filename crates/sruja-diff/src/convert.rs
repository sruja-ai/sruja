//! Convert DSL program to scan graph for comparison.

use sruja_language::traversal::collect_elements;
use sruja_language::{ElementKind, Program};
use sruja_scan::{
    Edge, EdgeEvidence, EdgeKind, Graph, Node, NodeKind, ResolvedContract, ResolvedError,
    ResolvedField, ResolvedStateMachine, ResolvedTransition,
};
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

        let (
            canonical_id,
            aliases,
            owner,
            domain,
            criticality,
            sources,
            gotchas,
            constraints,
            runbooks,
            state_machines,
            contracts,
        ) = if let Some(body) = &a.body {
            (
                body.canonical_id.clone(),
                body.aliases.clone(),
                body.owner.clone(),
                body.domain.clone(),
                body.criticality,
                body.sources.clone(),
                body.gotchas.clone(),
                body.operational_constraints.clone(),
                body.runbooks.clone(),
                body.state_machines
                    .iter()
                    .map(convert_state_machine)
                    .collect(),
                body.contracts.iter().map(convert_contract).collect(),
            )
        } else {
            (
                None,
                Vec::new(),
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
        };

        nodes.push(Node {
            id: fqn.clone(),
            kind,
            label,
            technology,
            path: None,
            metadata: HashMap::new(),
            canonical_id,
            aliases,
            owner,
            domain,
            criticality,
            sources,
            gotchas,
            operational_constraints: constraints,
            runbooks,
            confidence: None,
            state_machines,
            contracts,
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
                confidence: Default::default(),
            });
        }
    }

    let mut incidents = Vec::new();
    for item in &program.items {
        if let sruja_language::TopLevelItem::Incident(inc) = item {
            incidents.push(sruja_scan::Incident {
                id: inc.id.clone(),
                title: inc.title.clone(),
                date: inc.date.clone(),
                severity: inc.severity.clone(),
                affected: inc.affected.iter().map(|id| id.as_string()).collect(),
                cause: inc.cause.clone(),
                resolution: inc.resolution.clone(),
                lesson: inc.lesson.clone(),
            });
        }
    }

    Graph {
        metadata: HashMap::new(),
        nodes,
        edges,
        incidents,
        confidence: None,
        auto_context: Default::default(),
    }
}

fn element_kind_to_node_kind(kind: &ElementKind) -> NodeKind {
    match kind {
        ElementKind::Database | ElementKind::DataStore => NodeKind::new(NodeKind::DATABASE),
        ElementKind::ExternalSystem => NodeKind::new(NodeKind::EXTERNAL_API),
        ElementKind::Custom(ref s) => NodeKind::new(s.clone()),
        _ => NodeKind::new(NodeKind::MODULE),
    }
}

fn convert_state_machine(sm: &sruja_language::StateMachine) -> ResolvedStateMachine {
    let mut states_set = std::collections::HashSet::new();
    states_set.insert(sm.initial_state.clone());
    for s in &sm.terminal_states {
        states_set.insert(s.clone());
    }
    for t in &sm.transitions {
        states_set.insert(t.from.clone());
        states_set.insert(t.to.clone());
    }

    let mut states: Vec<_> = states_set.into_iter().collect();
    states.sort();

    ResolvedStateMachine {
        name: sm.name.clone(),
        states,
        initial_state: sm.initial_state.clone(),
        terminal_states: sm.terminal_states.clone(),
        transitions: sm
            .transitions
            .iter()
            .map(|t| ResolvedTransition {
                from: t.from.clone(),
                to: t.to.clone(),
                event: t.event.clone(),
                guard: t.guard.clone(),
                action: t.action.clone(),
            })
            .collect(),
    }
}

fn convert_contract(c: &sruja_language::Contract) -> ResolvedContract {
    ResolvedContract {
        name: c.name.clone(),
        description: c.description.clone(),
        inputs: c
            .inputs
            .iter()
            .map(|f| ResolvedField {
                name: f.name.clone(),
                spec: f.spec.clone(),
            })
            .collect(),
        outputs: c
            .outputs
            .iter()
            .map(|f| ResolvedField {
                name: f.name.clone(),
                spec: f.spec.clone(),
            })
            .collect(),
        errors: c
            .errors
            .iter()
            .map(|e| ResolvedError {
                code: e.code.clone(),
                description: e.description.clone(),
            })
            .collect(),
        constraints: c.constraints.clone(),
    }
}

fn relation_label_to_edge_kind(label: &str) -> EdgeKind {
    let lower = label.to_lowercase();
    if lower.contains("read") || lower == "reads" {
        EdgeKind::new(EdgeKind::READS_FROM)
    } else if lower.contains("write") || lower == "writes" {
        EdgeKind::new(EdgeKind::WRITES_TO)
    } else if lower.contains("depend") {
        EdgeKind::new(EdgeKind::DEPENDS_ON)
    } else if lower.contains("publish") {
        EdgeKind::new(EdgeKind::PUBLISHES_TO)
    } else if lower.contains("subscrib") {
        EdgeKind::new(EdgeKind::SUBSCRIBES_TO)
    } else if lower.contains("own") {
        EdgeKind::new(EdgeKind::OWNS)
    } else if lower.contains("contain") {
        EdgeKind::new(EdgeKind::CONTAINS)
    } else if lower.contains("use") {
        EdgeKind::new(EdgeKind::USES)
    } else {
        EdgeKind::new(EdgeKind::CALLS)
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
        assert_eq!(graph.edges[0].kind, EdgeKind::WRITES_TO);
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
        assert_eq!(graph.edges[0].kind, EdgeKind::DEPENDS_ON);
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
        assert_eq!(graph.edges[0].kind, EdgeKind::READS_FROM);
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
        assert_eq!(graph.edges[0].kind, EdgeKind::PUBLISHES_TO);
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
        assert_eq!(graph.edges[0].kind, EdgeKind::SUBSCRIBES_TO);
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
        assert_eq!(graph.edges[0].kind, EdgeKind::OWNS);
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
        assert_eq!(graph.edges[0].kind, EdgeKind::CONTAINS);
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
        assert_eq!(graph.edges[0].kind, EdgeKind::USES);
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
        assert_eq!(graph.edges[0].kind, EdgeKind::CALLS);
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
        assert_eq!(db.kind, NodeKind::DATABASE);
    }

    #[test]
    fn program_to_graph_single_node() {
        let program = parse_dsl(r#"S = system "My System" {}"#);
        let graph = program_to_graph(&program);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id, "S");
        assert_eq!(graph.nodes[0].kind, NodeKind::MODULE);
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn program_to_graph_multiple_nodes_and_edges() {
        let program = parse_dsl(
            r#"
A = system "A"
B = system "B"
C = system "C"
A -> B "calls"
A -> C "uses"
B -> C "depends on"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 3);
    }

    #[test]
    fn program_to_graph_person_becomes_module() {
        let program = parse_dsl(
            r#"
User = person "End User" {
  description "Application user"
}
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].kind, NodeKind::MODULE);
    }
}
