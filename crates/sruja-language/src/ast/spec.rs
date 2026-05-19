//! Scenario steps, state machines, and contracts.

use sruja_diagnostics::SourceLocation;

use super::relation::QualifiedIdent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioStep {
    pub from: Option<QualifiedIdent>,
    pub to: Option<QualifiedIdent>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub order: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMachine {
    pub location: SourceLocation,
    pub name: String,
    pub initial_state: String,
    pub terminal_states: Vec<String>,
    pub transitions: Vec<StateTransition>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransition {
    pub location: SourceLocation,
    pub from: String,
    pub to: String,
    pub event: String,
    pub guard: Option<String>,
    pub action: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contract {
    pub location: SourceLocation,
    pub name: String,
    pub description: Option<String>,
    pub inputs: Vec<ContractField>,
    pub outputs: Vec<ContractField>,
    pub errors: Vec<ContractError>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractField {
    pub name: String,
    pub spec: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractError {
    pub code: String,
    pub description: String,
}
