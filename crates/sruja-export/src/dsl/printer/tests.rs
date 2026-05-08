//! DSL printer tests.

use super::DslPrinter;
use crate::json::Exporter as JsonExporter;
use serde_json::Value;
use sruja_diagnostics::SourceLocation;
use sruja_language::Parser;
use std::fs;
use std::path::{Path, PathBuf};

fn parse(input: &str) -> sruja_language::Program {
    Parser::new("test.sruja".to_string())
        .parse(input)
        .expect("Failed to parse")
}

fn parse_with_filename(filename: &str, input: &str) -> sruja_language::Program {
    Parser::new(filename.to_string())
        .parse(input)
        .expect("Failed to parse")
}

fn export_model_json(program: &sruja_language::Program) -> Value {
    let exporter = JsonExporter::new();
    let json = exporter.export(program).expect("export json");
    serde_json::from_str(&json).expect("valid json")
}

fn normalize_model_json(mut model: Value) -> Value {
    let Some(obj) = model.as_object_mut() else {
        return model;
    };

    if let Some(Value::Object(metadata)) = obj.get_mut("_metadata") {
        metadata.remove("generated");
    }

    if let Some(Value::Array(relations)) = obj.get_mut("relations") {
        relations.sort_by_key(|a| a.to_string());
    }

    if let Some(Value::Object(sruja)) = obj.get_mut("sruja") {
        if let Some(Value::Array(requirements)) = sruja.get_mut("requirements") {
            for req in requirements.iter_mut() {
                let Some(req_obj) = req.as_object_mut() else {
                    continue;
                };
                let title = req_obj.get("title").and_then(|v| v.as_str());
                let description = req_obj.get("description").and_then(|v| v.as_str());
                if matches!((title, description), (Some(t), Some(d)) if t == d) {
                    req_obj.remove("description");
                }
            }
            requirements.sort_by_key(|a| a.to_string());
        }
    }

    model
}

fn assert_semantic_roundtrip(filename: &str, input: &str) {
    let program = parse_with_filename(filename, input);
    let printer = DslPrinter::new();
    let printed = printer.print(&program);
    let reparsed = parse_with_filename(filename, &printed);

    let a = normalize_model_json(export_model_json(&program));
    let b = normalize_model_json(export_model_json(&reparsed));
    if a != b {
        let left = serde_json::to_string(&a).unwrap_or_default();
        let right = serde_json::to_string(&b).unwrap_or_default();
        let min_len = left.len().min(right.len());
        let mut i = 0;
        while i < min_len && left.as_bytes()[i] == right.as_bytes()[i] {
            i += 1;
        }
        let start = i.saturating_sub(200);
        let end = (i + 200).min(min_len);
        panic!(
            "semantic mismatch in {} at byte {}.\nleft: {}\nright: {}",
            filename,
            i,
            &left[start..end],
            &right[start..end]
        );
    }
}

fn assert_idempotent(filename: &str, input: &str) {
    let program = parse_with_filename(filename, input);
    let printer = DslPrinter::new();
    let printed_once = printer.print(&program);
    let reparsed = parse_with_filename(filename, &printed_once);
    let printed_twice = printer.print(&reparsed);
    assert_eq!(printed_twice, printed_once);
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}

#[test]
fn test_empty_program_prints_empty() {
    let program = sruja_language::Program::new();
    let printer = DslPrinter::new();
    let out = printer.print(&program);
    assert_eq!(out, "");
}

#[test]
fn test_print_element_and_relation() {
    let input = r#"
person = kind "Person"
system = kind "System"

User = person "User"
A = system "System A" {
  description "A test system"
  technology "Rust"
}
A -> User "serves"
"#;
    let program = parse(input);
    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("User = person \"User\""));
    assert!(out.contains("A = system \"System A\""));
    assert!(out.contains("description \"A test system\""));
    assert!(out.contains("technology \"Rust\""));
    assert!(out.contains("A -> User \"serves\""));
}

#[test]
fn test_print_requirement_and_adr() {
    let input = r#"
R1 = requirement functional "Users must login"
ADR001 = adr "Use HTTPS" {
  status "Accepted"
  context "Security"
  decision "Use TLS"
  consequences "Encrypted"
}
"#;
    let program = parse(input);
    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("requirement"));
    assert!(out.contains("functional"));
    assert!(out.contains("Users must login"));
    assert!(out.contains("adr"));
    assert!(out.contains("Use HTTPS"));
    assert!(out.contains("status \"Accepted\""));
}

#[test]
fn test_print_import() {
    let input = r#"import { A, B } from "other.sruja"
"#;
    let program = parse(input);
    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("import {"));
    assert!(out.contains("A, B"));
    assert!(out.contains("from \"other.sruja\""));
}

#[test]
fn test_print_scenario() {
    let input = r#"
LoginFlow = scenario "User Login" {
  User -> System "Enter credentials"
  System -> Database "Validate user"
  Database -> System "Return result"
  System -> User "Display dashboard"
}
"#;
    let program = parse(input);
    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("scenario"));
    assert!(out.contains("LoginFlow"));
    assert!(out.contains("User Login"));
    assert!(out.contains("User -> System"));
    assert!(out.contains("Enter credentials"));
}

#[test]
fn test_print_flow() {
    let input = r#"
PaymentFlow = flow "Payment Processing" {
  User -> Checkout "Initiate payment"
  Checkout -> PaymentGateway "Process payment"
  PaymentGateway -> User "Show confirmation"
}
"#;
    let program = parse(input);
    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("flow"));
    assert!(out.contains("PaymentFlow"));
    assert!(out.contains("Payment Processing"));
    assert!(out.contains("User -> Checkout"));
}

#[test]
fn test_print_policy() {
    let input = r#"
SecurityPolicy = policy "Security Rules" {
  category "security"
  enforcement "error"
  description "Must use HTTPS"
  rule deny edge from { kind "container" } to { kind "datastore" }
}
"#;
    let program = parse(input);
    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("policy"));
    assert!(out.contains("SecurityPolicy"));
    assert!(out.contains("Security Rules"));
    assert!(out.contains("category \"security\""));
    assert!(out.contains("enforcement \"error\""));
    assert!(out.contains("description \"Must use HTTPS\""));
    assert!(out.contains("rule deny edge"));
}

#[test]
fn test_print_policy_without_body() {
    let input = r#"SimplePolicy = policy "Simple Policy"
"#;
    let program = parse(input);
    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("policy"));
    assert!(out.contains("SimplePolicy"));
    assert!(out.contains("Simple Policy"));
    assert!(!out.contains("{"));
}

#[test]
fn test_print_view() {
    let view = sruja_language::ViewDef {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "SystemView".to_string(),
        title: "System Architecture".to_string(),
        description: None,
        view_of: None,
        tags: Vec::new(),
        rules: vec![sruja_language::ViewRule {
            include: Some(sruja_language::ViewRuleExpr {
                wildcard: false,
                recursive: true,
                elements: vec!["System".to_string(), "Container".to_string()],
            }),
            exclude: Some(sruja_language::ViewRuleExpr {
                wildcard: false,
                recursive: false,
                elements: vec!["ExternalSystem".to_string()],
            }),
        }],
    };

    let program =
        sruja_language::Program::new().with_items(vec![sruja_language::TopLevelItem::View(view)]);

    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("view SystemView"));
    assert!(out.contains("System Architecture"));
    assert!(out.contains("include System Container"));
    assert!(out.contains("exclude ExternalSystem"));
}

#[test]
fn test_print_feedback_loop() {
    let feedback_loop = sruja_language::FeedbackLoop {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "RL1".to_string(),
        loop_type: sruja_language::FeedbackLoopType::Reinforcing,
        loop_id: Some("R1".to_string()),
        title: "Reinforcing Growth".to_string(),
        description: Some("Viral growth loop".to_string()),
        relationships: vec![
            sruja_language::Relation {
                location: SourceLocation::new("test.sruja".to_string(), 5, 1),
                from: sruja_language::QualifiedIdent::simple("User".to_string()),
                to: sruja_language::QualifiedIdent::simple("Feature".to_string()),
                label: Some("uses".to_string()),
                description: None,
                technology: None,
                tags: Vec::new(),
            },
            sruja_language::Relation {
                location: SourceLocation::new("test.sruja".to_string(), 6, 1),
                from: sruja_language::QualifiedIdent::simple("Feature".to_string()),
                to: sruja_language::QualifiedIdent::simple("User".to_string()),
                label: Some("invites".to_string()),
                description: None,
                technology: None,
                tags: Vec::new(),
            },
        ],
    };

    let program = sruja_language::Program::new().with_items(vec![
        sruja_language::TopLevelItem::FeedbackLoop(feedback_loop),
    ]);

    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("feedback"));
    assert!(out.contains("RL1"));
    assert!(out.contains("Reinforcing Growth"));
    assert!(out.contains("loop_type \"reinforcing\""));
    assert!(out.contains("loop_id \"R1\""));
    assert!(out.contains("description \"Viral growth loop\""));
    assert!(out.contains("User -> Feature"));
}

#[test]
fn test_print_causal_loop() {
    let causal_loop = sruja_language::CausalLoop {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "CL1".to_string(),
        loop_type: sruja_language::FeedbackLoopType::Reinforcing,
        loop_id: None,
        title: "Market Dynamics".to_string(),
        description: None,
        variables: vec![
            sruja_language::CausalLoopVariable {
                id: "Demand".to_string(),
                label: None,
            },
            sruja_language::CausalLoopVariable {
                id: "Supply".to_string(),
                label: None,
            },
        ],
        relationships: vec![
            sruja_language::CausalRelationship {
                from: "Demand".to_string(),
                to: "Supply".to_string(),
                effect: None,
                polarity: sruja_language::CausalPolarity::Positive,
                delay: None,
            },
            sruja_language::CausalRelationship {
                from: "Supply".to_string(),
                to: "Demand".to_string(),
                effect: Some("price increase".to_string()),
                polarity: sruja_language::CausalPolarity::Negative,
                delay: Some("quarters".to_string()),
            },
        ],
    };

    let program = sruja_language::Program::new()
        .with_items(vec![sruja_language::TopLevelItem::CausalLoop(causal_loop)]);

    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("causal_loop"));
    assert!(out.contains("CL1"));
    assert!(out.contains("Market Dynamics"));
    assert!(out.contains("loop_type \"reinforcing\""));
    assert!(out.contains("Demand -> Supply"));
    assert!(out.contains("effect \"price increase\""));
    assert!(out.contains("polarity \"+\""));
    assert!(out.contains("delay \"quarters\""));
}

#[test]
fn test_print_element_with_all_fields() {
    let element = sruja_language::ElementDef {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        assignment: sruja_language::ElementAssignment {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            name: "A".to_string(),
            kind: sruja_language::ElementKind::System,
            sub_kind: None,
            title: Some("System A".to_string()),
            tag_refs: Vec::new(),
            body: Some(sruja_language::ElementDefBody {
                description: Some("Test system".to_string()),
                technology: Some("Rust".to_string()),
                metadata: vec![
                    sruja_language::MetaEntry {
                        key: "owner".to_string(),
                        value: Some("team".to_string()),
                    },
                    sruja_language::MetaEntry {
                        key: "criticality".to_string(),
                        value: Some("high".to_string()),
                    },
                ],
                ..Default::default()
            }),
        },
    };

    let program =
        sruja_language::Program::new().with_items(vec![sruja_language::TopLevelItem::ElementDef(
            Box::new(element),
        )]);

    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("A = system \"System A\""));
    assert!(out.contains("description \"Test system\""));
    assert!(out.contains("technology \"Rust\""));
}

#[test]
fn test_print_kind_def() {
    let kind_def = sruja_language::ElementKindDef {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        kind: sruja_language::ElementKind::Custom("Microservice".to_string()),
        title: Some("A microservice component".to_string()),
        description: None,
        technology: None,
        style: None,
    };

    let program = sruja_language::Program::new()
        .with_items(vec![sruja_language::TopLevelItem::KindDef(kind_def)]);

    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("microservice"));
    assert!(out.contains("kind"));
    assert!(out.contains("A microservice component"));
}

#[test]
fn test_print_tag_def() {
    let tag_def = sruja_language::TagDef {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "criticality".to_string(),
        color: Some("#FF0000".to_string()),
    };

    let program = sruja_language::Program::new()
        .with_items(vec![sruja_language::TopLevelItem::TagDef(tag_def)]);

    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("tag"));
    assert!(out.contains("criticality"));
    assert!(out.contains("#FF0000"));
}

#[test]
fn test_print_overview() {
    let overview = sruja_language::OverviewBlock {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        summary: Some("E-commerce platform".to_string()),
        audience: Some("Developers".to_string()),
        scope: Some("System architecture".to_string()),
        goals: vec!["Scalable".to_string(), "Fast".to_string()],
        non_goals: vec!["Custom UI".to_string()],
        risks: vec!["DDoS".to_string(), "Data loss".to_string()],
    };

    let program = sruja_language::Program::new()
        .with_items(vec![sruja_language::TopLevelItem::Overview(overview)]);

    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("overview"));
    assert!(out.contains("summary \"E-commerce platform\""));
    assert!(out.contains("audience \"Developers\""));
    assert!(out.contains("scope \"System architecture\""));
    assert!(out.contains("goals"));
    assert!(out.contains("non_goals"));
    assert!(out.contains("risks"));
}

#[test]
fn test_print_deployment() {
    let deployment = sruja_language::DeploymentNode {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "Production".to_string(),
        label: Some("Production Environment".to_string()),
        technology: None,
        children: vec![sruja_language::DeploymentNode {
            location: SourceLocation::new("test.sruja".to_string(), 3, 3),
            id: "WebServer".to_string(),
            label: Some("WebServer".to_string()),
            technology: Some("Nginx".to_string()),
            children: vec![sruja_language::DeploymentNode {
                location: SourceLocation::new("test.sruja".to_string(), 5, 5),
                id: "App".to_string(),
                label: Some("App".to_string()),
                technology: Some("Node.js".to_string()),
                children: vec![],
            }],
        }],
    };

    let program = sruja_language::Program::new()
        .with_items(vec![sruja_language::TopLevelItem::Deployment(deployment)]);

    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("deployment"));
    assert!(out.contains("Production"));
    assert!(out.contains("Production Environment"));
    assert!(out.contains("deployment WebServer"));
    assert!(out.contains("deployment App"));
    assert!(out.contains("Nginx"));
    assert!(out.contains("Node.js"));
}

#[test]
fn test_print_constraints() {
    let constraints = sruja_language::ConstraintsBlock {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        entries: vec![
            sruja_language::ConstraintEntry {
                key: "max_components".to_string(),
                value: "10".to_string(),
            },
            sruja_language::ConstraintEntry {
                key: "no_monoliths".to_string(),
                value: String::new(),
            },
        ],
    };

    let program = sruja_language::Program::new()
        .with_items(vec![sruja_language::TopLevelItem::Constraints(constraints)]);

    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("constraints"));
    assert!(out.contains("max_components"));
    assert!(out.contains("10"));
    assert!(out.contains("no_monoliths"));
}

#[test]
fn test_print_conventions() {
    let conventions = sruja_language::ConventionsBlock {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        entries: vec![
            sruja_language::ConventionEntry {
                key: "naming".to_string(),
                value: "lowercase".to_string(),
            },
            sruja_language::ConventionEntry {
                key: "indentation".to_string(),
                value: "2 spaces".to_string(),
            },
        ],
    };

    let program = sruja_language::Program::new()
        .with_items(vec![sruja_language::TopLevelItem::Conventions(conventions)]);

    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("conventions"));
    assert!(out.contains("naming \"lowercase\""));
    assert!(out.contains("indentation \"2 spaces\""));
}

#[test]
fn test_print_extend() {
    let extend = sruja_language::ExtendElement {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        target: sruja_language::QualifiedIdent::simple("WebContainer".to_string()),
        assignments: vec![sruja_language::ElementAssignment {
            location: SourceLocation::new("test.sruja".to_string(), 2, 3),
            name: "".to_string(),
            kind: sruja_language::ElementKind::Custom("".to_string()),
            sub_kind: None,
            title: None,
            tag_refs: Vec::new(),
            body: Some(sruja_language::ElementDefBody {
                description: None,
                technology: None,
                scale: Some(sruja_language::ScaleBlock {
                    location: SourceLocation::new("test.sruja".to_string(), 2, 3),
                    min: None,
                    max: None,
                    metric: Some("large".to_string()),
                }),
                slo: Some(sruja_language::SloBlock {
                    location: SourceLocation::new("test.sruja".to_string(), 3, 3),
                    availability: Some(sruja_language::SloAvailability {
                        target: Some("99.9".to_string()),
                        window: None,
                        current: None,
                    }),
                    latency: None,
                    error_rate: None,
                    throughput: None,
                }),
                ..Default::default()
            }),
        }],
    };

    let program = sruja_language::Program::new()
        .with_items(vec![sruja_language::TopLevelItem::Extend(extend)]);

    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("extend"));
    assert!(out.contains("WebContainer"));
}

#[test]
fn test_print_style() {
    let style = sruja_language::StyleDecl {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        selector: "System".to_string(),
        properties: std::collections::HashMap::from([
            ("color".to_string(), "#4CAF50".to_string()),
            ("shape".to_string(), "rounded".to_string()),
        ]),
    };

    let program =
        sruja_language::Program::new().with_items(vec![sruja_language::TopLevelItem::Style(style)]);

    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("style"));
    assert!(out.contains("System"));
    assert!(out.contains("color \"#4CAF50\""));
    assert!(out.contains("shape \"rounded\""));
}

#[test]
fn test_print_fitness_roundtrip() {
    let input = r#"fitness AccuracyTarget {
  target "success_rate > 99.0%"
  measure "scripts/evaluate_accuracy.sh"
}
"#;
    let program = parse(input);
    let printer = DslPrinter::new();
    let out = printer.print(&program);

    assert!(out.contains("fitness"));
    assert!(out.contains("AccuracyTarget"));
    assert!(out.contains("target \"success_rate > 99.0%\""));
    assert!(out.contains("measure \"scripts/evaluate_accuracy.sh\""));
}

#[test]
fn test_roundtrip_and_idempotency_on_book_golden_files() {
    let dir = workspace_root().join("book/valid-examples");
    let mut files = Vec::new();
    for entry in fs::read_dir(&dir).expect("book/valid-examples should exist") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("sruja") {
            continue;
        }
        files.push(path);
    }
    files.sort();
    assert!(!files.is_empty(), "expected at least one golden file");

    for file in files {
        let content = fs::read_to_string(&file).expect("read golden file");
        let filename = file.to_string_lossy().to_string();
        assert_semantic_roundtrip(&filename, &content);
        assert_idempotent(&filename, &content);
    }
}
