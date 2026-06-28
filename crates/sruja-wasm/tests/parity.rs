use serde_json::Value;
use sruja_engine::Validator;
use sruja_export::mermaid::exporter::MermaidConfig;
use sruja_export::mermaid::MermaidExporter;
use sruja_language::Parser;
use wasm_bindgen_test::wasm_bindgen_test;

use sruja_wasm::{sruja_dsl_to_mermaid, sruja_get_diagnostics};

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
