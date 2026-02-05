//! Sruja WASM bindings for browser usage
//!
//! This crate provides WebAssembly bindings for Sruja functionality,
//! allowing to website and other frontend applications to use Sruja
//! in browser.

use serde_json::json;
use sruja_engine::Validator;
use sruja_export::dot::{DotConfig, DotExporter};
use sruja_export::json::Exporter as JsonExporter;
use sruja_export::markdown::{MarkdownExporter, MarkdownOptions};
use sruja_export::mermaid::exporter::MermaidConfig;
use sruja_export::mermaid::MermaidExporter;
use sruja_language::Parser;
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
    let start = std::time::Instant::now();
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

    let elapsed_ms = start.elapsed().as_millis() as u64;
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
pub fn sruja_dsl_to_dot(
    dsl: &str,
    view_level: Option<u8>,
    target_id: Option<String>,
    node_sizes_json: Option<String>,
    view_id: Option<String>,
    filename: Option<String>,
) -> Result<String, JsValue> {
    let filename = filename.unwrap_or_else(|| "input.sruja".to_string());
    let parser = Parser::new(filename.clone());
    let program = parser
        .parse(dsl)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {:?}", e)))?;

    // Parse node_sizes JSON if provided
    let mut node_sizes = std::collections::HashMap::new();
    if let Some(sizes_json) = node_sizes_json {
        if let Ok(sizes) = serde_json::from_str::<serde_json::Value>(&sizes_json) {
            if let Some(obj) = sizes.as_object() {
                for (key, value) in obj {
                    if let Some(arr) = value.as_array() {
                        if arr.len() >= 2 {
                            if let (Some(w), Some(h)) = (
                                arr.get(0).and_then(|v| v.as_f64()),
                                arr.get(1).and_then(|v| v.as_f64()),
                            ) {
                                node_sizes.insert(key.clone(), (w, h));
                            }
                        }
                    }
                }
            }
        }
    }

    let dot_config = DotConfig {
        rank_dir: "TB".to_string(),
        node_sep: 0.5,
        rank_sep: 0.8,
        view_level: view_level.unwrap_or(1),
        target_id,
        node_sizes,
        view_id,
        filename: Some(filename),
    };

    let exporter = DotExporter::new(dot_config);
    Ok(exporter.export(&program))
}

/// Export DOT and return JSON with both DOT string and projected relations
#[wasm_bindgen]
pub fn sruja_dsl_to_dot_with_relations(
    dsl: &str,
    view_level: Option<u8>,
    target_id: Option<String>,
    node_sizes_json: Option<String>,
    view_id: Option<String>,
    filename: Option<String>,
) -> Result<String, JsValue> {
    let filename = filename.unwrap_or_else(|| "input.sruja".to_string());
    let parser = Parser::new(filename.clone());
    let program = parser
        .parse(dsl)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {:?}", e)))?;

    // Parse node_sizes JSON if provided
    let mut node_sizes = std::collections::HashMap::new();
    if let Some(sizes_json) = node_sizes_json {
        if let Ok(sizes) = serde_json::from_str::<serde_json::Value>(&sizes_json) {
            if let Some(obj) = sizes.as_object() {
                for (key, value) in obj {
                    if let Some(arr) = value.as_array() {
                        if arr.len() >= 2 {
                            if let (Some(w), Some(h)) = (
                                arr.get(0).and_then(|v| v.as_f64()),
                                arr.get(1).and_then(|v| v.as_f64()),
                            ) {
                                node_sizes.insert(key.clone(), (w, h));
                            }
                        }
                    }
                }
            }
        }
    }

    let dot_config = DotConfig {
        rank_dir: "TB".to_string(),
        node_sep: 0.5,
        rank_sep: 0.8,
        view_level: view_level.unwrap_or(1),
        target_id,
        node_sizes,
        view_id,
        filename: Some(filename),
    };

    let exporter = DotExporter::new(dot_config);
    let (dot, view_elements, relations) = exporter.export_with_relations(&program);

    // Convert relations to JSON format
    let relations_json: Vec<serde_json::Value> = relations
        .iter()
        .map(|rel| {
            serde_json::json!({
                "from": rel.from.as_string(),
                "to": rel.to.as_string(),
                "label": rel.label.as_ref().or(rel.description.as_ref()),
            })
        })
        .collect();

    // Convert projected elements to JSON format
    let elements_json: std::collections::HashMap<String, serde_json::Value> = view_elements
        .iter()
        .map(|(fqn, elem)| {
            let kind = elem.assignment.kind.to_string();
            let title = elem.assignment.name.clone();
            let description = elem
                .assignment
                .body
                .as_ref()
                .and_then(|b| b.description.clone());
            let technology = elem
                .assignment
                .body
                .as_ref()
                .and_then(|b| b.technology.clone());
            let parent = if let Some(dot_idx) = fqn.rfind('.') {
                Some(fqn[..dot_idx].to_string())
            } else {
                None
            };

            (
                fqn.clone(),
                serde_json::json!({
                    "id": fqn,
                    "kind": kind,
                    "title": title,
                    "description": description,
                    "technology": technology,
                    "parent": parent,
                }),
            )
        })
        .collect();

    let result = serde_json::json!({
        "dot": dot,
        "elements": elements_json,
        "relations": relations_json,
    });

    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&format!("JSON error: {:?}", e)))
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
        || tags.map_or(false, |t| !t.is_empty())
        || metadata.map_or(false, |m| !m.is_empty())
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
        description.is_some() || technology.is_some() || tags.map_or(false, |t| !t.is_empty());

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
        || element_ids.map_or(false, |ids| !ids.is_empty())
        || include.map_or(false, |ids| !ids.is_empty())
        || exclude.map_or(false, |ids| !ids.is_empty());

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

    let diagnostics_json: Vec<serde_json::Value> = all_diagnostics.iter().map(|d| {
        // We currently only have (line, column) in `SourceLocation`; use a 1-character span.
        let end_character = d.location.column.saturating_add(1);
        json!({
            "range": {
                "start": {"line": d.location.line as u32, "character": d.location.column as u32},
                "end": {"line": d.location.line as u32, "character": end_character as u32}
            },
            "severity": if d.severity == sruja_diagnostics::Severity::Error { 1 } else { 2 },
            "code": d.code.clone(),
            "message": d.message.clone(),
            "source": "sruja"
        })
    }).collect();

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
