use sruja_export::dot::{DotConfig, DotExporter};
use sruja_language::Parser;
use std::collections::HashMap;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <sruja-file>", args[0]);
        eprintln!("\nDebugs edge rendering for a Sruja DSL file.");
        eprintln!("Shows which edges are projected and why.");
        std::process::exit(1);
    }

    let filename = &args[1];

    // Read DSL file
    let dsl = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            std::process::exit(1);
        }
    };

    println!("=== EDGE RENDERING DEBUGGER ===\n");
    println!("File: {}", filename);
    println!(
        "DSL Content (first 500 chars):\n{}\n...\n",
        if dsl.len() > 500 { &dsl[..500] } else { &dsl }
    );

    println!("=== PARSED ELEMENTS ===\n");

    let parser = Parser::new(filename.clone());
    let program = match parser.parse(&dsl) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to parse DSL:");
            for diag in &e {
                eprintln!(
                    "  Line {}: {} - {}",
                    diag.location.line, diag.code, diag.message
                );
            }
            std::process::exit(1);
        }
    };

    // Collect all elements and relations
    let (elements, all_relations) = sruja_language::collect_elements(&program);

    println!("Total elements: {}", elements.len());
    for (id, elem) in &elements {
        let kind = elem.assignment.kind.to_string();
        println!("  - {} [kind: {}]", id, kind);
    }

    println!("\nTotal relations in DSL: {}", all_relations.len());
    if all_relations.is_empty() {
        println!("  ⚠️  No relations found in DSL!");
    }
    for rel in &all_relations {
        println!(
            "  - {} -> {} (label: {:?})",
            rel.from.as_string(),
            rel.to.as_string(),
            rel.label
        );
    }

    println!("\n=== TESTING DIFFERENT VIEW LEVELS ===\n");

    // Collect view definitions from program
    let view_ids: Vec<String> = program
        .items
        .iter()
        .filter_map(|item| {
            if let sruja_language::TopLevelItem::View(view_def) = item {
                Some(view_def.id.clone())
            } else {
                None
            }
        })
        .collect();

    if !view_ids.is_empty() {
        println!(
            "Found {} view definition(s): {}",
            view_ids.len(),
            view_ids.join(", ")
        );
    } else {
        println!("No view definitions found in DSL");
    }

    // Test L1 (System Context)
    println!("\n--- LEVEL 1 (System Context) ---");
    test_level(&program, 1, None, None);

    // Test L2 (Container view with focus on first system)
    if let Some(first_system) = elements.iter().find(|(_, e)| {
        let kind_str = e.assignment.kind.to_string();
        kind_str.to_lowercase() == "system"
    }) {
        println!(
            "\n--- LEVEL 2 (Container view, focus={}) ---",
            first_system.0
        );
        test_level(&program, 2, Some(first_system.0), None);
    } else {
        println!("\n--- LEVEL 2 (Container view, no focus) ---");
        test_level(&program, 2, None, None);
    }

    // Test L3 (Component view, no focus)
    println!("\n--- LEVEL 3 (Component view, no focus) ---");
    test_level(&program, 3, None, None);

    // Test with each custom view definition
    for view_id in &view_ids {
        println!("\n--- CUSTOM VIEW: {} ---", view_id);
        test_level(&program, 1, None, Some(view_id));
    }

    println!("\n=== ANALYSIS ===\n");
    analyze_projection(&program, &elements, &all_relations);
}

fn test_level(
    program: &sruja_language::Program,
    level: u8,
    focus_id: Option<&str>,
    view_id: Option<&str>,
) {
    let config = DotConfig {
        rank_dir: "TB".to_string(),
        node_sep: 0.5,
        rank_sep: 0.8,
        view_level: level,
        target_id: focus_id.map(|s| s.to_string()),
        node_sizes: HashMap::new(),
        view_id: view_id.map(|s| s.to_string()),
        filename: Some("debug.sruja".to_string()),
    };

    let exporter = DotExporter::new(config);
    let (dot, elements, relations) = exporter.export_with_relations(program);

    println!(
        "Level: {}, Focus: {:?}, View: {:?}",
        level, focus_id, view_id
    );
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

    // Check DOT output for edge declarations
    let edge_count_in_dot = dot.matches("->").count();
    println!("Edge declarations in DOT: {}", edge_count_in_dot);

    if edge_count_in_dot > 0 && relations.is_empty() {
        println!("⚠️  WARNING: DOT has edges but relations array is empty!");
        println!("   This indicates a bug in export_with_relations()");
    } else if edge_count_in_dot == 0 && relations.len() > 0 {
        println!("⚠️  WARNING: Relations array has edges but DOT has none!");
        println!("   This indicates a bug in DOT generation");
    }

    // Show first 200 chars of DOT output
    println!("\nDOT output (first 300 chars):");
    let preview = if dot.len() > 300 { &dot[..300] } else { &dot };
    println!("{}...", preview);
}

fn analyze_projection(
    program: &sruja_language::Program,
    elements: &HashMap<String, sruja_language::ElementDef>,
    all_relations: &[sruja_language::Relation],
) {
    println!("Analyzing why edges might not render...\n");

    // Check for common issues

    // Issue 1: Edges where endpoints have different visibility levels
    println!("1. Checking edge endpoint visibility at each level:");

    for level in 1..=3 {
        let config = DotConfig {
            rank_dir: "TB".to_string(),
            node_sep: 0.5,
            rank_sep: 0.8,
            view_level: level,
            target_id: None,
            node_sizes: HashMap::new(),
            view_id: None, // Keep None for default level-based testing
            filename: Some("debug.sruja".to_string()),
        };

        let exporter = DotExporter::new(config);
        let (_dot, visible_elements, projected_relations) = exporter.export_with_relations(program);

        let visible_set: std::collections::HashSet<String> =
            visible_elements.keys().cloned().collect();

        println!(
            "\n  Level {} (visible: {} elements):",
            level,
            visible_elements.len()
        );

        for rel in all_relations {
            let from = rel.from.as_string();
            let to = rel.to.as_string();

            let from_kind = elements.get(&from).map(|e| e.assignment.kind.to_string());
            let to_kind = elements.get(&to).map(|e| e.assignment.kind.to_string());

            let from_visible = visible_set.contains(&from);
            let to_visible = visible_set.contains(&to);

            let is_projected = projected_relations
                .iter()
                .any(|r| r.from.as_string() == from && r.to.as_string() == to);

            if !from_visible || !to_visible {
                println!(
                    "    {} -> {} [from: {:?}, to: {:?}]",
                    from, to, from_kind, to_kind
                );
                println!("      Visible: from={}, to={}", from_visible, to_visible);

                if !is_projected {
                    println!("      ⚠️  NOT PROJECTED - endpoint(s) invisible");
                }
            }

            // Check for hierarchical relationships
            if from.contains('.') || to.contains('.') {
                if from.starts_with(&format!("{}.", to)) || to.starts_with(&format!("{}.", from)) {
                    println!("    ⚠️  {} -> {} is HIERARCHICAL (parent-child)", from, to);
                    if is_projected {
                        println!("      ERROR: Should have been filtered out!");
                    } else {
                        println!("      Correctly filtered out");
                    }
                }
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

    // Check if any edges involve non-visible kinds at L2
    let config = DotConfig {
        rank_dir: "TB".to_string(),
        node_sep: 0.5,
        rank_sep: 0.8,
        view_level: 2,
        target_id: None,
        node_sizes: HashMap::new(),
        view_id: None, // Keep None for default level-based testing
        filename: Some("debug.sruja".to_string()),
    };

    let exporter = DotExporter::new(config);
    let (_dot, visible_elements, _projected_relations) = exporter.export_with_relations(program);

    for rel in all_relations {
        let from = rel.from.as_string();
        let to = rel.to.as_string();

        let from_kind = elements
            .get(&from)
            .map(|e| e.assignment.kind.to_string().to_lowercase());
        let to_kind = elements
            .get(&to)
            .map(|e| e.assignment.kind.to_string().to_lowercase());

        let from_visible = visible_elements.contains_key(&from);
        let to_visible = visible_elements.contains_key(&to);

        if !from_visible && from_kind.is_some() {
            println!(
                "   {} -> {} blocked by kind: {}",
                from,
                to,
                from_kind.unwrap()
            );
        }
        if !to_visible && to_kind.is_some() {
            println!(
                "   {} -> {} blocked by kind: {}",
                from,
                to,
                to_kind.unwrap()
            );
        }
    }

    // Issue 3: Self-loops
    println!("\n3. Checking for self-loops:");
    let mut has_self_loops = false;
    for rel in all_relations {
        if rel.from.as_string() == rel.to.as_string() {
            println!(
                "   ⚠️  Self-loop detected: {} -> {}",
                rel.from.as_string(),
                rel.to.as_string()
            );
            has_self_loops = true;
        }
    }
    if !has_self_loops {
        println!("   ✓ No self-loops found");
    }

    // Issue 4: Empty or invalid relations
    println!("\n4. Checking for empty relations:");
    if all_relations.is_empty() {
        println!("   ⚠️  No relations defined in DSL!");
        println!("   This is the root cause - edges won't render without relations.");
    } else {
        println!("   ✓ {} relations found", all_relations.len());
    }

    // Issue 5: Projection to empty strings
    println!("\n5. Checking for projection to empty strings:");
    for rel in all_relations {
        let from = rel.from.as_string();
        let to = rel.to.as_string();

        // Check if any parent element exists (affects projection)
        if from.contains('.') {
            let parent = from.rsplit('.').nth(1);
            println!("   {} has parent: {:?}", from, parent);
        }
        if to.contains('.') {
            let parent = to.rsplit('.').nth(1);
            println!("   {} has parent: {:?}", to, parent);
        }
    }

    println!("\n=== RECOMMENDATIONS ===\n");

    if all_relations.is_empty() {
        println!("1. ⚠️  No relations in DSL - add edges using '->' syntax:");
        println!("   Example: user -> app \"uses\"");
    }

    let mut any_edges_at_any_level = false;
    for level in 1..=3 {
        let config = DotConfig {
            rank_dir: "TB".to_string(),
            node_sep: 0.5,
            rank_sep: 0.8,
            view_level: level,
            target_id: None,
            node_sizes: HashMap::new(),
            view_id: None, // Keep None for default level-based testing
            filename: Some("debug.sruja".to_string()),
        };

        let exporter = DotExporter::new(config);
        let (_dot, _elements, relations) = exporter.export_with_relations(program);
        if !relations.is_empty() {
            any_edges_at_any_level = true;
            break;
        }
    }

    if !any_edges_at_any_level && !all_relations.is_empty() {
        println!("2. ⚠️  Edges exist but are being filtered out at all levels:");
        println!("   - Check if endpoints are visible at desired level");
        println!("   - Check if edges are hierarchical (parent-child)");
        println!("   - Check if element kinds are visible at L2");
    }

    if any_edges_at_any_level {
        println!("3. ✓ Edges exist at some level - check if viewing correct level");
        println!("   Use focus to drill down into nested elements");
    }
}
