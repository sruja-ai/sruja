//! CLI commands implementation

use std::fs;
use std::path::Path;

use sruja_diagnostics::format_diagnostic;
use sruja_engine::Validator;
use sruja_export::context::ContextExporter;
use sruja_export::dsl::DslPrinter;
use sruja_export::json::exporter::{ExportError as JsonExportError, Exporter as JsonExporter};
use sruja_export::markdown::{MarkdownExporter, MarkdownOptions};
use sruja_export::mermaid::exporter::{MermaidConfig, MermaidExporter};
use sruja_graph::{merge_scan_into_graph, KnowledgeGraph};
use sruja_language::Parser;
use sruja_lsp::server::run_stdio;
use sruja_scan::{scan_repo, Graph, NodeKind};
use thiserror::Error;

use crate::modules::collect_sruja_files;
use crate::modules::validation::enrich_diagnostics_with_source;

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
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Scan error: {0}")]
    Scan(String),
    #[error("Drift error: {0}")]
    #[allow(dead_code)]
    Drift(String),
}

impl From<sruja_scan::ScanError> for CliError {
    fn from(e: sruja_scan::ScanError) -> Self {
        CliError::Scan(e.to_string())
    }
}

/// Print version information
pub fn version() -> Result<(), CliError> {
    println!("sruja version 2.0.0");
    Ok(())
}

/// Scan a repository and emit an inferred architecture graph (JSON).
pub async fn scan(repo_root: &str, output: &str) -> Result<(), CliError> {
    let graph =
        sruja_scan::scan_repo(Path::new(repo_root)).map_err(|e| CliError::Scan(e.to_string()))?;

    let json = serde_json::to_string_pretty(&graph)?;

    if output == "-" {
        println!("{}", json);
        return Ok(());
    }

    fs::write(output, json)?;
    println!("Wrote {}", output);
    Ok(())
}

/// Collect file references from scan graph edges for evidence display.
fn collect_file_evidence_from_scan(scan_graph: &Graph) -> Vec<String> {
    let mut files: std::collections::HashSet<String> = std::collections::HashSet::new();
    for edge in &scan_graph.edges {
        for ev in &edge.evidence {
            if let Some(ref f) = ev.file {
                files.insert(f.clone());
            }
        }
    }
    let mut v: Vec<_> = files.into_iter().collect();
    v.sort();
    v
}

/// Ask "why" questions about architecture
pub async fn why(question: &str, repo: &str, graph_file: Option<&str>) -> Result<(), CliError> {
    let mut kg = KnowledgeGraph::new();
    let scan_graph: Graph = if let Some(path) = graph_file {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| CliError::Json(e))?
    } else {
        scan_repo(Path::new(repo))?
    };

    let repo_path = graph_file.unwrap_or(repo);
    merge_scan_into_graph(&mut kg, &scan_graph, repo_path);

    match kg.query(question) {
        Ok(result) => {
            println!("{}\n", result.answer);
            println!("Confidence: {}%", (result.confidence * 100.0) as i32);
            if !result.evidence.is_empty() {
                println!("\nEvidence:");
                for ev in &result.evidence {
                    println!("  - {}", ev.excerpt);
                }
            }
            let file_refs = collect_file_evidence_from_scan(&scan_graph);
            if !file_refs.is_empty()
                && (question.to_lowercase().contains("depend")
                    || question.to_lowercase().contains("connect")
                    || question.to_lowercase().contains("how")
                    || question.to_lowercase().contains("why"))
            {
                println!("\nFile references (from scan):");
                for f in file_refs.iter().take(10) {
                    println!("  - {}", f);
                }
                if file_refs.len() > 10 {
                    println!("  ... and {} more", file_refs.len() - 10);
                }
            }
        }
        Err(e) => {
            return Err(CliError::Validation(format!("No answer found: {}", e)));
        }
    }

    Ok(())
}

/// Lint a Sruja file
pub async fn lint(file: &str) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());

    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(mut diagnostics) => {
            enrich_diagnostics_with_source(&content, &mut diagnostics);
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!(
                "Parsing failed with {} errors",
                diagnostics.len()
            )));
        }
    };

    let validator = Validator::with_default_rules();
    let mut diagnostics = validator.validate_sync(&program);
    enrich_diagnostics_with_source(&content, &mut diagnostics);

    if diagnostics.is_empty() {
        println!("✓ No issues found");
        return Ok(());
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for diag in diagnostics {
        match diag.severity {
            sruja_diagnostics::Severity::Error => errors.push(diag),
            sruja_diagnostics::Severity::Warning => warnings.push(diag),
            _ => {}
        }
    }

    for warning in &warnings {
        eprintln!("{}", format_diagnostic(warning));
    }

    for error in &errors {
        eprintln!("{}", format_diagnostic(error));
    }

    if !errors.is_empty() {
        eprintln!(
            "\n✗ Found {} error(s) and {} warning(s)",
            errors.len(),
            warnings.len()
        );
        return Err(CliError::Validation(format!(
            "Linting failed with {} errors",
            errors.len()
        )));
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

    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(mut diagnostics) => {
            enrich_diagnostics_with_source(&content, &mut diagnostics);
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!(
                "Parsing failed with {} errors",
                diagnostics.len()
            )));
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
                direction: "LR".to_string(),
                view_level,
                target_id: target.map(|s| s.to_string()),
            });
            let mmd = exporter.export(&program);
            println!("{}", mmd);
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
            return Err(CliError::Export(JsonExportError::Export(format!(
                "Unsupported export format: {}",
                format
            ))));
        }
    }

    Ok(())
}

/// Format a Sruja file
pub async fn fmt(file: &str, check: bool) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());

    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(mut diagnostics) => {
            enrich_diagnostics_with_source(&content, &mut diagnostics);
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!(
                "Formatting failed: file has {} parse errors",
                diagnostics.len()
            )));
        }
    };

    let printer = DslPrinter::new();
    let formatted = printer.print(&program);

    if formatted != content {
        if check {
            println!("Would reformat {}", file);
            return Err(CliError::Validation(format!(
                "File {} needs formatting",
                file
            )));
        } else {
            fs::write(file, formatted)?;
            println!("Formatted {}", file);
        }
    } else {
        println!("{} is already formatted", file);
    }

    Ok(())
}

/// List elements from a file
pub async fn list_elements(file: &str) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());

    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(mut diagnostics) => {
            enrich_diagnostics_with_source(&content, &mut diagnostics);
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!(
                "Parsing failed with {} errors",
                diagnostics.len()
            )));
        }
    };

    let (elements, _relations) = sruja_language::collect_elements(&program);

    println!("Elements:");
    for (fqn, elem) in &elements {
        let kind = elem.assignment.kind.to_string();
        let title = elem
            .assignment
            .title
            .clone()
            .unwrap_or_else(|| elem.assignment.name.clone());
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
        Err(mut diagnostics) => {
            enrich_diagnostics_with_source(&content, &mut diagnostics);
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!(
                "Parsing failed with {} errors",
                diagnostics.len()
            )));
        }
    };

    let (elements, _relations) = sruja_language::collect_elements(&program);

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
        let title = elem
            .assignment
            .title
            .clone()
            .unwrap_or_else(|| elem.assignment.name.clone());
        println!("{}{} ({}) - {}", indent, fqn, kind, title);

        if let Some(kids) = children.get(fqn) {
            for kid in kids {
                print_tree_node(kid, elements, children, depth + 1);
            }
        }
    }
}

/// Show differences between two architecture files
pub async fn diff(file1: &str, file2: &str, format: &str) -> Result<(), CliError> {
    let content1 = fs::read_to_string(file1)?;
    let content2 = fs::read_to_string(file2)?;

    let parser1 = Parser::new(file1.to_string());
    let parser2 = Parser::new(file2.to_string());

    let program1 = match parser1.parse(&content1) {
        Ok(p) => p,
        Err(mut diags) => {
            enrich_diagnostics_with_source(&content1, &mut diags);
            for diag in &diags {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse("Failed to parse first file".to_string()));
        }
    };

    let program2 = match parser2.parse(&content2) {
        Ok(p) => p,
        Err(mut diags) => {
            enrich_diagnostics_with_source(&content2, &mut diags);
            for diag in &diags {
                eprintln!("{}", format_diagnostic(diag));
            }
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
        let output = serde_json::json!({
            "added": added,
            "removed": removed
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
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
        Err(mut diags) => {
            enrich_diagnostics_with_source(&content, &mut diags);
            for diag in &diags {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!(
                "Parsing failed with {} errors",
                diags.len()
            )));
        }
    };

    let (elements, relations) = sruja_language::collect_elements(&program);

    let elem = elements
        .get(element_id)
        .ok_or_else(|| CliError::Parse(format!("Element '{}' not found", element_id)))?;

    let incoming: Vec<_> = relations
        .iter()
        .filter(|r| r.to.as_string() == element_id)
        .collect();
    let outgoing: Vec<_> = relations
        .iter()
        .filter(|r| r.from.as_string() == element_id)
        .collect();

    if json {
        let desc = elem
            .assignment
            .body
            .as_ref()
            .and_then(|b| b.description.as_ref())
            .cloned()
            .unwrap_or_default();
        let output = serde_json::json!({
            "id": element_id,
            "description": desc,
            "incoming_relations": incoming.len(),
            "outgoing_relations": outgoing.len()
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
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
        return Err(CliError::Parse(format!(
            "Unsupported import format: {}. Supported: json",
            format
        )));
    }

    let content = fs::read_to_string(file)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(arch) = json.get("architecture") {
        if let Some(systems) = arch.get("systems").and_then(|s| s.as_array()) {
            for sys in systems {
                if let (Some(id), Some(label)) = (
                    sys.get("id").and_then(|v| v.as_str()),
                    sys.get("label").and_then(|v| v.as_str()),
                ) {
                    println!("system {} \"{}\"", id, label);
                }
            }
            return Ok(());
        }
    }

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

    Err(CliError::Parse(
        "Could not identify architecture in JSON".to_string(),
    ))
}

/// Start LSP server
pub async fn lsp() -> Result<(), CliError> {
    run_stdio()
        .await
        .map_err(|e| CliError::Validation(format!("LSP server error: {:?}", e)))?;
    Ok(())
}

/// Validate architecture against rules and external constraints
pub async fn validate(
    file: &str,
    constraints: Vec<String>,
    fail_on_violations: bool,
    format_json: bool,
) -> Result<(), CliError> {
    let file_path = Path::new(file);

    if file_path.is_dir() {
        let files = collect_sruja_files(file_path)?;
        return validate_files(&files, constraints, fail_on_violations, format_json).await;
    }

    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());

    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(mut diagnostics) => {
            if !format_json {
                enrich_diagnostics_with_source(&content, &mut diagnostics);
            }
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!(
                "Parsing failed with {} errors",
                diagnostics.len()
            )));
        }
    };

    let validator = Validator::with_default_rules();

    for constraint_path in constraints {
        let constraint_content = fs::read_to_string(&constraint_path)?;
        let constraint_parser = Parser::new(constraint_path.clone());

        if let Ok(constraint_program) = constraint_parser.parse(&constraint_content) {
            let constraint_diagnostics = validator.validate_sync(&constraint_program);
            if !constraint_diagnostics.is_empty() {
                eprintln!("Errors in constraint file {}:", constraint_path);
                let mut constraint_diagnostics = constraint_diagnostics;
                if !format_json {
                    enrich_diagnostics_with_source(
                        &constraint_content,
                        &mut constraint_diagnostics,
                    );
                }
                for diag in &constraint_diagnostics {
                    eprintln!("  {}", format_diagnostic(diag));
                }
                return Err(CliError::Validation(format!(
                    "Constraint file '{}' has errors",
                    constraint_path
                )));
            }
        }
    }

    let mut diagnostics = validator.validate_sync(&program);
    if !format_json {
        enrich_diagnostics_with_source(&content, &mut diagnostics);
    }

    if format_json {
        let output = serde_json::json!({
            "file": file,
            "valid": diagnostics.is_empty(),
            "errors": diagnostics.len(),
            "diagnostics": diagnostics
        });
        println!("{}", output);
    }

    if diagnostics.is_empty() {
        if !format_json {
            println!("✓ Validation passed - no violations found");
        }
        return Ok(());
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for diag in diagnostics {
        match diag.severity {
            sruja_diagnostics::Severity::Error => errors.push(diag),
            sruja_diagnostics::Severity::Warning => warnings.push(diag),
            _ => {}
        }
    }

    if !format_json {
        for warning in &warnings {
            eprintln!("⚠ {}", format_diagnostic(warning));
        }

        for error in &errors {
            eprintln!("✗ {}", format_diagnostic(error));
        }

        eprintln!("\n✗ Found {} violation(s)", errors.len());
    }

    if fail_on_violations || !errors.is_empty() {
        Err(CliError::Validation(format!(
            "Validation failed with {} violation(s)",
            errors.len()
        )))
    } else {
        Ok(())
    }
}

async fn validate_files(
    files: &[String],
    constraints: Vec<String>,
    fail_on_violations: bool,
    format_json: bool,
) -> Result<(), CliError> {
    let mut results = Vec::new();

    for file_path in files {
        match validate_single_file(file_path, &constraints).await {
            Ok(()) => results.push((file_path.clone(), true)),
            Err(_) => results.push((file_path.clone(), false)),
        }
    }

    if format_json {
        let output = serde_json::json!({
            "total_files": results.len(),
            "valid": results.iter().filter(|(_, valid)| *valid).count(),
            "files": results
        });
        println!("{}", output);
    } else {
        println!("\nValidation Summary:");
        println!("  Total files: {}", results.len());
        let valid_count = results.iter().filter(|(_, valid)| *valid).count();
        println!("  Valid: {}", valid_count);
        println!("  Invalid: {}", results.len() - valid_count);

        if results.iter().any(|(_, valid)| !valid) && fail_on_violations {
            return Err(CliError::Validation(
                "Some files failed validation".to_string(),
            ));
        }
    }

    Ok(())
}

async fn validate_single_file(file: &str, _constraints: &[String]) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());

    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(diagnostics) => {
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!(
                "Parsing failed with {} errors",
                diagnostics.len()
            )));
        }
    };

    let validator = Validator::with_default_rules();
    let diagnostics = validator.validate_sync(&program);

    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
        .count();

    if error_count > 0 {
        Err(CliError::Validation(format!(
            "Validation failed with {} errors",
            error_count
        )))
    } else {
        Ok(())
    }
}

/// Compile a Sruja file
pub async fn compile(file: &str) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());

    let program = match parser.parse(&content) {
        Ok(program) => {
            println!("✓ Parsing successful");
            program
        }
        Err(mut diagnostics) => {
            enrich_diagnostics_with_source(&content, &mut diagnostics);
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse(format!(
                "Compilation failed with {} errors",
                diagnostics.len()
            )));
        }
    };

    let validator = Validator::with_default_rules();
    let mut diagnostics = validator.validate_sync(&program);
    enrich_diagnostics_with_source(&content, &mut diagnostics);

    if diagnostics.is_empty() {
        println!("✓ Validation successful");
        println!("✓ Compilation successful");
        return Ok(());
    }

    for diag in &diagnostics {
        eprintln!("{}", format_diagnostic(diag));
    }

    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
        .count();
    if error_count > 0 {
        eprintln!("\n✗ Compilation failed with {} error(s)", error_count);
        return Err(CliError::Validation(format!(
            "Compilation failed with {} errors",
            error_count
        )));
    }

    println!(
        "✓ Compilation successful (with {} warning(s))",
        diagnostics.len()
    );
    Ok(())
}

/// Detect architectural drift in codebase.
/// When architecture_path is provided, compares scanned repo against the DSL baseline.
/// Otherwise, reports drift from structural analysis (cycles, orphans, etc.).
pub async fn drift(
    repo_root: &str,
    architecture_path: Option<&str>,
    format: &str,
    _enrich: bool,
    violations_only: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let actual_graph = scan_repo(repo_path)?;

    if let Some(arch_path) = architecture_path {
        // Baseline drift: compare scan (actual) vs DSL (proposed)
        let arch_file = Path::new(arch_path);
        if !arch_file.exists() {
            return Err(CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Architecture file not found: {}", arch_path),
            )));
        }
        let content = fs::read_to_string(arch_file)?;
        let parser = sruja_language::Parser::new(arch_path);
        let program = parser.parse(&content).map_err(|diags| {
            CliError::Parse(format!(
                "Parse error in {}: {}",
                arch_path,
                diags.iter().map(|d| d.message.as_str()).collect::<Vec<_>>().join("; ")
            ))
        })?;
        let proposed_graph = sruja_diff::program_to_graph(&program);
        let diff_result = sruja_diff::compare_graphs(&actual_graph, &proposed_graph);

        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&diff_result)?);
            }
            _ => {
                print_diff_text(&diff_result, violations_only);
            }
        }

        if diff_result
            .violations
            .iter()
            .any(|v| matches!(v.severity, sruja_diff::Severity::Error))
        {
            std::process::exit(1);
        }
    } else {
        // Scan-only drift: structural analysis
        let drift_result = sruja_diff::detect_architectural_drift(&actual_graph);

        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&drift_result)?);
            }
            _ => {
                print_drift_text(&drift_result, violations_only);
            }
        }

        if drift_result
            .violations
            .iter()
            .any(|v| matches!(v.severity, sruja_diff::Severity::Error))
        {
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Quickstart: Get immediate architecture insights (zero-key, deterministic)
pub async fn quickstart(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    eprintln!("{}", "═".repeat(70));
    eprintln!("🚀 Sruja Quickstart - Architecture Intelligence");
    eprintln!("{}", "═".repeat(70));
    eprintln!();

    // Step 1: Scan repository
    eprintln!("📂 Scanning repository...");
    let graph = scan_repo(repo_path)?;
    eprintln!("   ✓ Found {} components", graph.nodes.len());
    eprintln!();

    // Step 2: Detect drift (consolidated sruja-diff)
    eprintln!("🔍 Analyzing architecture health...");
    let drift_report = sruja_diff::detect_architectural_drift(&graph);
    eprintln!("   ✓ Analysis complete");
    eprintln!();

    match format {
        "json" => {
            let output = QuickstartResult::from_drift_report(&drift_report, &graph, repo_root);
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            print_quickstart_summary(&drift_report, &graph, repo_root);
        }
    }

    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct QuickstartResult {
    pub repo: String,
    pub health_score: u8,
    pub inventory: InventorySummary,
    pub top_findings: Vec<Finding>,
    pub actionable_fixes: Vec<ActionableFix>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InventorySummary {
    pub modules: usize,
    pub services: usize,
    pub databases: usize,
    pub external_apis: usize,
    pub total_dependencies: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Finding {
    pub severity: String,
    pub kind: String,
    pub message: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ActionableFix {
    pub priority: String,
    pub description: String,
    pub impact: String,
    pub affected_components: Vec<String>,
}

impl QuickstartResult {
    fn from_drift_report(report: &sruja_diff::DriftReport, graph: &Graph, repo: &str) -> Self {
        let external_apis = graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::ExternalApi)
            .count();

        let mut all_violations: Vec<_> = report.violations.iter().collect();
        all_violations.sort_by(|a, b| {
            let severity_order = |s: &sruja_diff::Severity| match s {
                sruja_diff::Severity::Error => 0,
                sruja_diff::Severity::Warning => 1,
                sruja_diff::Severity::Info => 2,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        let top_findings: Vec<Finding> = all_violations
            .iter()
            .take(3)
            .map(|v| Finding {
                severity: match v.severity {
                    sruja_diff::Severity::Error => "error".to_string(),
                    sruja_diff::Severity::Warning => "warning".to_string(),
                    sruja_diff::Severity::Info => "info".to_string(),
                },
                kind: format!("{:?}", v.kind),
                message: v.message.clone(),
                evidence: v
                    .location
                    .as_ref()
                    .map(|s| vec![s.clone()])
                    .unwrap_or_default(),
            })
            .collect();

        let actionable_fixes = generate_actionable_fixes_from_violations(&report.violations);

        QuickstartResult {
            repo: repo.to_string(),
            health_score: report.health_score,
            inventory: InventorySummary {
                modules: report.total_modules,
                services: report.total_services,
                databases: report.total_databases,
                external_apis,
                total_dependencies: report.total_dependencies,
            },
            top_findings,
            actionable_fixes,
        }
    }
}

fn generate_actionable_fixes_from_violations(
    violations: &[sruja_diff::Violation],
) -> Vec<ActionableFix> {
    use sruja_diff::ViolationKind;
    let mut fixes = Vec::new();

    let circular: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::CircularDependency))
        .collect();
    if !circular.is_empty() {
        let affected: Vec<String> = circular.iter().filter_map(|v| v.location.clone()).collect();
        fixes.push(ActionableFix {
            priority: "high".to_string(),
            description:
                "Break circular dependencies by introducing interfaces or event-based communication"
                    .to_string(),
            impact: "Improves maintainability and reduces coupling".to_string(),
            affected_components: affected,
        });
    }

    let layer: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::LayerViolation))
        .collect();
    if !layer.is_empty() {
        let affected: Vec<String> = layer.iter().filter_map(|v| v.location.clone()).collect();
        fixes.push(ActionableFix {
            priority: "medium".to_string(),
            description: "Introduce proper service layers to abstract direct database access"
                .to_string(),
            impact: "Improves separation of concerns and testability".to_string(),
            affected_components: affected,
        });
    }

    let god: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::GodModule))
        .collect();
    if !god.is_empty() {
        let affected: Vec<String> = god.iter().filter_map(|v| v.location.clone()).collect();
        fixes.push(ActionableFix {
            priority: "medium".to_string(),
            description: "Refactor god modules into smaller, focused components".to_string(),
            impact: "Improves code maintainability and reduces cognitive load".to_string(),
            affected_components: affected,
        });
    }

    let orphans: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::OrphanComponent))
        .collect();
    if !orphans.is_empty() {
        let affected: Vec<String> = orphans.iter().filter_map(|v| v.location.clone()).collect();
        fixes.push(ActionableFix {
            priority: "low".to_string(),
            description: "Review orphan modules - integrate or remove unused code".to_string(),
            impact: "Reduces dead code and technical debt".to_string(),
            affected_components: affected,
        });
    }

    fixes.truncate(3);
    fixes
}

fn print_quickstart_summary(report: &sruja_diff::DriftReport, graph: &Graph, repo: &str) {
    println!("{}", "─".repeat(70));
    println!("📊 Architecture Inventory");
    println!("{}", "─".repeat(70));
    println!("  Repository: {}", repo);
    println!();
    println!("  Components detected:");
    println!("    • {} modules", report.total_modules);
    println!("    • {} services", report.total_services);
    println!("    • {} databases", report.total_databases);
    let external_apis = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::ExternalApi)
        .count();
    println!("    • {} external APIs", external_apis);
    println!("    • {} total dependencies", report.total_dependencies);
    println!();

    println!("{}", "─".repeat(70));
    println!("💚 Architecture Health Score: {}/100", report.health_score);
    println!("{}", "─".repeat(70));

    let score_bar = match report.health_score {
        80..=100 => "████████████████████ ✓ Good",
        60..=79 => "██████████████░░░░░░ ⚠ Fair",
        40..=59 => "██████████░░░░░░░░░░ ⚠ Needs Work",
        _ => "████░░░░░░░░░░░░░░░░ ✗ Critical",
    };
    println!("  {}", score_bar);
    println!();

    println!("{}", "─".repeat(70));
    println!("🔍 Top 3 Critical Findings");
    println!("{}", "─".repeat(70));

    let mut sorted: Vec<_> = report.violations.iter().collect();
    sorted.sort_by(|a, b| {
        let severity_order = |s: &sruja_diff::Severity| match s {
            sruja_diff::Severity::Error => 0,
            sruja_diff::Severity::Warning => 1,
            sruja_diff::Severity::Info => 2,
        };
        severity_order(&a.severity).cmp(&severity_order(&b.severity))
    });

    for (i, v) in sorted.iter().take(3).enumerate() {
        let icon = match v.severity {
            sruja_diff::Severity::Error => "🚨",
            sruja_diff::Severity::Warning => "⚠️",
            sruja_diff::Severity::Info => "ℹ️",
        };
        println!();
        println!("  {}. {} {}", i + 1, icon, v.message);
        if let Some(ref loc) = v.location {
            println!("     📍 Component: {}", loc);
        }
        if let Some(ref s) = v.suggestion {
            println!("     💡 Suggestion: {}", s);
        }
    }

    if sorted.is_empty() {
        println!();
        println!("  ✓ No critical issues found!");
    }
    println!();

    let fixes = generate_actionable_fixes_from_violations(&report.violations);

    if !fixes.is_empty() {
        println!("{}", "─".repeat(70));
        println!("🎯 Top 3 Actionable Fixes");
        println!("{}", "─".repeat(70));

        for (i, fix) in fixes.iter().enumerate() {
            let priority_icon = match fix.priority.as_str() {
                "high" => "🔴",
                "medium" => "🟡",
                _ => "🟢",
            };

            println!();
            println!(
                "  {}. {} [{}] {}",
                i + 1,
                priority_icon,
                fix.priority.to_uppercase(),
                fix.description
            );
            println!("     Impact: {}", fix.impact);
            if !fix.affected_components.is_empty() {
                println!("     Affected: {}", fix.affected_components.join(", "));
            }
        }
        println!();
    }

    // Evidence Section
    println!("{}", "─".repeat(70));
    println!("📎 Evidence References");
    println!("{}", "─".repeat(70));

    let sample_nodes: Vec<_> = graph.nodes.iter().take(5).collect();
    if !sample_nodes.is_empty() {
        println!();
        println!("  Sample components detected:");
        for node in &sample_nodes {
            println!(
                "    • {} ({:?}) - {}",
                node.id,
                node.kind,
                node.path.as_deref().unwrap_or("unknown")
            );
        }
    }
    println!();

    // Next Steps
    println!("{}", "─".repeat(70));
    println!("🚀 Next Steps");
    println!("{}", "─".repeat(70));
    println!();
    println!("  1. Review the findings above and prioritize fixes");
    println!("  2. Run 'sruja drift -r . --format json' for detailed analysis");
    println!("  3. Run 'sruja scan -r . -o architecture.json' to save the graph");
    println!("  4. Run 'sruja why \"your question\" -r .' to explore architecture decisions");
    println!();
    println!("{}", "═".repeat(70));
}

fn print_diff_text(result: &sruja_diff::DiffResult, violations_only: bool) {
    println!("{}", "═".repeat(60));
    println!("Baseline Drift: Scan vs DSL");
    println!("{}", "═".repeat(60));
    println!();

    if !violations_only {
        println!("📊 Summary");
        println!("{}", "-".repeat(40));
        let s = &result.summary;
        println!(
            "  Proposed: {} | Actual (scan): {}",
            s.proposed_components, s.existing_components
        );
        println!(
            "  New: {} | Missing: {} | Edges +{} -{}",
            s.new_components, s.missing_components,
            s.new_dependencies, s.removed_dependencies
        );
        println!("  Health Score: {}/100", s.health_score);
        println!();
    }

    let errors: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Error))
        .collect();
    let warnings: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Warning))
        .collect();
    let info: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Info))
        .collect();

    if !errors.is_empty() {
        println!("🚨 Errors ({})", errors.len());
        println!("{}", "-".repeat(40));
        for v in &errors {
            println!("  ✗ {}", v.message);
            if let Some(ref suggestion) = v.suggestion {
                println!("    → {}", suggestion);
            }
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("⚠️  Warnings ({})", warnings.len());
        println!("{}", "-".repeat(40));
        for v in &warnings {
            println!("  ⚠ {}", v.message);
            if let Some(ref suggestion) = v.suggestion {
                println!("    → {}", suggestion);
            }
        }
        println!();
    }

    if !violations_only && !info.is_empty() {
        println!("ℹ️  Info ({})", info.len());
        println!("{}", "-".repeat(40));
        for v in &info {
            println!("  ℹ {}", v.message);
        }
        println!();
    }

    if !violations_only && !result.suggestions.is_empty() {
        println!("💡 Suggestions");
        println!("{}", "-".repeat(40));
        for (i, s) in result.suggestions.iter().enumerate() {
            println!("  {}. {}", i + 1, s);
        }
        println!();
    }

    println!("{}", "═".repeat(60));
}

fn print_drift_text(result: &sruja_diff::DriftReport, violations_only: bool) {
    println!("{}", "═".repeat(60));
    println!("Architecture Drift Detection");
    println!("{}", "═".repeat(60));
    println!();

    if !violations_only {
        println!("📊 Summary");
        println!("{}", "-".repeat(40));
        println!(
            "  Modules: {} | Services: {} | Databases: {}",
            result.total_modules, result.total_services, result.total_databases
        );
        println!("  Dependencies: {}", result.total_dependencies);
        println!("  Health Score: {}/100", result.health_score);
        println!();
    }

    let errors: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Error))
        .collect();
    let warnings: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Warning))
        .collect();
    let info: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Info))
        .collect();

    if !errors.is_empty() {
        println!("🚨 Errors ({})", errors.len());
        println!("{}", "-".repeat(40));
        for v in &errors {
            println!("  ✗ {}", v.message);
            if let Some(ref suggestion) = v.suggestion {
                println!("    → {}", suggestion);
            }
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("⚠️  Warnings ({})", warnings.len());
        println!("{}", "-".repeat(40));
        for v in &warnings {
            println!("  ⚠ {}", v.message);
            if let Some(ref suggestion) = v.suggestion {
                println!("    → {}", suggestion);
            }
        }
        println!();
    }

    if !violations_only && !info.is_empty() {
        println!("ℹ️  Info ({})", info.len());
        println!("{}", "-".repeat(40));
        for v in &info {
            println!("  ℹ {}", v.message);
        }
        println!();
    }

    if !violations_only && !result.suggestions.is_empty() {
        println!("💡 Suggestions");
        println!("{}", "-".repeat(40));
        for (i, s) in result.suggestions.iter().enumerate() {
            println!("  {}. {}", i + 1, s);
        }
        println!();
    }

    println!("{}", "═".repeat(60));
}
