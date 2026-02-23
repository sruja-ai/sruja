//! E2E tests for `sruja why` - architecture intelligence layer.
//!
//! Exercises the same flow as `sruja why`: scan repo → merge into KnowledgeGraph → query.

use std::fs;
use tempfile::TempDir;

fn create_test_repo_with_tech() -> TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path();

    fs::create_dir_all(root.join("src")).ok();
    fs::write(
        root.join("src/api.ts"),
        r#"
import { db } from './db';
export async function getData() {
  return db.query('SELECT 1');
}
"#,
    )
    .expect("write api.ts");
    fs::write(
        root.join("src/db.ts"),
        r#"
// PostgreSQL database
export function query(sql: string) { return []; }
"#,
    )
    .expect("write db.ts");

    dir
}

fn merge_scan_into_graph(
    graph: &mut sruja_graph::KnowledgeGraph,
    scan_graph: &sruja_scan::Graph,
    repo_path: &str,
) {
    use chrono::Utc;
    use sruja_graph::SourceReference;
    use sruja_scan::{EdgeKind as ScanEdgeKind, NodeKind as ScanNodeKind};

    let now = Utc::now();
    let source = SourceReference::scanned_repo(repo_path);

    for node in &scan_graph.nodes {
        let kind = match node.kind {
            ScanNodeKind::Service => sruja_graph::NodeKind::Service,
            ScanNodeKind::Module => sruja_graph::NodeKind::Module,
            ScanNodeKind::Database => sruja_graph::NodeKind::Database,
            ScanNodeKind::ExternalApi => sruja_graph::NodeKind::ExternalApi,
        };
        let arch_node = sruja_graph::ArchitectureNode {
            id: node.id.clone(),
            kind,
            label: node.label.clone(),
            technology: node.technology.clone(),
            description: node.path.clone(),
            metadata: node.metadata.clone(),
            source: source.clone(),
            created_at: now,
            updated_at: now,
        };
        graph.merge_node(arch_node);
    }

    for edge in &scan_graph.edges {
        let kind = match edge.kind {
            ScanEdgeKind::Calls => sruja_graph::EdgeKind::Calls,
            ScanEdgeKind::ReadsFrom => sruja_graph::EdgeKind::ReadsFrom,
            ScanEdgeKind::WritesTo => sruja_graph::EdgeKind::WritesTo,
        };
        let edge_id = format!("{}-{}-{:?}", edge.source, edge.target, edge.kind);
        let arch_edge = sruja_graph::ArchitectureEdge {
            id: edge_id,
            source: edge.source.clone(),
            target: edge.target.clone(),
            kind,
            label: None,
            description: None,
            source_ref: source.clone(),
        };
        graph.merge_edge(arch_edge);
    }
}

#[test]
fn why_flow_scan_merge_query() {
    let repo = create_test_repo_with_tech();
    let mut kg = sruja_graph::KnowledgeGraph::new();

    let scan_graph = sruja_scan::scan_repo(repo.path()).expect("scan");
    merge_scan_into_graph(&mut kg, &scan_graph, repo.path().to_str().unwrap());

    let result = kg.query("what services do we have?").expect("query");
    assert!(!result.answer.is_empty());
    assert!(result.confidence > 0.0);
}

#[test]
fn why_flow_connectivity_query() {
    let repo = create_test_repo_with_tech();
    let mut kg = sruja_graph::KnowledgeGraph::new();

    let scan_graph = sruja_scan::scan_repo(repo.path()).expect("scan");
    merge_scan_into_graph(&mut kg, &scan_graph, repo.path().to_str().unwrap());

    let result = kg.query("how do components depend?").expect("query");
    assert!(!result.answer.is_empty());
}
