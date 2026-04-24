use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use sruja_scan::Graph;
use sruja_intent::model::{IntentModel, BoundaryRuleType};
use crate::commands::error::CliError;
use crate::commands::scan_repo_cached;

#[derive(Debug, Serialize, Deserialize)]
pub struct PreflightResult {
    pub constraints: Vec<PreflightConstraint>,
    pub risks: Vec<PreflightRisk>,
    pub contracts: Vec<PreflightContract>,
    pub supervision: PreflightSupervision,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreflightConstraint {
    #[serde(rename = "type")]
    pub constraint_type: String,  // "boundary", "policy", "convention"
    pub rule: String,
    pub applies_to: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreflightRisk {
    #[serde(rename = "type")]
    pub risk_type: String,  // "incident", "gotcha", "constraint"
    pub element: String,
    pub detail: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreflightContract {
    pub element: String,
    pub contract_name: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreflightSupervision {
    pub recent_velocity: String,        // "high", "moderate", "low"
    pub unproposed_changes_in_area: usize,
    pub recommendation: Option<String>,
}

pub async fn preflight(
    repo_path: &Path,
    target_files: Vec<String>,
    _intent: String,
) -> Result<PreflightResult, CliError> {
    let graph = scan_repo_cached(repo_path)?;
    
    // Load intent model (boundaries and policies)
    let intent_path = repo_path.join("architecture.sruja");
    let intent = if intent_path.exists() {
        IntentModel::from_sruja_file(&intent_path).unwrap_or_default()
    } else {
        IntentModel::default()
    };

    let mut constraints = Vec::new();
    let mut risks = Vec::new();
    let mut contracts = Vec::new();

    // Map files to elements
    let mut affected_elements = HashSet::new();
    for file in &target_files {
        let elements = sruja_diff::git_mapper::find_components_for_file(file, &graph);
        for el in elements {
            affected_elements.insert(el);
        }
    }

    // Expand to neighbors for broader context
    let mut expanded_elements = affected_elements.clone();
    for id in &affected_elements {
        let radius = graph.blast_radius(id, 1);
        for node in radius.downstream {
            expanded_elements.insert(node.id);
        }
        for node in radius.upstream {
            expanded_elements.insert(node.id);
        }
    }

    // Collect constraints from boundaries
    for boundary in &intent.boundaries {
        let is_affected = boundary.inside.iter().any(|id| expanded_elements.contains(id));
        if is_affected {
            for rule in &boundary.rules {
                constraints.push(PreflightConstraint {
                    constraint_type: "boundary".to_string(),
                    rule: rule.description.clone(),
                    applies_to: boundary.inside.clone(),
                });
            }
        }
    }

    // Collect risks and contracts from nodes
    for id in &expanded_elements {
        if let Some(node) = graph.nodes.iter().find(|n| &n.id == id) {
            for gotcha in &node.gotchas {
                risks.push(PreflightRisk {
                    risk_type: "gotcha".to_string(),
                    element: id.clone(),
                    detail: gotcha.clone(),
                });
            }
            
            for contract in &node.contracts {
                contracts.push(PreflightContract {
                    element: id.clone(),
                    contract_name: contract.name.clone(),
                    summary: contract.description.clone().unwrap_or_default(),
                });
            }
        }
    }

    // Supervision metrics
    let velocity = sruja_diff::git_mapper::architectural_velocity(
        repo_path,
        "HEAD~20", // Default heuristic
        "HEAD",
        &graph,
    ).ok();

    let (recent_velocity, unproposed_changes) = if let Some(v) = velocity {
        let rel_v = if v.supervision_ratio < 0.5 {
            "high (unsupervised)"
        } else if v.supervision_ratio < 0.8 {
            "moderate"
        } else {
            "low"
        };
        
        let area_unproposed = v.unsupervised_nodes.iter()
            .filter(|id| expanded_elements.contains(*id))
            .count();
            
        (rel_v.to_string(), area_unproposed)
    } else {
        ("unknown".to_string(), 0)
    };

    let recommendation = if unproposed_changes > 0 {
        Some(format!(
            "This area has {} recent unproposed changes. Consider running 'sruja propose' to document architectural intent.",
            unproposed_changes
        ))
    } else {
        None
    };

    Ok(PreflightResult {
        constraints,
        risks,
        contracts,
        supervision: PreflightSupervision {
            recent_velocity,
            unproposed_changes_in_area: unproposed_changes,
            recommendation,
        },
    })
}
