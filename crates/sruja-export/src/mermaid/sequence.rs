//! Mermaid sequence diagram generation for scenarios and flows.
//!
//! Emits `sequenceDiagram` blocks so scenarios are depicted using Mermaid
//! sequence diagrams (participants and messages).

use sruja_language::ast::ScenarioStep;

/// Escape a string for use inside Mermaid participant/label (avoid quotes and newlines).
fn escape_mermaid(s: &str) -> String {
    s.replace(['"', '\n', '\r'], " ").trim().to_string()
}

/// Generate Mermaid sequence diagram for a scenario (id, title, steps).
pub fn scenario_to_sequence_diagram(_id: &str, _title: &str, steps: &[ScenarioStep]) -> String {
    scenario_steps_to_sequence_diagram(steps)
}

/// Generate Mermaid sequence diagram for a flow (same structure as scenario).
pub fn flow_to_sequence_diagram(_id: &str, _title: &str, steps: &[ScenarioStep]) -> String {
    scenario_steps_to_sequence_diagram(steps)
}

/// Build `sequenceDiagram` content from scenario steps.
/// Participants are derived from step `from`/`to`; each step becomes a message.
fn scenario_steps_to_sequence_diagram(steps: &[ScenarioStep]) -> String {
    let mut out = String::new();
    out.push_str("sequenceDiagram\n");

    // Collect participants in order of first appearance
    let mut participants: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for step in steps {
        if let Some(ref from) = step.from {
            let name = escape_mermaid(&from.as_string());
            if !name.is_empty() && seen.insert(name.clone()) {
                participants.push(name);
            }
        }
        if let Some(ref to) = step.to {
            let name = escape_mermaid(&to.as_string());
            if !name.is_empty() && seen.insert(name.clone()) {
                participants.push(name);
            }
        }
    }

    for p in &participants {
        out.push_str(&format!("    participant {} as {}\n", sanitize_id(p), p));
    }
    if !participants.is_empty() {
        out.push('\n');
    }

    for step in steps {
        let from = step
            .from
            .as_ref()
            .map(|q| escape_mermaid(&q.as_string()))
            .unwrap_or_else(|| "?".to_string());
        let to = step
            .to
            .as_ref()
            .map(|q| escape_mermaid(&q.as_string()))
            .unwrap_or_else(|| "?".to_string());
        let msg = step
            .description
            .as_deref()
            .map(escape_mermaid)
            .unwrap_or_else(|| "".to_string());
        if from.is_empty() || to.is_empty() {
            continue;
        }
        let from_id = sanitize_id(&from);
        let to_id = sanitize_id(&to);
        // Mermaid sequenceDiagram: ->> is solid line with arrowhead (not ->>>)
        if msg.is_empty() {
            out.push_str(&format!("    {}->>{}\n", from_id, to_id));
        } else {
            out.push_str(&format!("    {}->>{}: {}\n", from_id, to_id, msg));
        }
    }

    out
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_language::ast::QualifiedIdent;

    #[test]
    fn test_sequence_diagram_two_steps() {
        let steps = vec![
            ScenarioStep {
                from: Some(QualifiedIdent::simple("User".to_string())),
                to: Some(QualifiedIdent::simple("API".to_string())),
                description: Some("Login".to_string()),
                tags: vec![],
                order: None,
            },
            ScenarioStep {
                from: Some(QualifiedIdent::simple("API".to_string())),
                to: Some(QualifiedIdent::simple("DB".to_string())),
                description: Some("Validate".to_string()),
                tags: vec![],
                order: None,
            },
        ];
        let out = scenario_steps_to_sequence_diagram(&steps);
        assert!(out.contains("sequenceDiagram"));
        assert!(out.contains("participant"));
        assert!(out.contains("User"));
        assert!(out.contains("API"));
        assert!(out.contains("DB"));
        assert!(out.contains("Login"));
        assert!(out.contains("Validate"));
    }
}
