use std::path::Path;

use sruja_scan::Graph;

use super::analysis::{
    discover_confidence, discover_kind_counts, discover_key_elements, discover_key_relationships,
    discover_next_steps, discover_reasoning, discover_suggested_questions,
    discover_surprising_connections, discover_top_directories,
};
use super::context::discover_context_json_from_graph;
use super::models::{
    DiscoverCommunity, DiscoverEdgeConfidenceBreakdown, DiscoverExplanationJson,
};
use crate::commands::CliError;

pub(crate) fn build_discover_explanation(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Result<DiscoverExplanationJson, CliError> {
    let context = discover_context_json_from_graph(repo, repo_path, graph)?;
    let top_directories = discover_top_directories(repo, repo_path, graph);
    let reasoning = discover_reasoning(&context, graph, &top_directories);
    let confidence = discover_confidence(&context, graph, &top_directories);
    let god_nodes = discover_key_elements(repo, repo_path, graph);
    let key_relationships = discover_key_relationships(graph);
    let surprising_connections = discover_surprising_connections(graph);
    let suggested_questions = discover_suggested_questions(&god_nodes, &surprising_connections);

    let mut extracted = 0;
    let mut inferred = 0;
    let mut ambiguous = 0;
    for edge in &graph.edges {
        match edge.confidence {
            sruja_scan::graph::EdgeConfidence::Extracted => extracted += 1,
            sruja_scan::graph::EdgeConfidence::Inferred => inferred += 1,
            sruja_scan::graph::EdgeConfidence::Ambiguous => ambiguous += 1,
        }
    }
    let edge_confidence_breakdown = DiscoverEdgeConfidenceBreakdown {
        extracted,
        inferred,
        ambiguous,
    };

    let mut digest_parts = Vec::new();
    digest_parts.push(format!(
        "The scanned repository '{}' is identified as a {} system primarily written in {}.",
        context.repo, context.architecture_style, context.primary_language
    ));
    if let Some(ref framework) = context.framework {
        digest_parts.push(format!(
            "It leverages the {} framework to organize its components.",
            framework
        ));
    }
    digest_parts.push(format!(
        "The static analysis discovered {} logical component(s) connected by {} relationship(s).",
        graph.nodes.len(),
        graph.edges.len()
    ));
    if !graph.nodes.is_empty() {
        let db_count = graph
            .nodes
            .iter()
            .filter(|n| n.kind == sruja_scan::NodeKind::DATABASE)
            .count();
        if db_count > 0 {
            digest_parts.push(format!(
                "The architecture integrates {} database datastore(s) for persistent storage.",
                db_count
            ));
        } else {
            digest_parts.push("The architecture appears to be modular without directly scanned database dependencies at this level.".to_string());
        }
    }
    let architecture_digest = digest_parts.join(" ");

    let raw_communities = sruja_scan::detect_communities(graph);
    let community_infos = sruja_scan::summarize_communities(graph, &raw_communities);
    let communities: Vec<DiscoverCommunity> = community_infos
        .into_iter()
        .map(|c| DiscoverCommunity {
            id: c.id,
            suggested_label: c.suggested_label,
            member_count: c.member_count,
            cohesion: c.cohesion,
            top_members: c.members.into_iter().take(5).collect(),
        })
        .collect();

    Ok(DiscoverExplanationJson {
        kind_counts: discover_kind_counts(graph),
        key_elements: god_nodes.clone(),
        god_nodes,
        key_relationships,
        surprising_connections,
        suggested_questions,
        next_steps: discover_next_steps(graph),
        reasoning,
        confidence,
        edge_confidence_breakdown,
        architecture_digest,
        communities,
        top_directories,
        context,
        enrichment: None,
    })
}

/// Build discovery explanation as a JSON value from a pre-scanned graph (avoids rescanning).
pub fn discover_explanation_value_from_graph(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Result<serde_json::Value, CliError> {
    let explanation = build_discover_explanation(repo, repo_path, graph)?;
    serde_json::to_value(&explanation).map_err(|e| CliError::validation(e.to_string()))
}

pub(crate) fn format_discovery_explanation(explanation: &DiscoverExplanationJson) -> String {
    let mut out = String::new();
    out.push_str("# Sruja Discovery Explanation\n\n");
    out.push_str(&format!("**Repo:** {}\n", explanation.context.repo));
    out.push_str(&format!(
        "**Scan summary:** {} node(s), {} relationship(s)\n",
        explanation.context.components, explanation.context.edges
    ));
    out.push_str(&format!(
        "**Primary language:** {}\n",
        explanation.context.primary_language
    ));
    if let Some(framework) = &explanation.context.framework {
        out.push_str(&format!("**Framework:** {}\n", framework));
    }
    out.push_str(&format!(
        "**Architecture style:** {}\n",
        explanation.context.architecture_style
    ));
    if let Some(domain) = &explanation.context.domain {
        out.push_str(&format!("**Domain hint:** {}\n", domain));
    }

    if !explanation.kind_counts.is_empty() {
        let counts = explanation
            .kind_counts
            .iter()
            .map(|(kind, count)| format!("{kind}: {count}"))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("**Element mix:** {}\n", counts));
    }

    out.push_str("\n## Architecture Digest\n\n");
    out.push_str(&explanation.architecture_digest);
    out.push_str("\n\n## Why Sruja Thinks That\n\n");
    for reason in &explanation.reasoning {
        out.push_str(&format!("- {}\n", reason));
    }

    if !explanation.top_directories.is_empty() {
        out.push_str("\n## Top Directories\n\n");
        for dir in &explanation.top_directories {
            out.push_str(&format!("- `{}`: {} node(s)\n", dir.area, dir.nodes));
        }
    }

    out.push_str("\n## Dependencies Confidence Breakdown\n\n");
    out.push_str("| Confidence Level | Count |\n");
    out.push_str("|---|---|\n");
    out.push_str(&format!(
        "| **Extracted** (AST, manifest) | {} |\n",
        explanation.edge_confidence_breakdown.extracted
    ));
    out.push_str(&format!(
        "| **Inferred** (naming, heuristics) | {} |\n",
        explanation.edge_confidence_breakdown.inferred
    ));
    out.push_str(&format!(
        "| **Ambiguous** (unresolved, weak) | {} |\n",
        explanation.edge_confidence_breakdown.ambiguous
    ));

    out.push_str("\n## Discovery Confidence\n\n");
    out.push_str(&format!("**Level:** {}\n", explanation.confidence.level));
    for signal in &explanation.confidence.signals {
        out.push_str(&format!("- [✓] {}\n", signal));
    }
    for spot in &explanation.confidence.blind_spots {
        out.push_str(&format!("- [?] {}\n", spot));
    }

    if !explanation.god_nodes.is_empty() {
        out.push_str("\n## God Nodes (High-Signal Elements)\n\n");
        out.push_str("| ID | Label | Kind | Incoming | Outgoing | PageRank | Why It Matters |\n");
        out.push_str("|---|---|---|---|---|---|---|\n");
        for element in &explanation.god_nodes {
            let path_suffix = element
                .path
                .as_deref()
                .map(|path| format!("<br>`{path}`"))
                .unwrap_or_default();
            out.push_str(&format!(
                "| `{}`{} | {} | `{}` | {} | {} | {:.4} | {} |\n",
                element.id,
                path_suffix,
                element.label,
                element.kind,
                element.incoming,
                element.outgoing,
                element.pagerank,
                element.why_it_matters
            ));
        }
    }

    if !explanation.key_relationships.is_empty() {
        out.push_str("\n## Key Relationships\n\n");
        out.push_str("| Source | Target | Kind | Confidence | Why It Matters |\n");
        out.push_str("|---|---|---|---|---|\n");
        for relationship in &explanation.key_relationships {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | {} |\n",
                relationship.source,
                relationship.target,
                relationship.kind,
                relationship.confidence,
                relationship.why_it_matters
            ));
        }
    }

    if !explanation.surprising_connections.is_empty() {
        out.push_str("\n## Surprising Connections\n\n");
        out.push_str("| Source | Target | Kind | Confidence | Why It Matters |\n");
        out.push_str("|---|---|---|---|---|\n");
        for relationship in &explanation.surprising_connections {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | {} |\n",
                relationship.source,
                relationship.target,
                relationship.kind,
                relationship.confidence,
                relationship.why_it_matters
            ));
        }
    }

    if !explanation.suggested_questions.is_empty() {
        out.push_str("\n## Suggested Questions\n\n");
        for question in &explanation.suggested_questions {
            out.push_str(&format!("- {}\n", question));
        }
    }

    if !explanation.communities.is_empty() {
        out.push_str("\n## Module Communities (LPA Clusters)\n\n");
        out.push_str(
            "| ID | Suggested Label | Members | Cohesion | Boundary Status | Top Members |\n",
        );
        out.push_str("|---|---|---|---|---|---|\n");
        for community in &explanation.communities {
            let boundary_status = if community.cohesion < 0.3 {
                "⚠️ **Weakly Bounded**"
            } else {
                "✅ Strongly Bounded"
            };
            let members_joined = community
                .top_members
                .iter()
                .map(|m| format!("`{m}`"))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "| {} | **{}** | {} | {:.2} | {} | {} |\n",
                community.id,
                community.suggested_label,
                community.member_count,
                community.cohesion,
                boundary_status,
                members_joined
            ));
        }
    }

    out.push_str("\n## Next Steps\n\n");
    for step in &explanation.next_steps {
        out.push_str(&format!("- {}\n", step));
    }

    if let Some(ref e) = explanation.enrichment {
        out.push_str("\n## LLM Enrichment (opt-in)\n\n");
        out.push_str(
            "- This section is **LLM-generated** and must be treated as **interpretation**, not truth.\n",
        );
        out.push_str(
            "- It is grounded in the JSON facts above; if it contradicts them, prefer the grounded scan output.\n\n",
        );
        out.push_str(&format!("- Status: `{}`\n", e.status));
        if let Some(ref p) = e.provider {
            out.push_str(&format!("- Provider: `{}`\n", p));
        }
        if let Some(ref m) = e.model {
            out.push_str(&format!("- Model: `{}`\n", m));
        }
        if let Some(ref err) = e.error {
            out.push_str(&format!("- Error: `{}`\n", err));
        }
        out.push('\n');
        if let Some(ref narrative) = e.narrative_markdown {
            out.push_str(narrative);
            out.push('\n');
        }
    }

    out
}

pub fn discover_explanation_markdown(repo: &str) -> Result<String, CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }
    let graph = crate::commands::scan_repo_cached(repo_path)?;
    let explanation = build_discover_explanation(repo, repo_path, &graph)?;
    Ok(format_discovery_explanation(&explanation))
}
