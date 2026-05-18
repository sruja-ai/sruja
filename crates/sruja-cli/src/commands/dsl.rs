//! DSL file commands: lint, export, fmt, list, tree, diff, explain, import, lsp, validate, compile.

use std::fs;
use std::path::Path;

use sruja_diagnostics::{format_diagnostic, format_github_actions_annotation};
use sruja_engine::Validator;
use sruja_export::context::ContextExporter;
use sruja_export::d2::{D2Config, D2Exporter};
use sruja_export::dsl::DslPrinter;
use sruja_export::json::exporter::{ExportError as JsonExportError, Exporter as JsonExporter};
use sruja_export::markdown::{MarkdownExporter, MarkdownOptions};
use sruja_export::mermaid::exporter::{MermaidConfig, MermaidExporter};

use crate::modules::collect_sruja_files;
use crate::modules::validation::enrich_diagnostics_with_source;

use super::CliError;

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
struct LintBaselineEntry {
    code: String,
    file: String,
    line: u32,
    column: u32,
    message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LintBaseline {
    schema_version: u32,
    entries: Vec<LintBaselineEntry>,
}

/// Machine-readable diagnostic for JSON output.
#[derive(serde::Serialize)]
struct LintDiagnostic {
    code: String,
    severity: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<LintLocation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    suggestions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    context: Vec<String>,
}

#[derive(serde::Serialize)]
struct LintLocation {
    file: String,
    line: u32,
    column: u32,
}

#[derive(serde::Serialize)]
struct LintOutput {
    ok: bool,
    error_count: usize,
    warning_count: usize,
    diagnostics: Vec<LintDiagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    baseline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_error_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_warning_count: Option<usize>,
    /// Present when the full diagnostic list was stored under `.sruja/vfs/diagnostics/`.
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostic_full: Option<crate::commands::diagnostic_vfs::TruncatedDiagnosticPayload>,
}

pub async fn lint(
    file: &str,
    format: &str,
    baseline: Option<&str>,
    write_baseline: Option<&str>,
) -> Result<(), CliError> {
    let github = matches!(format, "github" | "github-actions");
    let content = fs::read_to_string(file)?;
    let parser = sruja_language::Parser::new(file.to_string());

    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(mut diagnostics) => {
            enrich_diagnostics_with_source(&content, &mut diagnostics);
            if format == "json" {
                let out = lint_diagnostics_to_json(Path::new(file), &diagnostics, false)?;
                println!(
                    "{}",
                    serde_json::to_string(&out).map_err(|e| CliError::validation(e.to_string()))?
                );
                return Err(CliError::Parse {
                    file: file.to_string(),
                    message: format!("Parsing failed with {} errors", diagnostics.len()),
                    diagnostics,
                    help: Some("Fix the syntax errors reported by the parser.".into()),
                    fix: Some(
                        "After fixing syntax, run 'sruja fmt' (optional) and re-run 'sruja lint'."
                            .into(),
                    ),
                });
            }
            if github {
                for diag in &diagnostics {
                    println!("{}", format_github_actions_annotation(diag));
                }
                return Err(CliError::Parse {
                    file: file.to_string(),
                    message: format!("Parsing failed with {} errors", diagnostics.len()),
                    diagnostics,
                    help: Some(
                        "Check the GitHub Actions annotations for the exact location.".into(),
                    ),
                    fix: Some("Fix the syntax errors in the file, then re-run the check.".into()),
                });
            }
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse {
                file: file.to_string(),
                message: format!("Parsing failed with {} errors", diagnostics.len()),
                diagnostics,
                help: Some("Review the diagnostics printed above for the exact location.".into()),
                fix: Some("Fix the syntax errors in the file, then re-run 'sruja lint'.".into()),
            });
        }
    };

    let validator = Validator::with_default_rules();
    let mut diagnostics = validator.validate_sync(&program);
    enrich_diagnostics_with_source(&content, &mut diagnostics);
    sort_diagnostics(&mut diagnostics);

    if let Some(out_path) = write_baseline {
        write_lint_baseline(out_path, &diagnostics)?;
        if format == "json" {
            let mut out = lint_diagnostics_to_json(Path::new(file), &diagnostics, true)?;
            out.baseline = Some(out_path.to_string());
            out.total_error_count = Some(out.error_count);
            out.total_warning_count = Some(out.warning_count);
            out.error_count = 0;
            out.warning_count = 0;
            out.diagnostics.clear();
            println!("{}", serde_json::to_string(&out)?);
        } else {
            println!("Wrote baseline: {}", out_path);
        }
        return Ok(());
    }

    let (filtered_diagnostics, total_error_count, total_warning_count) =
        apply_lint_baseline(baseline, &diagnostics)?;

    if format == "json" {
        let error_count = filtered_diagnostics
            .iter()
            .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
            .count();
        let mut out =
            lint_diagnostics_to_json(Path::new(file), &filtered_diagnostics, error_count == 0)?;
        out.baseline = baseline.map(|s| s.to_string());
        out.total_error_count = Some(total_error_count);
        out.total_warning_count = Some(total_warning_count);
        println!(
            "{}",
            serde_json::to_string(&out).map_err(|e| CliError::validation(e.to_string()))?
        );
        if error_count > 0 {
            return Err(CliError::validation(format!(
                "Linting failed with {} errors",
                error_count
            )));
        }
        return Ok(());
    }

    if github {
        let error_count = filtered_diagnostics
            .iter()
            .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
            .count();
        for diag in &filtered_diagnostics {
            println!("{}", format_github_actions_annotation(diag));
        }
        if error_count > 0 {
            return Err(CliError::validation(format!(
                "Linting failed with {} errors",
                error_count
            )));
        }
        return Ok(());
    }

    if filtered_diagnostics.is_empty() {
        println!("✓ No issues found");
        return Ok(());
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for diag in filtered_diagnostics {
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
        use crate::utils::colors;
        println!("──────────────────────────────────────────────");
        println!(
            "{} Found {} error(s) and {} warning(s)",
            colors::error("✗"),
            errors.len(),
            warnings.len()
        );
        return Err(CliError::validation(format!(
            "Linting failed with {} errors",
            errors.len()
        )));
    }

    if !warnings.is_empty() {
        use crate::utils::colors;
        println!("──────────────────────────────────────────────");
        println!(
            "{} Found {} warning(s) (no errors)",
            colors::success("✓"),
            warnings.len()
        );
    } else {
        use crate::utils::colors;
        println!("──────────────────────────────────────────────");
        println!("{} No issues found", colors::success("✓"));
    }

    Ok(())
}

fn lint_storage_name(arch_file: &Path) -> String {
    let stem = arch_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("lint");
    format!("lint-{stem}.txt")
}

fn lint_repo_root(arch_file: &Path) -> &Path {
    arch_file.parent().unwrap_or_else(|| Path::new("."))
}

fn lint_diagnostics_to_json(
    arch_file: &Path,
    diagnostics: &[sruja_diagnostics::Diagnostic],
    ok: bool,
) -> Result<LintOutput, CliError> {
    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
        .count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Warning)
        .count();

    let repo = lint_repo_root(arch_file);
    let (diagnostics_for_output, diagnostic_full) =
        crate::commands::diagnostic_vfs::apply_lint_json_truncation(
            repo,
            &lint_storage_name(arch_file),
            diagnostics,
            crate::commands::diagnostic_vfs::LINT_JSON_DIAGNOSTIC_TOKEN_BUDGET,
        )?;

    let diagnostics: Vec<LintDiagnostic> = diagnostics_for_output
        .iter()
        .map(|d| {
            let severity = match d.severity {
                sruja_diagnostics::Severity::Error => "error",
                sruja_diagnostics::Severity::Warning => "warning",
                sruja_diagnostics::Severity::Info => "info",
                _ => "info",
            };
            let location = LintLocation {
                file: d.location.file.clone(),
                line: d.location.line,
                column: d.location.column,
            };
            LintDiagnostic {
                code: d.code.clone(),
                severity: severity.to_string(),
                message: d.message.clone(),
                location: Some(location),
                suggestions: d.suggestions.clone(),
                context: d.context.clone(),
            }
        })
        .collect();
    Ok(LintOutput {
        ok,
        error_count,
        warning_count,
        diagnostics,
        baseline: None,
        total_error_count: None,
        total_warning_count: None,
        diagnostic_full,
    })
}

fn sort_diagnostics(diagnostics: &mut [sruja_diagnostics::Diagnostic]) {
    diagnostics.sort_by(|a, b| {
        (
            a.location.file.as_str(),
            a.location.line,
            a.location.column,
            a.code.as_str(),
            a.message.as_str(),
        )
            .cmp(&(
                b.location.file.as_str(),
                b.location.line,
                b.location.column,
                b.code.as_str(),
                b.message.as_str(),
            ))
    });
}

fn apply_lint_baseline(
    baseline: Option<&str>,
    diagnostics: &[sruja_diagnostics::Diagnostic],
) -> Result<(Vec<sruja_diagnostics::Diagnostic>, usize, usize), CliError> {
    let total_error_count = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
        .count();
    let total_warning_count = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Warning)
        .count();

    let Some(baseline_path) = baseline else {
        return Ok((diagnostics.to_vec(), total_error_count, total_warning_count));
    };

    let baseline_content = fs::read_to_string(baseline_path)?;
    let baseline: LintBaseline = serde_json::from_str(&baseline_content)?;

    let set: std::collections::HashSet<LintBaselineEntry> = baseline.entries.into_iter().collect();
    let filtered = diagnostics
        .iter()
        .filter(|d| {
            let key_v1 = LintBaselineEntry {
                code: d.code.clone(),
                file: d.location.file.clone(),
                line: d.location.line,
                column: d.location.column,
                message: None,
            };
            let key_v2 = LintBaselineEntry {
                code: d.code.clone(),
                file: d.location.file.clone(),
                line: d.location.line,
                column: d.location.column,
                message: Some(d.message.clone()),
            };
            !(set.contains(&key_v2) || set.contains(&key_v1))
        })
        .cloned()
        .collect();

    Ok((filtered, total_error_count, total_warning_count))
}

fn write_lint_baseline(
    out_path: &str,
    diagnostics: &[sruja_diagnostics::Diagnostic],
) -> Result<(), CliError> {
    let mut entries: Vec<LintBaselineEntry> = diagnostics
        .iter()
        .map(|d| LintBaselineEntry {
            code: d.code.clone(),
            file: d.location.file.clone(),
            line: d.location.line,
            column: d.location.column,
            message: Some(d.message.clone()),
        })
        .collect();
    entries.sort();
    entries.dedup();

    let baseline = LintBaseline {
        schema_version: 1,
        entries,
    };

    let json = serde_json::to_string_pretty(&baseline)?;
    fs::write(out_path, json)?;
    Ok(())
}

pub struct ExportOptions {
    pub extended: bool,
    pub view_level: u8,
    pub target: Option<String>,
    pub view_name: Option<String>,
    pub all_views: bool,
    pub inject: Option<String>,
    pub hydrate: bool,
    pub from_scan: bool,
    pub repo: Option<String>,
    pub output_dir: Option<String>,
}

pub async fn export(format: &str, file: &str, options: ExportOptions) -> Result<(), CliError> {
    if options.from_scan {
        let repo_path = Path::new(options.repo.as_deref().unwrap_or("."));
        let graph = super::scan_repo_cached(repo_path)?;
        match format {
            "graphml" => {
                let output = sruja_export::GraphMLExporter::export(&graph);
                println!("{}", output);
                return Ok(());
            }
            "neo4j" | "cypher" => {
                let output = sruja_export::Neo4jExporter::export(&graph);
                println!("{}", output);
                return Ok(());
            }
            "obsidian" => {
                let out_dir = options.output_dir.as_deref().unwrap_or("./obsidian-vault");
                sruja_export::ObsidianExporter::export(&graph, Path::new(out_dir))?;
                println!("Successfully exported Obsidian vault to {}", out_dir);
                return Ok(());
            }
            _ => {
                return Err(CliError::validation(format!(
                    "Unsupported scan-based export format: {}. Supported: graphml, neo4j, cypher, obsidian",
                    format
                )));
            }
        }
    }

    let (_, program) = super::parse_sruja_file(file)?;

    let output_str = match format {
        "json" => {
            let exporter = JsonExporter::with_extended(options.extended);
            let mut dump = exporter.to_model_dump(&program);

            if options.hydrate {
                let repo_root = Path::new(file).parent().unwrap_or_else(|| Path::new("."));
                for (_fqn, element) in dump.elements.iter_mut() {
                    // Collect all sources for this element
                    let mut code_blocks = Vec::new();
                    for s in &element.sources {
                        let full_path = repo_root.join(&s.path);
                        if let Ok(content) = fs::read_to_string(&full_path) {
                            code_blocks.push(format!("// File: {}\n{}", s.path, content));
                        }
                    }
                    if !code_blocks.is_empty() {
                        element.hydration = Some(code_blocks.join("\n\n"));
                    }
                }
            }

            serde_json::to_string_pretty(&dump)?
        }
        "mermaid" => {
            let exporter = MermaidExporter::new(MermaidConfig {
                direction: "LR".to_string(),
                view_level: options.view_level,
                target_id: options.target.clone(),
            });
            exporter.export(&program)
        }
        "d2" => {
            let exporter = D2Exporter::new(D2Config {
                direction: "right".to_string(),
                view_level: options.view_level,
                target_id: options.target.clone(),
                link_template: None,
            });
            exporter.export(&program)
        }
        "markdown" => {
            let mut md_options = MarkdownOptions::default();
            if let Some(name) = options.view_name.as_deref() {
                md_options.use_views = true;
                md_options.view_name = Some(name.to_string());
            } else if options.all_views {
                md_options.use_views = true;
                md_options.include_all_views = true;
            }
            let exporter = MarkdownExporter::new(md_options);
            exporter.export(&program)
        }
        "context" => {
            let exporter = ContextExporter::new("general");
            exporter.export(&program)
        }
        "dsl" => {
            let printer = DslPrinter::new();
            printer.print(&program)
        }
        _ => {
            return Err(CliError::Export(JsonExportError::Export(format!(
                "Unsupported export format: {}",
                format
            ))));
        }
    };

    if let Some(inject_path) = options.inject {
        inject_into_file(&inject_path, &output_str, format)?;
        println!(
            "Successfully injected {} output into {}",
            format, inject_path
        );
    } else {
        println!("{}", output_str);
    }

    Ok(())
}

fn inject_into_file(path: &str, content: &str, format: &str) -> Result<(), CliError> {
    let file_content = fs::read_to_string(path).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read {}: {}", path, e),
        ))
    })?;

    let start_marker = "<!-- sruja:start -->";
    let end_marker = "<!-- sruja:end -->";

    let start_idx = file_content.find(start_marker);
    let end_idx = file_content.find(end_marker);

    match (start_idx, end_idx) {
        (Some(start), Some(end)) if start < end => {
            let mut new_content = String::new();
            new_content.push_str(&file_content[..start + start_marker.len()]);
            new_content.push('\n');

            if format == "mermaid" {
                new_content.push_str("```mermaid\n");
                new_content.push_str(content);
                if !content.ends_with('\n') {
                    new_content.push('\n');
                }
                new_content.push_str("```\n");
            } else if format == "d2" {
                new_content.push_str("```d2\n");
                new_content.push_str(content);
                if !content.ends_with('\n') {
                    new_content.push('\n');
                }
                new_content.push_str("```\n");
            } else {
                new_content.push_str(content);
                if !content.ends_with('\n') {
                    new_content.push('\n');
                }
            }

            new_content.push_str(&file_content[end..]);

            fs::write(path, new_content).map_err(|e| {
                CliError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to write {}: {}", path, e),
                ))
            })?;
            Ok(())
        }
        _ => Err(CliError::Export(JsonExportError::Export(format!(
            "Could not find valid <!-- sruja:start --> and <!-- sruja:end --> markers in {}",
            path
        )))),
    }
}

pub async fn fmt(file: &str, check: bool) -> Result<(), CliError> {
    let (content, program) = super::parse_sruja_file(file)?;

    let printer = DslPrinter::new();
    let formatted = printer.print(&program);

    if formatted != content {
        if check {
            println!("Would reformat {}", file);
            return Err(CliError::validation(format!(
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
    let (_, program) = super::parse_sruja_file(file)?;

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
    let (_, program) = super::parse_sruja_file(file)?;

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
    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
    struct ElementSnapshot {
        kind: String,
        title: String,
        description: Option<String>,
        technology: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
    struct ElementChange {
        id: String,
        before: ElementSnapshot,
        after: ElementSnapshot,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
    struct RelationSnapshot {
        from: String,
        to: String,
        label: Option<String>,
        description: Option<String>,
        technology: Option<String>,
        tags: Vec<String>,
    }

    fn element_snapshot(elem: &sruja_language::ElementDef) -> ElementSnapshot {
        let title = elem
            .assignment
            .title
            .clone()
            .unwrap_or_else(|| elem.assignment.name.clone());
        let (description, technology) = elem
            .assignment
            .body
            .as_ref()
            .map(|b| (b.description.clone(), b.technology.clone()))
            .unwrap_or((None, None));

        ElementSnapshot {
            kind: elem.assignment.kind.to_string(),
            title,
            description,
            technology,
        }
    }

    fn relation_snapshot(rel: &sruja_language::Relation) -> RelationSnapshot {
        let mut tags = rel.tags.clone();
        tags.sort();
        RelationSnapshot {
            from: rel.from.as_string(),
            to: rel.to.as_string(),
            label: rel.label.clone(),
            description: rel.description.clone(),
            technology: rel.technology.clone(),
            tags,
        }
    }

    let (_content1, program1) = super::parse_sruja_file(file1)?;
    let (_content2, program2) = super::parse_sruja_file(file2)?;
    let (elems1, rels1) = sruja_language::collect_elements(&program1);
    let (elems2, rels2) = sruja_language::collect_elements(&program2);

    let mut added: Vec<String> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    let mut changed: Vec<ElementChange> = Vec::new();

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

    for (fqn, e1) in &elems1 {
        if let Some(e2) = elems2.get(fqn) {
            let before = element_snapshot(e1);
            let after = element_snapshot(e2);
            if before != after {
                changed.push(ElementChange {
                    id: fqn.clone(),
                    before,
                    after,
                });
            }
        }
    }

    added.sort();
    removed.sort();
    changed.sort_by(|a, b| a.id.cmp(&b.id));

    use std::collections::BTreeSet;
    let set1: BTreeSet<RelationSnapshot> = rels1.iter().map(relation_snapshot).collect();
    let set2: BTreeSet<RelationSnapshot> = rels2.iter().map(relation_snapshot).collect();

    let mut added_relations: Vec<RelationSnapshot> = set2.difference(&set1).cloned().collect();
    let mut removed_relations: Vec<RelationSnapshot> = set1.difference(&set2).cloned().collect();
    added_relations.sort();
    removed_relations.sort();

    if format == "json" {
        let output = serde_json::json!({
            "added_elements": added,
            "removed_elements": removed,
            "changed_elements": changed,
            "added_relations": added_relations,
            "removed_relations": removed_relations
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if format == "markdown" || format == "llm" {
        let mut out = String::new();
        out.push_str("# Architectural Diff\n\n");
        out.push_str("> Generated by `sruja diff`\n\n");

        if added.is_empty()
            && removed.is_empty()
            && changed.is_empty()
            && added_relations.is_empty()
            && removed_relations.is_empty()
        {
            out.push_str("**No architectural changes detected.**\n");
        } else {
            if !added.is_empty() {
                out.push_str("## 🟢 Added Elements\n\n");
                for fqn in &added {
                    out.push_str(&format!("- `{}`\n", fqn));
                }
                out.push('\n');
            }
            if !removed.is_empty() {
                out.push_str("## 🔴 Removed Elements\n\n");
                for fqn in &removed {
                    out.push_str(&format!("- `{}`\n", fqn));
                }
                out.push('\n');
            }
            if !changed.is_empty() {
                out.push_str("## 🟡 Changed Elements\n\n");
                for c in &changed {
                    out.push_str(&format!("- `{}`\n", c.id));
                }
                out.push('\n');
            }
            if !added_relations.is_empty() {
                out.push_str("## 🟢 Added Relationships\n\n");
                for r in &added_relations {
                    let label = r.label.as_deref().unwrap_or("");
                    if label.is_empty() {
                        out.push_str(&format!("- `{}` -> `{}`\n", r.from, r.to));
                    } else {
                        out.push_str(&format!("- `{}` -> `{}` (*{}*)\n", r.from, r.to, label));
                    }
                }
                out.push('\n');
            }
            if !removed_relations.is_empty() {
                out.push_str("## 🔴 Removed Relationships\n\n");
                for r in &removed_relations {
                    let label = r.label.as_deref().unwrap_or("");
                    if label.is_empty() {
                        out.push_str(&format!("- `{}` -> `{}`\n", r.from, r.to));
                    } else {
                        out.push_str(&format!("- `{}` -> `{}` (*{}*)\n", r.from, r.to, label));
                    }
                }
                out.push('\n');
            }
        }
        println!("{}", out);
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
        if !changed.is_empty() {
            println!("Changed elements:");
            for c in &changed {
                println!("  ~ {}", c.id);
            }
        }
        if !added_relations.is_empty() {
            println!("Added relations:");
            for r in &added_relations {
                let label = r.label.as_deref().unwrap_or("");
                if label.is_empty() {
                    println!("  + {} -> {}", r.from, r.to);
                } else {
                    println!("  + {} -> {} \"{}\"", r.from, r.to, label);
                }
            }
        }
        if !removed_relations.is_empty() {
            println!("Removed relations:");
            for r in &removed_relations {
                let label = r.label.as_deref().unwrap_or("");
                if label.is_empty() {
                    println!("  - {} -> {}", r.from, r.to);
                } else {
                    println!("  - {} -> {} \"{}\"", r.from, r.to, label);
                }
            }
        }
        if added.is_empty()
            && removed.is_empty()
            && changed.is_empty()
            && added_relations.is_empty()
            && removed_relations.is_empty()
        {
            println!("No differences found");
        }
    }

    Ok(())
}

pub async fn explain(element_id: &str, file: Option<&str>, json: bool) -> Result<(), CliError> {
    let file_path = file.unwrap_or("architecture.sruja");
    let (_content, program) = super::parse_sruja_file(file_path)?;

    let (elements, relations) = sruja_language::collect_elements(&program);

    let elem = elements.get(element_id).ok_or_else(|| {
        let q = element_id.to_lowercase();
        let fuzzy: Vec<&String> = elements
            .keys()
            .filter(|k| k.to_lowercase().contains(&q))
            .collect();
        if fuzzy.is_empty() {
            CliError::validation(format!(
                "Element '{}' not found. Available: {} (or run `sruja list {}` to see all)",
                element_id,
                elements
                    .keys()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", "),
                file_path
            ))
        } else if fuzzy.len() == 1 {
            CliError::validation(format!(
                "Element '{}' not found. Did you mean '{}'?",
                element_id, fuzzy[0]
            ))
        } else {
            CliError::validation(format!(
                "Element '{}' not found. Similar: {}",
                element_id,
                fuzzy
                    .iter()
                    .take(5)
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
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
        return Err(CliError::Validation {
            message: format!("Unsupported import format: {}. Supported: json", format),
            help: Some("Use: sruja import json <file>".into()),
            fix: None,
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

    if let Some(elements) = json.get("elements").and_then(|e| e.as_object()) {
        let mut sorted: Vec<_> = elements.iter().collect();
        sorted.sort_by_key(|(k, _)| *k);

        for (fqn, elem) in &sorted {
            if let (Some(kind), Some(title)) = (
                elem.get("kind").and_then(|v| v.as_str()),
                elem.get("title").and_then(|v| v.as_str()),
            ) {
                let short_id = fqn.split('.').next_back().unwrap_or(fqn);
                println!(
                    "{} = {} \"{}\" {{",
                    short_id.replace('-', "_").replace(" ", "_"),
                    kind,
                    title
                );
                if let Some(desc) = elem.get("description").and_then(|v| v.as_str()) {
                    if !desc.is_empty() {
                        println!("  description \"{}\"", desc.replace('"', "\\\""));
                    }
                }
                if let Some(tech) = elem.get("technology").and_then(|v| v.as_str()) {
                    println!("  technology \"{}\"", tech);
                }
                println!("}}");
            }
        }

        if let Some(relations) = json.get("relations").and_then(|r| r.as_array()) {
            println!();
            for rel in relations {
                let from = rel
                    .get("source")
                    .and_then(|s| s.get("model"))
                    .and_then(|m| m.as_str());
                let to = rel
                    .get("target")
                    .and_then(|t| t.get("model"))
                    .and_then(|m| m.as_str());
                let label = rel.get("title").and_then(|t| t.as_str()).unwrap_or("");
                if let (Some(f), Some(t)) = (from, to) {
                    let from_short = f
                        .split('.')
                        .next_back()
                        .unwrap_or(f)
                        .replace('-', "_")
                        .replace(" ", "_");
                    let to_short = t
                        .split('.')
                        .next_back()
                        .unwrap_or(t)
                        .replace('-', "_")
                        .replace(" ", "_");
                    println!("{} -> {} \"{}\"", from_short, to_short, label);
                }
            }
        }

        return Ok(());
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

    Err(CliError::Validation {
        message: "Could not identify architecture in JSON".to_string(),
        help: Some("Expected { architecture: { systems: [...] } }, { elements: { ... } } (Sruja native), or { elements: [...] }.".into()),
        fix: Some("Re-export JSON from a supported tool or provide a JSON file matching the expected shape.".into()),
    })
}

pub async fn lsp() -> Result<(), CliError> {
    Err(CliError::validation(
        "LSP server is not available in this build".to_string(),
    ))
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

    let (content, program) = super::parse_sruja_file(file)?;

    let validator = Validator::with_default_rules();

    for constraint_path in &constraints {
        let constraint_content = fs::read_to_string(constraint_path)?;
        let constraint_parser = sruja_language::Parser::new(constraint_path.clone());

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
                return Err(CliError::validation(format!(
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
        Err(CliError::validation(format!(
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
            return Err(CliError::validation(
                "Some files failed validation".to_string(),
            ));
        }
    }

    Ok(())
}

async fn validate_single_file(file: &str, _constraints: &[String]) -> Result<(), CliError> {
    let (_content, program) = super::parse_sruja_file(file)?;

    let validator = Validator::with_default_rules();
    let diagnostics = validator.validate_sync(&program);

    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
        .count();

    if error_count > 0 {
        Err(CliError::validation(format!(
            "Validation failed with {} errors",
            error_count
        )))
    } else {
        Ok(())
    }
}

pub async fn compile(file: &str) -> Result<(), CliError> {
    let (content, program) = super::parse_sruja_file(file)?;
    println!("✓ Parsing successful");

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
        return Err(CliError::validation(format!(
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
