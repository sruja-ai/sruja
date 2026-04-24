//! Adversarial architectural critique engine

use serde::{Deserialize, Serialize};
use sruja_scan::{Graph, Criticality};
use sruja_language::Program;

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
    pub findings: Vec<CritiqueFinding>,
    pub risk_level: RiskLevel,
    pub summary: String,
    pub affected_elements: Vec<String>,
    pub blast_radius: BlastRadiusSummary,
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
}

impl CritiqueEngine {
    pub fn new(graph: Graph, program: Option<Program>) -> Self {
        Self { graph, program }
    }

    pub fn critique(&self, request: &CritiqueRequest) -> CritiqueReport {
        let affected = self.resolve_affected_elements(&request.changed_files);
        let mut findings = Vec::new();

        // Pass 1: Policy violations
        findings.extend(self.check_policy_violations(&affected));

        // Pass 2: Historical incident pattern matching
        findings.extend(self.check_incident_patterns(&affected, &request.description));

        // Pass 3: Constraint breaches
        findings.extend(self.check_constraint_breaches(&affected, &request.changed_files));

        // Pass 4: Blast radius assessment
        let (blast_summary, blast_findings) = self.assess_blast_radius(&affected);
        findings.extend(blast_findings);

        // Pass 5: Behavioral contract drift (Heuristic)
        if let Some(program) = &self.program {
            findings.extend(crate::behavioral_drift::check_behavioral_drift(
                &self.graph,
                program,
                &request.changed_files,
                &affected,
            ));
        }

        // Pass 6: Gotcha surfacing
        findings.extend(self.surface_gotchas(&affected));

        // Pass 7: Unproposed change detection
        if request.proposal_id.is_none() {
            findings.extend(self.check_unproposed_changes(&affected));
        }

        // Sort findings by severity
        findings.sort_by(|a, b| b.severity.cmp(&a.severity));

        let risk_level = self.compute_risk_level(&findings);
        let summary = self.generate_summary(&findings, &affected, &blast_summary);

        CritiqueReport {
            findings,
            risk_level,
            summary,
            affected_elements: affected,
            blast_radius: blast_summary,
        }
    }

    pub fn resolve_affected_elements(&self, changed_files: &[String]) -> Vec<String> {
        let mut affected = std::collections::HashSet::new();
        for file in changed_files {
            for node in &self.graph.nodes {
                if let Some(path) = &node.path {
                    // Simple heuristic: if path is contained in file path or vice versa
                    if file.contains(path) || path.contains(file) {
                        affected.insert(node.id.clone());
                    }
                }
            }
        }
        affected.into_iter().collect()
    }

    fn check_policy_violations(&self, _affected: &[String]) -> Vec<CritiqueFinding> {
        Vec::new()
    }

    fn check_incident_patterns(&self, affected: &[String], description: &Option<String>) -> Vec<CritiqueFinding> {
        let mut findings = Vec::new();
        if let Some(program) = &self.program {
            for item in &program.items {
                if let sruja_language::ast::TopLevelItem::Incident(incident) = item {
                    let is_affected = incident.affected.iter().any(|qi| affected.contains(&qi.to_string()));
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
                        });
                    }
                }
            }
        }
        findings
    }

    fn check_constraint_breaches(&self, affected: &[String], _files: &[String]) -> Vec<CritiqueFinding> {
        let mut findings = Vec::new();
        for id in affected {
            if let Some(node) = self.graph.nodes.iter().find(|n| &n.id == id) {
                for constraint in &node.operational_constraints {
                    findings.push(CritiqueFinding {
                        category: CritiqueCategory::ConstraintBreach,
                        severity: CritiqueSeverity::Medium,
                        title: format!("Constraint: {}", id),
                        detail: format!("Element '{}' has a declared constraint: '{}'", id, constraint),
                        evidence: vec![CritiqueEvidence {
                            source: "sruja".to_string(),
                            location: None,
                            detail: format!("Node: {}", id),
                        }],
                        suggestion: Some("Verify this change does not violate the constraint.".to_string()),
                        confidence: 0.9,
                    });
                }
            }
        }
        findings
    }

    fn assess_blast_radius(&self, affected: &[String]) -> (BlastRadiusSummary, Vec<CritiqueFinding>) {
        let mut total_downstream = 0;
        let mut max_depth = 0;
        let mut findings = Vec::new();

        for id in affected {
            let radius = self.graph.blast_radius(id, 2);
            total_downstream += radius.downstream.len();
            for n in &radius.downstream {
                max_depth = max_depth.max(n.depth);
            }
        }

        if total_downstream > 5 {
            findings.push(CritiqueFinding {
                category: CritiqueCategory::BlastRadius,
                severity: CritiqueSeverity::Medium,
                title: "Large Blast Radius".to_string(),
                detail: format!("This change affects {} downstream consumers. Consider the impact on dependent systems.", total_downstream),
                evidence: vec![],
                suggestion: Some("Consider running integration tests for downstream consumers.".to_string()),
                confidence: 1.0,
            });
        }

        (BlastRadiusSummary {
            total_affected_elements: affected.len(),
            downstream_consumers: total_downstream,
            max_depth,
        }, findings)
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
                    });
                }
            }
        }
        findings
    }

    fn check_unproposed_changes(&self, affected: &[String]) -> Vec<CritiqueFinding> {
        let mut findings = Vec::new();
        
        for id in affected {
            if let Some(node) = self.graph.nodes.iter().find(|n| &n.id == id) {
                let radius = self.graph.blast_radius(id, 2);
                let affected_count = radius.upstream.len();
                
                // Determine severity based on criticality and blast radius (upstream consumers)
                let severity = match (node.criticality, affected_count) {
                    (Some(Criticality::Critical), _) => CritiqueSeverity::Critical,
                    (Some(Criticality::High), _) | (_, 5..) => CritiqueSeverity::High,
                    (_, 2..=4) => CritiqueSeverity::Medium,
                    _ => CritiqueSeverity::Low,
                };
                
                // Only report Medium and above individually, or if it's the only one
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
                    });
                }
            }
        }
        
        // If we didn't add any specific findings but have affected elements, add a summary one
        if findings.is_empty() && !affected.is_empty() {
            findings.push(CritiqueFinding {
                category: CritiqueCategory::UnproposedChange,
                severity: CritiqueSeverity::Low,
                title: "Unproposed Architectural Changes".to_string(),
                detail: format!("This change affects {} architectural elements without a linked proposal.", affected.len()),
                evidence: vec![],
                suggestion: Some("Consider running 'sruja propose' to document architectural intent.".to_string()),
                confidence: 0.7,
            });
        }
        
        findings
    }

    fn compute_risk_level(&self, findings: &[CritiqueFinding]) -> RiskLevel {
        let mut max_severity = CritiqueSeverity::Low;
        for f in findings {
            if f.severity > max_severity {
                max_severity = f.severity;
            }
        }

        match max_severity {
            CritiqueSeverity::Critical => RiskLevel::Danger,
            CritiqueSeverity::High => RiskLevel::Warning,
            CritiqueSeverity::Medium => RiskLevel::Caution,
            CritiqueSeverity::Low => {
                if findings.is_empty() {
                    RiskLevel::Clear
                } else {
                    RiskLevel::Caution
                }
            }
        }
    }

    fn generate_summary(&self, findings: &[CritiqueFinding], affected: &[String], blast: &BlastRadiusSummary) -> String {
        format!("Critique found {} issues across {} affected elements. Blast radius includes {} downstream consumers.", 
            findings.len(), 
            affected.len(), 
            blast.downstream_consumers
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::{Node, NodeKind};

    #[test]
    fn test_critique_unproposed_change() {
        let mut graph = Graph::default();
        graph.nodes.push(Node {
            id: "TestNode".to_string(),
            kind: NodeKind::Component,
            label: "Test Node".to_string(),
            path: Some("src/test.rs".to_string()),
            ..Default::default()
        });

        let engine = CritiqueEngine::new(graph, None);
        let report = engine.critique(&CritiqueRequest {
            changed_files: vec!["src/test.rs".to_string()],
            description: None,
            proposal_id: None,
            base_ref: None,
            head_ref: None,
        });

        assert_eq!(report.affected_elements.len(), 1);
        assert!(report.findings.iter().any(|f| f.category == CritiqueCategory::UnproposedChange));
        assert_eq!(report.risk_level, RiskLevel::Caution);
    }

    #[test]
    fn test_critique_unproposed_change_escalation() {
        let mut graph = Graph::default();
        graph.nodes.push(Node {
            id: "CriticalService".to_string(),
            kind: NodeKind::Service,
            label: "Critical Service".to_string(),
            path: Some("src/critical.rs".to_string()),
            criticality: Some(Criticality::Critical),
            ..Default::default()
        });

        let engine = CritiqueEngine::new(graph, None);
        let report = engine.critique(&CritiqueRequest {
            changed_files: vec!["src/critical.rs".to_string()],
            description: None,
            proposal_id: None,
            base_ref: None,
            head_ref: None,
        });

        assert_eq!(report.affected_elements.len(), 1);
        let finding = report.findings.iter().find(|f| f.category == CritiqueCategory::UnproposedChange).expect("should have unproposed change finding");
        assert_eq!(finding.severity, CritiqueSeverity::Critical);
        assert_eq!(report.risk_level, RiskLevel::Danger);
    }

    #[test]
    fn test_critique_unproposed_change_blast_radius_escalation() {
        let mut graph = Graph::default();
        graph.nodes.push(Node {
            id: "CoreLib".to_string(),
            kind: NodeKind::Component,
            label: "Core Library".to_string(),
            path: Some("src/core.rs".to_string()),
            ..Default::default()
        });
        
        // Add 6 downstream consumers to trigger High severity
        for i in 0..6 {
            let consumer_id = format!("Consumer{}", i);
            graph.nodes.push(Node {
                id: consumer_id.clone(),
                kind: NodeKind::Component,
                label: consumer_id.clone(),
                ..Default::default()
            });
            graph.edges.push(sruja_scan::Edge {
                source: consumer_id,
                target: "CoreLib".to_string(),
                kind: sruja_scan::EdgeKind::Calls,
                evidence: vec![],
            });
        }

        let engine = CritiqueEngine::new(graph, None);
        let report = engine.critique(&CritiqueRequest {
            changed_files: vec!["src/core.rs".to_string()],
            description: None,
            proposal_id: None,
            base_ref: None,
            head_ref: None,
        });

        assert_eq!(report.affected_elements.len(), 1);
        let finding = report.findings.iter().find(|f| f.category == CritiqueCategory::UnproposedChange).expect("should have unproposed change finding");
        assert_eq!(finding.severity, CritiqueSeverity::High);
        assert_eq!(report.risk_level, RiskLevel::Warning);
    }
}
