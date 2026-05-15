use sruja_scan::Graph;

pub struct GraphMLExporter;

impl GraphMLExporter {
    /// Export a scan Graph to GraphML XML format string.
    pub fn export(graph: &Graph) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\"\n");
        xml.push_str("         xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"\n");
        xml.push_str("         xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">\n");

        // Key definitions
        xml.push_str(
            "  <key id=\"d_kind\" for=\"node\" attr.name=\"kind\" attr.type=\"string\"/>\n",
        );
        xml.push_str(
            "  <key id=\"d_label\" for=\"node\" attr.name=\"label\" attr.type=\"string\"/>\n",
        );
        xml.push_str("  <key id=\"d_technology\" for=\"node\" attr.name=\"technology\" attr.type=\"string\"/>\n");
        xml.push_str("  <key id=\"d_confidence\" for=\"node\" attr.name=\"confidence\" attr.type=\"string\"/>\n");
        xml.push_str(
            "  <key id=\"e_kind\" for=\"edge\" attr.name=\"kind\" attr.type=\"string\"/>\n",
        );
        xml.push_str("  <key id=\"e_confidence\" for=\"edge\" attr.name=\"confidence\" attr.type=\"string\"/>\n");

        xml.push_str("  <graph id=\"G\" edgedefault=\"directed\">\n");

        // Nodes
        for node in &graph.nodes {
            let id_esc = escape_xml(&node.id);
            xml.push_str(&format!("    <node id=\"{}\">\n", id_esc));
            xml.push_str(&format!(
                "      <data key=\"d_kind\">{}</data>\n",
                escape_xml(node.kind.as_str())
            ));
            xml.push_str(&format!(
                "      <data key=\"d_label\">{}</data>\n",
                escape_xml(&node.label)
            ));
            if let Some(ref tech) = node.technology {
                xml.push_str(&format!(
                    "      <data key=\"d_technology\">{}</data>\n",
                    escape_xml(tech)
                ));
            }
            if let Some(conf) = node.confidence {
                xml.push_str(&format!(
                    "      <data key=\"d_confidence\">{}</data>\n",
                    conf
                ));
            }
            xml.push_str("    </node>\n");
        }

        // Edges
        for (i, edge) in graph.edges.iter().enumerate() {
            let src_esc = escape_xml(&edge.source);
            let tgt_esc = escape_xml(&edge.target);
            xml.push_str(&format!(
                "    <edge id=\"e{}\" source=\"{}\" target=\"{}\">\n",
                i, src_esc, tgt_esc
            ));
            xml.push_str(&format!(
                "      <data key=\"e_kind\">{}</data>\n",
                escape_xml(edge.kind.kind_str())
            ));

            let conf_str = match edge.confidence {
                sruja_scan::graph::EdgeConfidence::Extracted => "extracted",
                sruja_scan::graph::EdgeConfidence::Inferred => "inferred",
                sruja_scan::graph::EdgeConfidence::Ambiguous => "ambiguous",
            };
            xml.push_str(&format!(
                "      <data key=\"e_confidence\">{}</data>\n",
                conf_str
            ));
            xml.push_str("    </edge>\n");
        }

        xml.push_str("  </graph>\n");
        xml.push_str("</graphml>\n");
        xml
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::{Edge, EdgeKind, Graph, Node, NodeKind};

    #[test]
    fn test_graphml_export() {
        let nodes = vec![
            Node {
                id: "A".to_string(),
                kind: NodeKind::new(NodeKind::MODULE),
                label: "A".to_string(),
                ..Default::default()
            },
            Node {
                id: "B".to_string(),
                kind: NodeKind::new(NodeKind::DATABASE),
                label: "B".to_string(),
                technology: Some("Postgres".to_string()),
                ..Default::default()
            },
        ];
        let edges = vec![Edge {
            source: "A".to_string(),
            target: "B".to_string(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            evidence: Vec::new(),
            confidence: sruja_scan::graph::EdgeConfidence::Extracted,
        }];
        let graph = Graph {
            nodes,
            edges,
            ..Default::default()
        };
        let out = GraphMLExporter::export(&graph);

        assert!(out.contains("node id=\"A\""));
        assert!(out.contains("node id=\"B\""));
        assert!(out.contains("<data key=\"d_technology\">Postgres</data>"));
        assert!(out.contains("edge id=\"e0\" source=\"A\" target=\"B\""));
    }
}
