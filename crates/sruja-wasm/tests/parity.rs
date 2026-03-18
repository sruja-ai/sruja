use serde_json::Value;
use sruja_engine::Validator;
use sruja_export::json::Exporter as JsonExporter;
use sruja_export::mermaid::exporter::MermaidConfig;
use sruja_export::mermaid::MermaidExporter;
use sruja_language::Parser;
use wasm_bindgen_test::wasm_bindgen_test;

use sruja_wasm::{sruja_dsl_to_mermaid, sruja_dsl_to_model, sruja_get_diagnostics};

const GOLDEN_MICROSERVICES: &str =
    include_str!("../../../book/valid-examples/pattern-microservices.sruja");

fn parse_ok(dsl: &str, filename: &str) -> sruja_language::Program {
    Parser::new(filename.to_string())
        .parse(dsl)
        .expect("parse ok")
}

fn diag_signature(d: &sruja_diagnostics::Diagnostic) -> (String, u8) {
    let severity = if d.severity == sruja_diagnostics::Severity::Error {
        1
    } else {
        2
    };
    (d.code.clone(), severity)
}

fn wasm_diag_signature(v: &Value) -> (String, u8) {
    let code = v
        .get("code")
        .and_then(|c| c.as_str())
        .unwrap_or_default()
        .to_string();
    let severity = v.get("severity").and_then(|s| s.as_u64()).unwrap_or(2) as u8;
    (code, severity)
}

fn first_json_mismatch(left: &Value, right: &Value) -> Option<(String, Value, Value)> {
    fn walk(path: &str, left: &Value, right: &Value) -> Option<(String, Value, Value)> {
        if left == right {
            return None;
        }

        match (left, right) {
            (Value::Object(a), Value::Object(b)) => {
                let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
                keys.sort();
                keys.dedup();
                for key in keys {
                    let next_path = format!("{path}.{}", key);
                    let lv = a.get(key).unwrap_or(&Value::Null);
                    let rv = b.get(key).unwrap_or(&Value::Null);
                    if let Some(m) = walk(&next_path, lv, rv) {
                        return Some(m);
                    }
                }
                Some((path.to_string(), left.clone(), right.clone()))
            }
            (Value::Array(a), Value::Array(b)) => {
                if a.len() != b.len() {
                    return Some((path.to_string(), left.clone(), right.clone()));
                }
                for (idx, (lv, rv)) in a.iter().zip(b.iter()).enumerate() {
                    let next_path = format!("{path}[{idx}]");
                    if let Some(m) = walk(&next_path, lv, rv) {
                        return Some(m);
                    }
                }
                Some((path.to_string(), left.clone(), right.clone()))
            }
            _ => Some((path.to_string(), left.clone(), right.clone())),
        }
    }

    walk("$", left, right)
}

fn normalize_model_json(v: &mut Value) {
    if let Some(obj) = v.as_object_mut() {
        if let Some(meta) = obj.get_mut("_metadata").and_then(|m| m.as_object_mut()) {
            meta.remove("generated");
        }
        if let Some(meta) = obj.get_mut("metadata").and_then(|m| m.as_object_mut()) {
            meta.remove("generated");
        }
    }
}

#[wasm_bindgen_test]
fn wasm_export_json_matches_core_exporter() {
    let mut expected = {
        let program = parse_ok(GOLDEN_MICROSERVICES, "pattern-microservices.sruja");
        let exporter = JsonExporter::new();
        serde_json::from_str::<Value>(&exporter.export(&program).expect("export ok"))
            .expect("valid json")
    };

    let mut actual = serde_json::from_str::<Value>(
        &sruja_dsl_to_model(
            GOLDEN_MICROSERVICES,
            Some("pattern-microservices.sruja".to_string()),
        )
        .expect("wasm export ok"),
    )
    .expect("valid json");

    normalize_model_json(&mut actual);
    normalize_model_json(&mut expected);

    if actual != expected {
        let (path, left, right) = first_json_mismatch(&actual, &expected)
            .unwrap_or_else(|| ("$".to_string(), actual.clone(), expected.clone()));
        panic!(
            "json mismatch at {path}\nactual: {}\nexpected: {}",
            serde_json::to_string_pretty(&left).unwrap(),
            serde_json::to_string_pretty(&right).unwrap()
        );
    }
}

#[wasm_bindgen_test]
fn wasm_export_mermaid_matches_core_exporter() {
    let program = parse_ok(GOLDEN_MICROSERVICES, "pattern-microservices.sruja");
    let expected = MermaidExporter::new(MermaidConfig::default()).export(&program);
    let actual = sruja_dsl_to_mermaid(GOLDEN_MICROSERVICES, None).expect("wasm mermaid ok");
    assert_eq!(actual, expected);
}

#[wasm_bindgen_test]
fn wasm_diagnostics_match_core_parser_and_validator_on_valid_file() {
    let actual = serde_json::from_str::<Vec<Value>>(
        &sruja_get_diagnostics(
            GOLDEN_MICROSERVICES,
            Some("pattern-microservices.sruja".to_string()),
        )
        .expect("wasm diagnostics ok"),
    )
    .expect("valid json");

    let expected = {
        let program = parse_ok(GOLDEN_MICROSERVICES, "pattern-microservices.sruja");
        let validator = Validator::with_default_rules();
        validator
            .validate_sync(&program)
            .iter()
            .map(diag_signature)
            .collect::<Vec<_>>()
    };

    let mut actual_sig = actual
        .into_iter()
        .map(|v| wasm_diag_signature(&v))
        .collect::<Vec<_>>();
    let mut expected_sig = expected;
    actual_sig.sort();
    expected_sig.sort();

    assert_eq!(actual_sig, expected_sig);
}

#[wasm_bindgen_test]
fn wasm_diagnostics_match_core_parser_and_validator_on_parse_error() {
    let invalid = "Foo = container \"Foo\" {\n  description \"x\"\n";

    let actual = serde_json::from_str::<Vec<Value>>(
        &sruja_get_diagnostics(invalid, Some("invalid.sruja".to_string()))
            .expect("wasm diagnostics ok"),
    )
    .expect("valid json");

    let expected = {
        let parser = Parser::new("invalid.sruja".to_string());
        let (parse_diags, program) = match parser.parse(invalid) {
            Ok(p) => (Vec::new(), Some(p)),
            Err(diags) => (diags, None),
        };

        let mut all = parse_diags;
        if let Some(p) = program {
            let validator = Validator::with_default_rules();
            all.extend(validator.validate_sync(&p));
        }
        all.iter().map(diag_signature).collect::<Vec<_>>()
    };

    let mut actual_sig = actual
        .into_iter()
        .map(|v| wasm_diag_signature(&v))
        .collect::<Vec<_>>();
    let mut expected_sig = expected;
    actual_sig.sort();
    expected_sig.sort();

    assert_eq!(actual_sig, expected_sig);
}
