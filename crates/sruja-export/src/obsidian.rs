use sruja_scan::Graph;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct ObsidianExporter;

impl ObsidianExporter {
    /// Export architecture graph as interlinked markdown files into output_dir.
    pub fn export(graph: &Graph, output_dir: &Path) -> std::io::Result<()> {
        fs::create_dir_all(output_dir)?;

        // Precompute dependencies to build depends_on / depended_on_by maps
        let mut depends_on: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut depended_on_by: HashMap<String, Vec<(String, String)>> = HashMap::new();

        for edge in &graph.edges {
            depends_on
                .entry(edge.source.clone())
                .or_default()
                .push((edge.target.clone(), edge.kind.kind_str().to_string()));
            depended_on_by
                .entry(edge.target.clone())
                .or_default()
                .push((edge.source.clone(), edge.kind.kind_str().to_string()));
        }

        // 1. Create individual node markdown files
        for node in &graph.nodes {
            let filename = format!("{}.md", sanitize_filename(&node.id));
            let filepath = output_dir.join(filename);

            let mut md = String::new();
            md.push_str(&format!("# {}\n\n", node.label));
            md.push_str(&format!("- **ID**: `{}`\n", node.id));
            md.push_str(&format!("- **Kind**: `{}`\n", node.kind.as_str()));
            if let Some(ref tech) = node.technology {
                md.push_str(&format!("- **Technology**: `{}`\n", tech));
            }
            if let Some(ref path) = node.path {
                md.push_str(&format!("- **Path**: `{}`\n", path));
            }
            md.push('\n');

            if let Some(desc) = node.metadata.get("description") {
                md.push_str("## Description\n\n");
                md.push_str(desc);
                md.push_str("\n\n");
            }

            md.push_str("## Depends On\n\n");
            if let Some(deps) = depends_on.get(&node.id) {
                let mut sorted_deps = deps.clone();
                sorted_deps.sort();
                for (target, kind) in sorted_deps {
                    let target_node_label = graph
                        .nodes
                        .iter()
                        .find(|n| n.id == target)
                        .map(|n| n.label.as_str())
                        .unwrap_or(&target);
                    md.push_str(&format!(
                        "- [[{}]] (*{}* via `{}`)\n",
                        sanitize_filename(&target),
                        target_node_label,
                        kind
                    ));
                }
            } else {
                md.push_str("*None*\n");
            }
            md.push('\n');

            md.push_str("## Depended On By\n\n");
            if let Some(revs) = depended_on_by.get(&node.id) {
                let mut sorted_revs = revs.clone();
                sorted_revs.sort();
                for (source, kind) in sorted_revs {
                    let source_node_label = graph
                        .nodes
                        .iter()
                        .find(|n| n.id == source)
                        .map(|n| n.label.as_str())
                        .unwrap_or(&source);
                    md.push_str(&format!(
                        "- [[{}]] (*{}* via `{}`)\n",
                        sanitize_filename(&source),
                        source_node_label,
                        kind
                    ));
                }
            } else {
                md.push_str("*None*\n");
            }

            fs::write(filepath, md)?;
        }

        // 2. Create _Index.md
        let mut index = String::new();
        index.push_str("# Sruja Architecture Index\n\n");
        index.push_str("## All Elements by Kind\n\n");

        let mut grouped: HashMap<String, Vec<(String, String)>> = HashMap::new();
        for node in &graph.nodes {
            grouped
                .entry(node.kind.as_str().to_string())
                .or_default()
                .push((node.label.clone(), node.id.clone()));
        }

        let mut kinds: Vec<String> = grouped.keys().cloned().collect();
        kinds.sort();

        for kind in kinds {
            index.push_str(&format!("### {}\n\n", capitalize(&kind)));
            let mut members = grouped.remove(&kind).unwrap_or_default();
            members.sort();
            for (label, id) in members {
                index.push_str(&format!("- [[{}]] - *{}*\n", sanitize_filename(&id), label));
            }
            index.push('\n');
        }

        fs::write(output_dir.join("_Index.md"), index)?;

        // 3. Create _Graph.md (Mermaid visualization)
        let mut graph_md = String::new();
        graph_md.push_str("# Architectural Map\n\n");
        graph_md.push_str("```mermaid\ngraph TD\n");
        for edge in &graph.edges {
            let src_san = sanitize_filename(&edge.source);
            let tgt_san = sanitize_filename(&edge.target);
            graph_md.push_str(&format!("  {} --> {}\n", src_san, tgt_san));
        }
        graph_md.push_str("```\n");

        fs::write(output_dir.join("_Graph.md"), graph_md)?;

        Ok(())
    }
}

fn sanitize_filename(id: &str) -> String {
    id.replace([':', '/', '.'], "_")
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::{Edge, EdgeKind, Graph, Node, NodeKind};

    #[test]
    fn test_obsidian_export() {
        let temp_dir = std::env::temp_dir().join("sruja_obsidian_test");
        let nodes = vec![
            Node {
                id: "A:B".to_string(),
                kind: NodeKind::new(NodeKind::MODULE),
                label: "A".to_string(),
                ..Default::default()
            },
            Node {
                id: "C".to_string(),
                kind: NodeKind::new(NodeKind::DATABASE),
                label: "C".to_string(),
                ..Default::default()
            },
        ];
        let edges = vec![Edge {
            source: "A:B".to_string(),
            target: "C".to_string(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            evidence: Vec::new(),
            confidence: sruja_scan::graph::EdgeConfidence::Extracted,
        }];
        let graph = Graph {
            nodes,
            edges,
            ..Default::default()
        };

        ObsidianExporter::export(&graph, &temp_dir).unwrap();

        assert!(temp_dir.join("A_B.md").exists());
        assert!(temp_dir.join("C.md").exists());
        assert!(temp_dir.join("_Index.md").exists());
        assert!(temp_dir.join("_Graph.md").exists());

        let a_content = fs::read_to_string(temp_dir.join("A_B.md")).unwrap();
        assert!(a_content.contains("[[C]]"));

        // Clean up
        fs::remove_dir_all(temp_dir).unwrap();
    }
}
