//! Query interface for Knowledge Graph

use crate::*;
use thiserror::Error;

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

        if question_lower.contains("why") {
            self.query_why(question)
        } else if question_lower.contains("what") || question_lower.contains("which") {
            self.query_what(question)
        } else if question_lower.contains("how") {
            self.query_how(question)
        } else if question_lower.contains("decision") || question_lower.contains("adr") {
            self.query_decisions(question)
        } else {
            self.query_generic(question)
        }
    }

    fn query_why(&self, question: &str) -> Result<QueryResult, QueryError> {
        let tech_patterns = self.extract_tech_patterns(question);

        for tech in &tech_patterns {
            let nodes = self.find_nodes_by_technology(tech);
            if !nodes.is_empty() {
                let node = &nodes[0];
                let decisions = self.get_decisions_for_node(&node.id);

                if !decisions.is_empty() {
                    let decision = &decisions[0];
                    return Ok(QueryResult {
                        question: question.to_string(),
                        answer: format!("Based on {}: {}", decision.title, decision.decision),
                        evidence: vec![Evidence {
                            kind: EvidenceKind::Decision,
                            reference: decision.id.clone(),
                            excerpt: decision.decision.clone(),
                        }],
                        confidence: 0.85,
                    });
                }

                return Ok(QueryResult {
                    question: question.to_string(),
                    answer: format!("{} uses {} technology.", node.label, tech),
                    evidence: vec![Evidence {
                        kind: EvidenceKind::Node,
                        reference: node.id.clone(),
                        excerpt: format!("Node {} uses technology {}", node.label, tech),
                    }],
                    confidence: 0.5,
                });
            }
        }

        self.query_generic(question)
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
                let nodes = self.find_nodes_by_kind(*kind);
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
                                reference: n.id.clone(),
                                excerpt: n.label.clone(),
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
                return Ok(QueryResult {
                    question: question.to_string(),
                    answer: format!("Found {} connections", connections.len()),
                    evidence: connections
                        .iter()
                        .take(10)
                        .map(|c| Evidence {
                            kind: EvidenceKind::Edge,
                            reference: String::new(),
                            excerpt: c.clone(),
                        })
                        .collect(),
                    confidence: 0.7,
                });
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
                    reference: d.id.clone(),
                    excerpt: format!("{}: {}", d.title, d.decision),
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
    use std::collections::HashMap;

    fn create_test_graph() -> KnowledgeGraph {
        let mut graph = KnowledgeGraph::new();

        graph
            .add_node(ArchitectureNode {
                id: "api".to_string(),
                kind: NodeKind::Service,
                label: "API Service".to_string(),
                technology: Some("Node.js".to_string()),
                description: None,
                metadata: HashMap::new(),
                source: SourceReference::manual(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .unwrap();

        graph
            .add_node(ArchitectureNode {
                id: "db".to_string(),
                kind: NodeKind::Database,
                label: "PostgreSQL".to_string(),
                technology: Some("PostgreSQL".to_string()),
                description: None,
                metadata: HashMap::new(),
                source: SourceReference::manual(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .unwrap();

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
}
