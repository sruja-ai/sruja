//! Sruja WASM bindings for browser usage
//!
//! This crate provides WebAssembly bindings for Sruja functionality,
//! allowing to website and other frontend applications to use Sruja
//! in browser.

use serde_json::json;
use sruja_engine::Validator;
use sruja_export::json::Exporter as JsonExporter;
use sruja_export::markdown::{MarkdownExporter, MarkdownOptions};
use sruja_export::mermaid::exporter::MermaidConfig;
use sruja_export::mermaid::MermaidExporter;
use sruja_export::mermaid::{flow_to_sequence_diagram, scenario_to_sequence_diagram};
use sruja_language::Parser;
use sruja_language::TopLevelItem;
use wasm_bindgen::prelude::*;

/// Initialize panic hook for better error messages in WASM
/// This should be called once when WASM module is loaded
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Unused macro - kept for potential future debugging
// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

#[wasm_bindgen]
pub fn sruja_dsl_to_model(dsl: &str, filename: Option<String>) -> Result<String, JsValue> {
    let filename = filename.unwrap_or_else(|| "input.sruja".to_string());
    let parser = Parser::new(filename.clone());
    let program = parser.parse(dsl).map_err(|e| {
        let error_msg = if e.is_empty() {
            "Parse error: unknown error".to_string()
        } else {
            format!(
                "Parse error: {}",
                e.iter()
                    .map(|d| format!("{}: {}", d.code, d.message))
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        };
        JsValue::from_str(&error_msg)
    })?;

    let exporter = JsonExporter::new();
    // export() already returns a JSON string, so we return it directly
    exporter
        .export(&program)
        .map_err(|e| JsValue::from_str(&format!("Export error: {}", e)))
}

/// Incremental parse: re-parses DSL and returns updated model JSON plus change metadata.
/// When `existing_ast_json` is not yet used for true incremental merge (Program (de)serialization
/// is future work), this performs a full parse and returns the same shape for API compatibility.
#[wasm_bindgen]
pub fn sruja_incremental_parse(
    dsl: &str,
    _change_start: u32,
    _change_end: u32,
    _existing_ast_json: &str,
    _context_lines: u32,
    filename: Option<String>,
) -> Result<String, JsValue> {
    let start_ms = js_sys::Date::now();
    let filename = filename.unwrap_or_else(|| "input.sruja".to_string());
    let parser = Parser::new(filename.clone());
    let program = parser.parse(dsl).map_err(|e| {
        let error_msg = if e.is_empty() {
            "Parse error: unknown error".to_string()
        } else {
            format!(
                "Parse error: {}",
                e.iter()
                    .map(|d| format!("{}: {}", d.code, d.message))
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        };
        JsValue::from_str(&error_msg)
    })?;

    let exporter = JsonExporter::new();
    let model_json = exporter
        .export(&program)
        .map_err(|e| JsValue::from_str(&format!("Export error: {}", e)))?;

    let elapsed_ms = (js_sys::Date::now() - start_ms).max(0.0).round() as u64;
    let result = json!({
        "updated_ast": serde_json::from_str::<serde_json::Value>(&model_json)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        "changed_elements": serde_json::Value::Array(vec![]),
        "changed_ranges": serde_json::Value::Array(vec![serde_json::Value::Array(vec![
            serde_json::json!(0),
            serde_json::json!(0),
        ])]),
        "parsing_time_ms": elapsed_ms,
    });
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&format!("JSON error: {:?}", e)))
}

#[wasm_bindgen]
pub fn sruja_dsl_to_mermaid(dsl: &str, config_json: Option<String>) -> Result<String, JsValue> {
    let parser = Parser::new("input.sruja".to_string());
    let program = parser
        .parse(dsl)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {:?}", e)))?;

    let mut mermaid_config = MermaidConfig::default();
    if let Some(config_str) = config_json {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&config_str) {
            if let Some(view_level) = config.get("viewLevel").and_then(|v| v.as_u64()) {
                mermaid_config.view_level = view_level as u8;
            }
            if let Some(target) = config.get("targetId").and_then(|v| v.as_str()) {
                mermaid_config.target_id = Some(target.to_string());
            }
        }
    }

    let exporter = MermaidExporter::new(mermaid_config);
    Ok(exporter.export(&program))
}

#[wasm_bindgen]
pub fn sruja_dsl_to_sequence_diagram(
    dsl: &str,
    config_json: Option<String>,
) -> Result<String, JsValue> {
    let parser = Parser::new("input.sruja".to_string());
    let program = parser
        .parse(dsl)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {:?}", e)))?;

    let mut kind: String = "scenario".to_string();
    let mut id: Option<String> = None;
    if let Some(config_str) = config_json {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&config_str) {
            if let Some(k) = config.get("kind").and_then(|v| v.as_str()) {
                kind = k.to_string();
            }
            if let Some(target) = config.get("id").and_then(|v| v.as_str()) {
                if !target.trim().is_empty() {
                    id = Some(target.to_string());
                }
            }
        }
    }

    let Some(target_id) = id else {
        return Ok(String::new());
    };

    let want_flow = kind.trim().eq_ignore_ascii_case("flow");

    for item in &program.items {
        match item {
            TopLevelItem::Flow(flow) if want_flow && flow.id == target_id => {
                return Ok(flow_to_sequence_diagram(&flow.id, &flow.title, &flow.steps));
            }
            TopLevelItem::Scenario(scenario) if !want_flow && scenario.id == target_id => {
                return Ok(scenario_to_sequence_diagram(
                    &scenario.id,
                    &scenario.title,
                    &scenario.steps,
                ));
            }
            _ => {}
        }
    }

    if want_flow {
        for item in &program.items {
            if let TopLevelItem::Scenario(scenario) = item {
                if scenario.id == target_id {
                    return Ok(scenario_to_sequence_diagram(
                        &scenario.id,
                        &scenario.title,
                        &scenario.steps,
                    ));
                }
            }
        }
    } else {
        for item in &program.items {
            if let TopLevelItem::Flow(flow) = item {
                if flow.id == target_id {
                    return Ok(flow_to_sequence_diagram(&flow.id, &flow.title, &flow.steps));
                }
            }
        }
    }

    Ok(String::new())
}

/// DOT export was removed. Use Mermaid export instead.
#[wasm_bindgen]
pub fn sruja_dsl_to_dot(
    _dsl: &str,
    _view_level: Option<u8>,
    _target_id: Option<String>,
    _node_sizes_json: Option<String>,
    _view_id: Option<String>,
    _filename: Option<String>,
) -> Result<String, JsValue> {
    Err(JsValue::from_str(
        "DOT export is no longer available. Use sruja_dsl_to_mermaid instead.",
    ))
}

/// DOT export with relations was removed. Use Mermaid export instead.
#[wasm_bindgen]
pub fn sruja_dsl_to_dot_with_relations(
    _dsl: &str,
    _view_level: Option<u8>,
    _target_id: Option<String>,
    _node_sizes_json: Option<String>,
    _view_id: Option<String>,
    _filename: Option<String>,
) -> Result<String, JsValue> {
    Err(JsValue::from_str(
        "DOT export is no longer available. Use sruja_dsl_to_mermaid instead.",
    ))
}

#[wasm_bindgen]
pub fn sruja_dsl_to_markdown(dsl: &str) -> Result<String, JsValue> {
    let parser = Parser::new("input.sruja".to_string());
    let program = parser
        .parse(dsl)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {:?}", e)))?;

    let exporter = MarkdownExporter::new(MarkdownOptions::default());
    Ok(exporter.export(&program))
}

#[wasm_bindgen]
pub fn sruja_model_to_dsl(model_json: &str) -> Result<String, JsValue> {
    // Parse JSON model
    let model: serde_json::Value = serde_json::from_str(model_json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {:?}", e)))?;

    let mut dsl = String::new();

    // Write specification metadata if present
    if let Some(spec) = model.get("specification") {
        write_specification(&mut dsl, spec);
    }

    // Write elements
    if let Some(elements) = model.get("elements").and_then(|v| v.as_object()) {
        let mut element_ids: Vec<_> = elements.keys().cloned().collect();
        element_ids.sort(); // Sort for consistent output

        for id in element_ids {
            if let Some(elem) = elements.get(&id) {
                write_element(&mut dsl, &id, elem, 0);
                dsl.push('\n');
            }
        }
    }

    // Write relations
    if let Some(relations) = model.get("relations").and_then(|v| v.as_array()) {
        for rel in relations {
            write_relation(&mut dsl, rel);
        }
    }

    // Write views if present
    if let Some(views) = model.get("views").and_then(|v| v.as_object()) {
        for (view_id, view) in views {
            write_view(&mut dsl, view_id, view);
            dsl.push('\n');
        }
    }

    Ok(dsl.trim().to_string())
}

fn write_specification(dsl: &mut String, spec: &serde_json::Value) {
    // Extract specification fields
    if let Some(name) = spec.get("name").and_then(|v| v.as_str()) {
        dsl.push_str(&format!("specification \"{}\"", name));

        let mut has_fields = false;
        let mut fields = String::new();

        if let Some(description) = spec.get("description").and_then(|v| v.as_str()) {
            fields.push_str(&format!("\n    description \"{}\"", description));
            has_fields = true;
        }

        if let Some(version) = spec.get("version").and_then(|v| v.as_str()) {
            fields.push_str(&format!("\n    version \"{}\"", version));
            has_fields = true;
        }

        if let Some(authors) = spec.get("authors").and_then(|v| v.as_array()) {
            let author_list: Vec<&str> = authors.iter().filter_map(|v| v.as_str()).collect();
            if !author_list.is_empty() {
                fields.push_str(&format!(
                    "\n    authors {}",
                    author_list
                        .iter()
                        .map(|a| format!("\"{}\"", a))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                has_fields = true;
            }
        }

        if has_fields {
            dsl.push_str(" {");
            dsl.push_str(&fields);
            dsl.push_str("\n}\n\n");
        } else {
            dsl.push('\n');
        }
    }
}

fn write_element(dsl: &mut String, id: &str, elem: &serde_json::Value, indent: usize) {
    let indent_str = " ".repeat(indent);
    let inner_indent = " ".repeat(indent + 4);

    // Get element properties
    let kind = elem
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("system");
    let title = elem.get("title").and_then(|v| v.as_str()).unwrap_or(id);
    let description = elem.get("description").and_then(|v| v.as_str());
    let technology = elem.get("technology").and_then(|v| v.as_str());
    let tags = elem.get("tags").and_then(|v| v.as_array());
    let metadata = elem.get("metadata").and_then(|v| v.as_object());
    let style = elem.get("style").and_then(|v| v.as_object());

    // Check if element has a body (nested elements or additional properties)
    let has_body = description.is_some()
        || technology.is_some()
        || tags.is_some_and(|t| !t.is_empty())
        || metadata.is_some_and(|m| !m.is_empty())
        || style.is_some();

    // Write element declaration
    dsl.push_str(&indent_str);
    dsl.push_str(kind);
    dsl.push(' ');
    dsl.push_str(id);
    dsl.push_str(" \"");
    dsl.push_str(title);
    dsl.push('\"');

    if has_body {
        dsl.push_str(" {");

        // Description
        if let Some(desc) = description {
            dsl.push('\n');
            dsl.push_str(&inner_indent);
            dsl.push_str(&format!("description \"{}\"", desc));
        }

        // Technology
        if let Some(tech) = technology {
            dsl.push('\n');
            dsl.push_str(&inner_indent);
            dsl.push_str(&format!("technology \"{}\"", tech));
        }

        // Tags
        if let Some(tag_list) = tags {
            if !tag_list.is_empty() {
                dsl.push('\n');
                dsl.push_str(&inner_indent);
                dsl.push_str("tags ");
                dsl.push_str(
                    &tag_list
                        .iter()
                        .map(|t| format!("\"{}\"", t.as_str().unwrap_or("")))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
            }
        }

        // Metadata
        if let Some(meta) = metadata {
            if !meta.is_empty() {
                for (key, value) in meta {
                    if let Some(val_str) = value.as_str() {
                        dsl.push('\n');
                        dsl.push_str(&inner_indent);
                        dsl.push_str(&format!("metadata {} = \"{}\"", key, val_str));
                    }
                }
            }
        }

        // Style
        if let Some(style_obj) = style {
            for (key, value) in style_obj {
                dsl.push('\n');
                dsl.push_str(&inner_indent);
                dsl.push_str("style ");
                dsl.push_str(key);
                dsl.push_str(" = ");

                if let Some(s) = value.as_str() {
                    dsl.push('"');
                    dsl.push_str(s);
                    dsl.push('"');
                } else if let Some(n) = value.as_f64() {
                    dsl.push_str(&n.to_string());
                } else if let Some(b) = value.as_bool() {
                    dsl.push_str(&b.to_string());
                }
            }
        }

        dsl.push('\n');
        dsl.push_str(&indent_str);
        dsl.push('}');
    } else {
        dsl.push('\n');
    }
}

fn write_relation(dsl: &mut String, rel: &serde_json::Value) {
    let source = rel.get("source").and_then(|v| v.as_str()).unwrap_or("");
    let target = rel.get("target").and_then(|v| v.as_str()).unwrap_or("");
    let title = rel.get("title").and_then(|v| v.as_str());
    let description = rel.get("description").and_then(|v| v.as_str());
    let technology = rel.get("technology").and_then(|v| v.as_str());
    let tags = rel.get("tags").and_then(|v| v.as_array());

    // Write relation
    dsl.push_str(source);
    dsl.push_str(" -> ");
    dsl.push_str(target);

    if let Some(label) = title {
        dsl.push_str(" \"");
        dsl.push_str(label);
        dsl.push('\"');
    }

    // Check if relation has a body
    let has_body =
        description.is_some() || technology.is_some() || tags.is_some_and(|t| !t.is_empty());

    if has_body {
        dsl.push_str(" {");

        if let Some(desc) = description {
            dsl.push_str(&format!("\n    description \"{}\"", desc));
        }

        if let Some(tech) = technology {
            dsl.push_str(&format!("\n    technology \"{}\"", tech));
        }

        if let Some(tag_list) = tags {
            if !tag_list.is_empty() {
                dsl.push_str(&format!(
                    "\n    tags {}",
                    tag_list
                        .iter()
                        .map(|t| format!("\"{}\"", t.as_str().unwrap_or("")))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        dsl.push_str("\n}\n");
    } else {
        dsl.push('\n');
    }
}

fn write_view(dsl: &mut String, view_id: &str, view: &serde_json::Value) {
    let title = view
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or(view_id);
    let description = view.get("description").and_then(|v| v.as_str());
    let kind = view
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("system");
    let element_ids = view.get("element_ids").and_then(|v| v.as_array());
    let include = view.get("include").and_then(|v| v.as_array());
    let exclude = view.get("exclude").and_then(|v| v.as_array());

    // Write view declaration
    dsl.push_str(&format!("view {} \"{}\"", view_id, title));

    // Check if view has a body
    let has_body = description.is_some()
        || element_ids.is_some_and(|ids| !ids.is_empty())
        || include.is_some_and(|ids| !ids.is_empty())
        || exclude.is_some_and(|ids| !ids.is_empty());

    if has_body {
        dsl.push_str(" {");

        if let Some(desc) = description {
            dsl.push_str(&format!("\n    description \"{}\"", desc));
        }

        dsl.push_str(&format!("\n    kind \"{}\"", kind));

        if let Some(ids) = element_ids {
            if !ids.is_empty() {
                dsl.push_str("\n    element_ids ");
                dsl.push_str(
                    &ids.iter()
                        .map(|id| format!("\"{}\"", id.as_str().unwrap_or("")))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
            }
        }

        if let Some(includes) = include {
            if !includes.is_empty() {
                for inc in includes {
                    if let Some(id) = inc.as_str() {
                        dsl.push_str(&format!("\n    include \"{}\"", id));
                    }
                }
            }
        }

        if let Some(excludes) = exclude {
            if !excludes.is_empty() {
                for exc in excludes {
                    if let Some(id) = exc.as_str() {
                        dsl.push_str(&format!("\n    exclude \"{}\"", id));
                    }
                }
            }
        }

        dsl.push_str("\n}\n");
    } else {
        dsl.push('\n');
    }
}

#[wasm_bindgen]
pub fn sruja_get_diagnostics(dsl: &str, filename: Option<String>) -> Result<String, JsValue> {
    let filename = filename.unwrap_or_else(|| "input.sruja".to_string());
    let parser = Parser::new(filename.clone());
    let (parse_diagnostics, program) = match parser.parse(dsl) {
        Ok(p) => (Vec::new(), Some(p)),
        Err(diags) => (diags, None),
    };

    let mut all_diagnostics = parse_diagnostics;

    if let Some(prog) = program {
        let validator = Validator::with_default_rules();
        let validation_diagnostics = validator.validate_sync(&prog);
        all_diagnostics.extend(validation_diagnostics);
    }

    let diagnostics_json: Vec<serde_json::Value> = all_diagnostics
        .iter()
        .map(|d| {
            // We currently only have (line, column) in `SourceLocation`; use a 1-character span.
            let end_character = d.location.column.saturating_add(1);
            json!({
                "range": {
                    "start": {"line": d.location.line, "character": d.location.column},
                    "end": {"line": d.location.line, "character": end_character}
                },
                "severity": if d.severity == sruja_diagnostics::Severity::Error { 1 } else { 2 },
                "code": d.code.clone(),
                "message": d.message.clone(),
                "source": "sruja"
            })
        })
        .collect();

    serde_json::to_string(&diagnostics_json)
        .map_err(|e| JsValue::from_str(&format!("JSON error: {:?}", e)))
}

#[wasm_bindgen]
pub fn sruja_calculate_architecture_score(dsl: &str) -> Result<String, JsValue> {
    let parser = Parser::new("input.sruja".to_string());
    let program = parser
        .parse(dsl)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {:?}", e)))?;

    let validator = Validator::with_default_rules();
    let diagnostics = validator.validate_sync(&program);

    // Calculate score (100 - deductions)
    let mut score: i32 = 100;
    let mut deductions = Vec::new();

    for diag in &diagnostics {
        let points = match diag.severity {
            sruja_diagnostics::Severity::Error => 5,
            sruja_diagnostics::Severity::Warning => 2,
            _ => 0,
        };
        if points > 0 {
            score = score.saturating_sub(points);
            deductions.push(json!({
                "message": diag.message.clone(),
                "points": points,
                "code": diag.code.clone()
            }));
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

    let result = json!({
        "score": score,
        "grade": grade,
        "deductions": deductions,
        "categories": {
            "structural": score,
            "documentation": score,
            "traceability": score,
            "complexity": score,
            "standardization": score
        }
    });

    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&format!("JSON error: {:?}", e)))
}

#[wasm_bindgen]
pub fn sruja_get_elements(dsl: &str, filename: Option<String>) -> Result<String, JsValue> {
    let filename = filename.unwrap_or_else(|| "input.sruja".to_string());
    let parser = Parser::new(filename.clone());
    let program = match parser.parse(dsl) {
        Ok(p) => p,
        Err(_) => {
            return Ok("[]".to_string());
        }
    };

    let (elements, _) = sruja_language::collect_elements(&program);

    let elements_json: Vec<serde_json::Value> = elements
        .iter()
        .map(|(fqn, elem)| {
            let doc = elem.assignment.body.as_ref().and_then(|b| b.doc.clone());
            let short_name = fqn.rsplit('.').next().unwrap_or(fqn.as_str());
            let (line, col) = if elem.location.line > 0 || elem.location.column > 0 {
                (elem.location.line, elem.location.column)
            } else if let Some((l, c)) = sruja_language::find_definition_line(dsl, short_name) {
                (l + 1, c + 1)
            } else {
                (1, 1)
            };
            let end_col = col + short_name.len() as u32;
            json!({
                "id": fqn,
                "kind": elem.assignment.kind.to_string(),
                "title": elem.assignment.title,
                "doc": doc,
                "range": {
                    "start": {"line": line, "character": col},
                    "end": {"line": line, "character": end_col}
                }
            })
        })
        .collect();

    serde_json::to_string(&elements_json)
        .map_err(|e| JsValue::from_str(&format!("JSON error: {:?}", e)))
}

#[wasm_bindgen]
pub fn sruja_get_document_symbols(dsl: &str, filename: Option<String>) -> Result<String, JsValue> {
    let filename = filename.unwrap_or_else(|| "input.sruja".to_string());
    let parser = Parser::new(filename.clone());
    let program = match parser.parse(dsl) {
        Ok(p) => p,
        Err(_) => {
            return Ok("[]".to_string());
        }
    };

    let (elements, _) = sruja_language::collect_elements(&program);

    fn symbol_range(
        dsl: &str,
        location: &sruja_diagnostics::SourceLocation,
        def_name: &str,
    ) -> (u32, u32, u32) {
        let (line, col) = if location.line > 0 || location.column > 0 {
            (location.line, location.column)
        } else if let Some((l, c)) = sruja_language::find_definition_line(dsl, def_name) {
            (l + 1, c + 1)
        } else {
            (1, 1)
        };
        let end_col = col + def_name.len() as u32;
        (line, col, end_col)
    }

    let mut symbols = Vec::new();

    for (fqn, elem) in elements {
        let kind = elem.assignment.kind.to_string();
        let short_name = fqn.rsplit('.').next().unwrap_or(fqn.as_str());
        let (line, ch, end_ch) = symbol_range(dsl, &elem.location, short_name);
        symbols.push(json!({
            "kind": "element",
            "name": fqn,
            "detail": kind,
            "range": {
                "start": {"line": line, "character": ch},
                "end": {"line": line, "character": end_ch}
            },
            "children": []
        }));
    }

    for item in &program.items {
        match item {
            sruja_language::TopLevelItem::View(view) => {
                let (line, ch, end_ch) = symbol_range(dsl, &view.location, &view.id);
                symbols.push(json!({
                    "kind": "view",
                    "name": view.id.clone(),
                    "detail": "View",
                    "range": {
                        "start": {"line": line, "character": ch},
                        "end": {"line": line, "character": end_ch}
                    },
                    "children": []
                }));
            }
            sruja_language::TopLevelItem::Scenario(scenario) => {
                let (line, ch, end_ch) = symbol_range(dsl, &scenario.location, &scenario.id);
                symbols.push(json!({
                    "kind": "scenario",
                    "name": scenario.id.clone(),
                    "detail": "Scenario",
                    "range": {
                        "start": {"line": line, "character": ch},
                        "end": {"line": line, "character": end_ch}
                    },
                    "children": []
                }));
            }
            sruja_language::TopLevelItem::Flow(flow) => {
                let (line, ch, end_ch) = symbol_range(dsl, &flow.location, &flow.id);
                symbols.push(json!({
                    "kind": "flow",
                    "name": flow.id.clone(),
                    "detail": "Flow",
                    "range": {
                        "start": {"line": line, "character": ch},
                        "end": {"line": line, "character": end_ch}
                    },
                    "children": []
                }));
            }
            sruja_language::TopLevelItem::Requirement(req) => {
                let (line, ch, end_ch) = symbol_range(dsl, &req.location, &req.id);
                symbols.push(json!({
                    "kind": "requirement",
                    "name": req.id.clone(),
                    "detail": req.r#type.clone(),
                    "range": {
                        "start": {"line": line, "character": ch},
                        "end": {"line": line, "character": end_ch}
                    },
                    "children": []
                }));
            }
            sruja_language::TopLevelItem::Adr(adr) => {
                let (line, ch, end_ch) = symbol_range(dsl, &adr.location, &adr.id);
                symbols.push(json!({
                    "kind": "adr",
                    "name": adr.id.clone(),
                    "detail": "ADR",
                    "range": {
                        "start": {"line": line, "character": ch},
                        "end": {"line": line, "character": end_ch}
                    },
                    "children": []
                }));
            }
            sruja_language::TopLevelItem::Policy(policy) => {
                let (line, ch, end_ch) = symbol_range(dsl, &policy.location, &policy.id);
                symbols.push(json!({
                    "kind": "policy",
                    "name": policy.id.clone(),
                    "detail": format!("{} ({})", policy.title, policy.category),
                    "range": {
                        "start": {"line": line, "character": ch},
                        "end": {"line": line, "character": end_ch}
                    },
                    "children": []
                }));
            }
            _ => {}
        }
    }

    serde_json::to_string(&symbols).map_err(|e| JsValue::from_str(&format!("JSON error: {:?}", e)))
}

#[cfg(test)]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn dsl_to_model_valid() {
        let dsl = r#"S = system "My System" { description "A system" }"#;
        let out = sruja_dsl_to_model(dsl, None).unwrap();
        assert!(out.contains("elements"));
        assert!(out.contains("My System"));
    }

    #[wasm_bindgen_test]
    fn dsl_to_model_invalid_returns_err() {
        let dsl = "{{{";
        let res = sruja_dsl_to_model(dsl, None);
        assert!(res.is_err());
    }

    #[wasm_bindgen_test]
    fn dsl_to_mermaid_valid() {
        let dsl = r#"S = system "S" { description "S" }"#;
        let out = sruja_dsl_to_mermaid(dsl, None).unwrap();
        assert!(!out.is_empty());
        assert!(out.contains("flowchart") || out.contains("graph"));
    }

    #[wasm_bindgen_test]
    fn get_diagnostics_valid_dsl() {
        let dsl = r#"S = system "S" { description "S" }"#;
        let out = sruja_get_diagnostics(dsl, None).unwrap();
        // Returns a JSON array of diagnostic objects
        assert!(out.starts_with('['));
    }

    #[wasm_bindgen_test]
    fn get_diagnostics_invalid_dsl_returns_diagnostics_array() {
        let dsl = "broken {{{";
        let out = sruja_get_diagnostics(dsl, None).unwrap();
        assert!(out.starts_with('['));
        let arr: Vec<serde_json::Value> = serde_json::from_str(&out).expect("valid JSON array");
        assert!(
            !arr.is_empty(),
            "invalid DSL should produce at least one diagnostic"
        );
    }

    #[wasm_bindgen_test]
    fn incremental_parse_returns_expected_shape() {
        let dsl = r#"S = system "S" { description "S" }"#;
        let out = sruja_incremental_parse(dsl, 0, 0, "{}", 0, None).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert!(v.get("updated_ast").is_some());
        assert!(v.get("changed_elements").is_some());
        assert!(v.get("changed_ranges").is_some());
        assert!(v.get("parsing_time_ms").is_some());
    }
}
