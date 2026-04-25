//! Query interface for Knowledge Graph
//!
//! Evidence is produced deterministically from graph data (no LLM). Templates
//! format nodes, edges, and decisions for consistent CLI output.

use crate::*;
use thiserror::Error;

/// Deterministic evidence excerpt for a decision (ADR or similar).
fn format_decision_evidence(d: &Decision) -> String {
    let snippet = d.decision.trim();
    let max_len = 200;
    if snippet.len() <= max_len {
        format!("[{}] {}", d.title, snippet)
    } else {
        format!("[{}] {}...", d.title, &snippet[..max_len])
    }
}

/// Deterministic evidence excerpt for a node (component).
fn format_node_evidence(node: &ArchitectureNode, tech: Option<&str>) -> String {
    let kind = format!("{}", node.kind);
    let tech_str = tech.or(node.technology.as_deref()).unwrap_or("(not set)");
    format!(
        "Component '{}' (kind={}, technology={})",
        node.label, kind, tech_str
    )
}

/// Deterministic evidence excerpt for an edge (relationship).
fn format_edge_evidence(
    src_label: &str,
    kind: &EdgeKind,
    tgt_label: &str,
    label: Option<&str>,
) -> String {
    let kind_str = format!("{}", kind);
    match label {
        Some(l) if !l.is_empty() => {
            format!("{} --[{}] {}--> {}", src_label, l, kind_str, tgt_label)
        }
        _ => format!("{} --{}--> {}", src_label, kind_str, tgt_label),
    }
}

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("No results found")]
    NoResults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub question: String,
    pub answer: String,
    pub evidence: Vec<Evidence>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: EvidenceKind,
    pub reference: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceKind {
    Decision,
    Policy,
    Requirement,
    Node,
    Edge,
}

impl KnowledgeGraph {
    pub fn query(&self, question: &str) -> Result<QueryResult, QueryError> {
        let question_lower = question.to_lowercase();

        let entity = self.resolve_entity(question);

        if question_lower.contains("why") {
            if let Some(node_id) = &entity {
                return self.query_why_entity(node_id, question);
            }
            let tech_patterns = self.extract_tech_patterns(question);
            if !tech_patterns.is_empty() {
                return self.query_why_tech(&tech_patterns, question);
            }
        } else if question_lower.contains("what") || question_lower.contains("which") {
            if let Some(node_id) = &entity {
                return self.query_what_entity(node_id, question);
            }
            return self.query_what(question);
        } else if question_lower.contains("how") {
            if let Some(node_id) = &entity {
                return self.query_connections(node_id, question);
            }
            return self.query_how(question);
        } else if question_lower.contains("decision") || question_lower.contains("adr") {
            return self.query_decisions(question);
        } else if let Some(node_id) = &entity {
            return self.query_describe(node_id, question);
        }

        self.query_generic(question)
    }

    fn resolve_entity(&self, question: &str) -> Option<String> {
        let q = question.to_lowercase();
        let words: Vec<&str> = q
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-' && c != '.')
            .filter(|w| w.len() >= 3)
            .collect();

        for (id, node) in &self.nodes {
            let label_lower = node.label.to_lowercase();
            let id_lower = id.to_lowercase();
            let id_short: Vec<&str> = id.split('.').collect();
            let short = id_short.last().unwrap_or(&id.as_str()).to_lowercase();

            if q.contains(&id_lower) || q.contains(&label_lower) {
                return Some(id.clone());
            }

            for word in &words {
                if label_lower == *word || short == *word {
                    return Some(id.clone());
                }
                if label_lower.contains(word) && word.len() >= 4 {
                    return Some(id.clone());
                }
            }
        }

        None
    }

    fn query_why_entity(&self, node_id: &str, question: &str) -> Result<QueryResult, QueryError> {
        let node = self.nodes.get(node_id).ok_or(QueryError::NoResults)?;

        let upstream: Vec<&ArchitectureEdge> =
            self.edges.iter().filter(|e| e.target == node_id).collect();
        let downstream: Vec<&ArchitectureEdge> =
            self.edges.iter().filter(|e| e.source == node_id).collect();

        let mut answer_parts = Vec::new();
        let mut evidence = Vec::new();

        answer_parts.push(format!(
            "'{}' (kind={}, technology={})",
            node.label,
            node.kind,
            node.technology.as_deref().unwrap_or("not set")
        ));

        if !upstream.is_empty() {
            let sources: Vec<String> = upstream
                .iter()
                .filter_map(|e| self.nodes.get(&e.source).map(|n| n.label.clone()))
                .collect();
            answer_parts.push(format!("is depended on by: {}", sources.join(", ")));
            for e in upstream.iter().take(5) {
                if let Some(src) = self.nodes.get(&e.source) {
                    evidence.push(Evidence {
                        kind: EvidenceKind::Edge,
                        reference: e.source_ref.summary(),
                        excerpt: format_edge_evidence(
                            &src.label,
                            &e.kind,
                            &node.label,
                            e.label.as_deref(),
                        ),
                    });
                }
            }
        }

        if !downstream.is_empty() {
            let targets: Vec<String> = downstream
                .iter()
                .filter_map(|e| self.nodes.get(&e.target).map(|n| n.label.clone()))
                .collect();
            answer_parts.push(format!("depends on: {}", targets.join(", ")));
            for e in downstream.iter().take(5) {
                if let Some(tgt) = self.nodes.get(&e.target) {
                    evidence.push(Evidence {
                        kind: EvidenceKind::Edge,
                        reference: e.source_ref.summary(),
                        excerpt: format_edge_evidence(
                            &node.label,
                            &e.kind,
                            &tgt.label,
                            e.label.as_deref(),
                        ),
                    });
                }
            }
        }

        let decisions = self.get_decisions_for_node(node_id);
        if !decisions.is_empty() {
            answer_parts.push(format!("has {} linked decision(s)", decisions.len()));
            for d in decisions.iter().take(3) {
                evidence.push(Evidence {
                    kind: EvidenceKind::Decision,
                    reference: d.source.summary(),
                    excerpt: format_decision_evidence(d),
                });
            }
        }

        if upstream.is_empty() && downstream.is_empty() {
            answer_parts.push("has no connections (orphan)".to_string());
        }

        let confidence = if !evidence.is_empty() { 0.8 } else { 0.4 };

        Ok(QueryResult {
            question: question.to_string(),
            answer: answer_parts.join(". "),
            evidence,
            confidence,
        })
    }

    fn query_why_tech(
        &self,
        tech_patterns: &[String],
        question: &str,
    ) -> Result<QueryResult, QueryError> {
        for tech in tech_patterns {
            let nodes = self.find_nodes_by_technology(tech);
            if !nodes.is_empty() {
                let node = &nodes[0];
                let decisions = self.get_decisions_for_node(&node.id);

                if !decisions.is_empty() {
                    let decision = &decisions[0];
                    let excerpt = format_decision_evidence(decision);
                    return Ok(QueryResult {
                        question: question.to_string(),
                        answer: format!("Based on {}: {}", decision.title, decision.decision),
                        evidence: vec![Evidence {
                            kind: EvidenceKind::Decision,
                            reference: decision.source.summary(),
                            excerpt,
                        }],
                        confidence: 0.85,
                    });
                }

                let excerpt = format_node_evidence(node, Some(tech.as_str()));
                return Ok(QueryResult {
                    question: question.to_string(),
                    answer: format!("{} uses {} technology.", node.label, tech),
                    evidence: vec![Evidence {
                        kind: EvidenceKind::Node,
                        reference: node.source.summary(),
                        excerpt,
                    }],
                    confidence: 0.5,
                });
            }
        }

        self.query_generic(question)
    }

    fn query_what_entity(&self, node_id: &str, question: &str) -> Result<QueryResult, QueryError> {
        let node = self.nodes.get(node_id).ok_or(QueryError::NoResults)?;

        let mut answer = format!(
            "'{}' is a {} (technology: {})",
            node.label,
            node.kind,
            node.technology.as_deref().unwrap_or("not set")
        );

        if let Some(desc) = &node.description {
            if !desc.is_empty() {
                answer = format!("{}. {}", answer, desc);
            }
        }

        let evidence = vec![Evidence {
            kind: EvidenceKind::Node,
            reference: node.source.summary(),
            excerpt: format_node_evidence(node, None),
        }];

        Ok(QueryResult {
            question: question.to_string(),
            answer,
            evidence,
            confidence: 0.8,
        })
    }

    fn query_connections(&self, node_id: &str, question: &str) -> Result<QueryResult, QueryError> {
        let node = self.nodes.get(node_id).ok_or(QueryError::NoResults)?;

        let downstream: Vec<&ArchitectureEdge> =
            self.edges.iter().filter(|e| e.source == node_id).collect();
        let upstream: Vec<&ArchitectureEdge> =
            self.edges.iter().filter(|e| e.target == node_id).collect();

        let mut parts = Vec::new();
        let mut evidence = Vec::new();

        if !downstream.is_empty() {
            let targets: Vec<String> = downstream
                .iter()
                .filter_map(|e| {
                    self.nodes.get(&e.target).map(|n| {
                        format!("{} ({})", n.label, e.label.as_deref().unwrap_or_default())
                    })
                })
                .collect();
            parts.push(format!(
                "{} connects to: {}",
                node.label,
                targets.join(", ")
            ));
        }

        if !upstream.is_empty() {
            let sources: Vec<String> = upstream
                .iter()
                .filter_map(|e| {
                    self.nodes.get(&e.source).map(|n| {
                        format!("{} ({})", n.label, e.label.as_deref().unwrap_or_default())
                    })
                })
                .collect();
            parts.push(format!("{} is used by: {}", node.label, sources.join(", ")));
        }

        for e in downstream.iter().take(5) {
            if let Some(tgt) = self.nodes.get(&e.target) {
                evidence.push(Evidence {
                    kind: EvidenceKind::Edge,
                    reference: e.source_ref.summary(),
                    excerpt: format_edge_evidence(
                        &node.label,
                        &e.kind,
                        &tgt.label,
                        e.label.as_deref(),
                    ),
                });
            }
        }

        if parts.is_empty() {
            parts.push(format!("{} has no connections", node.label));
        }

        let conf = if evidence.is_empty() { 0.4 } else { 0.75 };
        Ok(QueryResult {
            question: question.to_string(),
            answer: parts.join(". "),
            evidence,
            confidence: conf,
        })
    }

    fn query_describe(&self, node_id: &str, question: &str) -> Result<QueryResult, QueryError> {
        let node = self.nodes.get(node_id).ok_or(QueryError::NoResults)?;

        let downstream_count = self.edges.iter().filter(|e| e.source == node_id).count();
        let upstream_count = self.edges.iter().filter(|e| e.target == node_id).count();

        let mut answer = format!("'{}' is a {}", node.label, node.kind);
        if let Some(tech) = &node.technology {
            answer = format!("{} using {}", answer, tech);
        }
        answer = format!(
            "{}. {} outgoing and {} incoming connections.",
            answer, downstream_count, upstream_count
        );
        if let Some(desc) = &node.description {
            if !desc.is_empty() {
                answer = format!("{} {}", answer, desc);
            }
        }

        Ok(QueryResult {
            question: question.to_string(),
            answer,
            evidence: vec![Evidence {
                kind: EvidenceKind::Node,
                reference: node.source.summary(),
                excerpt: format_node_evidence(node, None),
            }],
            confidence: 0.7,
        })
    }

    fn query_what(&self, question: &str) -> Result<QueryResult, QueryError> {
        let kind_patterns = [
            ("service", NodeKind::Service),
            ("database", NodeKind::Database),
            ("queue", NodeKind::Queue),
            ("frontend", NodeKind::Frontend),
            ("api", NodeKind::ExternalApi),
            ("system", NodeKind::System),
        ];

        let question_lower = question.to_lowercase();

        for (pattern, kind) in &kind_patterns {
            if question_lower.contains(pattern) {
                let nodes = self.find_nodes_by_kind(kind.clone());
                if !nodes.is_empty() {
                    let labels: Vec<&str> = nodes.iter().map(|n| n.label.as_str()).collect();
                    return Ok(QueryResult {
                        question: question.to_string(),
                        answer: format!(
                            "Found {} {}(s): {}",
                            nodes.len(),
                            pattern,
                            labels.join(", ")
                        ),
                        evidence: nodes
                            .iter()
                            .take(5)
                            .map(|n| Evidence {
                                kind: EvidenceKind::Node,
                                reference: n.source.summary(),
                                excerpt: format_node_evidence(n, None),
                            })
                            .collect(),
                        confidence: 0.9,
                    });
                }
            }
        }

        self.query_generic(question)
    }

    fn query_how(&self, question: &str) -> Result<QueryResult, QueryError> {
        let question_lower = question.to_lowercase();

        if question_lower.contains("depend") || question_lower.contains("connect") {
            let mut connections: Vec<String> = Vec::new();

            for edge in &self.edges {
                if let (Some(src), Some(tgt)) =
                    (self.nodes.get(&edge.source), self.nodes.get(&edge.target))
                {
                    connections.push(format!("{} {} {}", src.label, edge.kind, tgt.label));
                }
            }

            if !connections.is_empty() {
                let evidence: Vec<Evidence> = self
                    .edges
                    .iter()
                    .take(10)
                    .filter_map(|e| {
                        let src = self.nodes.get(&e.source)?;
                        let tgt = self.nodes.get(&e.target)?;
                        Some(Evidence {
                            kind: EvidenceKind::Edge,
                            reference: e.source_ref.summary(),
                            excerpt: format_edge_evidence(
                                src.label.as_str(),
                                &e.kind,
                                tgt.label.as_str(),
                                e.label.as_deref(),
                            ),
                        })
                    })
                    .collect();
                if !evidence.is_empty() {
                    return Ok(QueryResult {
                        question: question.to_string(),
                        answer: format!("Found {} connections", connections.len()),
                        evidence,
                        confidence: 0.7,
                    });
                }
            }
        }

        self.query_generic(question)
    }

    fn query_decisions(&self, _question: &str) -> Result<QueryResult, QueryError> {
        let decisions: Vec<&Decision> = self.decisions.values().collect();

        if decisions.is_empty() {
            return Ok(QueryResult {
                question: _question.to_string(),
                answer: "No architecture decisions recorded yet.".to_string(),
                evidence: vec![],
                confidence: 1.0,
            });
        }

        let accepted_count = decisions
            .iter()
            .filter(|d| d.status == DecisionStatus::Accepted)
            .count();

        Ok(QueryResult {
            question: _question.to_string(),
            answer: format!(
                "Found {} decisions ({} accepted, {} proposed)",
                decisions.len(),
                accepted_count,
                decisions.len() - accepted_count
            ),
            evidence: decisions
                .iter()
                .take(5)
                .map(|d| Evidence {
                    kind: EvidenceKind::Decision,
                    reference: d.source.summary(),
                    excerpt: format_decision_evidence(d),
                })
                .collect(),
            confidence: 0.9,
        })
    }

    fn query_generic(&self, question: &str) -> Result<QueryResult, QueryError> {
        let stats = self.stats();

        Ok(QueryResult {
            question: question.to_string(),
            answer: format!(
                "The architecture has {} components, {} decisions, and {} policies. Try asking about specific services, technologies, or decisions.",
                stats.total_nodes,
                stats.total_decisions,
                stats.total_policies
            ),
            evidence: vec![],
            confidence: 0.3,
        })
    }

    fn extract_tech_patterns(&self, question: &str) -> Vec<String> {
        let techs = [
            "kafka",
            "rabbitmq",
            "redis",
            "postgres",
            "postgresql",
            "mysql",
            "mongodb",
            "elasticsearch",
            "nginx",
            "kubernetes",
            "docker",
            "react",
            "vue",
            "angular",
            "node",
            "python",
            "go",
            "rust",
            "java",
            "graphql",
            "rest",
            "grpc",
            "aws",
            "gcp",
            "azure",
        ];

        let question_lower = question.to_lowercase();
        techs
            .iter()
            .filter(|t| question_lower.contains(*t))
            .map(|t| t.to_string())
            .collect()
    }

    pub fn find_policy_violations(&self) -> Vec<PolicyViolation> {
        let mut violations = Vec::new();

        for policy in self.policies.values() {
            for rule in &policy.rules {
                for edge in &self.edges {
                    if let (Some(src), Some(tgt)) =
                        (self.nodes.get(&edge.source), self.nodes.get(&edge.target))
                    {
                        let source_matches = rule
                            .constraint
                            .source_kind
                            .as_ref()
                            .map(|k| src.kind == *k)
                            .unwrap_or(true);
                        let target_matches = rule
                            .constraint
                            .target_kind
                            .as_ref()
                            .map(|k| tgt.kind == *k)
                            .unwrap_or(true);

                        if source_matches && target_matches && !rule.constraint.allowed {
                            violations.push(PolicyViolation {
                                policy_id: policy.id.clone(),
                                policy_name: policy.name.clone(),
                                edge_id: edge.id.clone(),
                                source: edge.source.clone(),
                                target: edge.target.clone(),
                                message: rule.constraint.message.clone(),
                                severity: policy.severity,
                            });
                        }
                    }
                }
            }
        }

        violations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub policy_id: String,
    pub policy_name: String,
    pub edge_id: String,
    pub source: String,
    pub target: String,
    pub message: String,
    pub severity: PolicySeverity,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> KnowledgeGraph {
        let mut graph = KnowledgeGraph::new();

        graph.add_node(ArchitectureNode {
            id: "api".to_string(),
            kind: NodeKind::Service,
            label: "API Service".to_string(),
            technology: Some("Node.js".to_string()),
            ..ArchitectureNode::default()
        }).unwrap();

        graph.add_node(ArchitectureNode {
            id: "db".to_string(),
            kind: NodeKind::Database,
            label: "PostgreSQL".to_string(),
            technology: Some("PostgreSQL".to_string()),
            ..ArchitectureNode::default()
        }).unwrap();

        graph
    }

    #[test]
    fn test_query_what_services() {
        let graph = create_test_graph();
        let result = graph.query("what services do we have?").unwrap();
        assert!(result.answer.contains("service"));
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_query_why_technology() {
        let graph = create_test_graph();
        let result = graph.query("why are we using Node.js?").unwrap();
        assert!(result.answer.to_lowercase().contains("node") || result.confidence > 0.0);
    }

    #[test]
    fn test_query_what_databases() {
        let graph = create_test_graph();
        let result = graph.query("what databases do we have?").unwrap();
        assert!(result.answer.to_lowercase().contains("database"));
    }

    #[test]
    fn test_query_how_connected() {
        let graph = create_test_graph();
        let result = graph.query("how are components connected?").unwrap();
        assert!(!result.answer.is_empty());
    }

    #[test]
    fn test_query_decisions() {
        let graph = create_test_graph();
        let result = graph.query("show me all decisions").unwrap();
        assert!(result.answer.contains("decisions"));
    }

    #[test]
    fn test_query_generic() {
        let graph = create_test_graph();
        let result = graph.query("tell me about the architecture").unwrap();
        assert!(result.answer.contains("components"));
    }

    #[test]
    fn test_format_decision_evidence_short() {
        let decision = Decision {
            id: "adr-001".to_string(),
            title: "Use PostgreSQL".to_string(),
            status: DecisionStatus::Accepted,
            decision: "We chose PostgreSQL for its reliability.".to_string(),
            context: String::new(),
            consequences: String::new(),
            alternatives: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ratified_at: None,
            author: None,
            source: SourceReference::manual(),
            affects: vec![],
        };
        let evidence = format_decision_evidence(&decision);
        assert!(evidence.contains("Use PostgreSQL"));
    }

    #[test]
    fn test_format_decision_evidence_long() {
        let long_decision = "x".repeat(300);
        let decision = Decision {
            id: "adr-002".to_string(),
            title: "Long Decision".to_string(),
            status: DecisionStatus::Proposed,
            decision: long_decision.clone(),
            context: String::new(),
            consequences: String::new(),
            alternatives: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ratified_at: None,
            author: None,
            source: SourceReference::manual(),
            affects: vec![],
        };
        let evidence = format_decision_evidence(&decision);
        assert!(evidence.contains("..."));
        assert!(evidence.len() < long_decision.len() + 50);
    }

    #[test]
    fn test_format_node_evidence() {
        let node = ArchitectureNode {
            id: "svc".to_string(),
            kind: NodeKind::Service,
            label: "My Service".to_string(),
            technology: Some("Rust".to_string()),
            ..ArchitectureNode::default()
        };
        let evidence = format_node_evidence(&node, None);
        assert!(evidence.contains("My Service"));
        assert!(evidence.contains("Rust"));
    }

    #[test]
    fn test_format_node_evidence_no_tech() {
        let node = ArchitectureNode {
            id: "svc".to_string(),
            kind: NodeKind::Service,
            label: "No Tech Service".to_string(),
            ..ArchitectureNode::default()
        };
        let evidence = format_node_evidence(&node, None);
        assert!(evidence.contains("(not set)"));
    }

    #[test]
    fn test_format_edge_evidence_with_label() {
        let evidence = format_edge_evidence("Source", &EdgeKind::Calls, "Target", Some("HTTP"));
        assert!(evidence.contains("Source"));
        assert!(evidence.contains("Target"));
        assert!(evidence.contains("HTTP"));
    }

    #[test]
    fn test_format_edge_evidence_without_label() {
        let evidence = format_edge_evidence("A", &EdgeKind::ReadsFrom, "B", None);
        assert!(evidence.contains("A"));
        assert!(evidence.contains("B"));
    }

    #[test]
    fn test_query_result_serialization() {
        let result = QueryResult {
            question: "test?".to_string(),
            answer: "answer".to_string(),
            evidence: vec![],
            confidence: 0.5,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test?"));
    }

    #[test]
    fn test_evidence_kind_variants() {
        let kinds = vec![
            EvidenceKind::Decision,
            EvidenceKind::Policy,
            EvidenceKind::Requirement,
            EvidenceKind::Node,
            EvidenceKind::Edge,
        ];
        for kind in kinds {
            let evidence = Evidence {
                kind: kind.clone(),
                reference: "ref".to_string(),
                excerpt: "excerpt".to_string(),
            };
            let json = serde_json::to_string(&evidence).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_extract_tech_patterns() {
        let graph = create_test_graph();
        let patterns = graph.extract_tech_patterns("why use postgresql and redis?");
        assert!(patterns.contains(&"postgresql".to_string()));
        assert!(patterns.contains(&"redis".to_string()));
    }

    #[test]
    fn test_find_policy_violations_empty() {
        let graph = create_test_graph();
        let violations = graph.find_policy_violations();
        assert!(violations.is_empty());
    }
}
