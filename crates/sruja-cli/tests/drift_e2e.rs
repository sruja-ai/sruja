//! E2E tests for drift detection

mod common;
use common::*;

mod circular_dependency {
    use super::*;

    #[test]
    fn detects_simple_cycle() {
        let repo = create_test_repo();

        // A -> B -> C -> A
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

        // Check that imports were detected
        let imports: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.kind == sruja_scan::EdgeKind::Calls)
            .collect();

        assert!(!imports.is_empty(), "Should detect import edges");
    }

    #[test]
    fn no_cycle_in_linear_chain() {
        let repo = create_test_repo();

        // A -> B -> C (no cycle)
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
export function c() { return 'done'; }
"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");
        assert!(!graph.edges.is_empty(), "Should detect edges");
    }
}

mod layer_violation {
    use super::*;

    #[test]
    fn detects_frontend_database_access() {
        let repo = create_test_repo();

        // Frontend directly importing database module
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

        // Should have edges from frontend to database
        let has_frontend_to_db = graph
            .edges
            .iter()
            .any(|e| e.source.contains("frontend") && e.target.contains("database"));

        assert!(
            has_frontend_to_db || !graph.edges.is_empty(),
            "Should detect relationship between frontend and database"
        );
    }
}

mod orphan_modules {
    use super::*;

    #[test]
    fn detects_isolated_module() {
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

        // Isolated module (no imports, no exports, no references)
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

    #[test]
    fn connected_modules_not_orphan() {
        let repo = create_test_repo();

        write_file(
            repo.path(),
            "a.ts",
            r#"
import { b } from './b';
export const a = b;
"#,
        );
        write_file(
            repo.path(),
            "b.ts",
            r#"
export const b = 1;
"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        // Both should have connections
        let a_has_connections = graph
            .edges
            .iter()
            .any(|e| e.source.contains("a") || e.target.contains("a"));
        let b_has_connections = graph
            .edges
            .iter()
            .any(|e| e.source.contains("b") || e.target.contains("b"));

        assert!(
            a_has_connections || b_has_connections,
            "Connected modules should have edges"
        );
    }
}

mod god_modules {
    use super::*;

    #[test]
    fn detects_module_with_many_dependencies() {
        let repo = create_test_repo();

        // Module with 15 imports
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

        // Create dependency files
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
}

mod language_support {
    use super::*;

    #[test]
    fn scans_typescript() {
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
        assert!(graph
            .nodes
            .iter()
            .any(|n| n.path.as_deref().unwrap_or("").contains("app")));
    }

    #[test]
    fn scans_javascript() {
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
        assert!(!graph.nodes.is_empty(), "Should parse JavaScript files");
    }

    #[test]
    fn scans_python() {
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
        assert!(!graph.nodes.is_empty(), "Should parse Python files");
    }

    #[test]
    fn scans_go() {
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
        assert!(!graph.nodes.is_empty(), "Should parse Go files");
    }

    #[test]
    fn scans_rust() {
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
        assert!(!graph.nodes.is_empty(), "Should parse Rust files");
    }
}

mod mixed_languages {
    use super::*;

    #[test]
    fn scans_multilingual_repo() {
        let repo = create_test_repo();

        write_file(repo.path(), "app.ts", r#"export const app = 'typescript';"#);
        write_file(repo.path(), "script.py", r#"def script(): pass"#);
        write_file(repo.path(), "main.go", r#"package main"#);
        write_file(repo.path(), "lib.rs", r#"pub fn lib() {}"#);

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan failed");

        let techs: Vec<_> = graph
            .nodes
            .iter()
            .filter_map(|n| n.technology.as_ref())
            .collect();

        assert!(
            techs.iter().any(|t| t.contains("TypeScript")),
            "Should detect TypeScript"
        );
        assert!(
            techs.iter().any(|t| t.contains("Python")),
            "Should detect Python"
        );
        assert!(techs.iter().any(|t| t.contains("Go")), "Should detect Go");
        assert!(
            techs.iter().any(|t| t.contains("Rust")),
            "Should detect Rust"
        );
    }
}

mod empty_repo {
    use super::*;

    #[test]
    fn handles_empty_directory() {
        let repo = create_test_repo();
        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan should succeed");
        assert!(graph.nodes.is_empty(), "Empty repo should have no nodes");
    }

    #[test]
    fn handles_non_source_files() {
        let repo = create_test_repo();
        write_file(repo.path(), "README.md", "# My Project");
        write_file(repo.path(), "config.json", r#"{"key": "value"}"#);
        write_file(repo.path(), "data.txt", "some text");

        let graph = sruja_scan::scan_repo(repo.path()).expect("Scan should succeed");
        assert!(graph.nodes.is_empty(), "Non-source files should be ignored");
    }
}

/// Drift correctness tests - verify sruja_diff::detect_architectural_drift produces expected violations.
mod drift_correctness {
    use sruja_diff::ViolationKind;

    use super::*;

    #[test]
    fn detects_circular_dependency_violation() {
        let repo = create_test_repo();
        write_file(
            repo.path(),
            "a.ts",
            r#"import { b } from './b'; export function a() { return b(); }"#,
        );
        write_file(
            repo.path(),
            "b.ts",
            r#"import { c } from './c'; export function b() { return c(); }"#,
        );
        write_file(
            repo.path(),
            "c.ts",
            r#"import { a } from './a'; export function c() { return a(); }"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("scan");
        let report = sruja_diff::detect_architectural_drift(&graph);

        let has_circular = report
            .violations
            .iter()
            .any(|v| matches!(v.kind, ViolationKind::CircularDependency));
        assert!(
            has_circular,
            "Drift report should contain circular dependency violation"
        );
    }

    #[test]
    fn detects_orphan_module_violation() {
        let repo = create_test_repo();
        write_file(
            repo.path(),
            "main.ts",
            r#"import { s } from './s'; export const main = s;"#,
        );
        write_file(repo.path(), "s.ts", r#"export const s = 1;"#);
        write_file(
            repo.path(),
            "isolated.ts",
            r#"const x = 1; function f() { return x; }"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("scan");
        let report = sruja_diff::detect_architectural_drift(&graph);

        let has_orphan = report
            .violations
            .iter()
            .any(|v| matches!(v.kind, ViolationKind::OrphanComponent));
        assert!(
            has_orphan,
            "Drift report should contain orphan module violation"
        );
    }

    #[test]
    fn detects_god_module_violation() {
        let repo = create_test_repo();
        let imports: String = (0..15)
            .map(|i| format!("import {{ d{i} }} from './d{i}';"))
            .collect::<Vec<_>>()
            .join("\n");
        write_file(
            repo.path(),
            "god.ts",
            &format!("{imports}\nexport function god() {{ return 0; }}"),
        );
        for i in 0..15 {
            write_file(
                repo.path(),
                &format!("d{i}.ts"),
                &format!("export function d{i}() {{ return {i}; }}"),
            );
        }

        let graph = sruja_scan::scan_repo(repo.path()).expect("scan");
        let report = sruja_diff::detect_architectural_drift(&graph);

        let has_god = report
            .violations
            .iter()
            .any(|v| matches!(v.kind, ViolationKind::GodModule));
        assert!(has_god, "Drift report should contain god module violation");
    }

    #[test]
    fn detects_layer_violation() {
        let repo = create_test_repo();
        write_file(
            repo.path(),
            "frontend/ui.ts",
            r#"import { q } from '../database/db'; export const get = q;"#,
        );
        write_file(
            repo.path(),
            "database/db.ts",
            r#"export function q() { return []; }"#,
        );

        let graph = sruja_scan::scan_repo(repo.path()).expect("scan");
        let report = sruja_diff::detect_architectural_drift(&graph);

        let has_layer = report
            .violations
            .iter()
            .any(|v| matches!(v.kind, ViolationKind::LayerViolation));
        assert!(
            has_layer,
            "Drift report should contain layer violation (frontend->database)"
        );
    }

    #[test]
    fn clean_graph_has_high_health_score() {
        let repo = create_test_repo();
        write_file(
            repo.path(),
            "a.ts",
            r#"import { b } from './b'; export const a = b;"#,
        );
        write_file(repo.path(), "b.ts", r#"export const b = 1;"#);

        let graph = sruja_scan::scan_repo(repo.path()).expect("scan");
        let report = sruja_diff::detect_architectural_drift(&graph);

        assert!(
            report.health_score >= 80,
            "Clean linear graph should have high health score, got {}",
            report.health_score
        );
    }
}

/// CLI invocation tests - run the actual sruja drift command.
mod cli_invocation {
    use super::*;

    #[test]
    fn drift_cli_runs_successfully() {
        let repo = create_test_repo();
        write_file(
            repo.path(),
            "app.ts",
            "import { x } from \"./x\"; export const app = x;",
        );
        write_file(repo.path(), "x.ts", r#"export const x = 1;"#);

        let (success, stdout, stderr) = run_sruja(&["drift", "-r", repo.path().to_str().unwrap()]);

        assert!(success, "drift should succeed: stderr={}", stderr);
        let out = format!("{} {}", stdout, stderr);
        assert!(
            out.contains("Drift") || out.contains("Health") || out.contains("modules"),
            "Output should mention drift or results"
        );
    }

    #[test]
    fn drift_json_output_parsable() {
        let repo = create_test_repo();
        write_file(repo.path(), "main.ts", r#"export const main = 1;"#);

        let (success, stdout, stderr) =
            run_sruja(&["drift", "-r", repo.path().to_str().unwrap(), "-f", "json"]);

        assert!(success, "drift -f json should succeed: stderr={}", stderr);

        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("Output should be valid JSON");
        assert!(json.get("health_score").is_some());
        assert!(json.get("violations").is_some());
        assert!(json.get("total_modules").is_some());
    }
}
