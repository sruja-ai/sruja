use sruja_export::dot::{DotConfig, DotExporter};
use sruja_language::Parser;
use std::collections::HashMap;

fn main() {
    // Test DSL with nested edges
    let dsl = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

user = person "User"
web = system "Web App" {
    api = container "API"
    db = container "Database"
}

user -> web "uses"
user -> web.api "authenticates"
web.api -> web.db "queries"
"#;

    println!("=== EDGE RENDERING DEBUGGER ===\n");
    println!("DSL Content:");
    println!("{}", dsl);
    println!("\n=== PARSED ELEMENTS ===\n");

    let parser = Parser::new("debug.sruja".to_string());
    let program = match parser.parse(dsl) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to parse DSL:");
            for diag in &e {
                eprintln!("  {}: {}", diag.code, diag.message);
            }
            std::process::exit(1);
        }
    };

    // Collect all elements and relations
    let (elements, all_relations) = sruja_language::collect_elements(&program);

    println!("Total elements: {}", elements.len());
    for (id, elem) in &elements {
        let kind = elem.assignment.kind.to_string();
        println!("  - {} [{}]", id, kind);
    }

    println!("\nTotal relations in DSL: {}", all_relations.len());
    for rel in &all_relations {
        println!(
            "  - {} -> {} (label: {:?})",
            rel.from.as_string(),
            rel.to.as_string(),
            rel.label
        );
    }

    println!("\n=== TESTING DIFFERENT VIEW LEVELS ===\n");

    // Test L1 (System Context)
    println!("--- LEVEL 1 (System Context) ---");
    test_level(&program, 1, None);

    // Test L2 (Container view with focus on web)
    println!("\n--- LEVEL 2 (Container view, focus=web) ---");
    test_level(&program, 2, Some("web"));

    // Test L3 (Component view, no focus)
    println!("\n--- LEVEL 3 (Component view, no focus) ---");
    test_level(&program, 3, None);

    println!("\n=== ANALYSIS ===\n");
    analyze_projection(&program);
}

fn test_level(program: &sruja_language::Program, level: u8, focus_id: Option<&str>) {
    let config = DotConfig {
        rank_dir: "TB".to_string(),
        node_sep: 0.5,
        rank_sep: 0.8,
        view_level: level,
        target_id: focus_id.map(|s| s.to_string()),
        node_sizes: HashMap::new(),
        view_id: None,
        filename: Some("debug.sruja".to_string()),
    };

    let exporter = DotExporter::new(config);
    let (dot, elements, relations) = exporter.export_with_relations(program);

    println!("Focus: {:?}", focus_id);
    println!("Visible elements: {}", elements.len());
    for id in elements.keys() {
        println!("  - {}", id);
    }

    println!("Projected edges: {}", relations.len());
    for rel in &relations {
        println!(
            "  - {} -> {} (label: {:?})",
            rel.from.as_string(),
            rel.to.as_string(),
            rel.label
        );
    }

    if relations.is_empty() {
        println!("⚠️  WARNING: No edges projected at this level!");
    } else {
        println!("✓ Edges found");
    }

    // Show first 100 chars of DOT output
    println!("\nDOT output (first 200 chars):");
    let preview = if dot.len() > 200 { &dot[..200] } else { &dot };
    println!("{}...", preview);
}

fn analyze_projection(program: &sruja_language::Program) {
    let (elements, all_relations) = sruja_language::collect_elements(program);

    println!("Analyzing why edges might not render...\n");

    // Check for common issues

    // Issue 1: Edges where endpoints have different visibility levels
    println!("1. Checking edge endpoint visibility:");
    for rel in &all_relations {
        let from = rel.from.as_string();
        let to = rel.to.as_string();

        let from_kind = elements.get(&from).map(|e| e.assignment.kind.to_string());
        let to_kind = elements.get(&to).map(|e| e.assignment.kind.to_string());

        println!(
            "   {} -> {} [from: {:?}, to: {:?}]",
            from, to, from_kind, to_kind
        );

        // Check for nested relationships
        if from.contains('.') || to.contains('.') {
            if from.starts_with(&format!("{}.", to)) || to.starts_with(&format!("{}.", from)) {
                println!("      ⚠️  HIERARCHICAL EDGE (parent-child) - will be filtered");
            }
        }
    }

    // Issue 2: Element kind filtering at L2
    println!("\n2. Checking L2 kind filtering:");
    println!("   At L2, only these kinds are visible:");
    let visible_kinds = ["container", "datastore", "queue", "system", "person"];
    for kind in visible_kinds {
        println!("     - {}", kind);
    }

    println!("   Edges with non-visible kinds will be filtered out.");

    // Issue 3: Self-loops
    println!("\n3. Checking for self-loops:");
    for rel in &all_relations {
        if rel.from.as_string() == rel.to.as_string() {
            println!(
                "   ⚠️  Self-loop detected: {} -> {}",
                rel.from.as_string(),
                rel.to.as_string()
            );
        }
    }
}
