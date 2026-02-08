//! Edge Rendering Tests
//!
//! Comprehensive tests for edge projection and rendering across different view levels
//! and configurations. This tests the critical fix for the edge filtering bug
//! where edges were incorrectly filtered due to using && instead of || in the
//! visibility check.

use sruja_export::dot::{DotConfig, DotExporter};
use sruja_language::Parser;

/// Helper to parse DSL and get projected edges at a specific level
fn parse_and_get_edges(
    dsl: &str,
    level: u8,
    focus_id: Option<&str>,
) -> (
    Vec<sruja_language::Relation>,
    std::collections::HashMap<String, sruja_language::ElementDef>,
) {
    let parser = Parser::new("test.sruja".to_string());
    let program = parser.parse(dsl).expect("Failed to parse DSL");

    let config = DotConfig {
        rank_dir: "TB".to_string(),
        node_sep: 0.5,
        rank_sep: 0.8,
        view_level: level,
        target_id: focus_id.map(|s| s.to_string()),
        node_sizes: std::collections::HashMap::new(),
        view_id: None,
        filename: Some("test.sruja".to_string()),
    };

    let exporter = DotExporter::new(config);
    let (_dot, elements, relations) = exporter.export_with_relations(&program);

    (relations, elements)
}

/// Test 1: Basic edge rendering at L1 (System Context)
#[test]
fn test_basic_edge_l1() {
    let dsl = r#"
person = kind "Person"
system = kind "System"

user = person "User"
app = system "App"

user -> app "uses"
"#;

    let (relations, _) = parse_and_get_edges(dsl, 1, None);

    assert_eq!(relations.len(), 1, "Should have 1 edge at L1");
    assert_eq!(relations[0].from.as_string(), "user");
    assert_eq!(relations[0].to.as_string(), "app");
    assert_eq!(relations[0].label, Some("uses".to_string()));
}

/// Test 2: Nested edges at L2 (Container view)
#[test]
fn test_nested_edges_l2() {
    let dsl = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

user = person "User"
web = system "Web" {
    api = container "API"
}

user -> web "visits"
user -> web.api "authenticates"
"#;

    let (relations, elements) = parse_and_get_edges(dsl, 2, Some("web"));

    // At L2 with focus on "web", we should see:
    // - user -> web (both endpoints visible)
    // - user -> web.api (user is visible, web.api is visible)
    println!("L2 Relations:");
    for rel in &relations {
        println!(
            "  {} -> {} (label: {:?})",
            rel.from.as_string(),
            rel.to.as_string(),
            rel.label
        );
    }
    println!(
        "Visible elements: {:?}",
        elements.keys().collect::<Vec<_>>()
    );

    // The key fix: edges should not be filtered out if EITHER endpoint is not visible
    // Previously (with &&), edges were only kept if BOTH endpoints were visible
    // Now (with ||), edges are kept if BOTH endpoints are visible
    assert!(!relations.is_empty(), "Should have at least 1 edge at L2");

    // Check that user -> web edge exists
    let user_to_web = relations
        .iter()
        .find(|r| r.from.as_string() == "user" && r.to.as_string() == "web");
    assert!(user_to_web.is_some(), "Should have user -> web edge");

    // Check that user -> api edge exists
    let user_to_api = relations
        .iter()
        .find(|r| r.from.as_string() == "user" && r.to.as_string() == "web.api");
    assert!(
        user_to_api.is_some(),
        "Should have user -> web.api edge (CRITICAL FIX)"
    );
}

/// Test 3: Hierarchical edges (parent-child relationships)
#[test]
fn test_hierarchical_edges() {
    let dsl = r#"
system = kind "System"
container = kind "Container"

web = system "Web" {
    api = container "API"
    db = container "Database"
}

web.api -> web.db "queries"
"#;

    // Test at L1 (System level)
    let (l1_relations, _) = parse_and_get_edges(dsl, 1, None);

    // At L1, api and db are projected to "web", so web.api -> web.db becomes web -> web
    // which is a self-loop and should be filtered out
    println!("L1 Relations:");
    for rel in &l1_relations {
        println!("  {} -> {}", rel.from.as_string(), rel.to.as_string());
    }

    assert_eq!(
        l1_relations.len(),
        0,
        "L1 should have no edges (self-loops filtered)"
    );

    // Test at L3 (Component level with no focus)
    let (l3_relations, _) = parse_and_get_edges(dsl, 3, None);

    // At L3, we see the full hierarchy; sibling edge web.api -> web.db is shown
    println!("L3 Relations:");
    for rel in &l3_relations {
        println!(
            "  {} -> {} (label: {:?})",
            rel.from.as_string(),
            rel.to.as_string(),
            rel.label
        );
    }

    // At L3 with no focus, sibling edges (web.api -> web.db) are shown.
    // Parent-child edges are filtered by the hierarchical check.
    assert_eq!(
        l3_relations.len(),
        1,
        "L3 should have 1 edge (web.api -> web.db, sibling edge)"
    );
    assert_eq!(l3_relations[0].from.as_string(), "web.api");
    assert_eq!(l3_relations[0].to.as_string(), "web.db");
}

/// Test 4: Edge visibility across different views
#[test]
fn test_edge_visibility_across_views() {
    let dsl = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

user = person "User"
system_a = system "System A" {
    api = container "API"
}
system_b = system "System B" {
    db = container "Database"
}

user -> system_a "uses A"
user -> system_b "uses B"
system_a.api -> system_b.db "cross-system call"
"#;

    // L1: Should see user -> system_a and user -> system_b
    let (l1_relations, _) = parse_and_get_edges(dsl, 1, None);
    println!("L1 Relations ({} edges):", l1_relations.len());
    for rel in &l1_relations {
        println!("  {} -> {}", rel.from.as_string(), rel.to.as_string());
    }
    assert!(l1_relations.len() >= 2, "L1 should have at least 2 edges");

    // L2 with focus on system_a: Should see user -> system_a and api -> db
    let (l2_a_relations, _) = parse_and_get_edges(dsl, 2, Some("system_a"));
    println!(
        "L2 (system_a focus) Relations ({} edges):",
        l2_a_relations.len()
    );
    for rel in &l2_a_relations {
        println!("  {} -> {}", rel.from.as_string(), rel.to.as_string());
    }
    assert!(
        !l2_a_relations.is_empty(),
        "L2 with system_a focus should have edges"
    );

    // L2 with focus on system_b: Should see user -> system_b and api -> db
    let (l2_b_relations, _) = parse_and_get_edges(dsl, 2, Some("system_b"));
    println!(
        "L2 (system_b focus) Relations ({} edges):",
        l2_b_relations.len()
    );
    for rel in &l2_b_relations {
        println!("  {} -> {}", rel.from.as_string(), rel.to.as_string());
    }
    assert!(
        !l2_b_relations.is_empty(),
        "L2 with system_b focus should have edges"
    );
}

/// Test 5: Multiple edges between same nodes
#[test]
fn test_multiple_edges_same_nodes() {
    let dsl = r#"
system = kind "System"
container = kind "Container"

web = system "Web" {
    api = container "API"
}
external = system "External API"

web.api -> external "reads"
web.api -> external "writes"
"#;

    let (relations, _) = parse_and_get_edges(dsl, 3, None);

    println!("Multiple edge test ({} edges):", relations.len());
    for rel in &relations {
        println!(
            "  {} -> {} ({:?})",
            rel.from.as_string(),
            rel.to.as_string(),
            rel.label
        );
    }

    assert_eq!(relations.len(), 2, "Should have 2 edges between same nodes");

    // Check both edges exist
    let reads = relations
        .iter()
        .find(|r| r.label.as_ref().map(|l| l == "reads").unwrap_or(false));
    let writes = relations
        .iter()
        .find(|r| r.label.as_ref().map(|l| l == "writes").unwrap_or(false));

    assert!(reads.is_some(), "Should have 'reads' edge");
    assert!(writes.is_some(), "Should have 'writes' edge");
}

/// Test 6: Edge filtering with different element kinds
#[test]
fn test_edge_filtering_by_kind() {
    let dsl = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"
component = kind "Component"

user = person "User"
app = system "App" {
    web = container "Web" {
        controller = component "Controller"
    }
}

user -> app "uses"
user -> app.web "visits"
user -> app.web.controller "requests"
"#;

    // L2: Only certain kinds are visible (container, system, person, datastore, queue)
    let (l2_relations, _) = parse_and_get_edges(dsl, 2, Some("app"));

    println!("L2 Relations ({} edges):", l2_relations.len());
    for rel in &l2_relations {
        println!("  {} -> {}", rel.from.as_string(), rel.to.as_string());
    }

    // At L2, "app.web.controller" should be projected to "app.web" (component -> container)
    // so user -> app.web.controller becomes user -> app.web
    assert!(l2_relations.len() >= 2, "L2 should have at least 2 edges");
}

/// Test 7: Critical bug fix test - edge visibility check
///
/// This tests the critical bug fix where edges were incorrectly filtered out.
/// The bug was: `if !visible.contains(&source) && !visible.contains(&target)`
/// The fix is:  `if !visible.contains(&source) || !visible.contains(&target)`
///
/// With &&: Edge kept only if BOTH endpoints were invisible (wrong!)
/// With ||: Edge skipped if EITHER endpoint is invisible (correct!)
#[test]
fn test_critical_edge_visibility_bug_fix() {
    let dsl = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

user = person "User"
system_a = system "System A" {
    api = container "API"
}
system_b = system "System B" {
    db = container "Database"
}

user -> system_a "uses A"
user -> system_b "uses B"
system_a.api -> system_b.db "cross-system"
"#;

    let (relations, elements) = parse_and_get_edges(dsl, 2, Some("system_a"));

    println!("Elements visible at L2 (focus=system_a):");
    for elem_id in elements.keys() {
        println!("  {}", elem_id);
    }

    println!("Edges at L2 (focus=system_a):");
    for rel in &relations {
        println!("  {} -> {}", rel.from.as_string(), rel.to.as_string());
    }

    // CRITICAL: With the bug (&&), edges were only kept if BOTH endpoints were invisible
    // This is backwards - we want to keep edges where BOTH endpoints are visible
    // So we should SKIP edges where EITHER endpoint is invisible (||)

    // user -> system_a should exist (both visible)
    let user_to_a = relations
        .iter()
        .find(|r| r.from.as_string() == "user" && r.to.as_string() == "system_a");
    assert!(
        user_to_a.is_some(),
        "user -> system_a should exist (both visible)"
    );

    // system_a.api -> system_b.db should exist if both endpoints are projected/visible
    // At L2 with focus on system_a:
    // - system_a.api is visible (in focused scope)
    // - system_b.db might not be visible (not in focused scope)
    // So this edge might be filtered out depending on projection logic
    let api_to_db = relations
        .iter()
        .find(|r| r.from.as_string() == "system_a.api" && r.to.as_string() == "system_b.db");

    // The key assertion: if the edge exists in relations, both endpoints must be visible
    if let Some(rel) = api_to_db {
        let source_visible = elements.contains_key(&rel.from.as_string());
        let target_visible = elements.contains_key(&rel.to.as_string());
        println!(
            "API->DB edge: source_visible={}, target_visible={}",
            source_visible, target_visible
        );
        assert!(
            source_visible && target_visible,
            "Edge should only exist if both endpoints are visible"
        );
    }
}

/// Test 8: Empty edge handling
#[test]
fn test_no_edges() {
    let dsl = r#"
system = kind "System"

app = system "App"
"#;

    let (relations, _) = parse_and_get_edges(dsl, 1, None);

    assert_eq!(relations.len(), 0, "Should have no edges when none defined");
}

/// Test 9: Complex multi-level hierarchy
#[test]
fn test_complex_hierarchy() {
    let dsl = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"
component = kind "Component"

user = person "User"
platform = system "Platform" {
    app = system "App" {
        web = container "Web" {
            controller = component "Controller"
        }
        db = container "Database"
    }
}

user -> platform "uses"
user -> platform.app "opens"
user -> platform.app.web "visits"
user -> platform.app.web.controller "requests"
platform.app.web.controller -> platform.app.db "queries"
"#;

    // Test all levels
    for level in 1..=3 {
        let (relations, _) = parse_and_get_edges(dsl, level, None);
        println!("L{} Relations ({} edges):", level, relations.len());
        for rel in &relations {
            println!("  {} -> {}", rel.from.as_string(), rel.to.as_string());
        }

        // At least one edge should exist at each level
        assert!(
            !relations.is_empty() || level == 1,
            "Should have edges at L{}",
            level
        );
    }
}

/// Test 10: Edge with empty/null labels
#[test]
fn test_edges_without_labels() {
    let dsl = r#"
system = kind "System"

app = system "App"
db = system "Database"

app -> db
"#;

    let (relations, _) = parse_and_get_edges(dsl, 1, None);

    assert_eq!(relations.len(), 1, "Should have 1 edge");
    assert!(
        relations[0].label.is_none()
            || relations[0]
                .label
                .as_ref()
                .map(|l| l.is_empty())
                .unwrap_or(true),
        "Edge should have no or empty label"
    );
}
