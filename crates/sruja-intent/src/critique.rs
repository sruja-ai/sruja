//! Adversarial architectural critique engine
//!
//! @element Sruja.Intent.CritiqueEngine
//! @layer Secondary
//! @boundary Critique must use the real violation engine, not metadata restatement

use serde::{Deserialize, Serialize};
use sruja_language::Program;
use sruja_scan::{Criticality, Graph};

use crate::compare::{DriftDetector, Severity as DriftSeverity};
use crate::model::IntentModel;

/// Request for architectural critique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueRequest {
    /// Changed file paths (relative to repo root)
    pub changed_files: Vec<String>,
    /// Optional: description of what the change does
    pub description: Option<String>,
    /// Optional: proposal ID if this is a proposed change
    pub proposal_id: Option<String>,
    /// Optional: git base/head refs for diff-based critique
    pub base_ref: Option<String>,
    pub head_ref: Option<String>,
}

/// Result of an architectural critique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueReport {
    /// Real violations from the violation engine, scoped to the change
    pub violations: Vec<CritiqueFinding>,
    /// Awareness items (constraints, gotchas, incidents) — gated on accuracy.
    /// Empty in v1; the accuracy gate opens when relationship-diff semantics are available.
    pub context: Vec<CritiqueFinding>,
    pub risk_level: RiskLevel,
    pub summary: String,
    pub affected_elements: Vec<String>,
    pub blast_radius: BlastRadiusSummary,
    /// Whether a baseline (.sruja architecture file) was loaded.
    /// When false, the report is ungraded — Clear with no rules to check.
    pub baseline_present: bool,
}

/// A single finding from the critique engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueFinding {
    pub category: CritiqueCategory,
    pub severity: CritiqueSeverity,
    pub title: String,
    pub detail: String,
    pub evidence: Vec<CritiqueEvidence>,
    pub suggestion: Option<String>,
    pub confidence: f32,
    /// Stable rule identifier (e.g., "SRUJA-INTENT-POLICY-001").
    /// Populated by the real violation engine; None for legacy/context items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
}

/// Categories of critique findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CritiqueCategory {
    PolicyViolation,
    HistoricalPatternMatch,
    ConstraintBreach,
    BlastRadius,
    BehavioralContractDrift,
    GotchaWarning,
    UnproposedChange,
}

/// Severity of a critique finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CritiqueSeverity {
    /// FYI — awareness item
    Low,
    /// Worth considering — potential risk
    Medium,
    /// Strongly recommended — this has caused issues before
    High,
    /// Must fix — this will break something or violate critical policy
    Critical,
}

/// Overall risk level of the change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// No concerns found
    Clear,
    /// Minor issues, proceed with awareness
    Caution,
    /// Significant concerns, review carefully
    Warning,
    /// High-risk change, requires senior review
    Danger,
}

/// Summary of the blast radius
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusSummary {
    pub total_affected_elements: usize,
    pub downstream_consumers: usize,
    pub max_depth: usize,
}

/// Evidence for a critique finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueEvidence {
    pub source: String,
    pub location: Option<String>,
    pub detail: String,
}

pub struct CritiqueEngine {
    pub graph: Graph,
    pub program: Option<Program>,
    /// Declared architectural intent model, used by DriftDetector for policy/boundary violations.
    pub intent: Option<IntentModel>,
}

impl CritiqueEngine {
    pub fn new(graph: Graph, program: Option<Program>) -> Self {
        Self {
            graph,
            program,
            intent: None,
        }
    }

    /// Attach an intent model for policy/boundary violation detection via DriftDetector.
    pub fn with_intent(mut self, intent: Option<IntentModel>) -> Self {
        self.intent = intent;
        self
    }

    pub fn critique(&self, request: &CritiqueRequest) -> CritiqueReport {
        let affected = self.resolve_affected_elements(&request.changed_files);
        let baseline_present = self.program.is_some();

        // ── Violations tier: real detector output, scoped to the change ──
        let mut violations = self.check_policy_violations(&affected);

        // ── Context tier: constraints, gotchas, incidents (accuracy-gated) ──
        // In v1, the gate is closed (returns false). The passes are retained
        // for future revival when relationship-diff semantics become available.
        // Per R5: "a pass that cannot be accurate does not fire by default."
        let context = if self.context_gate_open() {
            let mut ctx = Vec::new();
            ctx.extend(self.check_incident_patterns(&affected, &request.description));
            ctx.extend(self.check_constraint_breaches(&affected));
            ctx.extend(self.surface_gotchas(&affected));
            ctx
        } else {
            Vec::new()
        };

        // ── Blast radius (report metadata, not a finding) ──
        let blast_summary = self.assess_blast_radius(&affected);

        // Sort violations by severity (highest first)
        violations.sort_by_key(|b| std::cmp::Reverse(b.severity));

        let risk_level = self.compute_risk_level(&violations);
        let summary = self.generate_summary(
            &violations,
            &context,
            &affected,
            &blast_summary,
            baseline_present,
        );

        CritiqueReport {
            violations,
            context,
            risk_level,
            summary,
            affected_elements: affected,
            blast_radius: blast_summary,
            baseline_present,
        }
    }

    /// v1 accuracy gate for the Context tier.
    /// Returns false — the relevance predicate requires relationship-diff
    /// semantics that are not yet available in this crate.
    fn context_gate_open(&self) -> bool {
        false
    }

    /// Resolve changed files to affected graph element IDs using exact attribution.
    /// Replaces the old bidirectional substring matcher with precise source/path matching.
    pub fn resolve_affected_elements(&self, changed_files: &[String]) -> Vec<String> {
        let mut affected = std::collections::HashSet::new();
        for file in changed_files {
            for node in &self.graph.nodes {
                let matches = node.sources.iter().any(|s| s.path == *file)
                    || node
                        .path
                        .as_ref()
                        .is_some_and(|p| p == file || is_within_dir(file, p));

                if matches {
                    affected.insert(node.id.clone());
                }
            }
        }
        affected.into_iter().collect()
    }

    fn check_policy_violations(&self, affected: &[String]) -> Vec<CritiqueFinding> {
        let mut findings = Vec::new();
        let affected_set: std::collections::HashSet<&str> =
            affected.iter().map(|s| s.as_str()).collect();

        // 1. Structural validation via sruja-engine (layer, cycle, orphan, etc.)
        if let Some(program) = &self.program {
            let validator =
                sruja_engine::Validator::with_profile(sruja_engine::RuleProfile::Minimal);
            let diagnostics = validator.validate_sync(program);
            for diag in diagnostics {
                let involves_affected = affected_set.iter().any(|id| {
                    diag.message.contains(id) || diag.context.iter().any(|c| c.contains(id))
                });

                if !involves_affected {
                    continue;
                }

                let severity = match diag.severity {
                    sruja_diagnostics::Severity::Error => CritiqueSeverity::High,
                    sruja_diagnostics::Severity::Warning => CritiqueSeverity::Medium,
                    sruja_diagnostics::Severity::Info | sruja_diagnostics::Severity::Hint => {
                        CritiqueSeverity::Low
                    }
                };

                findings.push(CritiqueFinding {
                    category: CritiqueCategory::PolicyViolation,
                    severity,
                    title: diag
                        .message
                        .split('.')
                        .next()
                        .unwrap_or(&diag.message)
                        .trim()
                        .to_string(),
                    detail: diag.message,
                    evidence: diag
                        .context
                        .iter()
                        .map(|c| CritiqueEvidence {
                            source: "sruja-engine".to_string(),
                            location: Some(format!(
                                "{}:{}",
                                diag.location.file, diag.location.line
                            )),
                            detail: c.clone(),
                        })
                        .collect(),
                    suggestion: diag.suggestions.first().cloned(),
                    confidence: 1.0,
                    rule_id: Some(diag.code),
                });
            }
        }

        // 2. Policy/boundary violations via DriftDetector (intent vs scanned reality)
        if let Some(intent) = &self.intent {
            let drifts = DriftDetector::evaluate_all_violations(intent, &self.graph);
            for drift in drifts {
                let involves_affected =
                    affected_set.iter().any(|id| drift.description.contains(id));

                if !involves_affected {
                    continue;
                }

                let severity = match drift.severity {
                    DriftSeverity::Critical => CritiqueSeverity::Critical,
                    DriftSeverity::High => CritiqueSeverity::High,
                    DriftSeverity::Medium => CritiqueSeverity::Medium,
                    DriftSeverity::Low | DriftSeverity::Info => CritiqueSeverity::Low,
                };

                let rule_id = crate::rule_ids::rule_id_for_drift_kind(drift.kind).to_string();
                findings.push(CritiqueFinding {
                    category: CritiqueCategory::PolicyViolation,
                    severity,
                    title: drift
                        .description
                        .split('.')
                        .next()
                        .unwrap_or(&drift.description)
                        .trim()
                        .to_string(),
                    detail: drift.description,
                    evidence: drift
                        .evidence
                        .iter()
                        .map(|e| CritiqueEvidence {
                            source: e.source.clone(),
                            location: e.location.clone(),
                            detail: e.detail.clone(),
                        })
                        .collect(),
                    suggestion: drift.suggestion.clone(),
                    confidence: 1.0,
                    rule_id: Some(rule_id),
                });
            }
        }

        findings
    }

    fn check_incident_patterns(
        &self,
        affected: &[String],
        description: &Option<String>,
    ) -> Vec<CritiqueFinding> {
        let mut findings = Vec::new();
        if let Some(program) = &self.program {
            for item in &program.items {
                if let sruja_language::ast::TopLevelItem::Incident(incident) = item {
                    let is_affected = incident
                        .affected
                        .iter()
                        .any(|qi| affected.contains(&qi.to_string()));
                    let mut matches_desc = false;

                    if let (Some(desc), Some(inc_cause)) = (description, &incident.cause) {
                        if desc.to_lowercase().contains(&inc_cause.to_lowercase()) {
                            matches_desc = true;
                        }
                    }

                    if is_affected || matches_desc {
                        findings.push(CritiqueFinding {
                            category: CritiqueCategory::HistoricalPatternMatch,
                            severity: CritiqueSeverity::High,
                            title: format!("Historical Match: {}", incident.title),
                            detail: format!("This change affects elements involved in a previous incident: {}. Cause: {}. Lesson: {}",
                                incident.title,
                                incident.cause.as_deref().unwrap_or("Unknown"),
                                incident.lesson.as_deref().unwrap_or("None")
                            ),
                            evidence: vec![CritiqueEvidence {
                                source: "sruja".to_string(),
                                location: None,
                                detail: format!("Incident ID: {}", incident.id),
                            }],
                            suggestion: incident.resolution.clone(),
                            confidence: 0.8,
                            rule_id: None,
                        });
                    }
                }
            }
        }
        findings
    }

    fn check_constraint_breaches(&self, affected: &[String]) -> Vec<CritiqueFinding> {
        let mut findings = Vec::new();
        for id in affected {
            if let Some(node) = self.graph.nodes.iter().find(|n| &n.id == id) {
                for constraint in &node.operational_constraints {
                    findings.push(CritiqueFinding {
                        category: CritiqueCategory::ConstraintBreach,
                        severity: CritiqueSeverity::Medium,
                        title: format!("Constraint: {}", id),
                        detail: format!(
                            "Element '{}' has a declared constraint: '{}'",
                            id, constraint
                        ),
                        evidence: vec![CritiqueEvidence {
                            source: "sruja".to_string(),
                            location: None,
                            detail: format!("Node: {}", id),
                        }],
                        suggestion: Some(
                            "Verify this change does not violate the constraint.".to_string(),
                        ),
                        confidence: 0.9,
                        rule_id: None,
                    });
                }
            }
        }
        findings
    }

    fn assess_blast_radius(&self, affected: &[String]) -> BlastRadiusSummary {
        let mut total_downstream = 0;
        let mut max_depth = 0;

        for id in affected {
            let radius = self.graph.blast_radius(id, 2);
            total_downstream += radius.downstream.len();
            for n in &radius.downstream {
                max_depth = max_depth.max(n.depth);
            }
        }

        BlastRadiusSummary {
            total_affected_elements: affected.len(),
            downstream_consumers: total_downstream,
            max_depth,
        }
    }

    fn surface_gotchas(&self, affected: &[String]) -> Vec<CritiqueFinding> {
        let mut findings = Vec::new();
        for id in affected {
            if let Some(node) = self.graph.nodes.iter().find(|n| &n.id == id) {
                for gotcha in &node.gotchas {
                    findings.push(CritiqueFinding {
                        category: CritiqueCategory::GotchaWarning,
                        severity: CritiqueSeverity::Medium,
                        title: format!("Gotcha: {}", id),
                        detail: gotcha.clone(),
                        evidence: vec![],
                        suggestion: None,
                        confidence: 1.0,
                        rule_id: None,
                    });
                }
            }
        }
        findings
    }

    /// Retained for potential future accuracy-gated revival.
    /// Not called by the default critique pipeline in v1.
    #[allow(dead_code)]
    fn check_unproposed_changes(&self, affected: &[String]) -> Vec<CritiqueFinding> {
        let mut findings = Vec::new();

        for id in affected {
            if let Some(node) = self.graph.nodes.iter().find(|n| &n.id == id) {
                let radius = self.graph.blast_radius(id, 2);
                let affected_count = radius.upstream.len();

                let severity = match (node.criticality, affected_count) {
                    (Some(Criticality::Critical), _) => CritiqueSeverity::Critical,
                    (Some(Criticality::High), _) | (_, 5..) => CritiqueSeverity::High,
                    (_, 2..=4) => CritiqueSeverity::Medium,
                    _ => CritiqueSeverity::Low,
                };

                if severity >= CritiqueSeverity::Medium || affected.len() == 1 {
                    findings.push(CritiqueFinding {
                        category: CritiqueCategory::UnproposedChange,
                        severity,
                        title: format!("Unproposed Change: {}", id),
                        detail: format!(
                            "Change affects '{}' (criticality: {:?}, {} downstream consumers) without a linked architectural proposal.",
                            id,
                            node.criticality.as_ref().map(|c| format!("{:?}", c)).unwrap_or_else(|| "Unknown".to_string()),
                            affected_count
                        ),
                        evidence: vec![CritiqueEvidence {
                            source: "sruja".to_string(),
                            location: node.path.clone(),
                            detail: format!("Blast radius: {} downstream consumers", affected_count),
                        }],
                        suggestion: Some("Run 'sruja propose' to document architectural intent and get formal review.".to_string()),
                        confidence: 0.9,
                        rule_id: None,
                    });
                }
            }
        }

        if findings.is_empty() && !affected.is_empty() {
            findings.push(CritiqueFinding {
                category: CritiqueCategory::UnproposedChange,
                severity: CritiqueSeverity::Low,
                title: "Unproposed Architectural Changes".to_string(),
                detail: format!(
                    "This change affects {} architectural elements without a linked proposal.",
                    affected.len()
                ),
                evidence: vec![],
                suggestion: Some(
                    "Consider running 'sruja propose' to document architectural intent."
                        .to_string(),
                ),
                confidence: 0.7,
                rule_id: None,
            });
        }

        findings
    }

    fn compute_risk_level(&self, violations: &[CritiqueFinding]) -> RiskLevel {
        let mut max_severity = CritiqueSeverity::Low;
        for f in violations {
            if f.severity > max_severity {
                max_severity = f.severity;
            }
        }

        match max_severity {
            CritiqueSeverity::Critical => RiskLevel::Danger,
            CritiqueSeverity::High => RiskLevel::Warning,
            CritiqueSeverity::Medium => RiskLevel::Caution,
            CritiqueSeverity::Low => {
                if violations.is_empty() {
                    RiskLevel::Clear
                } else {
                    RiskLevel::Caution
                }
            }
        }
    }

    fn generate_summary(
        &self,
        violations: &[CritiqueFinding],
        context: &[CritiqueFinding],
        affected: &[String],
        blast: &BlastRadiusSummary,
        baseline_present: bool,
    ) -> String {
        if !baseline_present {
            return format!(
                "No baseline loaded; 0 violations across {} affected elements.",
                affected.len()
            );
        }

        format!(
            "{} violations, {} context items across {} affected elements. Blast radius includes {} downstream consumers.",
            violations.len(),
            context.len(),
            affected.len(),
            blast.downstream_consumers
        )
    }
}

/// Check if `file` resides within the directory `dir_path` (any depth).
fn is_within_dir(file: &str, dir_path: &str) -> bool {
    file.starts_with(&format!("{}/", dir_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::{Node, NodeKind};

    #[test]
    fn report_with_violations_and_context_serializes_correctly() {
        let report = CritiqueReport {
            violations: vec![CritiqueFinding {
                category: CritiqueCategory::PolicyViolation,
                severity: CritiqueSeverity::High,
                title: "Layer violation".to_string(),
                detail: "A depends on B across layers".to_string(),
                evidence: vec![],
                suggestion: None,
                confidence: 1.0,
                rule_id: Some("SRUJA-LAYER-001".to_string()),
            }],
            context: vec![],
            risk_level: RiskLevel::Warning,
            summary: "1 violations, 0 context items".to_string(),
            affected_elements: vec!["A".to_string()],
            blast_radius: BlastRadiusSummary {
                total_affected_elements: 1,
                downstream_consumers: 0,
                max_depth: 0,
            },
            baseline_present: true,
        };

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"violations\""));
        assert!(json.contains("\"context\""));
        assert!(json.contains("\"baseline_present\":true"));
        assert!(json.contains("\"rule_id\":\"SRUJA-LAYER-001\""));
    }

    #[test]
    fn report_with_baseline_present_false() {
        let report = CritiqueReport {
            violations: vec![],
            context: vec![],
            risk_level: RiskLevel::Clear,
            summary: "No baseline loaded; 0 violations".to_string(),
            affected_elements: vec![],
            blast_radius: BlastRadiusSummary {
                total_affected_elements: 0,
                downstream_consumers: 0,
                max_depth: 0,
            },
            baseline_present: false,
        };

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"baseline_present\":false"));
    }

    #[test]
    fn resolve_affected_elements_exact_path_match() {
        let mut graph = Graph::default();
        graph.nodes.push(Node {
            id: "TestNode".to_string(),
            kind: NodeKind::new(NodeKind::COMPONENT),
            label: "Test Node".to_string(),
            path: Some("src/test.rs".to_string()),
            ..Default::default()
        });

        let engine = CritiqueEngine::new(graph, None);
        let affected = engine.resolve_affected_elements(&["src/test.rs".to_string()]);
        assert_eq!(affected.len(), 1);
        assert!(affected.contains(&"TestNode".to_string()));
    }

    #[test]
    fn resolve_affected_elements_no_substring_false_positive() {
        let mut graph = Graph::default();
        graph.nodes.push(Node {
            id: "DB".to_string(),
            kind: NodeKind::new(NodeKind::DATABASE),
            label: "Database".to_string(),
            path: Some("src/db".to_string()),
            ..Default::default()
        });

        let engine = CritiqueEngine::new(graph, None);
        // "src/db/migrations_v2.rs" should NOT match "src/db" as a substring
        // (it IS a direct child though, so it WILL match via directory prefix)
        // But "src/dashboard.rs" should NOT match
        let affected = engine.resolve_affected_elements(&["src/dashboard.rs".to_string()]);
        assert!(
            affected.is_empty(),
            "dashboard.rs should not match node path src/db"
        );
    }

    #[test]
    fn resolve_affected_elements_empty_files() {
        let graph = Graph::default();
        let engine = CritiqueEngine::new(graph, None);
        let affected = engine.resolve_affected_elements(&[]);
        assert!(affected.is_empty());
    }

    #[test]
    fn risk_from_violations_only() {
        let engine = CritiqueEngine::new(Graph::default(), None);

        let violations = vec![CritiqueFinding {
            category: CritiqueCategory::PolicyViolation,
            severity: CritiqueSeverity::High,
            title: "test".to_string(),
            detail: "test".to_string(),
            evidence: vec![],
            suggestion: None,
            confidence: 1.0,
            rule_id: None,
        }];

        assert_eq!(engine.compute_risk_level(&violations), RiskLevel::Warning);
        assert_eq!(engine.compute_risk_level(&[]), RiskLevel::Clear);
    }

    #[test]
    fn summary_distinguishes_counts() {
        let engine = CritiqueEngine::new(Graph::default(), None);
        let blast = BlastRadiusSummary {
            total_affected_elements: 1,
            downstream_consumers: 3,
            max_depth: 1,
        };
        let summary = engine.generate_summary(
            &vec![CritiqueFinding {
                category: CritiqueCategory::PolicyViolation,
                severity: CritiqueSeverity::Medium,
                title: "v".to_string(),
                detail: "v".to_string(),
                evidence: vec![],
                suggestion: None,
                confidence: 1.0,
                rule_id: None,
            }],
            &[],
            &["A".to_string()],
            &blast,
            true,
        );
        assert!(summary.contains("1 violations"));
        assert!(summary.contains("0 context items"));
    }

    #[test]
    fn summary_no_baseline() {
        let engine = CritiqueEngine::new(Graph::default(), None);
        let blast = BlastRadiusSummary {
            total_affected_elements: 0,
            downstream_consumers: 0,
            max_depth: 0,
        };
        let summary = engine.generate_summary(&[], &[], &[], &blast, false);
        assert!(summary.contains("No baseline loaded"));
        assert!(summary.contains("0 violations"));
    }
}
