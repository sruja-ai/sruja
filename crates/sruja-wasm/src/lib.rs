//! Sruja WASM bindings for browser usage
//!
//! This crate provides WebAssembly bindings for Sruja functionality,
//! allowing to website and other frontend applications to use Sruja
//! in browser.

use serde_json::json;
use sruja_engine::Validator;
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
                let mut sym = json!({
                    "kind": "requirement",
                    "name": req.id.clone(),
                    "detail": req.r#type.clone(),
                    "range": {
                        "start": {"line": line, "character": ch},
                        "end": {"line": line, "character": end_ch}
                    },
                    "children": []
                });
                if let Some(priority) = &req.priority {
                    sym["priority"] = json!(priority);
                }
                if let Some(status) = &req.status {
                    sym["status"] = json!(status);
                }
                if !req.affects.is_empty() {
                    sym["affects"] = json!(req.affects);
                }
                if !req.scenarios.is_empty() {
                    sym["scenarios"] = json!(req.scenarios);
                }
                if !req.adrs.is_empty() {
                    sym["adrs"] = json!(req.adrs);
                }
                if let Some(source) = &req.source {
                    sym["source"] = json!(source);
                }
                symbols.push(sym);
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
}
