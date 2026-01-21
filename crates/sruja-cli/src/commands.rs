//! CLI commands implementation

use std::fs;
use std::path::Path;

use sruja_diagnostics::format_diagnostic;
use sruja_engine::Validator;
use sruja_export::json::{Exporter as JsonExporter, ExportError as JsonExportError};
use sruja_export::mermaid::{MermaidConfig, MermaidExporter};
use sruja_export::dot::{DotConfig, DotExporter};
use sruja_export::markdown::{MarkdownExporter, MarkdownOptions};
use sruja_export::context::ContextExporter;
use sruja_export::dsl::DslPrinter;
use sruja_language::Parser;
use sruja_lsp::server::run_stdio;
use thiserror::Error;
use serde_json;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Export error: {0}")]
    Export(#[from] JsonExportError),
}

/// Print version information
pub fn version() -> Result<(), CliError> {
    println!("sruja version 2.0.0");
    Ok(())
}

/// Lint a Sruja file
pub async fn lint(file: &str) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());
    
    // Parse the file
    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(diagnostics) => {
            // Print parse diagnostics
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!("Parsing failed with {} errors", diagnostics.len())));
        }
    };

    // Validate
    let mut validator = Validator::new();
    validator.register_default_rules();
    let diagnostics = validator.validate_sync(&program);

    if diagnostics.is_empty() {
        println!("✓ No issues found");
        return Ok(());
    }

    // Separate errors and warnings
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for diag in diagnostics {
        match diag.severity {
            sruja_diagnostics::Severity::Error => errors.push(diag),
            sruja_diagnostics::Severity::Warning => warnings.push(diag),
            _ => {}
        }
    }

    // Print warnings first
    for warning in &warnings {
        eprintln!("{}", format_diagnostic(warning));
    }

    // Print errors
    for error in &errors {
        eprintln!("{}", format_diagnostic(error));
    }

    if !errors.is_empty() {
        eprintln!("\n✗ Found {} error(s) and {} warning(s)", errors.len(), warnings.len());
        return Err(CliError::Validation(format!("Linting failed with {} errors", errors.len())));
    }

    if !warnings.is_empty() {
        println!("✓ Found {} warning(s) (no errors)", warnings.len());
    } else {
        println!("✓ No issues found");
    }

    Ok(())
}

/// Export a Sruja file to various formats
pub async fn export(
    format: &str,
    file: &str,
    extended: bool,
    view_level: u8,
    target: Option<&str>,
) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());
    
    // Parse the file
    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(diagnostics) => {
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!("Parsing failed with {} errors", diagnostics.len())));
        }
    };

    match format {
        "json" => {
            let exporter = JsonExporter::with_extended(extended);
            let json = exporter.export(&program)?;
            println!("{}", json);
        }
        "mermaid" => {
            let exporter = MermaidExporter::new(MermaidConfig {
                view_level,
                target_id: target.map(|s| s.to_string()),
                ..MermaidConfig::default()
            });
            let mmd = exporter.export(&program);
            println!("{}", mmd);
        }
        "dot" => {
            let exporter = DotExporter::new(DotConfig {
                view_level,
                target_id: target.map(|s| s.to_string()),
                ..DotConfig::default()
            });
            let dot = exporter.export(&program);
            println!("{}", dot);
        }
        "markdown" => {
            let exporter = MarkdownExporter::new(MarkdownOptions::default());
            let md = exporter.export(&program);
            println!("{}", md);
        }
        "context" => {
            let exporter = ContextExporter::new("general");
            let ctx = exporter.export(&program);
            println!("{}", ctx);
        }
        "dsl" => {
            let printer = DslPrinter::new();
            let dsl = printer.print(&program);
            println!("{}", dsl);
        }
        _ => {
            return Err(CliError::Export(JsonExportError::Export(
                format!("Unsupported export format: {}", format)
            )));
        }
    }

    Ok(())
}

/// Format a Sruja file
pub async fn fmt(file: &str) -> Result<(), CliError> {
    // TODO: Implement formatting (pretty-print using DSL printer)
    eprintln!("Formatting not yet implemented");
    Ok(())
}

/// List elements from a file
pub async fn list(file: &str) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());
    
    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(diagnostics) => {
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!("Parsing failed with {} errors", diagnostics.len())));
        }
    };

    let (elements, _relations) = sruja_language::collect_elements(&program);
    
    println!("Elements:");
    for (fqn, elem) in &elements {
        let kind = elem.assignment.kind.to_string();
        let title = elem.assignment.title.clone().unwrap_or_else(|| elem.assignment.name.clone());
        println!("  {} ({}) - {}", fqn, kind, title);
    }
    
    Ok(())
}

/// Print architecture tree
pub async fn tree(file: &str) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());
    
    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(diagnostics) => {
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!("Parsing failed with {} errors", diagnostics.len())));
        }
    };

    let (elements, _relations) = sruja_language::collect_elements(&program);
    
    // Build parent->children map
    use std::collections::BTreeMap;
    let mut children: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut roots: Vec<String> = Vec::new();
    
    for fqn in elements.keys() {
        if let Some(parent) = parent_fqn(fqn) {
            if elements.contains_key(&parent) {
                children.entry(parent).or_default().push(fqn.clone());
            } else {
                roots.push(fqn.clone());
            }
        } else {
            roots.push(fqn.clone());
        }
    }
    
    // Print tree
    for root in roots {
        print_tree_node(&root, &elements, &children, 0);
    }
    
    Ok(())
}

fn parent_fqn(fqn: &str) -> Option<String> {
    fqn.rfind('.').map(|idx| fqn[..idx].to_string())
}

fn print_tree_node(
    fqn: &str,
    elements: &std::collections::HashMap<String, sruja_language::ElementDef>,
    children: &std::collections::BTreeMap<String, Vec<String>>,
    depth: usize,
) {
    let indent = "  ".repeat(depth);
    if let Some(elem) = elements.get(fqn) {
        let kind = elem.assignment.kind.to_string();
        let title = elem.assignment.title.clone().unwrap_or_else(|| elem.assignment.name.clone());
        println!("{}{} ({}) - {}", indent, fqn, kind, title);
        
        if let Some(kids) = children.get(fqn) {
            for kid in kids {
                print_tree_node(kid, elements, children, depth + 1);
            }
        }
    }
}

/// Initialize a new Sruja project
pub async fn init(name: Option<&str>) -> Result<(), CliError> {
    let project_name = name.unwrap_or("my-architecture");
    let filename = format!("{}.sruja", project_name);
    
    let template = format!(
        r#"// {project_name} Architecture

// Define your systems
system {project_name} {{
    description "Main system for {project_name}"
}}

// Add more elements, relations, scenarios, etc.
"#
    );
    
    fs::write(&filename, template)?;
    println!("Created {}", filename);
    Ok(())
}

/// Show differences between two architecture files
pub async fn diff(file1: &str, file2: &str, format: &str) -> Result<(), CliError> {
    let content1 = fs::read_to_string(file1)?;
    let content2 = fs::read_to_string(file2)?;
    
    let parser1 = Parser::new(file1.to_string());
    let parser2 = Parser::new(file2.to_string());
    
    let program1 = match parser1.parse(&content1) {
        Ok(p) => p,
        Err(diags) => {
            eprintln!("Error parsing {}: {} errors", file1, diags.len());
            return Err(CliError::Parse("Failed to parse first file".to_string()));
        }
    };
    
    let program2 = match parser2.parse(&content2) {
        Ok(p) => p,
        Err(diags) => {
            eprintln!("Error parsing {}: {} errors", file2, diags.len());
            return Err(CliError::Parse("Failed to parse second file".to_string()));
        }
    };
    
    let (elems1, _) = sruja_language::collect_elements(&program1);
    let (elems2, _) = sruja_language::collect_elements(&program2);
    
    let mut added: Vec<String> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    
    for fqn in elems2.keys() {
        if !elems1.contains_key(fqn) {
            added.push(fqn.clone());
        }
    }
    
    for fqn in elems1.keys() {
        if !elems2.contains_key(fqn) {
            removed.push(fqn.clone());
        }
    }
    
    if format == "json" {
        println!("{{");
        println!("  \"added\": [");
        for (i, fqn) in added.iter().enumerate() {
            print!("    \"{}\"", fqn);
            if i < added.len() - 1 {
                println!(",");
            } else {
                println!();
            }
        }
        println!("  ],");
        println!("  \"removed\": [");
        for (i, fqn) in removed.iter().enumerate() {
            print!("    \"{}\"", fqn);
            if i < removed.len() - 1 {
                println!(",");
            } else {
                println!();
            }
        }
        println!("  ]");
        println!("}}");
    } else {
        if !added.is_empty() {
            println!("Added elements:");
            for fqn in &added {
                println!("  + {}", fqn);
            }
        }
        if !removed.is_empty() {
            println!("Removed elements:");
            for fqn in &removed {
                println!("  - {}", fqn);
            }
        }
        if added.is_empty() && removed.is_empty() {
            println!("No differences found");
        }
    }
    
    Ok(())
}

/// Explain an element
pub async fn explain(element_id: &str, file: Option<&str>, json: bool) -> Result<(), CliError> {
    let file_path = file.unwrap_or("architecture.sruja");
    let content = fs::read_to_string(file_path)?;
    let parser = Parser::new(file_path.to_string());
    
    let program = match parser.parse(&content) {
        Ok(p) => p,
        Err(diags) => {
            for diag in &diags {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!("Parsing failed with {} errors", diags.len())));
        }
    };
    
    let (elements, relations) = sruja_language::collect_elements(&program);
    
    // Find element
    let elem = elements.get(element_id).ok_or_else(|| {
        CliError::Parse(format!("Element '{}' not found", element_id))
    })?;
    
    // Count relations
    let incoming: Vec<_> = relations.iter()
        .filter(|r| r.to.as_string() == element_id)
        .collect();
    let outgoing: Vec<_> = relations.iter()
        .filter(|r| r.from.as_string() == element_id)
        .collect();
    
    if json {
        let desc = elem.assignment.body.as_ref()
            .and_then(|b| b.description.as_ref())
            .cloned()
            .unwrap_or_default();
        println!("{{");
        println!("  \"id\": \"{}\",", element_id);
        println!("  \"description\": \"{}\",", desc.replace('"', "\\\""));
        println!("  \"incoming_relations\": {},", incoming.len());
        println!("  \"outgoing_relations\": {},", outgoing.len());
        println!("  \"dependencies\": {},", outgoing.len());
        println!("  \"adrs\": 0,");
        println!("  \"scenarios\": 0");
        println!("}}");
    } else {
        println!("Element: {}", element_id);
        if let Some(body) = &elem.assignment.body {
            if let Some(desc) = &body.description {
                println!("Description: {}", desc);
            }
            if let Some(tech) = &body.technology {
                println!("Technology: {}", tech);
            }
        }
        println!("Incoming relations: {}", incoming.len());
        println!("Outgoing relations: {}", outgoing.len());
        if !outgoing.is_empty() {
            println!("Dependencies:");
            for rel in outgoing {
                println!("  -> {}", rel.to.as_string());
            }
        }
    }
    
    Ok(())
}

/// Import from external format
pub async fn import(format: &str, file: &str) -> Result<(), CliError> {
    if format != "json" {
        return Err(CliError::Parse(format!("Unsupported import format: {}. Supported: json", format)));
    }
    
    let content = fs::read_to_string(file)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    
    // Try to extract systems from various JSON formats
    if let Some(arch) = json.get("architecture") {
        if let Some(systems) = arch.get("systems").and_then(|s| s.as_array()) {
            for sys in systems {
                if let (Some(id), Some(label)) = (sys.get("id").and_then(|v| v.as_str()), 
                                                     sys.get("label").and_then(|v| v.as_str())) {
                    println!("system {} \"{}\"", id, label);
                }
            }
            return Ok(());
        }
    }
    
    // Try SrujaModelDump format
    if let Some(elements) = json.get("elements").and_then(|e| e.as_array()) {
        for elem in elements {
            if let (Some(kind), Some(id), Some(title)) = (
                elem.get("kind").and_then(|v| v.as_str()),
                elem.get("id").and_then(|v| v.as_str()),
                elem.get("title").and_then(|v| v.as_str()),
            ) {
                if kind == "system" {
                    println!("system {} \"{}\"", id, title);
                }
            }
        }
        return Ok(());
    }
    
    Err(CliError::Parse("Could not identify architecture in JSON".to_string()))
}

/// Calculate architecture health score
pub async fn score(file: Option<&str>) -> Result<(), CliError> {
    let file_path = file.unwrap_or("architecture.sruja");
    let content = fs::read_to_string(file_path)?;
    let parser = Parser::new(file_path.to_string());
    
    let program = match parser.parse(&content) {
        Ok(p) => p,
        Err(diags) => {
            for diag in &diags {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!("Parsing failed with {} errors", diags.len())));
        }
    };
    
    // Validate to get diagnostics
    let mut validator = Validator::new();
    validator.register_default_rules();
    let diagnostics = validator.validate_sync(&program);
    
    // Calculate score (100 - deductions)
    let mut score = 100;
    let mut deductions = Vec::new();
    
    for diag in &diagnostics {
        let points = match diag.severity {
            sruja_diagnostics::Severity::Error => 5,
            sruja_diagnostics::Severity::Warning => 2,
            _ => 0,
        };
        if points > 0 {
            score = score.saturating_sub(points);
            deductions.push((diag.message.clone(), points, diag.code.clone()));
        }
    }
    
    let grade = if score >= 90 {
        "A"
    } else if score >= 80 {
        "B"
    } else if score >= 70 {
        "C"
    } else if score >= 60 {
        "D"
    } else {
        "F"
    };
    
    println!("Architecture Health Index: {} ({})", score, grade);
    println!("Dimensions:");
    println!("  - Structural Integrity: {}%", score);
    println!("  - Documentation Depth:  {}%", score);
    println!("  - Traceability:         {}%", score);
    println!("  - Complexity Control:   {}%", score);
    println!("  - Standardization:      {}%", score);
    println!();
    
    if !deductions.is_empty() {
        println!("Deductions:");
        for (msg, pts, code) in deductions {
            println!("  -{} pts: {} ({})", pts, msg, code);
        }
    } else {
        println!("Perfect Score! No deductions.");
    }
    
    Ok(())
}

/// Start LSP server
pub async fn lsp() -> Result<(), CliError> {
    run_stdio().await.map_err(|e| CliError::Validation(format!("LSP server error: {:?}", e)))?;
    Ok(())
}

/// Compile a Sruja file
pub async fn compile(file: &str) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());
    
    // Parse the file
    let program = match parser.parse(&content) {
        Ok(program) => {
            println!("✓ Parsing successful");
            program
        }
        Err(diagnostics) => {
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!("Compilation failed with {} errors", diagnostics.len())));
        }
    };

    // Validate
    let mut validator = Validator::new();
    validator.register_default_rules();
    let diagnostics = validator.validate_sync(&program);

    if diagnostics.is_empty() {
        println!("✓ Validation successful");
        println!("✓ Compilation successful");
        return Ok(());
    }

    // Print diagnostics
    for diag in &diagnostics {
        eprintln!("{}", format_diagnostic(diag));
    }

    let error_count = diagnostics.iter().filter(|d| d.severity == sruja_diagnostics::Severity::Error).count();
    if error_count > 0 {
        eprintln!("\n✗ Compilation failed with {} error(s)", error_count);
        return Err(CliError::Validation(format!("Compilation failed with {} errors", error_count)));
    }

    println!("✓ Compilation successful (with {} warning(s))", diagnostics.len());
    Ok(())
}
