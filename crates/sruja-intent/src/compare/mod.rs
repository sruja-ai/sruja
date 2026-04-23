//! Drift Detection
//!
//! Compares declared architectural intent against actual implementation
//! to detect boundary drift, intent violations, and undocumented changes.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
 
pub mod mapper;

use sruja_language::DomainSchema;
use sruja_scan::{Graph, Node};

use crate::model::{
    DeclaredBoundary, DeclaredPolicy, IntentModel, PolicyRuleContent, PolicySelector,
};

pub struct DriftDetector {
    #[allow(dead_code)]
    config: DriftConfig,
}

#[derive(Debug, Clone)]
pub struct DriftConfig {
    pub use_semantic_similarity: bool,
    pub min_confidence: f32,
    pub severity_threshold: Severity,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            use_semantic_similarity: false,
            min_confidence: 0.5,
            severity_threshold: Severity::Info,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub intent_source: String,
    pub reality_source: String,
    pub drifts: Vec<Drift>,
    pub drift_score: u8,
    pub health: DriftHealth,
    pub summary: DriftSummary,
}

impl DriftReport {
    /// Recompute summary counts from drifts and update drift_score and health.
    /// Call after appending additional drifts (e.g. policy violations).
    pub fn recompute_summary_and_score(&mut self) {
        self.summary.undocumented_components = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::UndocumentedComponent)
            .count();
        self.summary.missing_components = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::MissingComponent)
            .count();
        self.summary.undocumented_relationships = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::UndocumentedRelationship)
            .count();
        self.summary.boundary_violations = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::BoundaryViolation)
            .count();
        self.summary.policy_violations = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::PolicyViolation)
            .count();
        self.summary.schema_violations = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::SchemaViolation)
            .count();
        self.summary.taxonomy_mismatches = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::TaxonomyMismatch)
            .count();
        self.drift_score = DriftDetector::compute_drift_score(&self.summary, &self.drifts);
        self.health = DriftDetector::classify_health(self.drift_score);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftSummary {
    pub total_components_declared: usize,
    pub total_components_discovered: usize,
    pub undocumented_components: usize,
    pub missing_components: usize,
    pub undocumented_relationships: usize,
    pub boundary_violations: usize,
    pub policy_violations: usize,
    pub schema_violations: usize,
    pub taxonomy_mismatches: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drift {
    pub kind: DriftKind,
    pub severity: Severity,
    pub description: String,
    pub evidence: Vec<Evidence>,
    pub intent_ref: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftKind {
    UndocumentedComponent,
    MissingComponent,
    BoundaryViolation,
    UndocumentedRelationship,
    MissingRelationship,
    TechnologyMismatch,
    PolicyViolation,
    ConstraintViolation,
    SchemaViolation,
    TaxonomyMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftHealth {
    Healthy,
    MinorDrift,
    SignificantDrift,
    CriticalDrift,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub source: String,
    pub location: Option<String>,
    pub detail: String,
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl DriftDetector {
    pub fn new() -> Self {
        Self {
            config: DriftConfig::default(),
        }
    }

    pub fn with_config(config: DriftConfig) -> Self {
        Self { config }
    }

    pub fn detect(&self, intent: &IntentModel, reality: &Graph, schema: &DomainSchema) -> DriftReport {
        let mut drifts = Vec::new();
        let mapper = mapper::EvidenceMapper::new(schema);

        let declared_ids: HashSet<&str> = intent.components.iter().map(|c| c.id.as_str()).collect();
        let discovered_ids: HashSet<&str> = reality.nodes.iter().map(|n| n.id.as_str()).collect();

        for node in &reality.nodes {
            let discovered = &node.id;
            let node_kind = mapper.map_node_kind(node);

            if !schema.is_node_kind_allowed(&node_kind) {
                drifts.push(Drift {
                    kind: DriftKind::TaxonomyMismatch,
                    severity: Severity::Medium,
                    description: format!(
                        "Discovered component '{}' has kind '{}' which is not allowed in current schema '{}'",
                        discovered, node_kind, schema.name
                    ),
                    evidence: vec![Evidence {
                        source: "scan".to_string(),
                        location: node.path.clone(),
                        detail: format!("Scanner kind: {}", node.kind),
                    }],
                    intent_ref: None,
                    suggestion: Some(format!("Add '{}' to schema node_kinds or update scanner mapping", node_kind)),
                });
            }

            if !declared_ids.contains(discovered.as_str()) {
                drifts.push(Drift {
                    kind: DriftKind::UndocumentedComponent,
                    severity: Severity::Medium,
                    description: format!(
                        "Component '{}' exists in code but not in architecture docs",
                        discovered
                    ),
                    evidence: vec![Evidence {
                        source: "scan".to_string(),
                        location: node.path.clone(),
                        detail: format!("Discovered node of kind {}", node_kind),
                    }],
                    intent_ref: None,
                    suggestion: Some(format!(
                        "Add '{}' to architecture documentation or mark as temporary",
                        discovered
                    )),
                });
            }
        }

        for declared in &declared_ids {
            if !discovered_ids.contains(declared) {
                drifts.push(Drift {
                    kind: DriftKind::MissingComponent,
                    severity: Severity::High,
                    description: format!(
                        "Component '{}' declared in architecture but not found in code",
                        declared
                    ),
                    evidence: vec![],
                    intent_ref: intent
                        .find_component(declared)
                        .map(|c| c.source_ref.file.clone()),
                    suggestion: Some(format!(
                        "Either implement '{}' or remove it from architecture docs",
                        declared
                    )),
                });
            }
        }

        let declared_rels: HashSet<(&str, &str)> = intent
            .relationships
            .iter()
            .map(|r| (r.source.as_str(), r.target.as_str()))
            .collect();

        let discovered_rels: HashSet<(&str, &str)> = reality
            .edges
            .iter()
            .map(|e| (e.source.as_str(), e.target.as_str()))
            .collect();

        for (src, tgt) in &discovered_rels {
            if !declared_rels.contains(&(*src, *tgt)) {
                drifts.push(Drift {
                    kind: DriftKind::UndocumentedRelationship,
                    severity: Severity::Low,
                    description: format!(
                        "Relationship '{}' -> '{}' exists in code but not documented",
                        src, tgt
                    ),
                    evidence: vec![Evidence {
                        source: "scan".to_string(),
                        location: None,
                        detail: format!("Edge from {} to {}", src, tgt),
                    }],
                    intent_ref: None,
                    suggestion: Some(format!(
                        "Document the relationship {} -> {} in architecture",
                        src, tgt
                    )),
                });
            }
        }

        for (src, tgt) in &declared_rels {
            if !discovered_rels.contains(&(*src, *tgt)) {
                drifts.push(Drift {
                    kind: DriftKind::MissingRelationship,
                    severity: Severity::Low,
                    description: format!(
                        "Declared relationship '{}' -> '{}' not found in code",
                        src, tgt
                    ),
                    evidence: vec![],
                    intent_ref: None,
                    suggestion: Some(format!(
                        "Verify if {} -> {} is still needed, or remove from docs",
                        src, tgt
                    )),
                });
            }
        }

        for boundary in &intent.boundaries {
            let boundary_drifts = self.detect_boundary_violations(boundary, reality);
            drifts.extend(boundary_drifts);
        }

        for policy in &intent.policies {
            let policy_drifts = self.detect_policy_violations(policy, reality);
            drifts.extend(policy_drifts);
        }

        drifts.sort_by(|a, b| {
            let order = |s: Severity| match s {
                Severity::Critical => 0,
                Severity::High => 1,
                Severity::Medium => 2,
                Severity::Low => 3,
                Severity::Info => 4,
            };
            order(a.severity).cmp(&order(b.severity))
        });

        let summary = DriftSummary {
            total_components_declared: declared_ids.len(),
            total_components_discovered: discovered_ids.len(),
            undocumented_components: drifts
                .iter()
                .filter(|d| d.kind == DriftKind::UndocumentedComponent)
                .count(),
            missing_components: drifts
                .iter()
                .filter(|d| d.kind == DriftKind::MissingComponent)
                .count(),
            undocumented_relationships: drifts
                .iter()
                .filter(|d| d.kind == DriftKind::UndocumentedRelationship)
                .count(),
            boundary_violations: drifts
                .iter()
                .filter(|d| d.kind == DriftKind::BoundaryViolation)
                .count(),
            policy_violations: drifts
                .iter()
                .filter(|d| d.kind == DriftKind::PolicyViolation)
                .count(),
            schema_violations: drifts
                .iter()
                .filter(|d| d.kind == DriftKind::SchemaViolation)
                .count(),
            taxonomy_mismatches: drifts
                .iter()
                .filter(|d| d.kind == DriftKind::TaxonomyMismatch)
                .count(),
        };

        let drift_score = Self::compute_drift_score(&summary, &drifts);
        let health = Self::classify_health(drift_score);

        DriftReport {
            intent_source: intent.source.name.clone(),
            reality_source: "scanned codebase".to_string(),
            drifts,
            drift_score,
            health,
            summary,
        }
    }

    pub(crate) fn compute_drift_score(summary: &DriftSummary, drifts: &[Drift]) -> u8 {
        if summary.total_components_declared == 0 {
            return 0;
        }
        let mut score: f32 = 0.0;
        score += summary.undocumented_components as f32 * 5.0;
        score += summary.missing_components as f32 * 10.0;
        score += summary.undocumented_relationships as f32 * 2.0;
        score += summary.boundary_violations as f32 * 15.0;
        score += summary.policy_violations as f32 * 8.0;
        for d in drifts {
            score += match d.severity {
                Severity::Critical => 15.0,
                Severity::High => 10.0,
                Severity::Medium => 5.0,
                Severity::Low => 2.0,
                Severity::Info => 0.0,
            };
        }
        let max_score = (summary.total_components_declared.max(1) as f32) * 20.0;
        ((score / max_score * 100.0).min(100.0)) as u8
    }

    pub(crate) fn classify_health(score: u8) -> DriftHealth {
        match score {
            0..=20 => DriftHealth::Healthy,
            21..=50 => DriftHealth::MinorDrift,
            51..=75 => DriftHealth::SignificantDrift,
            _ => DriftHealth::CriticalDrift,
        }
    }

    fn detect_boundary_violations(
        &self,
        boundary: &DeclaredBoundary,
        reality: &Graph,
    ) -> Vec<Drift> {
        let mut drifts = Vec::new();

        let inside_set: HashSet<&str> = boundary.inside.iter().map(|s| s.as_str()).collect();

        for edge in &reality.edges {
            let src_inside = inside_set.contains(edge.source.as_str());
            let tgt_inside = inside_set.contains(edge.target.as_str());

            if src_inside && !tgt_inside {
                let allowed = boundary.allowed_connections.iter().any(|ac| {
                    inside_set.contains(edge.target.as_str()) || ac.target_boundary == edge.target
                });

                if !allowed {
                    for rule in &boundary.rules {
                        if rule.rule_type == crate::model::BoundaryRuleType::NoDirectDatabaseAccess
                            && (edge.target.contains("database") || edge.target.contains("db"))
                        {
                            drifts.push(Drift {
                                kind: DriftKind::BoundaryViolation,
                                severity: Severity::High,
                                description: format!(
                                    "Boundary '{}' violated: {} directly accesses {}",
                                    boundary.name, edge.source, edge.target
                                ),
                                evidence: vec![Evidence {
                                    source: "scan".to_string(),
                                    location: None,
                                    detail: format!("Edge {} -> {}", edge.source, edge.target),
                                }],
                                intent_ref: Some(boundary.source_ref.file.clone()),
                                suggestion: Some(rule.description.clone()),
                            });
                        }
                    }
                }
            }
        }

        drifts
    }

    fn detect_policy_violations(&self, policy: &DeclaredPolicy, reality: &Graph) -> Vec<Drift> {
        let mut drifts = Vec::new();

        for rule in &policy.rules {
            if let Some(ref content) = rule.content {
                match content {
                    PolicyRuleContent::DenyEdge {
                        from,
                        to,
                        except,
                        message,
                        suggestions,
                    } => {
                        for edge in &reality.edges {
                            let src_opt = reality.nodes.iter().find(|n| n.id == edge.source);
                            let tgt_opt = reality.nodes.iter().find(|n| n.id == edge.target);

                            if let (Some(src), Some(tgt)) = (src_opt, tgt_opt) {
                                if node_matches_selector(src, from)
                                    && node_matches_selector(tgt, to)
                                {
                                    // Check exceptions
                                    let is_except = except.iter().any(|e| {
                                        node_matches_selector(src, &e.from)
                                            && node_matches_selector(tgt, &e.to)
                                    });

                                    if !is_except {
                                        drifts.push(Drift {
                                            kind: DriftKind::PolicyViolation,
                                            severity: Severity::High,
                                            description: message.clone().unwrap_or_else(|| {
                                                format!(
                                                    "Policy '{}' violation: edge {} -> {} is forbidden",
                                                    policy.name, src.id, tgt.id
                                                )
                                            }),
                                            evidence: vec![Evidence {
                                                source: "scan".to_string(),
                                                location: None,
                                                detail: format!("Forbidden edge detected: {} -> {}", src.id, tgt.id),
                                            }],
                                            intent_ref: Some(policy.source_ref.file.clone()),
                                            suggestion: suggestions.first().cloned().or_else(|| Some("Remove this illegal dependency or update the architecture policy.".to_string())),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    PolicyRuleContent::RequireTags {
                        selector,
                        tags,
                        except,
                        message,
                        suggestions,
                    } => {
                        for node in &reality.nodes {
                            if node_matches_selector(node, selector) {
                                let is_except =
                                    except.iter().any(|e| node_matches_selector(node, e));
                                if is_except {
                                    continue;
                                }

                                for required_tag in tags {
                                    // Sruja tags are usually in metadata "tags" as a comma-separated string, or just keys.
                                    // We'll check for both.
                                    let has_tag = node.metadata.contains_key(required_tag)
                                        || node
                                            .metadata
                                            .get("tags")
                                            .map(|t| t.split(',').any(|s| s.trim() == required_tag))
                                            .unwrap_or(false);

                                    if !has_tag {
                                        drifts.push(Drift {
                                            kind: DriftKind::PolicyViolation,
                                            severity: Severity::Medium,
                                            description: message.clone().unwrap_or_else(|| {
                                                format!(
                                                    "Policy '{}' violation: component '{}' is missing required tag '{}'",
                                                    policy.name, node.id, required_tag
                                                )
                                            }),
                                            evidence: vec![Evidence {
                                                source: "scan".to_string(),
                                                location: None,
                                                detail: format!("Component {} is missing tag {}", node.id, required_tag),
                                            }],
                                            intent_ref: Some(policy.source_ref.file.clone()),
                                            suggestion: suggestions.first().cloned().or_else(|| Some(format!("Add tag '{}' to {}", required_tag, node.id))),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    PolicyRuleContent::RequireMetadata {
                        selector,
                        key,
                        value,
                        except,
                        message,
                        suggestions,
                    } => {
                        for node in &reality.nodes {
                            if node_matches_selector(node, selector) {
                                let is_except =
                                    except.iter().any(|e| node_matches_selector(node, e));
                                if is_except {
                                    continue;
                                }

                                let metadata_value = node.metadata.get(key);
                                let is_valid = match (metadata_value, value) {
                                    (Some(mv), Some(rv)) => mv == rv,
                                    (Some(_), None) => true,
                                    (None, _) => false,
                                };

                                if !is_valid {
                                    drifts.push(Drift {
                                        kind: DriftKind::PolicyViolation,
                                        severity: Severity::Medium,
                                        description: message.clone().unwrap_or_else(|| {
                                            format!(
                                                "Policy '{}' violation: component '{}' is missing or has incorrect metadata '{}'",
                                                policy.name, node.id, key
                                            )
                                        }),
                                        evidence: vec![Evidence {
                                            source: "scan".to_string(),
                                            location: None,
                                            detail: format!("Metadata check failed for {}: key={}", node.id, key),
                                        }],
                                        intent_ref: Some(policy.source_ref.file.clone()),
                                        suggestion: suggestions.first().cloned(),
                                    });
                                }
                            }
                        }
                    }
                    PolicyRuleContent::RequireSlo {
                        selector,
                        except,
                        message,
                        suggestions,
                    } => {
                        for node in &reality.nodes {
                            if node_matches_selector(node, selector) {
                                let is_except =
                                    except.iter().any(|e| node_matches_selector(node, e));
                                if is_except {
                                    continue;
                                }

                                let has_slo = node
                                    .metadata
                                    .keys()
                                    .any(|k| k.contains("slo") || k.contains("availability"));
                                if !has_slo {
                                    drifts.push(Drift {
                                        kind: DriftKind::PolicyViolation,
                                        severity: Severity::Medium,
                                        description: message.clone().unwrap_or_else(|| {
                                            format!(
                                                "Policy '{}' violation: component '{}' is missing required SLO metadata",
                                                policy.name, node.id
                                            )
                                        }),
                                        evidence: vec![Evidence {
                                            source: "scan".to_string(),
                                            location: None,
                                            detail: format!("SLO metadata missing for {}", node.id),
                                        }],
                                        intent_ref: Some(policy.source_ref.file.clone()),
                                        suggestion: suggestions.first().cloned().or_else(|| Some("Define SLO metadata (availability, latency) for this component.".to_string())),
                                    });
                                }
                            }
                        }
                    }
                    PolicyRuleContent::Phrase(phrase) => {
                        // Minimal heuristic for Phrases: "X must not call Y"
                        if phrase.contains("must not call") || phrase.contains("cannot call") {
                            let parts: Vec<&str> = if phrase.contains("must not call") {
                                phrase.split("must not call").collect()
                            } else {
                                phrase.split("cannot call").collect()
                            };
                            if parts.len() == 2 {
                                let from_pattern = parts[0].trim();
                                let to_pattern = parts[1].trim();

                                for edge in &reality.edges {
                                    if edge.source.contains(from_pattern)
                                        && edge.target.contains(to_pattern)
                                    {
                                        drifts.push(Drift {
                                            kind: DriftKind::PolicyViolation,
                                            severity: Severity::High,
                                            description: format!(
                                                "Policy '{}' violation: {} -> {} violates phrase '{}'",
                                                policy.name, edge.source, edge.target, phrase
                                            ),
                                            evidence: vec![Evidence {
                                                source: "scan".to_string(),
                                                location: None,
                                                detail: format!("Phrase match: {} -> {}", edge.source, edge.target),
                                            }],
                                            intent_ref: Some(policy.source_ref.file.clone()),
                                            suggestion: Some(format!("Refactor to remove dependency from {} to {}", edge.source, edge.target)),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        drifts
    }
}

fn node_matches_selector(node: &Node, selector: &PolicySelector) -> bool {
    if let Some(ref kind) = selector.kind {
        if format!("{:?}", node.kind).to_lowercase() != kind.to_lowercase() {
            return false;
        }
    }
    if let Some(ref id) = selector.id {
        if node.id != *id && !node.id.contains(id) {
            return false;
        }
    }
    if let Some(ref tech) = selector.technology {
        if node
            .technology
            .as_ref()
            .map(|t| t.to_lowercase() != tech.to_lowercase())
            .unwrap_or(true)
        {
            return false;
        }
    }
    for tag in &selector.tags {
        let has_tag = node.metadata.contains_key(tag)
            || node
                .metadata
                .get("tags")
                .map(|t: &String| t.split(',').any(|s| s.trim() == tag))
                .unwrap_or(false);
        if !has_tag {
            return false;
        }
    }
    for meta in &selector.meta {
        if let Some(ref val) = meta.value {
            if node.metadata.get(&meta.key) != Some(val) {
                return false;
            }
        } else if !node.metadata.contains_key(&meta.key) {
            return false;
        }
    }
    true
}

impl std::fmt::Display for DriftHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriftHealth::Healthy => write!(f, "Healthy"),
            DriftHealth::MinorDrift => write!(f, "Minor Drift"),
            DriftHealth::SignificantDrift => write!(f, "Significant Drift"),
            DriftHealth::CriticalDrift => write!(f, "Critical Drift"),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

impl std::fmt::Display for DriftKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriftKind::UndocumentedComponent => write!(f, "Undocumented Component"),
            DriftKind::MissingComponent => write!(f, "Missing Component"),
            DriftKind::BoundaryViolation => write!(f, "Boundary Violation"),
            DriftKind::UndocumentedRelationship => write!(f, "Undocumented Relationship"),
            DriftKind::MissingRelationship => write!(f, "Missing Relationship"),
            DriftKind::TechnologyMismatch => write!(f, "Technology Mismatch"),
            DriftKind::PolicyViolation => write!(f, "Policy Violation"),
            DriftKind::ConstraintViolation => write!(f, "Constraint Violation"),
            DriftKind::SchemaViolation => write!(f, "Schema Violation"),
            DriftKind::TaxonomyMismatch => write!(f, "Taxonomy Mismatch"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BoundaryRule, BoundaryRuleType, ConnectionType, DeclaredBoundary, DeclaredComponent,
        DeclaredPolicy, DeclaredRelationship, IntentSourceInfo, IntentSourceType,
        PolicyEdgeException, PolicyMetaSelector, PolicyRule, PolicyRuleContent, PolicySelector,
        SourceReference,
    };
    use sruja_scan::{Edge, EdgeKind, Graph as ScanGraph, Node, NodeKind};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn source_ref(element: &str) -> SourceReference {
        SourceReference {
            file: "test.sruja".to_string(),
            line: Some(1),
            element: Some(element.to_string()),
        }
    }

    fn scan_node(id: &str, label: &str, path: &str) -> Node {
        Node {
            id: id.to_string(),
            kind: NodeKind::Module,
            label: label.to_string(),
            path: Some(path.to_string()),
            technology: None,
            metadata: HashMap::new(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
        }
    }

    fn scan_node_with(
        id: &str,
        kind: NodeKind,
        technology: Option<&str>,
        metadata: HashMap<&str, &str>,
    ) -> Node {
        Node {
            id: id.to_string(),
            kind,
            label: id.to_string(),
            path: None,
            technology: technology.map(|t| t.to_string()),
            metadata: metadata
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
        }
    }

    fn create_test_graph() -> ScanGraph {
        ScanGraph {
            metadata: HashMap::new(),
            nodes: vec![
                scan_node("api", "API", "src/api"),
                scan_node("db", "Database", "src/db"),
            ],
            edges: vec![Edge {
                source: "api".to_string(),
                target: "db".to_string(),
                kind: EdgeKind::Calls,
                evidence: vec![],
            }],
            confidence: None,
        }
    }

    fn create_test_intent() -> IntentModel {
        let mut model = IntentModel::new(IntentSourceInfo {
            source_type: IntentSourceType::Manual,
            path: PathBuf::from("test.sruja"),
            name: "test".to_string(),
        });
        model.components.push(DeclaredComponent {
            id: "api".to_string(),
            kind: "service".to_string(),
            label: "API Service".to_string(),
            description: None,
            technology: None,
            source_ref: source_ref("api"),
        });
        model
    }

    #[test]
    fn test_detect_undocumented_component() {
        let detector = DriftDetector::new();
        let intent = create_test_intent();
        let reality = create_test_graph();

        let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

        assert!(report
            .drifts
            .iter()
            .any(|d| d.kind == DriftKind::UndocumentedComponent));
    }

    #[test]
    fn test_detect_missing_component() {
        let detector = DriftDetector::new();
        let intent = create_test_intent();
        let reality = ScanGraph::new();

        let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

        assert!(report
            .drifts
            .iter()
            .any(|d| d.kind == DriftKind::MissingComponent));
    }

    #[test]
    fn test_healthy_alignment() {
        let detector = DriftDetector::new();
        let mut intent = create_test_intent();
        intent.components.push(DeclaredComponent {
            id: "db".to_string(),
            kind: "service".to_string(),
            label: "Database".to_string(),
            description: None,
            technology: None,
            source_ref: source_ref("db"),
        });
        let reality = create_test_graph();

        let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

        assert!(report.drift_score < 50);
    }

    #[test]
    fn test_detect_undocumented_relationship() {
        let detector = DriftDetector::new();
        let intent = create_test_intent();
        let reality = create_test_graph();

        let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

        assert!(report
            .drifts
            .iter()
            .any(|d| d.kind == DriftKind::UndocumentedRelationship));
    }

    #[test]
    fn test_detect_missing_relationship() {
        let detector = DriftDetector::new();
        let mut intent = create_test_intent();
        intent.relationships.push(DeclaredRelationship {
            source: "api".to_string(),
            target: "db".to_string(),
            kind: "calls".to_string(),
            label: None,
            source_ref: source_ref("api -> db"),
        });

        let mut reality = ScanGraph::new();
        reality.nodes.push(scan_node("api", "API", "src/api"));
        reality.nodes.push(scan_node("db", "Database", "src/db"));

        let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

        assert!(report
            .drifts
            .iter()
            .any(|d| d.kind == DriftKind::MissingRelationship));
    }

    #[test]
    fn test_detect_boundary_violation_no_direct_database_access() {
        let detector = DriftDetector::new();
        let mut intent = create_test_intent();
        intent.boundaries.push(DeclaredBoundary {
            name: "Core".to_string(),
            inside: vec!["api".to_string()],
            allowed_connections: Vec::new(),
            rules: vec![BoundaryRule {
                rule_type: BoundaryRuleType::NoDirectDatabaseAccess,
                description: "Use a repository/service boundary instead of direct DB access."
                    .to_string(),
            }],
            source_ref: source_ref("Core"),
        });
        let reality = create_test_graph();

        let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

        assert!(report
            .drifts
            .iter()
            .any(|d| d.kind == DriftKind::BoundaryViolation));
    }

    #[test]
    fn test_boundary_violation_not_reported_when_allowed_connection_matches_target() {
        let detector = DriftDetector::new();
        let mut intent = create_test_intent();
        intent.boundaries.push(DeclaredBoundary {
            name: "Core".to_string(),
            inside: vec!["api".to_string()],
            allowed_connections: vec![crate::model::AllowedConnection {
                target_boundary: "db".to_string(),
                via: ConnectionType::Database,
            }],
            rules: vec![BoundaryRule {
                rule_type: BoundaryRuleType::NoDirectDatabaseAccess,
                description: "Use a repository/service boundary instead of direct DB access."
                    .to_string(),
            }],
            source_ref: source_ref("Core"),
        });
        let reality = create_test_graph();

        let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

        assert!(!report
            .drifts
            .iter()
            .any(|d| d.kind == DriftKind::BoundaryViolation));
    }

    #[test]
    fn test_policy_deny_edge_detects_violation_and_respects_exceptions() {
        let detector = DriftDetector::new();
        let mut intent = create_test_intent();
        intent.policies.push(DeclaredPolicy {
            name: "NoDbCalls".to_string(),
            description: "Services must not call the DB directly.".to_string(),
            category: String::new(),
            enforcement: String::new(),
            scope: Vec::new(),
            rules: vec![PolicyRule {
                description: "No direct api->db".to_string(),
                constraint: "deny".to_string(),
                content: Some(PolicyRuleContent::DenyEdge {
                    from: PolicySelector {
                        id: Some("api".to_string()),
                        ..Default::default()
                    },
                    to: PolicySelector {
                        id: Some("db".to_string()),
                        ..Default::default()
                    },
                    except: vec![PolicyEdgeException {
                        from: PolicySelector {
                            id: Some("api".to_string()),
                            ..Default::default()
                        },
                        to: PolicySelector {
                            id: Some("db".to_string()),
                            ..Default::default()
                        },
                    }],
                    message: None,
                    suggestions: vec!["Introduce an API boundary layer.".to_string()],
                }),
            }],
            source_ref: source_ref("NoDbCalls"),
        });
        let reality = create_test_graph();

        let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

        assert!(!report
            .drifts
            .iter()
            .any(|d| d.kind == DriftKind::PolicyViolation));

        intent.policies[0].rules[0].content = Some(PolicyRuleContent::DenyEdge {
            from: PolicySelector {
                id: Some("api".to_string()),
                ..Default::default()
            },
            to: PolicySelector {
                id: Some("db".to_string()),
                ..Default::default()
            },
            except: Vec::new(),
            message: None,
            suggestions: vec![],
        });

        let report = detector.detect(&intent, &reality, &DomainSchema::architecture());
        assert!(report
            .drifts
            .iter()
            .any(|d| d.kind == DriftKind::PolicyViolation && d.severity == Severity::High));
    }

    #[test]
    fn test_policy_require_tags_and_metadata_and_slo() {
        let detector = DriftDetector::new();
        let mut intent = create_test_intent();
        intent.policies.push(DeclaredPolicy {
            name: "MetadataRules".to_string(),
            description: "Ensure required tags/metadata exist.".to_string(),
            category: String::new(),
            enforcement: String::new(),
            scope: Vec::new(),
            rules: vec![
                PolicyRule {
                    description: "API must be tagged".to_string(),
                    constraint: "tags".to_string(),
                    content: Some(PolicyRuleContent::RequireTags {
                        selector: PolicySelector {
                            id: Some("api".to_string()),
                            ..Default::default()
                        },
                        tags: vec!["public".to_string()],
                        except: Vec::new(),
                        message: None,
                        suggestions: Vec::new(),
                    }),
                },
                PolicyRule {
                    description: "API must have tier metadata".to_string(),
                    constraint: "meta".to_string(),
                    content: Some(PolicyRuleContent::RequireMetadata {
                        selector: PolicySelector {
                            id: Some("api".to_string()),
                            ..Default::default()
                        },
                        key: "tier".to_string(),
                        value: Some("1".to_string()),
                        except: Vec::new(),
                        message: None,
                        suggestions: Vec::new(),
                    }),
                },
                PolicyRule {
                    description: "API must define SLO".to_string(),
                    constraint: "slo".to_string(),
                    content: Some(PolicyRuleContent::RequireSlo {
                        selector: PolicySelector {
                            id: Some("api".to_string()),
                            ..Default::default()
                        },
                        except: Vec::new(),
                        message: None,
                        suggestions: Vec::new(),
                    }),
                },
            ],
            source_ref: source_ref("MetadataRules"),
        });

        let mut reality = ScanGraph::new();
        reality.nodes.push(scan_node_with(
            "api",
            NodeKind::Service,
            Some("Rust"),
            HashMap::new(),
        ));
        reality.nodes.push(scan_node_with(
            "db",
            NodeKind::Database,
            Some("PostgreSQL"),
            HashMap::new(),
        ));
        reality.edges.push(Edge {
            source: "api".to_string(),
            target: "db".to_string(),
            kind: EdgeKind::Calls,
            evidence: vec![],
        });

        let report = detector.detect(&intent, &reality, &DomainSchema::architecture());
        assert!(report
            .drifts
            .iter()
            .any(|d| d.kind == DriftKind::PolicyViolation));

        let mut reality = reality;
        let mut meta = HashMap::new();
        meta.insert("tags", "public,internal");
        meta.insert("tier", "1");
        meta.insert("availability_slo", "99.9");
        reality.nodes[0] = scan_node_with("api", NodeKind::Service, Some("Rust"), meta);

        let report = detector.detect(&intent, &reality, &DomainSchema::architecture());
        assert!(!report
            .drifts
            .iter()
            .any(|d| d.kind == DriftKind::PolicyViolation));
    }

    #[test]
    fn test_policy_phrase_compatibility_shim() {
        let detector = DriftDetector::new();
        let mut intent = create_test_intent();
        intent.policies.push(DeclaredPolicy {
            name: "NoApiDbPhrase".to_string(),
            description: "Legacy phrase rule".to_string(),
            category: String::new(),
            enforcement: String::new(),
            scope: Vec::new(),
            rules: vec![PolicyRule {
                description: "Legacy phrase".to_string(),
                constraint: "api must not call db".to_string(),
                content: Some(PolicyRuleContent::Phrase(
                    "api must not call db".to_string(),
                )),
            }],
            source_ref: source_ref("NoApiDbPhrase"),
        });
        let reality = create_test_graph();

        let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

        assert!(report
            .drifts
            .iter()
            .any(|d| d.kind == DriftKind::PolicyViolation));
    }

    #[test]
    fn test_node_matches_selector_filters_kind_id_technology_tags_and_meta() {
        let mut metadata = HashMap::new();
        metadata.insert("tags", "public, pci");
        metadata.insert("tier", "1");
        metadata.insert("pci", "");
        let node = scan_node_with("payments.api", NodeKind::Service, Some("Rust"), metadata);

        let selector = PolicySelector {
            kind: Some("service".to_string()),
            id: Some("api".to_string()),
            technology: Some("rust".to_string()),
            tags: vec!["public".to_string(), "pci".to_string()],
            meta: vec![
                PolicyMetaSelector {
                    key: "tier".to_string(),
                    value: Some("1".to_string()),
                },
                PolicyMetaSelector {
                    key: "pci".to_string(),
                    value: None,
                },
            ],
        };

        assert!(node_matches_selector(&node, &selector));

        let selector = PolicySelector {
            technology: Some("Go".to_string()),
            ..selector
        };
        assert!(!node_matches_selector(&node, &selector));
    }

    #[test]
    fn test_compute_drift_score_zero_when_no_declared_components() {
        let summary = DriftSummary {
            total_components_declared: 0,
            total_components_discovered: 0,
            undocumented_components: 1,
            missing_components: 1,
            undocumented_relationships: 1,
            boundary_violations: 1,
            policy_violations: 1,
            schema_violations: 0,
            taxonomy_mismatches: 0,
        };
        let drifts = vec![Drift {
            kind: DriftKind::PolicyViolation,
            severity: Severity::Critical,
            description: "x".to_string(),
            evidence: vec![],
            intent_ref: None,
            suggestion: None,
        }];

        assert_eq!(DriftDetector::compute_drift_score(&summary, &drifts), 0);
    }

    #[test]
    fn test_drift_health_classification() {
        let _detector = DriftDetector::new();

        assert_eq!(DriftDetector::classify_health(10), DriftHealth::Healthy);
        assert_eq!(DriftDetector::classify_health(35), DriftHealth::MinorDrift);
        assert_eq!(
            DriftDetector::classify_health(60),
            DriftHealth::SignificantDrift
        );
        assert_eq!(
            DriftDetector::classify_health(80),
            DriftHealth::CriticalDrift
        );
    }

    #[test]
    fn test_detect_with_loaded_intent_from_fixture_dir() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let sruja_path = temp_dir.path().join("arch.sruja");
        let minimal_sruja = r#"
api = container "API" {
  technology "Node.js"
  description "HTTP API"
}
"#;
        std::fs::write(&sruja_path, minimal_sruja).expect("write sruja");

        let mut intelligence = crate::IntentContext::new();
        let models = intelligence
            .load_from_directory(temp_dir.path())
            .expect("load from directory");

        assert!(
            !models.is_empty(),
            "should load at least one model from .sruja"
        );

        let mut merged = crate::IntentModel::default();
        for m in models {
            merged.merge(m);
        }

        let detector = DriftDetector::new();
        let reality = create_test_graph();
        let report = detector.detect(&merged, &reality, &DomainSchema::architecture());

        assert!(report.drift_score <= 100);
        assert!(!report.intent_source.is_empty());
        assert!(!report.reality_source.is_empty());
    }
}
