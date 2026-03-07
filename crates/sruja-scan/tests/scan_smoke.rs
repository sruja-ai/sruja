use std::path::PathBuf;

#[test]
fn scan_workspace_smoke_test() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("expected crates/sruja-scan under repo root")
        .to_path_buf();

    let graph = sruja_scan::scan_repo(&repo_root).expect("scan should succeed for cargo repo");

    assert!(!graph.nodes.is_empty(), "expected at least one node");
    // This repo is a workspace; expect some internal edges.
    assert!(
        !graph.edges.is_empty(),
        "expected at least one edge in workspace graph"
    );

    let json = serde_json::to_string(&graph).expect("graph should serialize");
    assert!(json.contains("\"nodes\""));
    assert!(json.contains("\"edges\""));
}
