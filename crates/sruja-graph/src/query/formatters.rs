use crate::*;

pub(super) fn format_decision_evidence(d: &Decision) -> String {
    let snippet = d.decision.trim();
    let max_len = 200;
    if snippet.chars().count() <= max_len {
        format!("[{}] {}", d.title, snippet)
    } else {
        format!(
            "[{}] {}...",
            d.title,
            snippet.chars().take(max_len).collect::<String>()
        )
    }
}

pub(super) fn format_node_evidence(node: &ArchitectureNode, tech: Option<&str>) -> String {
    let kind = format!("{}", node.kind);
    let binding = node.technology();
    let tech_str = tech.or(binding).unwrap_or("(not set)");
    format!(
        "Component '{}' (kind={}, technology={})",
        node.label, kind, tech_str
    )
}

pub(super) fn format_edge_evidence(
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
