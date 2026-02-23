//! DSL file commands: lint, export, fmt, list, tree, diff, explain, import, lsp, validate, compile.

use std::fs;
use std::path::Path;

use sruja_diagnostics::format_diagnostic;
use sruja_engine::Validator;
use sruja_export::context::ContextExporter;
use sruja_export::dsl::DslPrinter;
use sruja_export::json::exporter::{ExportError as JsonExportError, Exporter as JsonExporter};
use sruja_export::markdown::{MarkdownExporter, MarkdownOptions};
use sruja_export::mermaid::exporter::{MermaidConfig, MermaidExporter};
use sruja_language::Parser;
use sruja_lsp::server::run_stdio;

use crate::modules::collect_sruja_files;
use crate::modules::validation::enrich_diagnostics_with_source;

use super::CliError;

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
            return Err(CliError::Parse {
                file: file.to_string(),
                message: format!("Parsing failed with {} errors", diagnostics.len()),
            });
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
            return Err(CliError::Parse {
                file: file.to_string(),
                message: format!("Parsing failed with {} errors", diagnostics.len()),
            });
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
            return Err(CliError::Parse {
                file: file.to_string(),
                message: format!(
                    "Formatting failed: file has {} parse errors",
                    diagnostics.len()
                ),
            });
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
            return Err(CliError::Parse {
                file: file.to_string(),
                message: format!("Parsing failed with {} errors", diagnostics.len()),
            });
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
            return Err(CliError::Parse {
                file: file.to_string(),
                message: format!("Parsing failed with {} errors", diagnostics.len()),
            });
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
            return Err(CliError::Parse {
                file: file1.to_string(),
                message: "Failed to parse first file".to_string(),
            });
        }
    };

    let program2 = match parser2.parse(&content2) {
        Ok(p) => p,
        Err(mut diags) => {
            enrich_diagnostics_with_source(&content2, &mut diags);
            for diag in &diags {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse {
                file: file2.to_string(),
                message: "Failed to parse second file".to_string(),
            });
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
            return Err(CliError::Parse {
                file: file_path.to_string(),
                message: format!("Parsing failed with {} errors", diags.len()),
            });
        }
    };

    let (elements, relations) = sruja_language::collect_elements(&program);

    let elem = elements.get(element_id).ok_or_else(|| CliError::Parse {
        file: element_id.to_string(),
        message: "Element not found".to_string(),
    })?;

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

pub async fn import(format: &str, file: &str) -> Result<(), CliError> {
    if format != "json" {
        return Err(CliError::Parse {
            file: file.to_string(),
            message: format!(
                "Unsupported import format: {}. Supported: json",
                format
            ),
        });
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

    Err(CliError::Parse {
        file: "import".to_string(),
        message: "Could not identify architecture in JSON".to_string(),
    })
}

pub async fn lsp() -> Result<(), CliError> {
    run_stdio()
        .await
        .map_err(|e| CliError::Validation(format!("LSP server error: {:?}", e)))?;
    Ok(())
}

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
            return Err(CliError::Parse {
                file: file.to_string(),
                message: format!("Parsing failed with {} errors", diagnostics.len()),
            });
        }
    };

    let validator = Validator::with_default_rules();

    for constraint_path in &constraints {
        let constraint_content = fs::read_to_string(constraint_path)?;
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

pub async fn validate_files(
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
            return Err(CliError::Parse {
                file: file.to_string(),
                message: format!("Parsing failed with {} errors", diagnostics.len()),
            });
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
            return Err(CliError::Parse {
                file: String::new(),
                message: format!("Compilation failed with {} errors", diagnostics.len()),
            });
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
