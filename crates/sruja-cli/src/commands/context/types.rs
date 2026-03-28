use serde::{Deserialize, Serialize};
use sruja_scan::{BlastRadiusResult, NodeKind};

#[derive(Debug)]
pub struct TokenBudget {
    pub max_tokens: usize,
    pub used_tokens: usize,
    pub truncated: bool,
}

impl TokenBudget {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens: max_tokens.max(1),
            used_tokens: 0,
            truncated: false,
        }
    }

    pub fn estimate_tokens(s: &str) -> usize {
        s.len().div_ceil(4)
    }

    pub fn push_str(&mut self, out: &mut String, s: &str) -> bool {
        if self.truncated {
            return false;
        }
        let t = Self::estimate_tokens(s);
        if self.used_tokens.saturating_add(t) > self.max_tokens {
            self.truncated = true;
            return false;
        }
        out.push_str(s);
        self.used_tokens = self.used_tokens.saturating_add(t);
        true
    }

    pub fn finish(&mut self, out: &mut String) {
        if self.truncated {
            out.push_str("\n[Context truncated due to token limits]\n");
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchitectureContext {
    pub repo: String,
    pub summary: ContextSummary,
    pub layers: Vec<LayerInfo>,
    pub boundaries: Vec<BoundaryRule>,
    pub forbidden_patterns: Vec<String>,
    pub focus: Option<FocusContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_context: Option<SystemContext>,
    #[serde(skip)]
    pub max_tokens: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemContext {
    pub index_path: String,
    pub total_repos: usize,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub total_conflicts: usize,
    pub cross_repo_elements: Vec<CrossRepoElement>,
    pub cross_repo_edges: Vec<CrossRepoEdge>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossRepoElement {
    pub canonical_id: String,
    pub kind: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    pub repo_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criticality: Option<sruja_language::ast::Criticality>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossRepoEdge {
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub repo_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiRepoArchitectureContext {
    pub repos: Vec<ArchitectureContext>,
    pub combined_summary: ContextSummary,
    pub cross_repo_rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FocusContext {
    pub file: String,
    pub intent: Option<String>,
    pub depth: usize,
    pub matched_nodes: Vec<FocusNode>,
    pub blast_radius: Option<BlastRadiusResult>,
    pub suggested_checks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FocusNode {
    pub id: String,
    pub kind: NodeKind,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criticality: Option<sruja_language::ast::Criticality>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextSummary {
    pub total_modules: usize,
    pub total_services: usize,
    pub total_databases: usize,
    pub total_external_apis: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LayerInfo {
    pub name: String,
    pub modules: usize,
    pub can_depend_on: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoundaryRule {
    pub from: String,
    pub to: String,
    pub allowed: bool,
    pub reason: String,
}
