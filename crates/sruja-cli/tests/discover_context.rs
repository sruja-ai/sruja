mod common;
use common::{create_test_repo, write_file, run_sruja};

#[test]
fn discover_context_text_contains_header() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 1; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) = run_sruja(&["discover", "--context", "-r", repo_str]);
    assert!(ok, "discover --context should succeed: stderr={}", stderr);
    assert!(
        stdout.contains("# Repo context (for contextual discovery questions)"),
        "discover --context should include header"
    );
}
