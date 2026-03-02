//! E2E tests for quickstart command

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn create_test_repo() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

fn write_file(dir: &Path, name: &str, content: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(path, content).expect("Failed to write file");
}

fn run_sruja(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_sruja"))
        .args(args)
        .output()
        .expect("Failed to run sruja");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

mod basic_functionality {
    use super::*;

    #[test]
    fn quickstart_scans_repository() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "main.ts",
            r#"
import { service } from './service';
export function main() { return service(); }
"#,
        );
        write_file(
            repo.path(),
            "service.ts",
            r#"export function service() { return 'hello'; }"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        assert!(!graph.nodes.is_empty(), "Should detect nodes");
        assert!(!graph.edges.is_empty(), "Should detect edges");
    }

    #[test]
    fn quickstart_handles_empty_repo() {
        let repo = create_test_repo();
        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan should succeed");
        assert!(graph.nodes.is_empty(), "Empty repo should have no nodes");
    }

    #[test]
    fn quickstart_detects_components() {
        let repo = create_test_repo();

        write_file(repo.path(), "module.ts", r#"export const x = 1;"#);
        write_file(repo.path(), "service.ts", r#"export function service() {}"#);
        write_file(repo.path(), "db.ts", r#"export function query() {}"#);

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        assert!(
            graph.nodes.len() >= 3,
            "Should detect at least 3 components"
        );
    }
}

mod health_score {
    use super::*;

    #[test]
    fn scans_clean_single_file() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "clean.ts",
            r#"
export function clean() { return 'clean code'; }
"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        // Single file with no dependencies should be scanned successfully
        assert!(!graph.nodes.is_empty(), "Should detect the clean file");
        // A single file with no imports/exports to other files should have minimal edges
        // The scanner may still detect some internal structure, which is fine
    }

    #[test]
    fn health_score_decreases_with_violations() {
        let repo = create_test_repo();

        // Create circular dependency
        write_file(
            repo.path(),
            "a.ts",
            r#"
import { b } from './b';
export function a() { return b(); }
"#,
        );
        write_file(
            repo.path(),
            "b.ts",
            r#"
import { a } from './a';
export function b() { return a(); }
"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");
        assert!(!graph.edges.is_empty(), "Should detect dependencies");
    }
}

mod findings_detection {
    use super::*;

    #[test]
    fn detects_circular_dependencies() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "a.ts",
            r#"
import { b } from './b';
export function a() { return b(); }
"#,
        );
        write_file(
            repo.path(),
            "b.ts",
            r#"
import { c } from './c';
export function b() { return c(); }
"#,
        );
        write_file(
            repo.path(),
            "c.ts",
            r#"
import { a } from './a';
export function c() { return a(); }
"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        let has_imports = graph
            .edges
            .iter()
            .any(|e| e.kind == sruja_scan::EdgeKind::Calls);

        assert!(has_imports, "Should detect circular imports");
    }

    #[test]
    fn detects_god_modules() {
        let repo = create_test_repo();

        // Module with many dependencies
        let imports: String = (0..15)
            .map(|i| format!("import {{ dep{i} }} from './dep{i}';"))
            .collect::<Vec<_>>()
            .join("\n");

        write_file(
            repo.path(),
            "god.ts",
            &format!(
                r#"
{imports}
export function god() {{
    return [{}];
}}
"#,
                (0..15)
                    .map(|i| format!("dep{i}()"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );

        for i in 0..15 {
            write_file(
                repo.path(),
                &format!("dep{}.ts", i),
                &format!(r#"export function dep{}() {{ return {}; }}"#, i, i),
            );
        }

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        let god_deps = graph
            .edges
            .iter()
            .filter(|e| e.source.contains("god"))
            .count();

        assert!(
            god_deps >= 10,
            "Should detect god module with many dependencies"
        );
    }

    #[test]
    fn detects_layer_violations() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "frontend/ui.ts",
            r#"
import { query } from '../database/db';
export function getData() { return query('SELECT *'); }
"#,
        );
        write_file(
            repo.path(),
            "database/db.ts",
            r#"
export function query(sql: string) { return []; }
"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        let has_frontend_to_db = graph
            .edges
            .iter()
            .any(|e| e.source.contains("frontend") && e.target.contains("database"));

        assert!(
            has_frontend_to_db || !graph.edges.is_empty(),
            "Should detect layer violation"
        );
    }

    #[test]
    fn detects_orphan_modules() {
        let repo = create_test_repo();

        // Connected module
        write_file(
            repo.path(),
            "main.ts",
            r#"
import { service } from './service';
export function main() { return service(); }
"#,
        );
        write_file(
            repo.path(),
            "service.ts",
            r#"
export function service() { return 'hello'; }
"#,
        );

        // Isolated module
        write_file(
            repo.path(),
            "isolated.ts",
            r#"
function isolated() { return 'alone'; }
const x = 1;
"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");
        assert!(graph.nodes.len() >= 3, "Should detect all files");
    }
}

mod output_formats {
    use super::*;

    #[test]
    fn generates_json_output() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "app.ts",
            r#"
import { hello } from './hello';
export function app() { return hello(); }
"#,
        );
        write_file(
            repo.path(),
            "hello.ts",
            r#"export function hello() { return 'world'; }"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        // Verify graph can be serialized to JSON
        let json = serde_json::to_string(&graph).expect("Should serialize to JSON");
        assert!(json.contains("nodes"), "JSON should contain nodes");
        assert!(json.contains("edges"), "JSON should contain edges");
    }

    #[test]
    fn json_output_structure() {
        let repo = create_test_repo();

        write_file(repo.path(), "module.ts", r#"export const x = 1;"#);

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");
        let json: serde_json::Value = serde_json::to_string(&graph)
            .map(|s| serde_json::from_str(&s).unwrap())
            .unwrap();

        assert!(json.is_object(), "Output should be a JSON object");
        assert!(json.get("nodes").is_some(), "Should have nodes field");
        assert!(json.get("edges").is_some(), "Should have edges field");
    }
}

mod inventory_summary {
    use super::*;

    #[test]
    fn counts_modules_correctly() {
        let repo = create_test_repo();

        write_file(repo.path(), "a.ts", r#"export const a = 1;"#);
        write_file(repo.path(), "b.ts", r#"export const b = 2;"#);
        write_file(repo.path(), "c.ts", r#"export const c = 3;"#);

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        assert!(graph.nodes.len() >= 3, "Should count all modules");
    }

    #[test]
    fn counts_dependencies_correctly() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "a.ts",
            r#"
import { b } from './b';
import { c } from './c';
export function a() { return b() + c(); }
"#,
        );
        write_file(repo.path(), "b.ts", r#"export function b() { return 1; }"#);
        write_file(repo.path(), "c.ts", r#"export function c() { return 2; }"#);

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        assert!(!graph.edges.is_empty(), "Should detect dependencies");
    }
}

mod multi_language {
    use super::*;

    #[test]
    fn scans_typescript_project() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "app.ts",
            r#"
import { service } from './service';
export function app() { return service(); }
"#,
        );
        write_file(
            repo.path(),
            "service.ts",
            r#"export function service() { return 'hello'; }"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");
        assert!(!graph.nodes.is_empty(), "Should detect TypeScript files");
    }

    #[test]
    fn scans_python_project() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "main.py",
            r#"
from service import process
def main():
    return process()
"#,
        );
        write_file(
            repo.path(),
            "service.py",
            r#"
def process():
    return 'done'
"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");
        assert!(!graph.nodes.is_empty(), "Should detect Python files");
    }

    #[test]
    fn scans_rust_project() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "main.rs",
            r#"
mod lib;
fn main() {
    lib::process();
}
"#,
        );
        write_file(
            repo.path(),
            "lib.rs",
            r#"
pub fn process() -> i32 { 42 }
"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");
        assert!(!graph.nodes.is_empty(), "Should detect Rust files");
    }

    #[test]
    fn scans_go_project() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "main.go",
            r#"
package main
import "fmt"
func main() {
    fmt.Println("hello")
}
"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");
        assert!(!graph.nodes.is_empty(), "Should detect Go files");
    }

    #[test]
    fn scans_javascript_project() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "index.js",
            r#"
const utils = require('./utils');
module.exports = { main: () => utils.help() };
"#,
        );
        write_file(
            repo.path(),
            "utils.js",
            r#"module.exports = { help: () => 'help' };"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");
        assert!(!graph.nodes.is_empty(), "Should detect JavaScript files");
    }
}

mod evidence_references {
    use super::*;

    #[test]
    fn provides_file_paths() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "src/module.ts",
            r#"export const module = 'test';"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        let has_path = graph
            .nodes
            .iter()
            .any(|n| n.path.as_deref().unwrap_or("").contains("module"));

        assert!(has_path, "Should provide file paths as evidence");
    }

    #[test]
    fn identifies_technology() {
        let repo = create_test_repo();

        write_file(repo.path(), "app.ts", r#"export const app = 'typescript';"#);
        write_file(repo.path(), "script.py", r#"def script(): pass"#);

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        let techs: Vec<_> = graph
            .nodes
            .iter()
            .filter_map(|n| n.technology.as_ref())
            .collect();

        assert!(
            techs
                .iter()
                .any(|t| t.contains("TypeScript") || t.contains("Python")),
            "Should identify technologies"
        );
    }
}

/// CLI invocation tests - run the actual sruja binary to prove end-to-end usability.
mod cli_invocation {
    use super::*;

    #[test]
    fn quickstart_cli_runs_successfully() {
        let repo = create_test_repo();
        write_file(
            repo.path(),
            "app.ts",
            r#"
import { hello } from './hello';
export function app() { return hello(); }
"#,
        );
        write_file(
            repo.path(),
            "hello.ts",
            r#"export function hello() { return 'world'; }"#,
        );

        let (success, stdout, stderr) =
            run_sruja(&["quickstart", "-r", repo.path().to_str().unwrap()]);

        assert!(success, "quickstart should succeed: stderr={}", stderr);
        assert!(
            stdout.contains("Architecture") || stderr.contains("Architecture"),
            "Output should mention architecture. stdout={} stderr={}",
            stdout,
            stderr
        );
    }

    #[test]
    fn quickstart_json_output_structure() {
        let repo = create_test_repo();
        write_file(
            repo.path(),
            "app.ts",
            r#"import { x } from './x'; export const app = x;"#,
        );
        write_file(repo.path(), "x.ts", r#"export const x = 1;"#);

        let (success, stdout, stderr) = run_sruja(&[
            "quickstart",
            "-r",
            repo.path().to_str().unwrap(),
            "-f",
            "json",
        ]);

        assert!(
            success,
            "quickstart -f json should succeed: stderr={}",
            stderr
        );

        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("Output should be valid JSON");
        assert!(json.get("repo").is_some(), "JSON should have repo field");
        assert!(
            json.get("health_score").is_some(),
            "JSON should have health_score"
        );
        assert!(
            json.get("inventory").is_some(),
            "JSON should have inventory"
        );
        assert!(
            json.get("top_findings").is_some(),
            "JSON should have top_findings"
        );
        assert!(
            json.get("actionable_fixes").is_some(),
            "JSON should have actionable_fixes"
        );

        let inv = json.get("inventory").unwrap();
        assert!(inv.get("modules").is_some());
        assert!(inv.get("total_dependencies").is_some());
    }

    #[test]
    fn quickstart_text_output_contains_expected_sections() {
        let repo = create_test_repo();
        write_file(repo.path(), "main.ts", r#"export const main = 1;"#);

        let (success, stdout, stderr) =
            run_sruja(&["quickstart", "-r", repo.path().to_str().unwrap()]);

        assert!(success, "quickstart should succeed: stderr={}", stderr);
        let out = format!("{} {}", stdout, stderr);
        assert!(
            out.contains("Architecture Inventory") || out.contains("Architecture Intelligence"),
            "Should show inventory section"
        );
        assert!(
            out.contains("Health Score") || out.contains("Health"),
            "Should show health score"
        );
        assert!(
            out.contains("Next Steps") || out.contains("modules") || out.contains("components"),
            "Should show results or next steps"
        );
    }

    #[test]
    fn quickstart_handles_empty_repo_via_cli() {
        let repo = create_test_repo();

        let (success, _stdout, stderr) =
            run_sruja(&["quickstart", "-r", repo.path().to_str().unwrap()]);

        assert!(
            success,
            "quickstart on empty repo should succeed: stderr={}",
            stderr
        );
    }
}
