use sruja_scan::Graph;

pub struct Neo4jExporter;

impl Neo4jExporter {
    /// Generate Cypher CREATE statements for nodes and relationships.
    pub fn export(graph: &Graph) -> String {
        let mut cypher = String::new();
        cypher.push_str("// Sruja Architecture Neo4j Cypher Export\n\n");

        cypher.push_str("// Create Nodes\n");
        for node in &graph.nodes {
            let id_esc = escape_cypher(&node.id);
            let label_esc = escape_cypher(&node.label);
            let kind_esc = escape_cypher(node.kind.as_str());

            let tech_part = if let Some(ref tech) = node.technology {
                format!(", technology: '{}'", escape_cypher(tech))
            } else {
                "".to_string()
            };

            let path_part = if let Some(ref path) = node.path {
                format!(", path: '{}'", escape_cypher(path))
            } else {
                "".to_string()
            };

            cypher.push_str(&format!(
                "CREATE (n:Component {{id: '{}', label: '{}', kind: '{}'{}{}}});\n",
                id_esc, label_esc, kind_esc, tech_part, path_part
            ));
        }

        cypher.push_str("\n// Create Relationships\n");
        for edge in &graph.edges {
            let src_esc = escape_cypher(&edge.source);
            let tgt_esc = escape_cypher(&edge.target);
            let kind_esc = escape_cypher(edge.kind.kind_str());

            let conf_str = match edge.confidence {
                sruja_scan::graph::EdgeConfidence::Extracted => "extracted",
                sruja_scan::graph::EdgeConfidence::Inferred => "inferred",
                sruja_scan::graph::EdgeConfidence::Ambiguous => "ambiguous",
            };

            cypher.push_str(&format!(
                "MATCH (a:Component {{id: '{}'}}), (b:Component {{id: '{}'}})\n\
                 CREATE (a)-[:DEPENDS_ON {{kind: '{}', confidence: '{}'}}]->(b);\n",
                src_esc, tgt_esc, kind_esc, conf_str
            ));
        }

        cypher
    }
}

fn escape_cypher(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::{Edge, EdgeKind, Graph, Node, NodeKind};

    #[test]
    fn test_neo4j_export() {
        let nodes = vec![
            Node {
                id: "A".to_string(),
                kind: NodeKind::Module,
                label: "A".to_string(),
                ..Default::default()
            },
            Node {
                id: "B".to_string(),
                kind: NodeKind::Database,
                label: "B".to_string(),
                technology: Some("MySQL".to_string()),
                ..Default::default()
            },
        ];
        let edges = vec![Edge {
            source: "A".to_string(),
            target: "B".to_string(),
            kind: EdgeKind::Calls,
            evidence: Vec::new(),
            confidence: sruja_scan::graph::EdgeConfidence::Extracted,
        }];
        let graph = Graph {
            nodes,
            edges,
            ..Default::default()
        };
        let out = Neo4jExporter::export(&graph);

        assert!(out.contains("id: 'A'"));
        assert!(out.contains("id: 'B'"));
        assert!(out.contains("technology: 'MySQL'"));
        assert!(out.contains("MATCH (a:Component {id: 'A'}), (b:Component {id: 'B'})"));
        assert!(out.contains("[:DEPENDS_ON {kind: 'calls', confidence: 'extracted'}]"));
    }
}
