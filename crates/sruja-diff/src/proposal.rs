use serde::{Deserialize, Serialize};
use sruja_diagnostics::SourceLocation;
use sruja_intent::DriftDetector;
use sruja_intent::IntentModel;
use sruja_intent::{Drift, DriftKind, Evidence, Severity};
use sruja_language::ast::{
    ElementAssignment, ElementDef, ElementKind, Program, QualifiedIdent, Relation, TopLevelItem,
};
use sruja_language::DomainSchema;
use sruja_scan::graph::EdgeConfidence;
use sruja_scan::{Edge, EdgeKind, Graph};
use std::path::{Path, PathBuf};

/// A structured architectural change proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,             // Slug-like ID, e.g., "add-payment-retry"
    pub title: String,          // Human-readable title
    pub description: String,    // Why this change is needed
    pub author: Option<String>, // Who proposed it (agent ID or human)
    pub created_at: String,     // ISO 8601 timestamp
    pub status: ProposalStatus,
    pub changes: Vec<ProposalChange>, // The actual modifications
    pub validation: Option<ProposalValidation>, // Result of pre-validation
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Draft,
    Pending, // Awaiting human review
    Approved,
    Rejected,
    Implemented, // Code matches the proposal
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProposalChange {
    AddElement {
        id: String,
        kind: String, // "container", "component", "database", etc.
        label: String,
        technology: Option<String>,
        parent: Option<String>, // Dot-notation parent, e.g., "PaymentSystem"
        description: Option<String>,
    },
    RemoveElement {
        id: String,
        reason: Option<String>,
    },
    ModifyElement {
        id: String,
        field: String, // "technology", "description", "kind"
        old_value: Option<String>,
        new_value: String,
    },
    AddRelationship {
        source: String,
        target: String,
        label: Option<String>,
        kind: Option<String>, // "calls", "stores", "publishes"
    },
    RemoveRelationship {
        source: String,
        target: String,
        reason: Option<String>,
    },
}

/// Result of validating a proposal against existing architecture + policies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProposalValidation {
    pub is_valid: bool,
    pub policy_violations: Vec<String>,
    pub tribal_warnings: Vec<String>, // Gotchas from affected components
    pub blast_radius: Vec<String>,    // IDs of downstream affected components
    pub suggestions: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProposalError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Apply error: {0}")]
    Apply(String),
}

impl Proposal {
    pub fn new(id: String, title: String, description: String) -> Self {
        Self {
            id,
            title,
            description,
            author: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            status: ProposalStatus::Draft,
            changes: Vec::new(),
            validation: None,
        }
    }

    pub fn save(&self, repo_root: &Path) -> Result<PathBuf, ProposalError> {
        let proposals_dir = repo_root.join(".sruja").join("proposals");
        if !proposals_dir.exists() {
            std::fs::create_dir_all(&proposals_dir)?;
        }
        let file_path = proposals_dir.join(format!("{}.json", self.id));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&file_path, content)?;
        Ok(file_path)
    }

    pub fn load_all(repo_root: &Path) -> Result<Vec<Proposal>, ProposalError> {
        let proposals_dir = repo_root.join(".sruja").join("proposals");
        if !proposals_dir.exists() {
            return Ok(Vec::new());
        }

        let mut proposals = Vec::new();
        for entry in std::fs::read_dir(proposals_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                let content = std::fs::read_to_string(&path)?;
                let proposal: Proposal = serde_json::from_str(&content)?;
                proposals.push(proposal);
            }
        }
        Ok(proposals)
    }

    pub fn validate(&mut self, graph: &Graph, _intent: &IntentModel) -> ProposalValidation {
        let mut validation = ProposalValidation {
            is_valid: true,
            ..Default::default()
        };

        // 1. Cross-reference tribal knowledge
        let affected_ids = self.get_affected_ids();
        for node in &graph.nodes {
            if affected_ids.contains(&node.id) {
                for gotcha in &node.gotchas {
                    validation
                        .tribal_warnings
                        .push(format!("⚠️ Gotcha on {}: {}", node.id, gotcha));
                }
                for constraint in &node.operational_constraints {
                    validation
                        .tribal_warnings
                        .push(format!("🔒 Constraint on {}: {}", node.id, constraint));
                }
            }
        }

        for incident in &graph.incidents {
            for affected in &incident.affected {
                if affected_ids.contains(affected) {
                    validation.tribal_warnings.push(format!(
                        "📋 Incident {} affected {}: {}",
                        incident.id,
                        affected,
                        incident.lesson.as_deref().unwrap_or(&incident.title)
                    ));
                }
            }
        }

        let mut tmp_graph = graph.clone();
        apply_relationship_changes(&mut tmp_graph, &self.changes);

        let schema = DomainSchema::architecture();
        let drift = DriftDetector::new().detect(_intent, &tmp_graph, &schema);
        for d in drift
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::PolicyViolation)
        {
            if drift_is_relevant_to_affected_ids(d, &affected_ids) {
                validation
                    .policy_violations
                    .push(format!("{}: {}", d.severity, d.description));
                if let Some(s) = d.suggestion.as_deref() {
                    if !s.is_empty() {
                        validation.suggestions.push(s.to_string());
                    }
                }
            }
        }

        let mut blast: Vec<String> = Vec::new();
        for id in &affected_ids {
            if tmp_graph.nodes.iter().any(|n| n.id == *id) {
                let r = tmp_graph.blast_radius(id, 2);
                for n in r.downstream {
                    blast.push(n.id);
                }
            }
        }
        blast.sort();
        blast.dedup();
        validation.blast_radius = blast;

        validation.policy_violations.sort();
        validation.policy_violations.dedup();
        validation.suggestions.sort();
        validation.suggestions.dedup();
        if !validation.policy_violations.is_empty() {
            validation.is_valid = false;
        }

        self.validation = Some(validation.clone());
        validation
    }

    pub fn get_affected_ids(&self) -> std::collections::HashSet<String> {
        let mut ids = std::collections::HashSet::new();
        for change in &self.changes {
            match change {
                ProposalChange::AddElement { id, .. } => {
                    ids.insert(id.clone());
                }
                ProposalChange::RemoveElement { id, .. } => {
                    ids.insert(id.clone());
                }
                ProposalChange::ModifyElement { id, .. } => {
                    ids.insert(id.clone());
                }
                ProposalChange::AddRelationship { source, target, .. } => {
                    ids.insert(source.clone());
                    ids.insert(target.clone());
                }
                ProposalChange::RemoveRelationship { source, target, .. } => {
                    ids.insert(source.clone());
                    ids.insert(target.clone());
                }
            }
        }
        ids
    }

    pub fn apply(&self, baseline: &Program) -> Result<Program, ProposalError> {
        let mut program = baseline.clone();

        for change in &self.changes {
            match change {
                ProposalChange::AddElement {
                    id,
                    kind,
                    label,
                    technology,
                    parent: _,
                    description,
                } => {
                    // Check if already exists
                    if self.find_element(&program, id).is_some() {
                        return Err(ProposalError::Apply(format!(
                            "Element '{}' already exists",
                            id
                        )));
                    }

                    let mut assignment =
                        ElementAssignment::new(id.clone(), parse_element_kind(kind));
                    assignment.title = Some(label.clone());

                    let body = sruja_language::ast::ElementDefBody {
                        technology: technology.clone(),
                        description: description.clone(),
                        ..Default::default()
                    };

                    // Note: parent is handled via naming convention or nesting.
                    // In flat DSL, it's usually part of the ID or handled via dot notation.

                    assignment.body = Some(body);

                    program.push_item(TopLevelItem::ElementDef(Box::new(ElementDef {
                        location: SourceLocation::new(String::new(), 0, 0),
                        assignment,
                    })));
                }
                ProposalChange::RemoveElement { id, .. } => {
                    let pos = program.items.iter().position(|item| {
                        if let TopLevelItem::ElementDef(def) = item {
                            def.assignment.name == *id
                        } else {
                            false
                        }
                    });
                    if let Some(i) = pos {
                        program.items.remove(i);
                    } else {
                        return Err(ProposalError::Apply(format!("Element '{}' not found", id)));
                    }
                }
                ProposalChange::ModifyElement {
                    id,
                    field,
                    new_value,
                    ..
                } => {
                    let element = self.find_element_mut(&mut program, id).ok_or_else(|| {
                        ProposalError::Apply(format!("Element '{}' not found", id))
                    })?;

                    let body = element.assignment.body.get_or_insert_with(Default::default);
                    match field.as_str() {
                        "technology" => body.technology = Some(new_value.clone()),
                        "description" => body.description = Some(new_value.clone()),
                        "kind" => element.assignment.kind = parse_element_kind(new_value),
                        _ => {
                            return Err(ProposalError::Apply(format!("Unknown field '{}'", field)))
                        }
                    }
                }
                ProposalChange::AddRelationship {
                    source,
                    target,
                    label,
                    kind,
                } => {
                    let rel = Relation {
                        location: SourceLocation::new(String::new(), 0, 0),
                        from: parse_qualified_ident(source),
                        to: parse_qualified_ident(target),
                        label: label.clone(),
                        description: None,
                        technology: kind.clone(), // Mapping kind to technology for now or use specific field
                        tags: Vec::new(),
                    };
                    program.push_item(TopLevelItem::Relation(rel));
                }
                ProposalChange::RemoveRelationship { source, target, .. } => {
                    let pos = program.items.iter().position(|item| {
                        if let TopLevelItem::Relation(rel) = item {
                            rel.from.as_string() == *source && rel.to.as_string() == *target
                        } else {
                            false
                        }
                    });
                    if let Some(i) = pos {
                        program.items.remove(i);
                    } else {
                        return Err(ProposalError::Apply(format!(
                            "Relationship '{} -> {}' not found",
                            source, target
                        )));
                    }
                }
            }
        }

        Ok(program)
    }

    fn find_element<'a>(&self, program: &'a Program, id: &str) -> Option<&'a ElementDef> {
        program.items.iter().find_map(|item| {
            if let TopLevelItem::ElementDef(def) = item {
                if def.assignment.name == id {
                    Some(def.as_ref())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn find_element_mut<'a>(
        &self,
        program: &'a mut Program,
        id: &str,
    ) -> Option<&'a mut ElementDef> {
        program.items.iter_mut().find_map(|item| {
            if let TopLevelItem::ElementDef(def) = item {
                if def.assignment.name == id {
                    Some(def.as_mut())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

fn apply_relationship_changes(graph: &mut Graph, changes: &[ProposalChange]) {
    for change in changes {
        match change {
            ProposalChange::AddRelationship {
                source,
                target,
                kind,
                ..
            } => {
                let kind = kind
                    .as_deref()
                    .unwrap_or("depends_on")
                    .parse::<EdgeKind>()
                    .unwrap_or_else(|_| EdgeKind::Custom(kind.clone().unwrap_or_default()));
                graph.edges.push(Edge {
                    source: source.clone(),
                    target: target.clone(),
                    kind,
                    evidence: Vec::new(),
                    confidence: EdgeConfidence::default(),
                });
            }
            ProposalChange::RemoveRelationship { source, target, .. } => {
                graph
                    .edges
                    .retain(|e| !(e.source == *source && e.target == *target));
            }
            _ => {}
        }
    }
}

fn drift_is_relevant_to_affected_ids(
    drift: &Drift,
    affected_ids: &std::collections::HashSet<String>,
) -> bool {
    for id in affected_ids {
        if drift.description.contains(id) {
            return true;
        }
        for e in &drift.evidence {
            if e.location.as_deref().is_some_and(|l| l.contains(id)) {
                return true;
            }
            if e.detail.contains(id) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_intent::model::{
        DeclaredPolicy, IntentSourceInfo, IntentSourceType, PolicyRule, PolicyRuleContent,
        PolicySelector, SourceReference,
    };
    use std::path::PathBuf;

    fn source_ref() -> SourceReference {
        SourceReference {
            file: "test".to_string(),
            line: None,
            element: None,
        }
    }

    #[test]
    fn proposal_validate_flags_policy_violation_for_added_edge() {
        let mut intent = IntentModel::new(IntentSourceInfo {
            source_type: IntentSourceType::Manual,
            path: PathBuf::from("."),
            name: "test".to_string(),
        });
        intent.policies.push(DeclaredPolicy {
            name: "no_a_to_b".to_string(),
            description: "deny A -> B".to_string(),
            category: "".to_string(),
            enforcement: "error".to_string(),
            scope: Vec::new(),
            rules: vec![PolicyRule {
                description: "deny".to_string(),
                constraint: "deny".to_string(),
                content: Some(PolicyRuleContent::DenyEdge {
                    from: PolicySelector {
                        id: Some("A".to_string()),
                        ..Default::default()
                    },
                    to: PolicySelector {
                        id: Some("B".to_string()),
                        ..Default::default()
                    },
                    except: Vec::new(),
                    message: Some("A must not depend on B".to_string()),
                    suggestions: vec!["Remove the dependency".to_string()],
                }),
            }],
            source_ref: source_ref(),
        });

        let mut graph = Graph::new();
        graph.nodes.push(sruja_scan::Node {
            id: "A".to_string(),
            label: "A".to_string(),
            ..Default::default()
        });
        graph.nodes.push(sruja_scan::Node {
            id: "B".to_string(),
            label: "B".to_string(),
            ..Default::default()
        });

        let mut proposal = Proposal::new("p1".to_string(), "t".to_string(), "d".to_string());
        proposal.changes.push(ProposalChange::AddRelationship {
            source: "A".to_string(),
            target: "B".to_string(),
            label: None,
            kind: Some("depends_on".to_string()),
        });

        let v = proposal.validate(&graph, &intent);
        assert!(!v.is_valid);
        assert!(!v.policy_violations.is_empty());
    }
}

fn parse_element_kind(kind: &str) -> ElementKind {
    match kind.to_lowercase().as_str() {
        "person" => ElementKind::Person,
        "system" => ElementKind::System,
        "container" => ElementKind::Container,
        "component" => ElementKind::Component,
        "database" => ElementKind::Database,
        "queue" => ElementKind::Queue,
        _ => ElementKind::Custom(kind.to_string()),
    }
}

fn parse_qualified_ident(s: &str) -> QualifiedIdent {
    QualifiedIdent::qualified(s.split('.').map(|p| p.to_string()).collect())
}

pub fn detect_unproposed_changes(
    previous_graph: &Graph,
    current_graph: &Graph,
    approved_proposals: &[Proposal],
) -> Vec<Drift> {
    let mut drifts = Vec::new();

    // 1. Find nodes that are in current but not in previous
    let previous_ids: std::collections::HashSet<_> =
        previous_graph.nodes.iter().map(|n| &n.id).collect();

    for node in &current_graph.nodes {
        if !previous_ids.contains(&node.id) {
            // New node. Check if it was proposed.
            let is_proposed = approved_proposals.iter().any(|p| {
                p.status == ProposalStatus::Approved
                    && p.changes.iter().any(|c| match c {
                        ProposalChange::AddElement { id, .. } => id == &node.id,
                        _ => false,
                    })
            });

            if !is_proposed {
                drifts.push(Drift {
                    kind: DriftKind::UnproposedChange,
                    severity: Severity::High,
                    description: format!(
                        "New component '{}' appeared in code without an approved proposal",
                        node.id
                    ),
                    evidence: vec![Evidence {
                        source: "scan".to_string(),
                        location: node.path.clone(),
                        detail: format!("Kind: {}", node.kind),
                    }],
                    intent_ref: None,
                    suggestion: Some("Run 'sruja propose create' to document this change or remove it from code.".to_string()),
                });
            }
        }
    }

    // 2. Find new relationships
    let previous_edges: std::collections::HashSet<_> = previous_graph
        .edges
        .iter()
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect();

    for edge in &current_graph.edges {
        if !previous_edges.contains(&(edge.source.clone(), edge.target.clone())) {
            let is_proposed = approved_proposals.iter().any(|p| {
                p.status == ProposalStatus::Approved
                    && p.changes.iter().any(|c| match c {
                        ProposalChange::AddRelationship { source, target, .. } => {
                            source == &edge.source && target == &edge.target
                        }
                        _ => false,
                    })
            });

            if !is_proposed {
                drifts.push(Drift {
                    kind: DriftKind::UnproposedChange,
                    severity: Severity::Medium,
                    description: format!(
                        "New relationship '{} -> {}' appeared in code without an approved proposal",
                        edge.source, edge.target
                    ),
                    evidence: vec![Evidence {
                        source: "scan".to_string(),
                        location: None,
                        detail: "Detected via structural scan".to_string(),
                    }],
                    intent_ref: None,
                    suggestion: Some("Document this relationship in a proposal.".to_string()),
                });
            }
        }
    }

    drifts
}
