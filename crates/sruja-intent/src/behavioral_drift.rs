//! Behavioral drift detection for critique engine

use sruja_scan::{Graph, ResolvedStateMachine, ResolvedContract, Node};
use sruja_language::Program;
use crate::critique::{CritiqueFinding, CritiqueCategory, CritiqueSeverity, CritiqueEvidence};

/// Check if changed files could violate behavioral contracts.
pub fn check_behavioral_drift(
    graph: &Graph,
    _program: &Program,
    changed_files: &[String],
    affected_elements: &[String],
) -> Vec<CritiqueFinding> {
    let mut findings = Vec::new();

    for element_id in affected_elements {
        let node = match graph.nodes.iter().find(|n| &n.id == element_id) {
            Some(n) => n,
            None => continue,
        };

        // Check state machine contracts
        for sm in &node.state_machines {
            findings.extend(check_state_machine_impact(sm, node, changed_files));
        }

        // Check API contracts
        for contract in &node.contracts {
            findings.extend(check_contract_impact(contract, node, changed_files));
        }
    }

    findings
}

fn check_state_machine_impact(
    sm: &ResolvedStateMachine,
    node: &Node,
    changed_files: &[String],
) -> Vec<CritiqueFinding> {
    let mut findings = Vec::new();
    let id = &node.id;

    // Heuristic: if a file specifically linked to this node changed
    let is_directly_affected = if let Some(path) = &node.path {
        changed_files.iter().any(|f| f.contains(path) || path.contains(f))
    } else {
        false
    };

    if is_directly_affected {
        findings.push(CritiqueFinding {
            category: CritiqueCategory::BehavioralContractDrift,
            severity: CritiqueSeverity::High,
            title: format!("State Machine Impact: {}", sm.name),
            detail: format!("File change detected in '{}' which implements state machine '{}'. Ensure all states and transitions are preserved.", id, sm.name),
            evidence: vec![CritiqueEvidence {
                source: "sruja".to_string(),
                location: node.path.clone(),
                detail: format!("State machine: {}", sm.name),
            }],
            suggestion: Some("Review state transition logic against the architectural specification.".to_string()),
            confidence: 0.8,
        });
    }

    findings
}

fn check_contract_impact(
    contract: &ResolvedContract,
    node: &Node,
    changed_files: &[String],
) -> Vec<CritiqueFinding> {
    let mut findings = Vec::new();
    let id = &node.id;

    let is_directly_affected = if let Some(path) = &node.path {
        changed_files.iter().any(|f| f.contains(path) || path.contains(f))
    } else {
        false
    };

    if is_directly_affected {
        findings.push(CritiqueFinding {
            category: CritiqueCategory::BehavioralContractDrift,
            severity: CritiqueSeverity::High,
            title: format!("API Contract Impact: {}", contract.name),
            detail: format!("File change detected in '{}' which implements API contract '{}'. Verify input/output shapes still match.", id, contract.name),
            evidence: vec![CritiqueEvidence {
                source: "sruja".to_string(),
                location: node.path.clone(),
                detail: format!("Contract: {}", contract.name),
            }],
            suggestion: Some("Run API compatibility tests or manually verify the implementation against the contract.".to_string()),
            confidence: 0.8,
        });
    }

    findings
}
