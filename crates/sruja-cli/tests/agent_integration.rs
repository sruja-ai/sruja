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
fn ai_output_includes_task_context_section() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 1; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "ai",
        "-r",
        repo_str,
        "--file",
        "src/app.ts",
        "--max-tokens",
        "500",
    ]);
    assert!(ok, "ai should succeed: stderr={stderr}");
    assert!(
        stdout.contains("## Task Context"),
        "expected task context section: stdout={stdout}"
    );
}

#[test]
fn ai_writes_output_file() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 1; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");
    let out_path = repo.path().join("ai.txt");
    let out_str = out_path.to_str().expect("utf-8");

    let (ok, _stdout, stderr) = run_sruja(&[
        "ai",
        "-r",
        repo_str,
        "--file",
        "src/app.ts",
        "--max-tokens",
        "500",
        "-o",
        out_str,
    ]);
    assert!(ok, "ai -o should succeed: stderr={stderr}");
    assert!(out_path.exists(), "expected output file to be written");
}

#[test]
fn agent_run_plan_json_succeeds() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "agent",
        "run",
        "-r",
        repo_str,
        "--goal",
        "test goal",
        "--file",
        "src/lib.rs",
        "--mode",
        "plan",
        "-f",
        "json",
    ]);
    assert!(ok, "agent run --mode plan should succeed: stderr={stderr}");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert!(
        parsed.get("plan").is_some() || stdout.contains("agent_plan_output/v1"),
        "expected agent run output to include plan: stdout={stdout}"
    );
}

#[test]
fn agent_record_and_history_roundtrip_json() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (record_ok, _, record_err) = run_sruja(&[
        "agent",
        "record",
        "-r",
        repo_str,
        "-c",
        "test context",
        "-H",
        "test hypothesis",
        "-o",
        "failed",
        "-g",
        "test guardrail",
        "-s",
        "test reason",
    ]);
    assert!(
        record_ok,
        "agent record should succeed: stderr={record_err}"
    );

    let (history_ok, stdout, history_err) =
        run_sruja(&["agent", "history", "-r", repo_str, "-f", "json"]);
    assert!(
        history_ok,
        "agent history -f json should succeed: stderr={history_err}"
    );
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    let entries = parsed.as_array().cloned().unwrap_or_default();
    assert!(!entries.is_empty(), "expected at least one entry");
}

#[test]
fn agent_crud_commands_succeed() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    run_sruja(&[
        "agent",
        "record",
        "-r",
        repo_str,
        "-c",
        "ctx 1",
        "-H",
        "hyp 1",
        "-o",
        "failed",
        "-g",
        "guardrail 1",
    ]);
    run_sruja(&[
        "agent",
        "record",
        "-r",
        repo_str,
        "-c",
        "ctx 2",
        "-H",
        "hyp 2",
        "-o",
        "failed",
        "-g",
        "guardrail 2",
    ]);

    let (history_ok, history_out, history_err) =
        run_sruja(&["agent", "history", "-r", repo_str, "-f", "json"]);
    assert!(
        history_ok,
        "agent history should succeed: stderr={history_err}"
    );
    let parsed: serde_json::Value = serde_json::from_str(history_out.trim()).expect("valid JSON");
    let entries = parsed.as_array().cloned().unwrap_or_default();
    assert!(entries.len() >= 2, "expected >=2 entries: {history_out}");
    let id1 = entries[0]
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let id2 = entries[1]
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert!(!id1.is_empty() && !id2.is_empty());

    let (update_ok, _, update_err) = run_sruja(&[
        "agent",
        "update",
        "-r",
        repo_str,
        "-i",
        id1,
        "-c",
        "updated ctx",
    ]);
    assert!(
        update_ok,
        "agent update should succeed: stderr={update_err}"
    );

    let (curate_ok, curate_out, curate_err) =
        run_sruja(&["agent", "curate", "-r", repo_str, "-f", "json"]);
    assert!(
        curate_ok,
        "agent curate should succeed: stderr={curate_err}"
    );
    let _parsed: serde_json::Value = serde_json::from_str(curate_out.trim()).expect("valid JSON");

    let (clusters_ok, clusters_out, clusters_err) =
        run_sruja(&["agent", "clusters", "-r", repo_str, "-f", "json"]);
    assert!(
        clusters_ok,
        "agent clusters should succeed: stderr={clusters_err}"
    );
    let _parsed: serde_json::Value = serde_json::from_str(clusters_out.trim()).expect("valid JSON");

    let (delete_ok, _, delete_err) =
        run_sruja(&["agent", "delete", "-r", repo_str, "-i", id2, "-y"]);
    assert!(
        delete_ok,
        "agent delete should succeed: stderr={delete_err}"
    );

    let (clear_ok, _, clear_err) = run_sruja(&["agent", "clear", "-r", repo_str, "-y"]);
    assert!(clear_ok, "agent clear should succeed: stderr={clear_err}");
}

#[test]
fn agent_plan_prints_plan_json() {
    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    let repo_str = repo.path().to_str().expect("utf-8");

    let (ok, stdout, stderr) = run_sruja(&[
        "agent",
        "plan",
        "-r",
        repo_str,
        "--goal",
        "test goal",
        "--file",
        "src/lib.rs",
        "--print",
    ]);
    assert!(ok, "agent plan should succeed: stderr={stderr}");

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(
        parsed
            .get("schema_version")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "agent_plan_output/v1"
    );
}

#[test]
fn run_show_json_includes_snapshots() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 1; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (focus_ok, focus_out, focus_err) = run_sruja(&[
        "focus",
        "-r",
        repo_str,
        "--file",
        "src/app.ts",
        "-f",
        "json",
    ]);
    assert!(focus_ok, "focus should succeed: stderr={focus_err}");
    let focus_json: serde_json::Value = serde_json::from_str(focus_out.trim()).expect("valid JSON");
    let run_id = focus_json
        .get("run_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    assert!(!run_id.is_empty(), "expected run_id: stdout={focus_out}");

    let (show_ok, show_out, show_err) = run_sruja(&[
        "run", "show", "-r", repo_str, "--run-id", &run_id, "-f", "json",
    ]);
    assert!(show_ok, "run show should succeed: stderr={show_err}");
    let show_json: serde_json::Value = serde_json::from_str(show_out.trim()).expect("valid JSON");
    assert_eq!(
        show_json.get("schema_version").and_then(|v| v.as_str()),
        Some("run_show/v1")
    );
    assert_eq!(
        show_json.get("run_id").and_then(|v| v.as_str()),
        Some(run_id.as_str())
    );
    assert!(show_json.get("files").is_some());
    assert!(show_json.get("snapshots").is_some());
}

#[test]
fn run_export_writes_manifest() {
    let repo = create_test_repo();
    write_file(
        repo.path(),
        "src/app.ts",
        r#"
export function app() { return 1; }
"#,
    );
    let repo_str = repo.path().to_str().expect("utf-8");

    let (focus_ok, focus_out, focus_err) = run_sruja(&[
        "focus",
        "-r",
        repo_str,
        "--file",
        "src/app.ts",
        "-f",
        "json",
    ]);
    assert!(focus_ok, "focus should succeed: stderr={focus_err}");
    let focus_json: serde_json::Value = serde_json::from_str(focus_out.trim()).expect("valid JSON");
    let run_id = focus_json
        .get("run_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    assert!(!run_id.is_empty(), "expected run_id: stdout={focus_out}");

    let out_dir = repo.path().join("exported_run");
    let out_dir_str = out_dir.to_str().expect("utf-8");
    let (export_ok, export_out, export_err) = run_sruja(&[
        "run",
        "export",
        "-r",
        repo_str,
        "--run-id",
        &run_id,
        "--out",
        out_dir_str,
        "--events-limit",
        "10",
    ]);
    assert!(export_ok, "run export should succeed: stderr={export_err}");
    assert!(
        export_out.contains(out_dir_str),
        "expected out_dir to be printed: stdout={export_out}"
    );

    let manifest_path = out_dir.join("manifest.json");
    assert!(
        manifest_path.exists(),
        "manifest.json must exist at {}",
        manifest_path.display()
    );
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).expect("read manifest"))
            .expect("manifest must be valid JSON");
    assert_eq!(
        manifest.get("schema_version").and_then(|v| v.as_str()),
        Some("run_export/v1")
    );
    assert_eq!(
        manifest.get("run_id").and_then(|v| v.as_str()),
        Some(run_id.as_str())
    );
    assert!(out_dir.join("context_events.json").exists());
}

fn mcp_send(stdin: &mut std::process::ChildStdin, value: serde_json::Value) -> std::io::Result<()> {
    use std::io::Write;
    writeln!(
        stdin,
        "{}",
        serde_json::to_string(&value).unwrap_or_default()
    )?;
    stdin.flush()
}

fn mcp_recv_matching_id(
    rx: &std::sync::mpsc::Receiver<serde_json::Value>,
    id: serde_json::Value,
) -> serde_json::Value {
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_secs(5);

    loop {
        let now = Instant::now();
        let wait = deadline.saturating_duration_since(now);
        let msg = rx
            .recv_timeout(wait)
            .unwrap_or_else(|_| panic!("timed out waiting for MCP response id={id}"));
        if msg.get("id") == Some(&id) {
            return msg;
        }
    }
}

fn mcp_recv_notification_method(
    rx: &std::sync::mpsc::Receiver<serde_json::Value>,
    method: &str,
) -> serde_json::Value {
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_secs(5);

    loop {
        let now = Instant::now();
        let wait = deadline.saturating_duration_since(now);
        let msg = rx
            .recv_timeout(wait)
            .unwrap_or_else(|_| panic!("timed out waiting for MCP notification {method}"));
        if msg.get("method").and_then(|v| v.as_str()) == Some(method) {
            return msg;
        }
    }
}

fn mcp_extract_text_result(response: &serde_json::Value) -> String {
    response
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c0| c0.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or_default()
        .to_string()
}

#[test]
fn mcp_server_roundtrip_tools_resources_prompts_and_tools_call() {
    use std::io::BufRead;
    use std::process::Stdio;

    let repo = create_test_repo();
    write_minimal_cargo_repo(repo.path());
    write_file(repo.path(), "repo.sruja", MINIMAL_VALID_SRUJA);
    let repo_str = repo.path().to_str().expect("utf-8");

    let (init_ok, _init_out, init_err) = run_sruja(&["init", "--auto", "-r", repo_str]);
    assert!(init_ok, "init --auto should succeed: stderr={init_err}");

    let (wf_ok, _wf_out, wf_err) = run_sruja(&[
        "workflow", "init", "-r", repo_str, "--title", "MCP Test", "--id", "wf-mcp",
    ]);
    assert!(wf_ok, "workflow init should succeed: stderr={wf_err}");

    let bundles_dir = repo.path().join("bundles");
    std::fs::create_dir_all(&bundles_dir).expect("create bundles dir");
    let bundle_path = bundles_dir.join("a.repo.bundle.json");
    let bundle_str = bundle_path.to_str().expect("utf-8");
    let (pub_ok, _pub_out, pub_err) = run_sruja(&[
        "publish",
        "-r",
        repo_str,
        "--repo-id",
        "repo-a",
        "-o",
        bundle_str,
    ]);
    assert!(pub_ok, "publish should succeed: stderr={pub_err}");
    assert!(bundle_path.exists(), "bundle must exist");

    let system_index_path = repo.path().join("system.index.json");
    let system_index_str = system_index_path.to_str().expect("utf-8");
    let (compose_ok, _compose_out, compose_err) =
        run_sruja(&["compose", "-i", bundle_str, "-o", system_index_str]);
    assert!(compose_ok, "compose should succeed: stderr={compose_err}");
    assert!(system_index_path.exists(), "system.index.json must exist");

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_sruja"))
        .args(["mcp", "-r", repo_str])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mcp server");

    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");

    let (tx, rx) = std::sync::mpsc::channel::<serde_json::Value>();
    let reader_handle = std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                let _ = tx.send(v);
            }
        }
    });

    let init = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "initializationOptions": {
                "tool_profile": "full",
                "watch_drift": true
            }
        }
    });
    mcp_send(&mut stdin, init).expect("send initialize");
    let init_resp = mcp_recv_matching_id(&rx, serde_json::json!(1));
    assert_eq!(
        init_resp["result"]["protocolVersion"]
            .as_str()
            .unwrap_or(""),
        "2025-06-18"
    );

    let initialized = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });
    mcp_send(&mut stdin, initialized).expect("send notifications/initialized");
    let drift_note = mcp_recv_notification_method(&rx, "notifications/drift_state");
    assert!(
        drift_note.get("params").is_some(),
        "expected drift_state params: {drift_note}"
    );

    mcp_send(
        &mut stdin,
        serde_json::json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {} }),
    )
    .expect("send tools/list");
    let tools_list = mcp_recv_matching_id(&rx, serde_json::json!(2));
    let tools = tools_list["result"]["tools"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(!tools.is_empty(), "expected non-empty tools list");

    mcp_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "resources/list",
            "params": { "path": repo_str }
        }),
    )
    .expect("send resources/list");
    let resources_list = mcp_recv_matching_id(&rx, serde_json::json!(3));
    let resources = resources_list["result"]["resources"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        !resources.is_empty(),
        "expected resources list to be non-empty"
    );
    let first_uri = resources[0]["uri"].as_str().unwrap_or_default().to_string();
    assert!(!first_uri.is_empty());

    mcp_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "resources/read",
            "params": { "path": repo_str, "uri": first_uri }
        }),
    )
    .expect("send resources/read");
    let resources_read = mcp_recv_matching_id(&rx, serde_json::json!(4));
    assert!(
        resources_read.get("result").is_some() || resources_read.get("error").is_some(),
        "expected resources/read response"
    );

    mcp_send(
        &mut stdin,
        serde_json::json!({ "jsonrpc": "2.0", "id": 5, "method": "prompts/list", "params": {} }),
    )
    .expect("send prompts/list");
    let prompts_list = mcp_recv_matching_id(&rx, serde_json::json!(5));
    let prompts = prompts_list["result"]["prompts"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(!prompts.is_empty(), "expected prompts list to be non-empty");
    let prompt_name = prompts[0]["name"].as_str().unwrap_or_default().to_string();
    assert!(!prompt_name.is_empty());

    mcp_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "prompts/get",
            "params": { "name": prompt_name }
        }),
    )
    .expect("send prompts/get");
    let prompts_get = mcp_recv_matching_id(&rx, serde_json::json!(6));
    assert!(
        prompts_get.get("result").is_some() || prompts_get.get("error").is_some(),
        "expected prompts/get response"
    );

    for (id, name, arguments) in [
        (
            10,
            "sruja_get_repomap",
            serde_json::json!({ "path": repo_str }),
        ),
        (
            11,
            "sruja_list_architecture_index",
            serde_json::json!({ "path": repo_str, "max_tokens": 500 }),
        ),
        (
            12,
            "sruja_get_topology",
            serde_json::json!({ "path": repo_str, "id": "App", "depth": 1, "max_tokens": 1200 }),
        ),
        (
            13,
            "sruja_get_elements",
            serde_json::json!({ "path": repo_str, "ids": ["App"], "max_tokens": 1500 }),
        ),
        (
            14,
            "sruja_get_drift_state",
            serde_json::json!({ "path": repo_str }),
        ),
        (
            15,
            "sruja_reindex_memory",
            serde_json::json!({ "path": repo_str }),
        ),
        (
            16,
            "sruja_search_memory",
            serde_json::json!({ "path": repo_str, "query": "App", "limit": 5 }),
        ),
        (
            17,
            "sruja_get_memory_timeline",
            serde_json::json!({ "path": repo_str, "before": 1, "after": 1 }),
        ),
        (
            18,
            "sruja_get_context_score",
            serde_json::json!({ "path": repo_str, "format": "json" }),
        ),
        (
            19,
            "sruja_check_drift",
            serde_json::json!({ "path": repo_str, "architecture": "repo.sruja" }),
        ),
        (
            20,
            "sruja_get_workflow",
            serde_json::json!({ "path": repo_str, "workflow_id": "wf-mcp" }),
        ),
        (
            33,
            "sruja_workflow_summary",
            serde_json::json!({ "path": repo_str, "workflow_id": "wf-mcp" }),
        ),
        (
            34,
            "sruja_workflow_next_steps",
            serde_json::json!({ "path": repo_str, "workflow_id": "wf-mcp" }),
        ),
        (
            21,
            "sruja_record_decision_event",
            serde_json::json!({ "path": repo_str, "kind": "guardrail", "summary": "test decision event", "outcome": "ok" }),
        ),
        (
            22,
            "sruja_get_context_events",
            serde_json::json!({ "path": repo_str, "limit": 5 }),
        ),
        (
            25,
            "sruja_get_task_context",
            serde_json::json!({ "path": repo_str, "file": "src/lib.rs", "max_tokens": 2000 }),
        ),
        (
            26,
            "sruja_get_operational_context",
            serde_json::json!({ "path": repo_str }),
        ),
        (
            27,
            "sruja_get_system_context",
            serde_json::json!({ "path": repo_str }),
        ),
        (
            28,
            "sruja_list_elements",
            serde_json::json!({ "path": repo_str, "kind": "system" }),
        ),
        (
            29,
            "sruja_add_element",
            serde_json::json!({
                "path": repo_str,
                "id": "Worker2",
                "kind": "system",
                "title": "Worker 2",
                "description": "Background worker system"
            }),
        ),
        (
            30,
            "sruja_add_relationship",
            serde_json::json!({
                "path": repo_str,
                "source": "App",
                "target": "Worker2",
                "label": "calls"
            }),
        ),
        (
            31,
            "sruja_propose_change",
            serde_json::json!({
                "path": repo_str,
                "description": "Propose element via structured args",
                "add_elements": [{ "id": "Billing", "kind": "system", "label": "Billing", "technology": "Rust" }],
                "add_relationships": [{ "source": "App", "target": "Billing", "label": "calls" }]
            }),
        ),
        (
            32,
            "sruja_unknown_tool_for_coverage",
            serde_json::json!({ "path": repo_str }),
        ),
    ] {
        mcp_send(
            &mut stdin,
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "tools/call",
                "params": { "name": name, "arguments": arguments }
            }),
        )
        .unwrap_or_else(|e| panic!("send tools/call {name} failed: {e}"));
        let resp = mcp_recv_matching_id(&rx, serde_json::json!(id));
        assert_eq!(resp.get("jsonrpc").and_then(|v| v.as_str()), Some("2.0"));
        let text = mcp_extract_text_result(&resp);
        assert!(
            !text.trim().is_empty(),
            "expected tools/call to return text: tool={name} resp={resp}"
        );
    }

    mcp_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 40,
            "method": "tools/call",
            "params": {
                "name": "sruja_create_decision_record",
                "arguments": {
                    "path": repo_str,
                    "title": "MCP Decision",
                    "record_type": "guardrail",
                    "scope": "repo"
                }
            }
        }),
    )
    .expect("send create decision record");
    let created = mcp_recv_matching_id(&rx, serde_json::json!(40));
    let created_text = mcp_extract_text_result(&created);
    let created_json: serde_json::Value =
        serde_json::from_str(created_text.trim()).expect("create_decision_record must return JSON");
    let decision_id = created_json["id"].as_str().unwrap_or_default().to_string();
    assert!(
        !decision_id.is_empty(),
        "expected decision id: {created_text}"
    );

    mcp_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 41,
            "method": "tools/call",
            "params": {
                "name": "sruja_link_decision_to_element",
                "arguments": {
                    "path": repo_str,
                    "decision_id": decision_id,
                    "element_id": "App"
                }
            }
        }),
    )
    .expect("send link decision");
    let link_resp = mcp_recv_matching_id(&rx, serde_json::json!(41));
    assert!(!mcp_extract_text_result(&link_resp).trim().is_empty());

    mcp_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "tools/call",
            "params": {
                "name": "sruja_get_decisions",
                "arguments": { "path": repo_str }
            }
        }),
    )
    .expect("send get decisions");
    let decisions_resp = mcp_recv_matching_id(&rx, serde_json::json!(42));
    assert!(!mcp_extract_text_result(&decisions_resp).trim().is_empty());

    drop(stdin);
    let _ = child.wait();
    let _ = reader_handle.join();
}
