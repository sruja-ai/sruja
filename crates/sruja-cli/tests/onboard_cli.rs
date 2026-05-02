mod common;

use common::{create_test_repo, run_sruja, write_file};

#[test]
fn onboard_markdown_smoke() {
    let repo = create_test_repo();
    write_file(repo.path(), "src/main.rs", "fn main() {}\n");

    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) = run_sruja(&["onboard", "-r", repo_str]);
    assert!(ok, "onboard should succeed: stderr={}", stderr);
    assert!(stdout.contains("# Sruja Onboarding Brief"));
    assert!(stdout.contains("Entrypoints"));
    assert!(stdout.contains("High-Signal Elements"));
    assert!(stdout.contains("Suggested Commands"));
}

#[test]
fn onboard_json_smoke() {
    let repo = create_test_repo();
    write_file(repo.path(), "src/main.rs", "fn main() {}\n");

    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) = run_sruja(&["onboard", "-r", repo_str, "-f", "json"]);
    assert!(ok, "onboard json should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert!(parsed.get("repo").is_some());
    assert!(parsed.get("context_score").is_some());
    assert!(parsed.get("entrypoints").is_some());
    assert!(parsed.get("suggested_commands").is_some());
}

#[test]
fn onboard_github_actions_smoke() {
    let repo = create_test_repo();
    write_file(repo.path(), "src/main.rs", "fn main() {}\n");

    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) = run_sruja(&["onboard", "-r", repo_str, "-f", "github-actions"]);
    assert!(
        ok,
        "onboard github-actions should succeed: stderr={}",
        stderr
    );
    assert!(stdout.contains("::notice"));
    assert!(stdout.contains("Sruja Onboard"));
}

#[test]
fn onboard_is_deterministic_for_same_repo() {
    let repo = create_test_repo();
    write_file(repo.path(), "src/lib.rs", "pub fn a() {}\n");
    write_file(repo.path(), "src/mod.rs", "pub mod lib;\n");

    let repo_str = repo.path().to_str().expect("utf-8");
    let args = ["onboard", "-r", repo_str, "-f", "json", "--max-items", "8"];

    let (ok1, stdout1, stderr1) = run_sruja(&args);
    assert!(ok1, "first run should succeed: stderr={}", stderr1);
    let (ok2, stdout2, stderr2) = run_sruja(&args);
    assert!(ok2, "second run should succeed: stderr={}", stderr2);
    assert_eq!(stdout1.trim(), stdout2.trim());
}

#[test]
fn onboard_enrich_cmd_populates_enrichment_field() {
    let repo = create_test_repo();
    write_file(repo.path(), "src/main.rs", "fn main() {}\n");

    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) = run_sruja(&[
        "onboard",
        "-r",
        repo_str,
        "-f",
        "json",
        "--enrich-cmd",
        "cat >/dev/null; echo 'ENRICHED: ok'",
    ]);
    assert!(ok, "onboard enrich-cmd should succeed: stderr={}", stderr);

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid json");
    let enrichment = parsed.get("enrichment").expect("enrichment field");
    assert_eq!(
        enrichment.get("status").and_then(|v| v.as_str()),
        Some("ok")
    );
    assert_eq!(
        enrichment.get("provider").and_then(|v| v.as_str()),
        Some("external_cmd")
    );
    assert!(
        enrichment
            .get("narrative_markdown")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .contains("ENRICHED: ok"),
        "expected narrative_markdown to contain enrichment output"
    );
}

#[test]
fn onboard_enrich_uses_repo_config_when_cmd_not_provided() {
    let repo = create_test_repo();
    write_file(repo.path(), "src/main.rs", "fn main() {}\n");
    write_file(
        repo.path(),
        ".sruja/config.toml",
        r#"
[integrations]
default_provider = "cmd"
cmd = "cat >/dev/null; echo 'ENRICHED_FROM_CONFIG'"
timeout_ms = 15000
max_bytes = 20000
"#,
    );

    let repo_str = repo.path().to_str().expect("utf-8");
    let (ok, stdout, stderr) = run_sruja(&["onboard", "-r", repo_str, "-f", "json", "--enrich"]);
    assert!(ok, "onboard enrich should succeed: stderr={}", stderr);

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid json");
    let enrichment = parsed.get("enrichment").expect("enrichment field");
    assert_eq!(
        enrichment.get("status").and_then(|v| v.as_str()),
        Some("ok")
    );
    assert!(
        enrichment
            .get("narrative_markdown")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .contains("ENRICHED_FROM_CONFIG"),
        "expected narrative_markdown to include config-driven output"
    );
}
