mod common;
use common::{create_test_repo, write_file, run_sruja};

#[test]
fn discover_repomap_prints_repository_map() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/index.ts",
        r#"
export function main() { return 42; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) =
        run_sruja(&["discover", "--repomap", "-r", repo_str, "--max-files", "10", "--max-tokens", "1000"]);
    assert!(ok, "discover --repomap should succeed: stderr={}", stderr);
    assert!(
        stdout.contains("# Repository Map"),
        "repomap should include header"
    );
}
