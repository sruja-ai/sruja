mod common;
use common::{create_test_repo, run_sruja, write_file};

const MINIMAL_VALID_SRUJA: &str = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User" {
  description "End user"
}

App = system "My App" {
  description "Main application"

  Web = container "Web" {
    technology "React"
    description "UI"
  }
}
User -> App "uses"
"#;

fn write_minimal_cargo_repo(repo_root: &std::path::Path) {
    write_file(
        repo_root,
        "Cargo.toml",
        r#"[package]
name = "dummy"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(repo_root, "src/lib.rs", "pub fn foo() {}");
}

#[test]
fn discover_explain_text_highlights_reasoning() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "package.json",
        r#"{"dependencies":{"express":"4.18.0"}}"#,
    );
    write_file(
        repo.path(),
        "src/server.ts",
        r#"
import { query } from "./db";
export function start() { return query(); }
"#,
    );
    write_file(
        repo.path(),
        "src/db.ts",
        r#"export function query() { return []; }"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&["discover", "--explain", "-r", repo_str]);

    assert!(
        success,
        "discover --explain should succeed: stderr={}",
        stderr
    );
    assert!(stdout.contains("# Sruja Discovery Explanation"));
    assert!(stdout.contains("Why Sruja Thinks That"));
    assert!(stdout.contains("Next Steps"));
}

#[test]
fn discover_subcommands_cover_context_repomap_questions_and_enrichment() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "package.json",
        r#"{"name":"fixture","version":"0.1.0","dependencies":{"express":"4.18.0"}}"#,
    );
    write_file(
        repo.path(),
        "src/server.ts",
        r#"
import { query } from "./db";
export function start() { return query(); }
"#,
    );
    write_file(
        repo.path(),
        "src/db.ts",
        r#"export function query() { return []; }"#,
    );

    let repo_str = repo.path().to_str().expect("utf-8");

    let (ctx_ok, ctx_out, ctx_err) =
        run_sruja(&["discover", "-r", repo_str, "--format", "json", "context"]);
    assert!(ctx_ok, "discover context should succeed: stderr={ctx_err}");
    assert!(
        !ctx_out.trim().is_empty(),
        "discover context should produce output"
    );
    let ctx_trim = ctx_out.trim_start();
    if ctx_trim.starts_with('{') || ctx_trim.starts_with('[') {
        let _parsed: serde_json::Value =
            serde_json::from_str(ctx_out.trim()).expect("discover context JSON");
    } else {
        assert!(
            ctx_out.contains("Context") || ctx_out.contains("Repo") || ctx_out.contains("Sruja"),
            "discover context should look like a summary: stdout={ctx_out}"
        );
    }

    let (rm_ok, rm_out, rm_err) = run_sruja(&[
        "discover",
        "-r",
        repo_str,
        "--max-files",
        "25",
        "--max-tokens",
        "1200",
        "--update",
        "repomap",
    ]);
    assert!(rm_ok, "discover repomap should succeed: stderr={rm_err}");
    assert!(
        rm_out.contains("# Sruja Repomap")
            || rm_out.contains("# Repository Map")
            || rm_out.contains("Repomap"),
        "expected repomap header: stdout={rm_out}"
    );

    let (q_ok, q_out, q_err) =
        run_sruja(&["discover", "-r", repo_str, "--format", "json", "questions"]);
    assert!(q_ok, "discover questions should succeed: stderr={q_err}");
    assert!(
        !q_out.trim().is_empty(),
        "discover questions should produce output"
    );
    let q_trim = q_out.trim_start();
    if q_trim.starts_with('{') || q_trim.starts_with('[') {
        let questions: serde_json::Value =
            serde_json::from_str(q_out.trim()).expect("discover questions JSON");
        assert!(
            questions.as_array().is_some() || questions.get("questions").is_some(),
            "expected questions payload: {q_out}"
        );
    } else {
        assert!(
            q_out.to_lowercase().contains("question"),
            "discover questions should include a header: stdout={q_out}"
        );
    }

    let report_path = repo.path().join("GRAPH_REPORT.md");
    let report_str = report_path.to_str().expect("utf-8");
    let (ex_ok, ex_out, ex_err) = run_sruja(&[
        "discover",
        "-r",
        repo_str,
        "--format",
        "json",
        "--enrich",
        "--enrich-provider",
        "cmd",
        "--enrich-cmd",
        "cat",
        "--export-report",
        report_str,
        "explain",
    ]);
    assert!(ex_ok, "discover explain should succeed: stderr={ex_err}");
    assert!(report_path.exists(), "expected report file to be written");
    let combined = format!("{}{}", ex_out, ex_err);
    if !combined.trim().is_empty() {
        let trim = combined.trim_start();
        if trim.starts_with('{') || trim.starts_with('[') {
            let _parsed: serde_json::Value =
                serde_json::from_str(trim).expect("discover explain JSON");
        }
    }
}

#[test]
fn daily_alias_refreshes_context_and_prints_next_steps() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
import { helper } from "./helper";
export function app() { return helper(); }
"#,
    );
    write_file(
        repo.path(),
        "src/helper.ts",
        r#"export function helper() { return "ok"; }"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&["daily", "-r", repo_str]);

    assert!(success, "daily should succeed: stderr={}", stderr);

    let out = format!("{} {}", stdout, stderr);
    assert!(
        out.contains("Top Actions:") || out.contains("Recommended Actions:"),
        "daily output should include next actions. stdout={} stderr={}",
        stdout,
        stderr
    );
    assert!(
        out.contains("sruja start -r") || out.contains("sruja watch -r"),
        "daily output should include workflow guidance. stdout={} stderr={}",
        stdout,
        stderr
    );

    assert!(
        repo.path().join(".sruja/context.json").exists(),
        "daily should refresh .sruja/context.json"
    );
    assert!(
        repo.path().join(".sruja/cache/scan.json").exists(),
        "daily should refresh .sruja/cache/scan.json"
    );
}

#[test]
fn quickstart_generates_baseline_and_emits_json() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(
        repo.path(),
        "src/main.rs",
        "fn main() { println!(\"hi\"); }\n",
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "quickstart",
        "-r",
        repo_str,
        "-f",
        "json",
        "--generate-baseline",
        "--advisory",
    ]);
    assert!(ok, "quickstart should succeed: stderr={stderr}");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(
        parsed.get("repo").is_some()
            && parsed.get("health_score").is_some()
            && parsed.get("inventory").is_some(),
        "expected quickstart json payload: stdout={stdout}"
    );
    assert!(
        repo.path().join("repo.sruja.draft").exists(),
        "quickstart --generate-baseline should write repo.sruja.draft"
    );
}

#[test]
fn scan_succeeds_on_repo_with_cargo_toml() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let path_str = repo.path().to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&["scan", path_str, "--output", "-"]);

    assert!(
        success,
        "scan should succeed on Cargo repo: stderr={}",
        stderr
    );
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("scan outputs JSON");
    assert!(
        parsed.get("nodes").is_some()
            || parsed.get("elements").is_some()
            || stdout.contains("\"nodes\""),
        "scan output should contain graph structure"
    );
}

#[test]
fn learn_json_labels_hypothesis_artifact() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (init_ok, _, init_stderr) = run_sruja(&["init", "-r", repo_str]);
    assert!(init_ok, "init should succeed: stderr={}", init_stderr);

    let (success, stdout, stderr) =
        run_sruja(&["learn", "-r", repo_str, "-f", "json", "--skip-proposals"]);
    assert!(success, "learn should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(
        parsed.get("artifact_kind").and_then(|v| v.as_str()),
        Some("learned_hypothesis")
    );
    assert!(parsed.get("metric_description").is_some());
    assert!(parsed.get("fact_count").is_some());
}

#[test]
fn context_multi_repo_json_includes_combined_summary_and_repos() {
    let repo_a = create_test_repo();
    write_minimal_cargo_repo(repo_a.path());
    let repo_a_str = repo_a.path().to_str().expect("utf-8");

    let repo_b = create_test_repo();
    write_minimal_cargo_repo(repo_b.path());
    let repo_b_str = repo_b.path().to_str().expect("utf-8");

    let (success, stdout, stderr) = run_sruja(&[
        "ai-context",
        "-r",
        repo_a_str,
        "-r",
        repo_b_str,
        "-f",
        "json",
    ]);

    assert!(
        success,
        "context multi-repo should succeed: stderr={}",
        stderr
    );
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");

    let repos = parsed
        .get("repos")
        .and_then(|v| v.as_array())
        .expect("repos must be array");
    assert_eq!(repos.len(), 2, "expected 2 repos in context output");

    let combined = parsed
        .get("combined_summary")
        .expect("combined_summary must exist");
    assert!(
        combined
            .get("total_modules")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0,
        "combined summary should include modules"
    );
}

#[test]
fn ai_context_cursor_rules_includes_header() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "ai-context",
        "-r",
        repo_str,
        "-f",
        "cursor-rules",
        "--max-tokens",
        "2000",
    ]);
    assert!(
        ok,
        "ai-context -f cursor-rules should succeed: stderr={stderr}"
    );
    assert!(
        stdout.contains("# Sruja Architecture Context"),
        "expected cursor-rules header: stdout={stdout}"
    );
}

#[test]
fn ai_context_multi_repo_cursor_rules_includes_combined_summary() {
    let repo_a = create_test_repo();
    write_minimal_cargo_repo(repo_a.path());
    let repo_a_str = repo_a.path().to_str().expect("utf-8");

    let repo_b = create_test_repo();
    write_minimal_cargo_repo(repo_b.path());
    let repo_b_str = repo_b.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "ai-context",
        "-r",
        repo_a_str,
        "-r",
        repo_b_str,
        "-f",
        "cursor-rules",
        "--max-tokens",
        "2500",
    ]);
    assert!(
        ok,
        "ai-context multi -f cursor-rules should succeed: stderr={stderr}"
    );
    assert!(
        stdout.contains("Multi-Repo") && stdout.contains("Combined Summary"),
        "expected multi-repo cursor-rules output: stdout={stdout}"
    );
}

#[test]
fn ai_context_multi_repo_copilot_instructions_includes_header() {
    let repo_a = create_test_repo();
    write_minimal_cargo_repo(repo_a.path());
    let repo_a_str = repo_a.path().to_str().expect("utf-8");

    let repo_b = create_test_repo();
    write_minimal_cargo_repo(repo_b.path());
    let repo_b_str = repo_b.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "ai-context",
        "-r",
        repo_a_str,
        "-r",
        repo_b_str,
        "-f",
        "copilot-instructions",
        "--max-tokens",
        "2500",
    ]);
    assert!(
        ok,
        "ai-context multi -f copilot-instructions should succeed: stderr={stderr}"
    );
    assert!(
        stdout.contains("GitHub Copilot") && stdout.contains("Combined Summary"),
        "expected copilot instructions output: stdout={stdout}"
    );
}

#[test]
fn ai_context_markdown_and_repomap_formats_succeed() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/index.ts",
        r#"
import { helper } from "./helper";
export function main() { return helper(); }
"#,
    );
    write_file(
        repo.path(),
        "src/helper.ts",
        r#"
export function helper() { return 1; }
"#,
    );
    write_file(
        repo.path(),
        "package.json",
        r#"{"name":"fixture","version":"0.1.0"}"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (md_ok, md_out, md_err) = run_sruja(&[
        "ai-context",
        "-r",
        repo_str,
        "-f",
        "markdown",
        "--max-tokens",
        "2500",
    ]);
    assert!(
        md_ok,
        "ai-context -f markdown should succeed: stderr={md_err}"
    );
    assert!(
        md_out.contains("# Architecture Context"),
        "expected markdown header: stdout={md_out}"
    );

    let (rm_ok, rm_out, rm_err) = run_sruja(&[
        "ai-context",
        "-r",
        repo_str,
        "-f",
        "repomap",
        "--max-tokens",
        "1500",
    ]);
    assert!(
        rm_ok,
        "ai-context -f repomap should succeed: stderr={rm_err}"
    );
    assert!(
        rm_out.contains("# Sruja Repomap"),
        "expected repomap header: stdout={rm_out}"
    );
}

#[test]
fn context_score_json_reports_breakdown() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    run_sruja(&["sync", "-r", repo_str]);

    let (success, stdout, stderr) = run_sruja(&["context-score", "-r", repo_str, "-f", "json"]);

    assert!(success, "context-score should succeed: stderr={}", stderr);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(parsed.get("score").is_some());
    assert!(parsed.get("architecture_coverage").is_some());
    assert!(parsed.get("quick_wins").is_some());
    assert_eq!(
        parsed.get("metric_type").and_then(|v| v.as_str()),
        Some("ai_readiness")
    );
    assert!(parsed.get("metric_description").is_some());
}

#[test]
fn index_embeddings_succeeds() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");
    let arch_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();
    let out_path = repo.path().join("vectors.json");
    let out_str = out_path.to_str().expect("utf-8");

    let (success, _stdout, stderr) = run_sruja(&[
        "index", "semantic", "-r", repo_str, "-a", &arch_str, "-o", out_str,
    ]);

    assert!(success, "index should succeed: stderr={}", stderr);
    assert!(out_path.exists(), "vectors.json should be created");
}
