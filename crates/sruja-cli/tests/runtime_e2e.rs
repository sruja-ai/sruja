//! E2E tests for `sruja runtime analyze` command

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn run_sruja(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_sruja"))
        .args(args)
        .output()
        .expect("Failed to run sruja");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

const SAMPLE_TRACES_JSON: &str = r#"[
  {
    "id": "span-1",
    "name": "agent.run",
    "start": "2025-01-15T10:00:00Z",
    "end": "2025-01-15T10:00:05Z",
    "attributes": [],
    "children": [
      {
        "id": "span-2",
        "name": "llm.generate",
        "start": "2025-01-15T10:00:01Z",
        "end": "2025-01-15T10:00:04Z",
        "attributes": [],
        "children": []
      }
    ]
  }
]"#;

mod runtime_command {
    use super::*;

    #[test]
    fn runtime_analyze_runs_successfully() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let traces_path = dir.path().join("traces.json");
        fs::write(&traces_path, SAMPLE_TRACES_JSON).expect("Failed to write traces");

        let (success, stdout, stderr) = run_sruja(&[
            "runtime",
            "analyze",
            "-t",
            traces_path.to_str().unwrap(),
            "-f",
            "text",
        ]);

        assert!(success, "runtime analyze should succeed: stderr={}", stderr);
        let out = format!("{} {}", stdout, stderr);
        assert!(out.contains("Root traces") || out.contains("Total spans"), "out={}", out);
    }

    #[test]
    fn runtime_analyze_json_output() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let traces_path = dir.path().join("traces.json");
        fs::write(&traces_path, SAMPLE_TRACES_JSON).expect("Failed to write traces");

        let (success, stdout, stderr) = run_sruja(&[
            "runtime",
            "analyze",
            "-t",
            traces_path.to_str().unwrap(),
            "-f",
            "json",
        ]);

        assert!(success, "runtime analyze -f json should succeed: stderr={}", stderr);

        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("Output should be valid JSON");
        assert!(json.get("trace_count").is_some());
        assert!(json.get("total_spans").is_some());
        assert!(json.get("max_depth").is_some());
        assert!(json.get("emergent_cycles").is_some());
        assert!(json.get("hotspots").is_some());
        assert!(json.get("execution_graph").is_some());
    }

    #[test]
    fn runtime_analyze_reports_emergent_cycles() {
        // Trace with cycle: planner -> executor -> planner (repeated twice for min_occurrences)
        let traces_with_cycle = r#"[
          {
            "id": "1",
            "name": "planner",
            "start": "2025-01-15T10:00:00Z",
            "end": "2025-01-15T10:00:05Z",
            "attributes": [],
            "children": [
              {
                "id": "2",
                "name": "executor",
                "start": "2025-01-15T10:00:01Z",
                "end": "2025-01-15T10:00:04Z",
                "attributes": [],
                "children": [
                  {
                    "id": "3",
                    "name": "planner",
                    "start": "2025-01-15T10:00:02Z",
                    "end": "2025-01-15T10:00:03Z",
                    "attributes": [],
                    "children": []
                  }
                ]
              }
            ]
          },
          {
            "id": "4",
            "name": "planner",
            "start": "2025-01-15T10:01:00Z",
            "end": "2025-01-15T10:01:05Z",
            "attributes": [],
            "children": [
              {
                "id": "5",
                "name": "executor",
                "start": "2025-01-15T10:01:01Z",
                "end": "2025-01-15T10:01:04Z",
                "attributes": [],
                "children": [
                  {
                    "id": "6",
                    "name": "planner",
                    "start": "2025-01-15T10:01:02Z",
                    "end": "2025-01-15T10:01:03Z",
                    "attributes": [],
                    "children": []
                  }
                ]
              }
            ]
          }
        ]"#;

        let dir = TempDir::new().expect("Failed to create temp dir");
        let traces_path = dir.path().join("traces.json");
        fs::write(&traces_path, traces_with_cycle).expect("Failed to write traces");

        let (success, stdout, stderr) = run_sruja(&[
            "runtime",
            "analyze",
            "-t",
            traces_path.to_str().unwrap(),
            "-f",
            "json",
        ]);

        assert!(success, "runtime analyze should succeed: stderr={}", stderr);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let cycles = json.get("emergent_cycles").and_then(|c| c.as_array()).unwrap();
        assert!(!cycles.is_empty(), "Should detect emergent cycles");
        let pattern = cycles[0].get("pattern").and_then(|p| p.as_array()).unwrap();
        assert_eq!(pattern, &["planner", "executor", "planner"]);
    }

    #[test]
    fn runtime_analyze_missing_file_fails() {
        let (success, _stdout, stderr) =
            run_sruja(&["runtime", "analyze", "-t", "/nonexistent/traces.json", "-f", "text"]);

        assert!(!success, "runtime analyze on missing file should fail");
        assert!(stderr.contains("not found") || stderr.contains("NotFound"), "stderr={}", stderr);
    }
}
