//! Sruja WASM bindings for browser usage
//!
//! This crate provides WebAssembly bindings for Sruja functionality,
//! allowing the website and other frontend applications to use Sruja
//! in the browser.

use wasm_bindgen::prelude::*;
use sruja_language::Parser;
use sruja_engine::Validator;
use sruja_export::json::Exporter as JsonExporter;
use sruja_export::mermaid::{MermaidExporter, MermaidConfig};
use sruja_export::dot::{DotExporter, DotConfig};
use sruja_export::markdown::{MarkdownExporter, MarkdownOptions};
use sruja_export::dsl::DslPrinter;
use serde_json::json;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn sruja_dsl_to_model(dsl: &str, filename: Option<String>) -> Result<String, JsValue> {
    let parser = Parser::new(filename.unwrap_or_else(|| "input.sruja".to_string()));
    let program = parser.parse(dsl).map_err(|e| {
        JsValue::from_str(&format!("Parse error: {:?}", e))
    })?;
    
    let exporter = JsonExporter::new();
    let result = exporter.export(&program).map_err(|e| {
        JsValue::from_str(&format!("Export error: {:?}", e))
    })?;
    
    serde_json::to_string(&result).map_err(|e| {
        JsValue::from_str(&format!("JSON error: {:?}", e))
    })
}

#[wasm_bindgen]
pub fn sruja_dsl_to_mermaid(dsl: &str, config_json: Option<String>) -> Result<String, JsValue> {
    let parser = Parser::new("input.sruja".to_string());
    let program = parser.parse(dsl).map_err(|e| {
        JsValue::from_str(&format!("Parse error: {:?}", e))
    })?;
    
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
pub fn sruja_dsl_to_dot(dsl: &str, view_level: Option<u8>, target_id: Option<String>) -> Result<String, JsValue> {
    let parser = Parser::new("input.sruja".to_string());
    let program = parser.parse(dsl).map_err(|e| {
        JsValue::from_str(&format!("Parse error: {:?}", e))
    })?;
    
    let dot_config = DotConfig {
        view_level: view_level.unwrap_or(1),
        target_id,
        ..DotConfig::default()
    };
    
    let exporter = DotExporter::new(dot_config);
    Ok(exporter.export(&program))
}

#[wasm_bindgen]
pub fn sruja_dsl_to_markdown(dsl: &str) -> Result<String, JsValue> {
    let parser = Parser::new("input.sruja".to_string());
    let program = parser.parse(dsl).map_err(|e| {
        JsValue::from_str(&format!("Parse error: {:?}", e))
    })?;
    
    let exporter = MarkdownExporter::new(MarkdownOptions::default());
    Ok(exporter.export(&program))
}

#[wasm_bindgen]
pub fn sruja_model_to_dsl(model_json: &str) -> Result<String, JsValue> {
    // Parse JSON model
    let model: serde_json::Value = serde_json::from_str(model_json).map_err(|e| {
        JsValue::from_str(&format!("JSON parse error: {:?}", e))
    })?;
    
    // For now, return a basic DSL representation
    // TODO: Implement full model-to-DSL conversion
    let mut dsl = String::new();
    
    if let Some(elements) = model.get("elements").and_then(|v| v.as_array()) {
        for elem in elements {
            if let (Some(kind), Some(id), Some(title)) = (
                elem.get("kind").and_then(|v| v.as_str()),
                elem.get("id").and_then(|v| v.as_str()),
                elem.get("title").and_then(|v| v.as_str()),
            ) {
                dsl.push_str(&format!("{} {} \"{}\"\n", kind.to_lowercase(), id, title));
            }
        }
    }
    
    Ok(dsl)
}

#[wasm_bindgen]
pub fn sruja_get_diagnostics(dsl: &str, filename: Option<String>) -> Result<String, JsValue> {
    let parser = Parser::new(filename.unwrap_or_else(|| "input.sruja".to_string()));
    let (parse_diagnostics, program) = match parser.parse(dsl) {
        Ok(p) => (Vec::new(), Some(p)),
        Err(diags) => (diags, None),
    };
    
    let mut all_diagnostics = parse_diagnostics;
    
    if let Some(prog) = program {
        let mut validator = Validator::new();
        validator.register_default_rules();
        let validation_diagnostics = validator.validate_sync(&prog);
        all_diagnostics.extend(validation_diagnostics);
    }
    
    let diagnostics_json: Vec<serde_json::Value> = all_diagnostics.iter().map(|d| {
        json!({
            "range": {
                "start": {"line": d.location.line as u32, "character": d.location.column as u32},
                "end": {"line": d.location.line as u32, "character": (d.location.column + d.location.length) as u32}
            },
            "severity": if d.severity == sruja_diagnostics::Severity::Error { 1 } else { 2 },
            "code": d.code.clone(),
            "message": d.message.clone(),
            "source": "sruja"
        })
    }).collect();
    
    serde_json::to_string(&diagnostics_json).map_err(|e| {
        JsValue::from_str(&format!("JSON error: {:?}", e))
    })
}

#[wasm_bindgen]
pub fn sruja_calculate_architecture_score(dsl: &str) -> Result<String, JsValue> {
    let parser = Parser::new("input.sruja".to_string());
    let program = parser.parse(dsl).map_err(|e| {
        JsValue::from_str(&format!("Parse error: {:?}", e))
    })?;
    
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
    
    serde_json::to_string(&result).map_err(|e| {
        JsValue::from_str(&format!("JSON error: {:?}", e))
    })
}
