//! State machine integrity validation rule

use crate::DomainSchema;
use std::collections::HashSet;

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{ElementDefBodyItem, Program, TopLevelItem, StateMachine};

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
    if !sm.transitions.iter().any(|t| t.from == sm.initial_state) && !sm.terminal_states.contains(&sm.initial_state) {
        diagnostics.push(Diagnostic::new(
            sruja_diagnostics::codes::CODE_SM_INITIAL_NOT_FOUND,
            Severity::Error,
            format!("Initial state '{}' has no outgoing transitions and is not a terminal state.", sm.initial_state),
            sm.location.clone(),
        ));
    }

    // E312: Terminal state has outgoing transitions
    for terminal in &sm.terminal_states {
        if sm.transitions.iter().any(|t| &t.from == terminal) {
            diagnostics.push(Diagnostic::new(
                sruja_diagnostics::codes::CODE_SM_TERMINAL_HAS_OUTGOING,
                Severity::Error,
                format!("Terminal state '{}' cannot have outgoing transitions.", terminal),
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
                format!("State '{}' is unreachable from initial state '{}'.", state, sm.initial_state),
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
                format!("State '{}' is a dead end (non-terminal with no outgoing transitions).", state),
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

fn compute_reachable(initial: &str, transitions: &[sruja_language::StateTransition]) -> HashSet<String> {
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
