use sruja_language::Parser;

fn main() {
    let dsl = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

user = person "User"
web = system "Web App" {
    api = container "API"
}

view index {
    title "Test View"
    include *
}

view specific {
    title "Specific View"
    include web, user
}

view recursive {
    title "Recursive View"
    include web.*
}
"#;

    println!("=== VIEW PARSING TEST ===\n");
    println!("DSL to parse:");
    println!("{}", dsl);
    println!();

    let parser = Parser::new("test.sruja".to_string());
    match parser.parse(dsl) {
        Ok(program) => {
            println!("✅ Parse successful!\n");

            // Find all view definitions
            let views: Vec<_> = program
                .items
                .iter()
                .filter_map(|item| {
                    if let sruja_language::TopLevelItem::View(view_def) = item {
                        Some(view_def)
                    } else {
                        None
                    }
                })
                .collect();

            if views.is_empty() {
                println!("❌ No view definitions found!");
            } else {
                println!("📋 Found {} view definition(s):\n", views.len());

                for (i, view_def) in views.iter().enumerate() {
                    println!("{}. View: {}", i + 1, view_def.id);
                    println!("   Title: {:?}", view_def.title);
                    println!("   Description: {:?}", view_def.description);
                    println!("   View of: {:?}", view_def.view_of);
                    println!("   Tags: {:?}", view_def.tags);
                    println!("   Rules: {} rule(s)", view_def.rules.len());

                    for (j, rule) in view_def.rules.iter().enumerate() {
                        println!("\n   {}. Rule {}:", j + 1, i + 1);

                        if let Some(ref include_expr) = rule.include {
                            println!("      INCLUDE:");
                            println!("      - Wildcard: {}", include_expr.wildcard);
                            println!("      - Recursive: {}", include_expr.recursive);
                            println!("      - Elements: {:?}", include_expr.elements);
                        }

                        if let Some(ref exclude_expr) = rule.exclude {
                            println!("      EXCLUDE:");
                            println!("      - Wildcard: {}", exclude_expr.wildcard);
                            println!("      - Recursive: {}", exclude_expr.recursive);
                            println!("      - Elements: {:?}", exclude_expr.elements);
                        }

                        if rule.include.is_none() && rule.exclude.is_none() {
                            println!("      (Empty rule)");
                        }
                    }

                    if view_def.rules.is_empty() {
                        println!("   ⚠️  NO RULES FOUND - This is the bug!");
                    }

                    println!();
                }
            }
        }
        Err(diagnostics) => {
            println!("❌ Parse failed!\n");
            for diag in &diagnostics {
                println!(
                    "  [{}] Line {}: {}",
                    diag.code, diag.location.line, diag.message
                );
            }
            std::process::exit(1);
        }
    }
}
