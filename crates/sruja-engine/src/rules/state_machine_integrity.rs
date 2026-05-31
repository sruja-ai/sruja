//! State machine integrity validation rule

use crate::DomainSchema;
use std::collections::HashSet;

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{ElementDefBodyItem, Program, StateMachine, TopLevelItem};

use crate::validator::Rule;

pub struct StateMachineIntegrityRule;

impl Rule for StateMachineIntegrityRule {
    fn name(&self) -> &str {
        "State Machine Integrity"
    }

    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for item in &program.items {
            if let TopLevelItem::ElementDef(elem) = item {
                if let Some(body) = &elem.assignment.body {
                    validate_body(body, &mut diagnostics);
                }
            }
        }

        diagnostics
    }
}

fn validate_body(body: &sruja_language::ElementDefBody, diagnostics: &mut Vec<Diagnostic>) {
    for sm in &body.state_machines {
        validate_state_machine(sm, diagnostics);
    }

    for item in &body.items {
        if let ElementDefBodyItem::ElementDef(elem) = item {
            if let Some(nested_body) = &elem.assignment.body {
                validate_body(nested_body, diagnostics);
            }
        }
    }
}

fn validate_state_machine(sm: &StateMachine, diagnostics: &mut Vec<Diagnostic>) {
    let mut all_states = HashSet::new();
    all_states.insert(sm.initial_state.clone());
    for s in &sm.terminal_states {
        all_states.insert(s.clone());
    }
    for t in &sm.transitions {
        all_states.insert(t.from.clone());
        all_states.insert(t.to.clone());
    }

    // E311: Initial state not found in any transition
    if !sm.transitions.iter().any(|t| t.from == sm.initial_state)
        && !sm.terminal_states.contains(&sm.initial_state)
    {
        diagnostics.push(Diagnostic::new(
            sruja_diagnostics::codes::CODE_SM_INITIAL_NOT_FOUND,
            Severity::Error,
            format!(
                "Initial state '{}' has no outgoing transitions and is not a terminal state.",
                sm.initial_state
            ),
            sm.location.clone(),
        ));
    }

    // E312: Terminal state has outgoing transitions
    for terminal in &sm.terminal_states {
        if sm.transitions.iter().any(|t| &t.from == terminal) {
            diagnostics.push(Diagnostic::new(
                sruja_diagnostics::codes::CODE_SM_TERMINAL_HAS_OUTGOING,
                Severity::Error,
                format!(
                    "Terminal state '{}' cannot have outgoing transitions.",
                    terminal
                ),
                sm.location.clone(),
            ));
        }
    }

    // W311: Unreachable states
    let reachable = compute_reachable(&sm.initial_state, &sm.transitions);
    for state in &all_states {
        if !reachable.contains(state) {
            diagnostics.push(Diagnostic::new(
                sruja_diagnostics::codes::CODE_SM_UNREACHABLE_STATE,
                Severity::Warning,
                format!(
                    "State '{}' is unreachable from initial state '{}'.",
                    state, sm.initial_state
                ),
                sm.location.clone(),
            ));
        }
    }

    // W312: Dead states (non-terminal with no outgoing)
    for state in &all_states {
        if !sm.terminal_states.contains(state) && !sm.transitions.iter().any(|t| &t.from == state) {
            diagnostics.push(Diagnostic::new(
                sruja_diagnostics::codes::CODE_SM_DEAD_STATE,
                Severity::Warning,
                format!(
                    "State '{}' is a dead end (non-terminal with no outgoing transitions).",
                    state
                ),
                sm.location.clone(),
            ));
        }
    }

    // W313: Duplicate transitions (same from + event)
    let mut seen_transitions = HashSet::new();
    for t in &sm.transitions {
        let key = (t.from.clone(), t.event.clone());
        if seen_transitions.contains(&key) {
            diagnostics.push(Diagnostic::new(
                sruja_diagnostics::codes::CODE_SM_DUPLICATE_TRANSITION,
                Severity::Warning,
                format!("Duplicate transition from '{}' on event '{}'. This makes the state machine non-deterministic.", t.from, t.event),
                t.location.clone(),
            ));
        } else {
            seen_transitions.insert(key);
        }
    }

    // W314: No terminal states
    if sm.terminal_states.is_empty() {
        diagnostics.push(Diagnostic::new(
            sruja_diagnostics::codes::CODE_SM_NO_TERMINAL,
            Severity::Warning,
            format!("State machine '{}' has no terminal states.", sm.name),
            sm.location.clone(),
        ));
    }
}

fn compute_reachable(
    initial: &str,
    transitions: &[sruja_language::StateTransition],
) -> HashSet<String> {
    let mut reachable = HashSet::new();
    let mut stack = vec![initial.to_string()];
    reachable.insert(initial.to_string());

    while let Some(current) = stack.pop() {
        for t in transitions {
            if t.from == current && !reachable.contains(&t.to) {
                reachable.insert(t.to.clone());
                stack.push(t.to.clone());
            }
        }
    }

    reachable
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DomainSchema;
    use sruja_diagnostics::codes::{
        CODE_SM_DEAD_STATE, CODE_SM_DUPLICATE_TRANSITION, CODE_SM_INITIAL_NOT_FOUND,
        CODE_SM_NO_TERMINAL, CODE_SM_TERMINAL_HAS_OUTGOING, CODE_SM_UNREACHABLE_STATE,
    };
    use sruja_language::Parser;

    fn parse_program(input: &str) -> Program {
        Parser::new("test.sruja".to_string())
            .parse(input)
            .expect("parse")
    }

    fn codes(diags: &[Diagnostic]) -> Vec<&str> {
        diags.iter().map(|d| d.code.as_str()).collect()
    }

    #[test]
    fn rule_name_is_state_machine_integrity() {
        assert_eq!(StateMachineIntegrityRule.name(), "State Machine Integrity");
    }

    #[test]
    fn valid_state_machine_has_no_diagnostics() {
        let program = parse_program(
            r#"
Svc = component "Service" {
  state_machine "Lifecycle" {
    initial "Created"
    terminal ["Done"]

    "Created" -> "Done" on "finish"
  }
}
"#,
        );
        let rule = StateMachineIntegrityRule;
        let diags = rule.validate(&program, &DomainSchema::architecture());
        assert!(diags.is_empty(), "valid SM should pass: {diags:?}");
    }

    #[test]
    fn initial_state_without_outgoing_and_not_terminal_is_error() {
        let program = parse_program(
            r#"
Svc = component "Service" {
  state_machine "Broken" {
    initial "Created"
    terminal ["Done"]
  }
}
"#,
        );
        let rule = StateMachineIntegrityRule;
        let diags = rule.validate(&program, &DomainSchema::architecture());
        assert!(
            codes(&diags).contains(&CODE_SM_INITIAL_NOT_FOUND),
            "expected initial-not-found: {diags:?}"
        );
    }

    #[test]
    fn terminal_state_with_outgoing_transition_is_error() {
        let program = parse_program(
            r#"
Svc = component "Service" {
  state_machine "Broken" {
    initial "Created"
    terminal ["Done"]

    "Created" -> "Done" on "finish"
    "Done" -> "Created" on "reopen"
  }
}
"#,
        );
        let rule = StateMachineIntegrityRule;
        let diags = rule.validate(&program, &DomainSchema::architecture());
        assert!(
            codes(&diags).contains(&CODE_SM_TERMINAL_HAS_OUTGOING),
            "expected terminal-has-outgoing: {diags:?}"
        );
    }

    #[test]
    fn unreachable_state_emits_warning() {
        let program = parse_program(
            r#"
Svc = component "Service" {
  state_machine "Broken" {
    initial "Created"
    terminal ["Done"]

    "Created" -> "Done" on "finish"
    "Orphan" -> "Done" on "skip"
  }
}
"#,
        );
        let rule = StateMachineIntegrityRule;
        let diags = rule.validate(&program, &DomainSchema::architecture());
        assert!(
            codes(&diags).contains(&CODE_SM_UNREACHABLE_STATE),
            "expected unreachable warning: {diags:?}"
        );
    }

    #[test]
    fn dead_non_terminal_state_emits_warning() {
        let program = parse_program(
            r#"
Svc = component "Service" {
  state_machine "Broken" {
    initial "Created"

    "Created" -> "Processing" on "start"
  }
}
"#,
        );
        let rule = StateMachineIntegrityRule;
        let diags = rule.validate(&program, &DomainSchema::architecture());
        assert!(
            codes(&diags).contains(&CODE_SM_DEAD_STATE),
            "expected dead-state warning: {diags:?}"
        );
    }

    #[test]
    fn duplicate_transition_on_same_event_emits_warning() {
        let program = parse_program(
            r#"
Svc = component "Service" {
  state_machine "Broken" {
    initial "Created"
    terminal ["Done"]

    "Created" -> "Done" on "finish"
    "Created" -> "Done" on "finish"
  }
}
"#,
        );
        let rule = StateMachineIntegrityRule;
        let diags = rule.validate(&program, &DomainSchema::architecture());
        assert!(
            codes(&diags).contains(&CODE_SM_DUPLICATE_TRANSITION),
            "expected duplicate transition warning: {diags:?}"
        );
    }

    #[test]
    fn missing_terminal_states_emits_warning() {
        let program = parse_program(
            r#"
Svc = component "Service" {
  state_machine "Broken" {
    initial "Created"

    "Created" -> "Processing" on "start"
  }
}
"#,
        );
        let rule = StateMachineIntegrityRule;
        let diags = rule.validate(&program, &DomainSchema::architecture());
        assert!(
            codes(&diags).contains(&CODE_SM_NO_TERMINAL),
            "expected no-terminal warning: {diags:?}"
        );
    }
}
