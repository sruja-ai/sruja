mod tests {
    use super::super::*;
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use std::fs;
    use std::sync::{Arc, OnceLock};
    use tokio::sync::Mutex;

    static ENV_LOCK: OnceLock<std::sync::Mutex<()>> = OnceLock::new();

    #[test]
    fn mcp_initialize_result_includes_capabilities() {
        let server = McpServer::new(".".to_string());
        let resp = server.handle_initialize(
            Some(json!(1)),
            Some(&json!({ "protocolVersion": MCP_PROTOCOL_VERSION })),
        );

        assert_eq!(resp.get("jsonrpc").and_then(|v| v.as_str()), Some("2.0"));
        assert_eq!(resp.get("id").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(
            resp.pointer("/result/protocolVersion")
                .and_then(|v| v.as_str()),
            Some(MCP_PROTOCOL_VERSION)
        );
        assert!(resp.pointer("/result/capabilities/tools").is_some());
        assert!(resp.pointer("/result/capabilities/resources").is_some());
        assert!(resp.pointer("/result/capabilities/prompts").is_some());
        assert_eq!(
            resp.pointer("/result/capabilities/experimental/watchDrift")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[tokio::test]
    async fn mcp_watch_drift_env_enables_notification() {
        let mut server = McpServer::new(".".to_string());
        let _guard = ENV_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .expect("env lock");
        std::env::set_var(ENV_MCP_WATCH_DRIFT, "1");
        let _ = server
            .handle_message(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": { "protocolVersion": MCP_PROTOCOL_VERSION }
            }))
            .await;
        let _ = server
            .handle_message(json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            }))
            .await;
        std::env::remove_var(ENV_MCP_WATCH_DRIFT);
        let pending = server.drain_pending_notifications();
        assert!(
            pending
                .iter()
                .any(|n| n.get("method").and_then(|m| m.as_str())
                    == Some("notifications/drift_state")),
            "expected drift_state from SRUJA_MCP_WATCH_DRIFT, got: {pending:?}"
        );
    }

    #[tokio::test]
    async fn mcp_watch_drift_emits_notification_after_initialized() {
        let mut server = McpServer::new(".".to_string());
        let _ = server
            .handle_message(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "initializationOptions": { "watch_drift": true }
                }
            }))
            .await;
        let _ = server
            .handle_message(json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            }))
            .await;
        let pending = server.drain_pending_notifications();
        assert!(
            pending.iter().any(|n| {
                n.get("method").and_then(|m| m.as_str()) == Some("notifications/drift_state")
                    && n.pointer("/params/schema_version").and_then(|v| v.as_str())
                        == Some("drift_state/v1")
            }),
            "expected drift_state notification, got: {pending:?}"
        );
    }

    #[test]
    fn mcp_resources_list_includes_invariant_uri() {
        let resources = crate::commands::mcp_resources::list_resources(".").expect("list");
        assert!(resources
            .iter()
            .any(|r| r.uri == "sruja://context/invariant.md"));
    }

    #[test]
    fn mcp_prompts_list_includes_mcp_guide() {
        let prompts = crate::commands::mcp_prompts::list_prompts();
        assert!(prompts.iter().any(|p| p.name == "sruja_mcp_guide"));
    }

    #[test]
    fn invariant_brief_fits_token_budget() {
        let ctx = crate::commands::context::types::ArchitectureContext {
            repo: "test".to_string(),
            summary: crate::commands::context::types::ContextSummary {
                total_modules: 10,
                total_services: 2,
                total_databases: 1,
                total_external_apis: 0,
            },
            layers: vec![],
            boundaries: vec![crate::commands::context::types::BoundaryRule {
                from: "ui".to_string(),
                to: "data".to_string(),
                allowed: false,
                reason: "use services".to_string(),
            }],
            forbidden_patterns: vec![],
            active_decisions: vec![],
            focus: None,
            system_context: None,
            max_tokens: 700,
        };
        let brief = crate::commands::context::format_invariant_brief(&ctx);
        assert!(brief.len() < 4_000, "brief should stay compact");
        assert!(brief.contains("sruja_list_architecture_index"));
    }

    #[test]
    fn mutating_mcp_tool_detection() {
        assert!(is_mutating_mcp_tool("sruja_record_learning"));
        assert!(is_mutating_mcp_tool("sruja_record_context_event"));
        assert!(is_mutating_mcp_tool("sruja_record_decision_event"));
        assert!(is_mutating_mcp_tool("sruja_create_decision_record"));
        assert!(is_mutating_mcp_tool("sruja_link_decision_to_element"));
        assert!(is_mutating_mcp_tool("sruja_sandbox"));
        assert!(is_mutating_mcp_tool("sruja_agent_run"));
        assert!(!is_mutating_mcp_tool("sruja_check_drift"));
        assert!(!is_mutating_mcp_tool("sruja_hybrid_query"));
        for ladder in [
            "sruja_list_architecture_index",
            "sruja_get_topology",
            "sruja_get_elements",
            "sruja_get_diagnostic_full",
            "sruja_suggest_context_prune",
            "sruja_get_drift_state",
        ] {
            assert!(
                !is_mutating_mcp_tool(ladder),
                "ladder tool {ladder} must be read-only"
            );
        }
    }

    #[test]
    fn mcp_readonly_list_excludes_all_mutating_tools() {
        let full = mcp_tools_for_list_with_readonly(false);
        let ro = mcp_tools_for_list_with_readonly(true);
        assert!(ro.len() < full.len());
        for t in &ro {
            let n = t.get("name").and_then(|x| x.as_str()).expect("name");
            assert!(
                !is_mutating_mcp_tool(n),
                "readonly list leaked mutating tool {n}"
            );
        }
        for m in MCP_MUTATING_TOOLS {
            assert!(!ro
                .iter()
                .any(|t| t.get("name").and_then(|n| n.as_str()) == Some(*m)));
        }
    }

    #[tokio::test]
    async fn mcp_tools_list_returns_sruja_tools() {
        let server = McpServer::new(".".to_string());
        let resp = server.handle_tools_list(json!(1));
        let tools = resp
            .pointer("/result/tools")
            .and_then(|v| v.as_array())
            .expect("tools list");

        let names: Vec<String> = tools
            .iter()
            .filter_map(|t| {
                t.get("name")
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        assert!(names.contains(&"sruja_get_repomap".to_string()));
        assert!(names.contains(&"sruja_get_architecture_context".to_string()));
        assert!(names.contains(&"sruja_explain_discovery".to_string()));
        assert!(names.contains(&"sruja_check_drift".to_string()));
        assert!(names.contains(&"sruja_list_architecture_index".to_string()));
        assert!(names.contains(&"sruja_get_topology".to_string()));
        assert!(names.contains(&"sruja_get_elements".to_string()));
        assert!(names.contains(&"sruja_get_diagnostic_full".to_string()));
        assert!(names.contains(&"sruja_suggest_context_prune".to_string()));
        assert!(names.contains(&"sruja_get_drift_state".to_string()));
        assert!(names.contains(&"sruja_get_context_events".to_string()));
        assert!(names.contains(&"sruja_get_decisions".to_string()));
        assert!(names.contains(&"sruja_get_decision_trace".to_string()));
        assert!(names.contains(&"sruja_record_context_event".to_string()));
        assert!(names.contains(&"sruja_record_decision_event".to_string()));
        assert!(names.contains(&"sruja_create_decision_record".to_string()));
        assert!(names.contains(&"sruja_link_decision_to_element".to_string()));
        assert!(names.contains(&"sruja_get_learned_facts".to_string()));
        assert!(names.contains(&"sruja_get_evidence_graph".to_string()));
        assert!(names.contains(&"sruja_get_author_evidence".to_string()));
        assert!(names.contains(&"sruja_get_agent_learnings".to_string()));
        assert!(names.contains(&"sruja_search_memory".to_string()));
        assert!(names.contains(&"sruja_get_memory_timeline".to_string()));
    }

    #[tokio::test]
    async fn mcp_tool_call_repomap_returns_markdown() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("main.rs"), "fn main() { println!(\"hello\"); }\n").expect("write");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_repomap",
            &json!({ "path": dir.path().to_string_lossy() }),
            ".",
            &cache,
        )
        .await
        .expect("repomap");
        assert!(out.contains("# Repository Map"));
    }

    #[tokio::test]
    async fn mcp_tool_call_discovery_explanation_returns_summary() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"express":"4.18.0"}}"#,
        )
        .expect("package");
        fs::write(
            src.join("server.ts"),
            "import { query } from './db';\nexport function start() { return query(); }\n",
        )
        .expect("server");
        fs::write(
            src.join("db.ts"),
            "export function query() { return []; }\n",
        )
        .expect("db");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_explain_discovery",
            &json!({ "path": dir.path().to_string_lossy() }),
            ".",
            &cache,
        )
        .await
        .expect("discovery explanation");

        assert!(out.contains("# Sruja Discovery Explanation"));
        assert!(out.contains("Why Sruja Thinks That"));
    }

    #[tokio::test]
    async fn mcp_tool_call_neighbors_returns_neighbors() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("main.rs"), "mod sub;\nfn main() {}\n").expect("main");
        fs::write(src.join("sub.rs"), "pub fn run() {}\n").expect("sub");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_neighbors",
            &json!({ "path": dir.path().to_string_lossy(), "id": "src_sub_rs" }),
            ".",
            &cache,
        )
        .await
        .expect("neighbors");

        assert!(out.contains("# Neighbors of src_sub_rs"));
        assert!(out.contains("Upstream"));
        assert!(out.contains("Downstream"));
    }

    #[tokio::test]
    async fn mcp_tool_call_find_path_returns_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            src.join("main.rs"),
            "use crate::sub;\nfn main() { sub::run(); }\n",
        )
        .expect("main");
        fs::write(src.join("sub.rs"), "pub fn run() {}\n").expect("sub");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_find_path",
            &json!({
                "path": dir.path().to_string_lossy(),
                "source": "src_main_rs",
                "target": "src_sub_rs"
            }),
            ".",
            &cache,
        )
        .await
        .expect("path");

        assert!(out.contains("# Path from src_main_rs to src_sub_rs"));
        assert!(out.contains("src_main_rs -> src_sub_rs"));
    }

    #[tokio::test]
    async fn mcp_tool_call_architecture_index_from_dsl_returns_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("repo.sruja"),
            r#"
MySystem = system "My System" {
  description "Test system"

  Api = container "API" {
    technology "Rust"
    description "HTTP API"
  }

  Db = database "DB" {
    technology "PostgreSQL"
    description "Data store"
  }
}

MySystem.Api -> MySystem.Db "SQL"
"#,
        )
        .expect("repo.sruja");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_list_architecture_index",
            &json!({ "path": dir.path().to_string_lossy(), "max_tokens": 2000 }),
            ".",
            &cache,
        )
        .await
        .expect("index");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            parsed.get("schema_version").and_then(|v| v.as_str()),
            Some("architecture_index/v1")
        );
        assert!(parsed.get("elements").and_then(|v| v.as_array()).is_some());
        assert!(parsed.get("estimated_tokens").is_some());
        assert_eq!(
            parsed.get("next_suggested_tool").and_then(|v| v.as_str()),
            Some("sruja_get_topology")
        );
    }

    #[tokio::test]
    async fn mcp_tool_call_topology_from_dsl_returns_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("repo.sruja"),
            r#"
MySystem = system "My System" {
  description "Test system"

  Api = container "API" {
    technology "Rust"
    description "HTTP API"
  }

  Db = database "DB" {
    technology "PostgreSQL"
    description "Data store"
  }
}

MySystem.Api -> MySystem.Db "SQL"
"#,
        )
        .expect("repo.sruja");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_topology",
            &json!({ "path": dir.path().to_string_lossy(), "id": "MySystem.Api", "depth": 1 }),
            ".",
            &cache,
        )
        .await
        .expect("topology");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            parsed.get("schema_version").and_then(|v| v.as_str()),
            Some("topology/v1")
        );
        assert!(parsed.get("upstream").is_some());
        assert!(parsed.get("downstream").is_some());
        assert_eq!(
            parsed.get("next_suggested_tool").and_then(|v| v.as_str()),
            Some("sruja_get_elements")
        );
        assert!(parsed
            .get("element_ids")
            .and_then(|v| v.as_array())
            .is_some());
    }

    #[tokio::test]
    async fn mcp_tool_call_get_elements_from_dsl_returns_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("repo.sruja"),
            r#"
MySystem = system "My System" {
  description "Test system"

  Api = container "API" {
    technology "Rust"
    description "HTTP API"
  }
}
"#,
        )
        .expect("repo.sruja");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_elements",
            &json!({ "path": dir.path().to_string_lossy(), "ids": ["MySystem.Api"] }),
            ".",
            &cache,
        )
        .await
        .expect("elements");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            parsed.get("schema_version").and_then(|v| v.as_str()),
            Some("elements/v1")
        );
        assert!(parsed.get("elements").and_then(|v| v.as_array()).is_some());
        assert_eq!(
            parsed.get("next_suggested_tool").and_then(|v| v.as_str()),
            Some("sruja_get_task_context")
        );
    }

    #[tokio::test]
    async fn mcp_tool_call_topology_element_ids_include_neighbors() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("repo.sruja"),
            r#"
MySystem = system "My System" {
  description "Test system"
  Api = container "API" { technology "Rust" description "HTTP API" }
  Db = database "DB" { technology "PostgreSQL" description "Data store" }
}
MySystem.Api -> MySystem.Db "SQL"
"#,
        )
        .expect("repo.sruja");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_topology",
            &json!({ "path": dir.path().to_string_lossy(), "id": "MySystem.Api", "depth": 1 }),
            ".",
            &cache,
        )
        .await
        .expect("topology");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        let ids: Vec<String> = parsed
            .get("element_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        assert!(ids.iter().any(|id| id == "MySystem.Api"));
        assert!(ids.iter().any(|id| id == "MySystem.Db"));
    }

    #[tokio::test]
    async fn mcp_tool_call_topology_resolves_short_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("repo.sruja"),
            r#"
MySystem = system "My System" {
  description "Test"
  Api = container "API" { technology "Rust" description "API" }
}
"#,
        )
        .expect("repo.sruja");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_topology",
            &json!({ "path": dir.path().to_string_lossy(), "id": "Api", "depth": 1 }),
            ".",
            &cache,
        )
        .await
        .expect("topology");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            parsed.get("target").and_then(|v| v.as_str()),
            Some("MySystem.Api")
        );
    }

    #[tokio::test]
    async fn mcp_tool_call_architecture_index_truncates_when_budget_low() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut dsl = String::new();
        dsl.push_str("App = system \"App\" {\n  description \"Many elements\"\n");
        for i in 0..80 {
            dsl.push_str(&format!(
                "  S{i} = container \"S{i}\" {{ technology \"Go\" description \"Service {i}\" }}\n"
            ));
        }
        dsl.push_str("}\n");
        fs::write(dir.path().join("repo.sruja"), dsl).expect("repo.sruja");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_list_architecture_index",
            &json!({ "path": dir.path().to_string_lossy(), "max_tokens": 400 }),
            ".",
            &cache,
        )
        .await
        .expect("index");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            parsed.get("truncated").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[tokio::test]
    async fn mcp_tool_call_get_diagnostic_full_reads_vfs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let uri = crate::commands::diagnostic_vfs::write_vfs_diagnostic(
            dir.path(),
            "sample.txt",
            "full diagnostic body\n",
        )
        .expect("write");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_diagnostic_full",
            &json!({
                "path": dir.path().to_string_lossy(),
                "uri": uri
            }),
            ".",
            &cache,
        )
        .await
        .expect("diagnostic full");

        assert_eq!(out.trim(), "full diagnostic body");
    }

    #[tokio::test]
    async fn mcp_tool_call_architecture_index_scan_fallback_without_dsl() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("lib.rs"), "pub fn hello() {}\n").expect("write");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_list_architecture_index",
            &json!({ "path": dir.path().to_string_lossy() }),
            ".",
            &cache,
        )
        .await
        .expect("index");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            parsed.pointer("/source/kind").and_then(|v| v.as_str()),
            Some("scan")
        );
    }

    #[tokio::test]
    async fn mcp_tool_call_uses_default_root_when_path_is_omitted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("main.rs"), "fn main() { println!(\"hello\"); }\n").expect("write");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_get_repomap",
            &json!({}),
            &dir.path().to_string_lossy(),
            &cache,
        )
        .await
        .expect("repomap");

        assert!(out.contains("# Repository Map"));
    }

    #[tokio::test]
    async fn mcp_tool_call_query_graph_returns_grounded_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("main.rs"), "mod sub;\nfn main() {}\n").expect("main");
        fs::write(src.join("sub.rs"), "pub fn run() {}\n").expect("sub");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_query_graph",
            &json!({
                "path": dir.path().to_string_lossy(),
                "query": "main sub module",
                "enrich": false
            }),
            ".",
            &cache,
        )
        .await
        .expect("query graph");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON output");
        assert_eq!(
            parsed.get("query").and_then(|v| v.as_str()),
            Some("main sub module")
        );
        assert!(parsed.get("matched_nodes").is_some());
        assert!(parsed.get("relationships").is_some());
    }

    #[tokio::test]
    async fn mcp_tool_call_explain_element_returns_grounded_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("main.rs"), "mod sub;\nfn main() {}\n").expect("main");
        fs::write(src.join("sub.rs"), "pub fn run() {}\n").expect("sub");

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let out = run_tool(
            "sruja_explain_element",
            &json!({
                "path": dir.path().to_string_lossy(),
                "id": "src_sub_rs",
                "enrich": false
            }),
            ".",
            &cache,
        )
        .await
        .expect("explain element");

        let parsed: Value = serde_json::from_str(&out).expect("valid JSON output");
        assert_eq!(
            parsed.pointer("/element/id").and_then(|v| v.as_str()),
            Some("src_sub_rs")
        );
        assert!(parsed.get("neighbors").is_some());
        assert!(parsed.get("notes").is_some());
    }
}
