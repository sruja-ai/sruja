use std::fs;

#[test]
fn scan_npm_single_package() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    let manifest = root.join("package.json");
    fs::write(
        &manifest,
        r#"{"name":"my-app","version":"1.0.0","dependencies":{"lodash":"^4.0.0"}}"#,
    )
    .expect("write package.json");

    let graph = sruja_scan::scan_repo_manifests(root).expect("scan should succeed for npm repo");

    assert_eq!(graph.nodes.len(), 1, "single package => one node");
    assert_eq!(graph.nodes[0].id, "npm:my-app");
    assert_eq!(graph.nodes[0].label, "my-app");
    assert!(graph.edges.is_empty(), "no workspace deps => no edges");
}

#[test]
fn scan_npm_workspace() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();

    fs::write(
        root.join("package.json"),
        r#"{"name":"root","private":true,"workspaces":["packages/*"]}"#,
    )
    .expect("write root package.json");

    let pkg_a = root.join("packages").join("a");
    let pkg_b = root.join("packages").join("b");
    fs::create_dir_all(&pkg_a).expect("mkdir packages/a");
    fs::create_dir_all(&pkg_b).expect("mkdir packages/b");

    fs::write(
        pkg_a.join("package.json"),
        r#"{"name":"pkg-a","version":"1.0.0","dependencies":{"pkg-b":"*"}}"#,
    )
    .expect("write packages/a/package.json");
    fs::write(
        pkg_b.join("package.json"),
        r#"{"name":"pkg-b","version":"1.0.0"}"#,
    )
    .expect("write packages/b/package.json");

    let graph =
        sruja_scan::scan_repo_manifests(root).expect("scan should succeed for npm workspace");

    assert_eq!(graph.nodes.len(), 2, "workspace has two packages");
    assert!(
        graph.nodes.iter().any(|n| n.id == "npm:pkg-a"),
        "expected pkg-a node"
    );
    assert!(
        graph.nodes.iter().any(|n| n.id == "npm:pkg-b"),
        "expected pkg-b node"
    );
    assert_eq!(graph.edges.len(), 1, "pkg-a -> pkg-b");
    assert_eq!(graph.edges[0].source, "npm:pkg-a");
    assert_eq!(graph.edges[0].target, "npm:pkg-b");
}
