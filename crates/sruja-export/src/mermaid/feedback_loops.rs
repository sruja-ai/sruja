//! Mermaid diagram generation for feedback loops and causal loops.
//!
//! Emits `graph` blocks for feedback loops and causal loops with
//! special styling for loop types (reinforcing/balancing) and causal relationships.

use sruja_language::ast::{CausalLoop, CausalPolarity, FeedbackLoop};

/// Escape a string for use inside Mermaid labels.
fn escape_mermaid(s: &str) -> String {
    s.replace(['"', '\n', '\r'], " ").trim().to_string()
}

/// Sanitize ID for Mermaid.
fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
}

/// Generate Mermaid diagram for a feedback loop.
pub fn feedback_loop_to_diagram(loop_data: &FeedbackLoop) -> String {
    let mut out = String::new();
    out.push_str(&format!("graph {}\n", "LR"));
    out.push('\n');

    // Add comment for the loop
    if let Some(ref loop_id) = loop_data.loop_id {
        out.push_str(&format!(
            "    %% Feedback Loop: {} - {}\n",
            loop_id, loop_data.title
        ));
    } else {
        out.push_str(&format!("    %% Feedback Loop: {}\n", loop_data.title));
    }
    out.push('\n');

    // Add loop type indicator
    let loop_type_str = loop_data.loop_type.to_string();
    let loop_symbol = loop_data.loop_type.to_symbol();
    out.push_str(&format!(
        "    %% Type: {} ({})\n",
        loop_type_str, loop_symbol
    ));
    out.push('\n');

    // Add styles
    write_feedback_loop_styles(&mut out);
    out.push('\n');

    // Write all nodes and edges from relations
    let mut nodes = std::collections::HashSet::<String>::new();

    for rel in &loop_data.relationships {
        let from = sanitize_id(&rel.from.as_string());
        let to = sanitize_id(&rel.to.as_string());

        // Write nodes if not already written
        if !nodes.contains(&from) {
            out.push_str(&format!("    {from}[\"{from}\"]\n"));
            nodes.insert(from.clone());
        }
        if !nodes.contains(&to) {
            out.push_str(&format!("    {to}[\"{to}\"]\n"));
            nodes.insert(to.clone());
        }

        // Write edge with label
        if let Some(ref label) = rel.label {
            out.push_str(&format!("    {from} -->|\"{label}\"| {to}\n"));
        } else {
            out.push_str(&format!("    {from} --> {to}\n"));
        }
    }

    out.push('\n');

    // Add loop indicator (circular arrow on first node)
    if !loop_data.relationships.is_empty() {
        let first_node = sanitize_id(&loop_data.relationships[0].from.as_string());
        let class = match loop_data.loop_type {
            sruja_language::ast::FeedbackLoopType::Reinforcing => "reinforcing",
            sruja_language::ast::FeedbackLoopType::Balancing => "balancing",
        };
        out.push_str(&format!(
            "    {first_node} ==>|{loop_symbol}| {first_node}\n"
        ));
        out.push_str(&format!("    class {first_node} {class}\n"));
    }

    out.push_str("end\n");
    out
}

/// Generate Mermaid diagram for a causal loop.
pub fn causal_loop_to_diagram(loop_data: &CausalLoop) -> String {
    let mut out = String::new();
    out.push_str(&format!("graph {}\n", "LR"));
    out.push('\n');

    // Add comment for the loop
    if let Some(ref loop_id) = loop_data.loop_id {
        out.push_str(&format!(
            "    %% Causal Loop: {} - {}\n",
            loop_id, loop_data.title
        ));
    } else {
        out.push_str(&format!("    %% Causal Loop: {}\n", loop_data.title));
    }
    out.push('\n');

    // Add loop type indicator
    let loop_type_str = loop_data.loop_type.to_string();
    let loop_symbol = loop_data.loop_type.to_symbol();
    out.push_str(&format!(
        "    %% Type: {} ({})\n",
        loop_type_str, loop_symbol
    ));
    out.push('\n');

    // Add styles
    write_causal_loop_styles(&mut out);
    out.push('\n');

    // Write all nodes and edges from causal relationships
    let mut nodes = std::collections::HashSet::<String>::new();

    for rel in &loop_data.relationships {
        let from = sanitize_id(&rel.from);
        let to = sanitize_id(&rel.to);

        // Write nodes if not already written
        if !nodes.contains(&from) {
            out.push_str(&format!("    {from}[\"{from}\"]\n"));
            nodes.insert(from.clone());
        }
        if !nodes.contains(&to) {
            out.push_str(&format!("    {to}[\"{to}\"]\n"));
            nodes.insert(to.clone());
        }

        // Write edge with polarity
        let polarity_symbol = rel.polarity.to_string();
        let effect = rel
            .effect
            .as_deref()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "".to_string());
        let effect = escape_mermaid(&effect);

        // Check for delay
        if let Some(ref delay) = rel.delay {
            if effect.is_empty() {
                out.push_str(&format!("    {from} --|\"|| {delay}\"| {to}\n"));
            } else {
                out.push_str(&format!("    {from} --|\"{effect} (|| {delay})\"| {to}\n"));
            }
        } else if effect.is_empty() {
            out.push_str(&format!("    {from} -->|\"{polarity_symbol}\"| {to}\n"));
        } else {
            out.push_str(&format!(
                "    {from} -->|\"{effect} ({polarity_symbol})\"| {to}\n"
            ));
        }

        // Apply style based on polarity
        let class = match rel.polarity {
            CausalPolarity::Positive => "positive",
            CausalPolarity::Negative => "negative",
        };
        out.push_str(&format!("    class {to} {class}\n"));
    }

    out.push('\n');

    // Add loop indicator (circular arrow on first node)
    if !loop_data.relationships.is_empty() {
        let first_node = sanitize_id(&loop_data.relationships[0].from);
        let class = match loop_data.loop_type {
            sruja_language::ast::FeedbackLoopType::Reinforcing => "reinforcing",
            sruja_language::ast::FeedbackLoopType::Balancing => "balancing",
        };
        out.push_str(&format!(
            "    {first_node} ==>|{loop_symbol}| {first_node}\n"
        ));
        out.push_str(&format!("    class {first_node} {class}\n"));
    }

    out.push_str("end\n");
    out
}

/// Write styles for feedback loop diagrams.
fn write_feedback_loop_styles(out: &mut String) {
    out.push_str("    classDef reinforcing fill:#C6F6D5,stroke:#38A169,stroke-width:2px\n");
    out.push_str("    classDef balancing fill:#FED7D7,stroke:#E53E3E,stroke-width:2px\n");
}

/// Write styles for causal loop diagrams.
fn write_causal_loop_styles(out: &mut String) {
    out.push_str("    classDef reinforcing fill:#C6F6D5,stroke:#38A169,stroke-width:2px\n");
    out.push_str("    classDef balancing fill:#FED7D7,stroke:#E53E3E,stroke-width:2px\n");
    out.push_str("    classDef positive fill:#90EE90,stroke:#2E8B57,stroke-width:2px\n");
    out.push_str("    classDef negative fill:#FFB6C1,stroke:#E53E3E,stroke-width:2px\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_language::ast::{CausalRelationship, FeedbackLoopType, QualifiedIdent, Relation};

    #[test]
    fn test_feedback_loop_diagram() {
        let loop_data = FeedbackLoop {
            location: sruja_diagnostics::SourceLocation::new(String::new(), 0, 0),
            id: "R1".to_string(),
            loop_type: FeedbackLoopType::Reinforcing,
            loop_id: Some("R1".to_string()),
            title: "Growth Loop".to_string(),
            description: None,
            relationships: vec![
                Relation {
                    location: sruja_diagnostics::SourceLocation::new(String::new(), 0, 0),
                    from: QualifiedIdent::simple("A".to_string()),
                    to: QualifiedIdent::simple("B".to_string()),
                    label: Some("leads to".to_string()),
                    description: None,
                    technology: None,
                    tags: vec![],
                    body: None,
                },
                Relation {
                    location: sruja_diagnostics::SourceLocation::new(String::new(), 0, 0),
                    from: QualifiedIdent::simple("B".to_string()),
                    to: QualifiedIdent::simple("C".to_string()),
                    label: Some("causes".to_string()),
                    description: None,
                    technology: None,
                    tags: vec![],
                    body: None,
                },
            ],
        };

        let out = feedback_loop_to_diagram(&loop_data);
        assert!(out.contains("graph LR"));
        assert!(out.contains("Feedback Loop"));
        assert!(out.contains("Type: reinforcing (+)"));
        assert!(out.contains("A"));
        assert!(out.contains("B"));
        assert!(out.contains("C"));
        assert!(out.contains("leads to"));
        assert!(out.contains("causes"));
    }

    #[test]
    fn test_causal_loop_diagram() {
        let loop_data = CausalLoop {
            location: sruja_diagnostics::SourceLocation::new(String::new(), 0, 0),
            id: "B1".to_string(),
            loop_type: FeedbackLoopType::Balancing,
            loop_id: Some("B1".to_string()),
            title: "Balancing Loop".to_string(),
            description: None,
            variables: vec![],
            relationships: vec![
                CausalRelationship {
                    from: "Pressure".to_string(),
                    to: "Effort".to_string(),
                    effect: Some("leads to".to_string()),
                    polarity: CausalPolarity::Positive,
                    delay: None,
                },
                CausalRelationship {
                    from: "Effort".to_string(),
                    to: "Result".to_string(),
                    effect: Some("increases".to_string()),
                    polarity: CausalPolarity::Positive,
                    delay: Some("1-2 weeks".to_string()),
                },
            ],
        };

        let out = causal_loop_to_diagram(&loop_data);
        assert!(out.contains("graph LR"));
        assert!(out.contains("Causal Loop"));
        assert!(out.contains("Type: balancing (-)"));
        assert!(out.contains("Pressure"));
        assert!(out.contains("Effort"));
        assert!(out.contains("Result"));
        assert!(out.contains("leads to (+)"));
        assert!(out.contains("|| 1-2 weeks"));
    }
}
