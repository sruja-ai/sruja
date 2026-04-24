use sruja_graph::KnowledgeGraph;

pub struct HtmlExporter;

impl Default for HtmlExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlExporter {
    pub fn new() -> Self {
        Self
    }

    pub fn export(&self, graph: &KnowledgeGraph) -> Result<String, Box<dyn std::error::Error>> {
        let nodes_json = serde_json::to_string(&graph.nodes.values().collect::<Vec<_>>())?;
        let edges_json = serde_json::to_string(&graph.edges)?;
        let decisions_json = serde_json::to_string(&graph.decisions.values().collect::<Vec<_>>())?;

        let template = include_str!("template.html");
        let html = template
            .replace("{{NODES_JSON}}", &nodes_json)
            .replace("{{EDGES_JSON}}", &edges_json)
            .replace("{{DECISIONS_JSON}}", &decisions_json)
            .replace("{{REPO_NAME}}", &graph.metadata.name);

        Ok(html)
    }
}
