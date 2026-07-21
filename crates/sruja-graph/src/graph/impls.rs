use crate::graph::GraphStats;
use crate::learning::LearningEntry;
use crate::*;
use chrono::Utc;

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
    pub fn merge_node(&mut self, mut node: ArchitectureNode) {
        if let Some(existing) = self.nodes.get(&node.id) {
            // Merge tribal knowledge if not already present
            let mut node_gotchas = node.gotchas();
            for g in existing.gotchas() {
                if !node_gotchas.contains(&g) {
                    node_gotchas.push(g);
                }
            }
            node.set_gotchas(node_gotchas);

            let mut node_constraints = node.operational_constraints();
            for c in existing.operational_constraints() {
                if !node_constraints.contains(&c) {
                    node_constraints.push(c);
                }
            }
            node.set_operational_constraints(node_constraints);

            let mut node_runbooks = node.runbooks();
            for r in existing.runbooks() {
                if !node_runbooks.contains(&r) {
                    node_runbooks.push(r);
                }
            }
            node.set_runbooks(node_runbooks);
        }
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

    pub fn add_incident(&mut self, incident: Incident) -> Result<(), GraphError> {
        self.incidents.insert(incident.id.clone(), incident);
        self.touch();
        Ok(())
    }

    pub fn get_incident(&self, id: &str) -> Option<&Incident> {
        self.incidents.get(id)
    }

    pub fn add_learning(&mut self, learning: LearningEntry) {
        self.learnings.insert(learning.id.clone(), learning);
        self.touch();
    }

    pub fn get_learning(&self, id: &str) -> Option<&LearningEntry> {
        self.learnings.get(id)
    }

    /// Get learnings that directly affect a specific node.
    pub fn get_learnings_for_node(&self, node_id: &str) -> Vec<&LearningEntry> {
        self.learnings
            .values()
            .filter(|l| l.affected_elements.contains(&node_id.to_string()))
            .collect()
    }

    /// Get learnings affecting any node in the given cluster (deduplicated).
    pub fn get_learnings_for_cluster(&self, node_ids: &[String]) -> Vec<&LearningEntry> {
        let id_set: std::collections::HashSet<&String> = node_ids.iter().collect();
        self.learnings
            .values()
            .filter(|l| l.affected_elements.iter().any(|e| id_set.contains(e)))
            .collect()
    }

    /// Traverse related_ids to find neighboring learnings.
    pub fn get_learning_neighbors(&self, learning_id: &str) -> Vec<&LearningEntry> {
        let Some(learning) = self.learnings.get(learning_id) else {
            return Vec::new();
        };
        learning
            .related_ids
            .iter()
            .filter_map(|rid| self.learnings.get(rid))
            .collect()
    }

    /// Get recent events that reference a specific element.
    pub fn get_events_for_node(&self, node_id: &str) -> Vec<&ContextEventSummary> {
        self.recent_events
            .iter()
            .filter(|e| e.elements.contains(&node_id.to_string()))
            .collect()
    }

    /// Get recent events that reference any node in the given cluster.
    pub fn get_events_for_cluster(&self, node_ids: &[String]) -> Vec<&ContextEventSummary> {
        let id_set: std::collections::HashSet<&String> = node_ids.iter().collect();
        self.recent_events
            .iter()
            .filter(|e| e.elements.iter().any(|el| id_set.contains(el)))
            .collect()
    }

    /// Set recent events (used during graph build).
    pub fn set_recent_events(&mut self, events: Vec<ContextEventSummary>) {
        self.recent_events = events;
        self.touch();
    }

    /// Get decisions affecting any node in the blast radius of a target node.
    pub fn get_decisions_for_blast_radius(&self, target_id: &str) -> Vec<&Decision> {
        let mut affected = std::collections::HashSet::new();
        affected.insert(target_id.to_string());

        // Collect upstream and downstream nodes (1 hop)
        for edge in &self.edges {
            if edge.source == target_id {
                affected.insert(edge.target.clone());
            }
            if edge.target == target_id {
                affected.insert(edge.source.clone());
            }
        }

        self.decisions
            .values()
            .filter(|d| d.affects.iter().any(|a| affected.contains(a)))
            .collect()
    }

    /// Traverse supersedes links to get a decision chain.
    pub fn get_decision_chain(&self, decision_id: &str) -> Vec<&Decision> {
        let mut chain = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut current = decision_id;

        while let Some(decision) = self.decisions.get(current) {
            if !visited.insert(current.to_string()) {
                break;
            }
            chain.push(decision);
            // Check if this decision supersedes another
            if let Some(supersedes_id) = decision
                .alternatives
                .iter()
                .find(|a| a.starts_with("supersedes:"))
            {
                current = &supersedes_id[11..];
            } else {
                break;
            }
        }

        chain
    }

    pub fn find_nodes_by_kind(&self, kind: NodeKind) -> Vec<&ArchitectureNode> {
        self.nodes.values().filter(|n| n.kind == kind).collect()
    }

    pub fn find_nodes_by_technology(&self, tech: &str) -> Vec<&ArchitectureNode> {
        self.nodes
            .values()
            .filter(|n| {
                n.technology()
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
            total_learnings: self.learnings.len(),
        }
    }
}
