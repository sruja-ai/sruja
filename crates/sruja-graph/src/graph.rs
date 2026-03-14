//! Knowledge Graph implementation

use crate::*;
use chrono::Utc;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: HashMap<NodeId, ArchitectureNode>,
    pub edges: Vec<ArchitectureEdge>,
    pub decisions: HashMap<DecisionId, Decision>,
    pub policies: HashMap<PolicyId, Policy>,
    pub requirements: HashMap<RequirementId, Requirement>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: String,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            name: "Architecture Graph".to_string(),
            description: None,
            created_at: now,
            updated_at: now,
            version: "1.0.0".to_string(),
        }
    }
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(name: impl Into<String>) -> Self {
        let mut graph = Self::new();
        graph.metadata.name = name.into();
        graph
    }

    pub fn touch(&mut self) {
        self.metadata.updated_at = Utc::now();
    }

    pub fn add_node(&mut self, node: ArchitectureNode) -> Result<(), GraphError> {
        if self.nodes.contains_key(&node.id) {
            return Err(GraphError::DuplicateNode(node.id));
        }
        self.nodes.insert(node.id.clone(), node);
        self.touch();
        Ok(())
    }

    /// Merge a node into the graph. Inserts if new, updates if existing.
    /// Used when loading context from scanned code.
    pub fn merge_node(&mut self, node: ArchitectureNode) {
        self.nodes.insert(node.id.clone(), node);
        self.touch();
    }

    pub fn remove_node(&mut self, id: &str) -> Result<ArchitectureNode, GraphError> {
        let node = self
            .nodes
            .remove(id)
            .ok_or(GraphError::NodeNotFound(id.to_string()))?;
        self.edges.retain(|e| e.source != id && e.target != id);
        self.touch();
        Ok(node)
    }

    pub fn get_node(&self, id: &str) -> Option<&ArchitectureNode> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut ArchitectureNode> {
        self.nodes.get_mut(id)
    }

    pub fn add_edge(&mut self, edge: ArchitectureEdge) -> Result<(), GraphError> {
        if !self.nodes.contains_key(&edge.source) {
            return Err(GraphError::NodeNotFound(edge.source.clone()));
        }
        if !self.nodes.contains_key(&edge.target) {
            return Err(GraphError::NodeNotFound(edge.target.clone()));
        }
        self.edges.push(edge);
        self.touch();
        Ok(())
    }

    /// Merge an edge into the graph. Skips if source/target nodes don't exist.
    /// Used when loading context from scanned code.
    pub fn merge_edge(&mut self, edge: ArchitectureEdge) {
        if self.nodes.contains_key(&edge.source)
            && self.nodes.contains_key(&edge.target)
            && !self
                .edges
                .iter()
                .any(|e| e.source == edge.source && e.target == edge.target && e.kind == edge.kind)
        {
            self.edges.push(edge);
            self.touch();
        }
    }

    pub fn remove_edge(&mut self, id: &str) -> Option<ArchitectureEdge> {
        if let Some(pos) = self.edges.iter().position(|e| e.id == id) {
            let edge = self.edges.remove(pos);
            self.touch();
            Some(edge)
        } else {
            None
        }
    }

    pub fn get_edges_from(&self, node_id: &str) -> Vec<&ArchitectureEdge> {
        self.edges.iter().filter(|e| e.source == node_id).collect()
    }

    pub fn get_edges_to(&self, node_id: &str) -> Vec<&ArchitectureEdge> {
        self.edges.iter().filter(|e| e.target == node_id).collect()
    }

    pub fn add_decision(&mut self, decision: Decision) -> Result<(), GraphError> {
        self.decisions.insert(decision.id.clone(), decision);
        self.touch();
        Ok(())
    }

    pub fn get_decision(&self, id: &str) -> Option<&Decision> {
        self.decisions.get(id)
    }

    pub fn get_decisions_for_node(&self, node_id: &str) -> Vec<&Decision> {
        self.decisions
            .values()
            .filter(|d| d.affects.contains(&node_id.to_string()))
            .collect()
    }

    pub fn accept_decision(&mut self, id: &str) -> Result<(), GraphError> {
        let decision = self
            .decisions
            .get_mut(id)
            .ok_or(GraphError::DecisionNotFound(id.to_string()))?;
        decision.status = DecisionStatus::Accepted;
        decision.ratified_at = Some(Utc::now());
        self.touch();
        Ok(())
    }

    pub fn add_policy(&mut self, policy: Policy) -> Result<(), GraphError> {
        self.policies.insert(policy.id.clone(), policy);
        self.touch();
        Ok(())
    }

    pub fn get_policy(&self, id: &str) -> Option<&Policy> {
        self.policies.get(id)
    }

    pub fn add_requirement(&mut self, requirement: Requirement) -> Result<(), GraphError> {
        self.requirements
            .insert(requirement.id.clone(), requirement);
        self.touch();
        Ok(())
    }

    pub fn get_requirement(&self, id: &str) -> Option<&Requirement> {
        self.requirements.get(id)
    }

    pub fn find_nodes_by_kind(&self, kind: NodeKind) -> Vec<&ArchitectureNode> {
        self.nodes.values().filter(|n| n.kind == kind).collect()
    }

    pub fn find_nodes_by_technology(&self, tech: &str) -> Vec<&ArchitectureNode> {
        self.nodes
            .values()
            .filter(|n| {
                n.technology
                    .as_deref()
                    .map(|t| t.to_lowercase() == tech.to_lowercase())
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn to_json(&self) -> Result<String, GraphError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(json: &str) -> Result<Self, GraphError> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn stats(&self) -> GraphStats {
        GraphStats {
            total_nodes: self.nodes.len(),
            total_edges: self.edges.len(),
            total_decisions: self.decisions.len(),
            accepted_decisions: self
                .decisions
                .values()
                .filter(|d| d.status == DecisionStatus::Accepted)
                .count(),
            proposed_decisions: self
                .decisions
                .values()
                .filter(|d| d.status == DecisionStatus::Proposed)
                .count(),
            total_policies: self.policies.len(),
            total_requirements: self.requirements.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub total_decisions: usize,
    pub accepted_decisions: usize,
    pub proposed_decisions: usize,
    pub total_policies: usize,
    pub total_requirements: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_node(id: &str) -> ArchitectureNode {
        ArchitectureNode {
            id: id.to_string(),
            kind: NodeKind::Service,
            label: id.to_string(),
            technology: None,
            description: None,
            metadata: HashMap::new(),
            source: SourceReference::manual(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_add_node() {
        let mut graph = KnowledgeGraph::new();
        let node = test_node("api");
        graph.add_node(node).unwrap();
        assert!(graph.get_node("api").is_some());
    }

    #[test]
    fn test_duplicate_node() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node(test_node("api")).unwrap();
        let result = graph.add_node(test_node("api"));
        assert!(result.is_err());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node(test_node("api")).unwrap();
        graph.add_node(test_node("db")).unwrap();

        let edge = ArchitectureEdge {
            id: "edge1".to_string(),
            source: "api".to_string(),
            target: "db".to_string(),
            kind: EdgeKind::DependsOn,
            label: None,
            description: None,
            source_ref: SourceReference::manual(),
        };

        graph.add_edge(edge).unwrap();
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_edge_to_nonexistent_node() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node(test_node("api")).unwrap();

        let edge = ArchitectureEdge {
            id: "edge1".to_string(),
            source: "api".to_string(),
            target: "nonexistent".to_string(),
            kind: EdgeKind::DependsOn,
            label: None,
            description: None,
            source_ref: SourceReference::manual(),
        };

        let result = graph.add_edge(edge);
        assert!(result.is_err());
    }

    fn test_edge(id: &str, source: &str, target: &str, kind: EdgeKind) -> ArchitectureEdge {
        ArchitectureEdge {
            id: id.to_string(),
            source: source.to_string(),
            target: target.to_string(),
            kind,
            label: None,
            description: None,
            source_ref: SourceReference::manual(),
        }
    }

    #[test]
    fn test_remove_node_removes_edges() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node(test_node("api")).unwrap();
        graph.add_node(test_node("db")).unwrap();
        graph.add_edge(test_edge("e1", "api", "db", EdgeKind::DependsOn)).unwrap();

        let removed = graph.remove_node("api").unwrap();
        assert_eq!(removed.id, "api");
        assert!(graph.get_node("api").is_none());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_node_errors() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node(test_node("api")).unwrap();
        let result = graph.remove_node("missing");
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_node_inserts_or_updates() {
        let mut graph = KnowledgeGraph::new();
        let n1 = test_node("api");
        graph.merge_node(n1);
        assert_eq!(graph.nodes.len(), 1);

        let mut n2 = test_node("api");
        n2.label = "API v2".to_string();
        graph.merge_node(n2);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.get_node("api").unwrap().label, "API v2");
    }

    #[test]
    fn test_merge_edge_skips_if_nodes_missing() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node(test_node("api")).unwrap();
        graph.merge_edge(test_edge("e1", "api", "nonexistent", EdgeKind::Calls));
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_merge_edge_adds_when_nodes_exist() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node(test_node("api")).unwrap();
        graph.add_node(test_node("db")).unwrap();
        graph.merge_edge(test_edge("e1", "api", "db", EdgeKind::ReadsFrom));
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_stats_counts_correctly() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node(test_node("a")).unwrap();
        graph.add_node(test_node("b")).unwrap();
        graph.add_edge(test_edge("e1", "a", "b", EdgeKind::Calls)).unwrap();

        let stats = graph.stats();
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.total_edges, 1);
    }

    #[test]
    fn test_with_name_sets_metadata() {
        let graph = KnowledgeGraph::with_name("My Architecture");
        assert_eq!(graph.metadata.name, "My Architecture");
    }

    #[test]
    fn test_find_nodes_by_kind() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node(test_node("api")).unwrap();
        let mut db_node = test_node("db");
        db_node.kind = NodeKind::Database;
        graph.add_node(db_node).unwrap();

        let services = graph.find_nodes_by_kind(NodeKind::Service);
        let dbs = graph.find_nodes_by_kind(NodeKind::Database);
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "api");
        assert_eq!(dbs.len(), 1);
        assert_eq!(dbs[0].id, "db");
    }

    #[test]
    fn test_get_edges_from_and_to() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node(test_node("a")).unwrap();
        graph.add_node(test_node("b")).unwrap();
        graph.add_node(test_node("c")).unwrap();
        graph.add_edge(test_edge("e1", "a", "b", EdgeKind::Calls)).unwrap();
        graph.add_edge(test_edge("e2", "a", "c", EdgeKind::Calls)).unwrap();

        let from_a = graph.get_edges_from("a");
        let to_b = graph.get_edges_to("b");
        assert_eq!(from_a.len(), 2);
        assert_eq!(to_b.len(), 1);
        assert_eq!(to_b[0].source, "a");
    }
}
