use std::collections::HashSet;

use sruja_language::DomainSchema;
use sruja_scan::Graph;

use crate::model::{
    DeclaredBoundary, DeclaredPolicy, IntentModel, PolicyRuleContent,
};

use super::format;
use super::mapper;
use super::types::*;

impl DriftDetector {
    pub fn new() -> Self {
        Self {
            config: DriftConfig::default(),
        }
    }

    pub fn with_config(config: DriftConfig) -> Self {
        Self { config }
    }

    pub fn detect(
        &self,
        intent: &IntentModel,
        reality: &Graph,
        schema: &DomainSchema,
    ) -> DriftReport {
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
            unproposed_changes: drifts
                .iter()
                .filter(|d| d.kind == DriftKind::UnproposedChange)
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

    /// Evaluate policy violations from an IntentModel against a scan graph.
    /// Returns only PolicyViolation drifts.
    pub fn evaluate_policy_violations(intent: &IntentModel, reality: &Graph) -> Vec<Drift> {
        let schema = DomainSchema::architecture();
        let report = DriftDetector::new().detect(intent, reality, &schema);
        report
            .drifts
            .into_iter()
            .filter(|d| d.kind == DriftKind::PolicyViolation)
            .collect()
    }

    /// Evaluate all architecture violations (policy + boundary) from an IntentModel against a scan graph.
    pub fn evaluate_all_violations(intent: &IntentModel, reality: &Graph) -> Vec<Drift> {
        let schema = DomainSchema::architecture();
        let report = DriftDetector::new().detect(intent, reality, &schema);
        report
            .drifts
            .into_iter()
            .filter(|d| {
                d.kind == DriftKind::PolicyViolation || d.kind == DriftKind::BoundaryViolation
            })
            .collect()
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
        let max_depth = boundary.max_depth.unwrap_or(2);

        for start_node_id in &boundary.inside {
            let radius = reality.blast_radius(start_node_id, max_depth);

            for downstream in &radius.downstream {
                if !inside_set.contains(downstream.id.as_str()) {
                    let is_allowed = boundary
                        .allowed_connections
                        .iter()
                        .any(|ac| ac.target_boundary == downstream.id);

                    if !is_allowed {
                        for rule in &boundary.rules {
                            match rule.rule_type {
                                crate::model::BoundaryRuleType::NoDirectDatabaseAccess => {
                                    let target_id_lower = downstream.id.to_lowercase();
                                    if target_id_lower.contains("database")
                                        || target_id_lower.contains("db")
                                    {
                                        let is_transitive = downstream.depth > 1;
                                        let prefix = if is_transitive { "Transitive " } else { "" };

                                        drifts.push(Drift {
                                            kind: DriftKind::BoundaryViolation,
                                            severity: Severity::High,
                                            description: format!(
                                                "Boundary '{}' violated: {} {}accesses database {} (depth {})",
                                                boundary.name,
                                                start_node_id,
                                                if is_transitive { "transitively " } else { "directly " },
                                                downstream.id,
                                                downstream.depth
                                            ),
                                            evidence: vec![Evidence {
                                                source: "scan".to_string(),
                                                location: None,
                                                detail: format!(
                                                    "{} dependency chain detected from {} to {} (hop count: {})",
                                                    prefix, start_node_id, downstream.id, downstream.depth
                                                ),
                                            }],
                                            intent_ref: Some(boundary.source_ref.file.clone()),
                                            suggestion: Some(rule.description.clone()),
                                        });
                                    }
                                }
                                _ => {
                                    drifts.push(Drift {
                                        kind: DriftKind::BoundaryViolation,
                                        severity: Severity::Medium,
                                        description: format!(
                                            "Boundary '{}' violation: {} depends on external component {} (depth {}) which is not in allowed_connections",
                                            boundary.name, start_node_id, downstream.id, downstream.depth
                                        ),
                                        evidence: vec![Evidence {
                                            source: "scan".to_string(),
                                            location: None,
                                            detail: format!("Unallowed transitive dependency: {} -> ... -> {}", start_node_id, downstream.id),
                                        }],
                                        intent_ref: Some(boundary.source_ref.file.clone()),
                                        suggestion: Some(format!("Add {} to allowed_connections for boundary {}", downstream.id, boundary.name)),
                                    });
                                }
                            }
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
                                if format::node_matches_selector(src, from)
                                    && format::node_matches_selector(tgt, to)
                                {
                                    let is_except = except.iter().any(|e| {
                                        format::node_matches_selector_strict(src, &e.from)
                                            && format::node_matches_selector_strict(tgt, &e.to)
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
                            if format::node_matches_selector(node, selector) {
                                let is_except =
                                    except.iter().any(|e| format::node_matches_selector(node, e));
                                if is_except {
                                    continue;
                                }

                                for required_tag in tags {
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
                            if format::node_matches_selector(node, selector) {
                                let is_except =
                                    except.iter().any(|e| format::node_matches_selector(node, e));
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
                            if format::node_matches_selector(node, selector) {
                                let is_except =
                                    except.iter().any(|e| format::node_matches_selector(node, e));
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
