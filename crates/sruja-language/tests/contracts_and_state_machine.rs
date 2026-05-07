use sruja_language::Parser;

fn parse_single_element_body(input: &str) -> sruja_language::ast::ElementDefBody {
    let parser = Parser::new("test.sruja".to_string());
    let program = parser.parse(input).expect("expected parse to succeed");
    let item = program
        .items
        .into_iter()
        .find_map(|i| match i {
            sruja_language::ast::TopLevelItem::ElementDef(def) => Some(*def),
            _ => None,
        })
        .expect("expected a top-level element definition");

    item.assignment
        .body
        .expect("expected element body to be present")
}

#[test]
fn parses_contract_block_in_element_body() {
    let body = parse_single_element_body(
        r#"
Api = component "API" {
  contract "GetUser" {
    description "Fetch user data"
    input {
      user_id "uuid"
    }
    output {
      user_name "string"
    }
    error {
      "NOT_FOUND" "No such user"
    }
    constraint "idempotent"
  }
}
"#,
    );

    assert_eq!(body.contracts.len(), 1);
    let c = &body.contracts[0];
    assert_eq!(c.name, "GetUser");
    assert_eq!(c.description.as_deref(), Some("Fetch user data"));
    assert_eq!(c.inputs.len(), 1);
    assert_eq!(c.inputs[0].name, "user_id");
    assert_eq!(c.inputs[0].spec, "uuid");
    assert_eq!(c.outputs.len(), 1);
    assert_eq!(c.outputs[0].name, "user_name");
    assert_eq!(c.outputs[0].spec, "string");
    assert_eq!(c.errors.len(), 1);
    assert_eq!(c.errors[0].code, "NOT_FOUND");
    assert_eq!(c.errors[0].description, "No such user");
    assert_eq!(c.constraints, vec!["idempotent".to_string()]);
}

#[test]
fn parses_state_machine_with_transition_metadata() {
    let body = parse_single_element_body(
        r#"
Svc = component "Service" {
  state_machine "Lifecycle" {
    description "Simple lifecycle"
    initial "Created"
    terminal ["Done", "Cancelled"]

    "Created" -> "Done" on "finish" {
      guard "is_valid"
      action "persist"
      description "Finish and persist"
    }
  }
}
"#,
    );

    assert_eq!(body.state_machines.len(), 1);
    let sm = &body.state_machines[0];
    assert_eq!(sm.name, "Lifecycle");
    assert_eq!(sm.description.as_deref(), Some("Simple lifecycle"));
    assert_eq!(sm.initial_state, "Created");
    assert_eq!(
        sm.terminal_states,
        vec!["Done".to_string(), "Cancelled".to_string()]
    );
    assert_eq!(sm.transitions.len(), 1);
    let t = &sm.transitions[0];
    assert_eq!(t.from, "Created");
    assert_eq!(t.to, "Done");
    assert_eq!(t.event, "finish");
    assert_eq!(t.guard.as_deref(), Some("is_valid"));
    assert_eq!(t.action.as_deref(), Some("persist"));
    assert_eq!(t.description.as_deref(), Some("Finish and persist"));
}
