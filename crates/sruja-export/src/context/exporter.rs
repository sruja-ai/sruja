//! Context exporter implementation (initial parity pass)
//!
//! Ports Go context exporter for AI-friendly architecture descriptions.
//! Generates structured text optimized for LLM consumption.

use sruja_language::{collect_elements, ElementKind, Program};

pub struct ContextExporter {
    pub template: String, // "proposal", "security", "general"
}

impl ContextExporter {
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }

    pub fn export(&self, program: &Program) -> String {
        if program.items.is_empty() {
            return String::new();
        }

        let (elements, relations) = collect_elements(program);
        let mut out = String::new();

        // Extract sections
        let systems: Vec<_> = elements
            .values()
            .filter(|e| e.assignment.kind == ElementKind::System)
            .collect();
        let persons: Vec<_> = elements
            .values()
            .filter(|e| e.assignment.kind == ElementKind::Person)
            .collect();

        // Write header based on template
        match self.template.as_str() {
            "proposal" => {
                out.push_str("# Architecture Proposal\n\n");
                out.push_str("This document describes the proposed system architecture.\n\n");
            }
            "security" => {
                out.push_str("# Security Architecture Review\n\n");
                out.push_str(
                    "This document provides a security-focused view of the architecture.\n\n",
                );
            }
            _ => {
                out.push_str("# System Architecture Context\n\n");
            }
        }

        // Write systems
        if !systems.is_empty() {
            out.push_str("## Systems\n\n");
            for sys in systems {
                let title = sys
                    .assignment
                    .title
                    .clone()
                    .unwrap_or_else(|| sys.assignment.name.clone());
                out.push_str(&format!("### {}\n\n", title));
                if let Some(body) = &sys.assignment.body {
                    if let Some(desc) = &body.description {
                        out.push_str(&format!("{}\n\n", desc));
                    }
                    if let Some(tech) = &body.technology {
                        out.push_str(&format!("Technology: {}\n\n", tech));
                    }
                }
            }
        }

        // Write persons
        if !persons.is_empty() {
            out.push_str("## Actors\n\n");
            for person in persons {
                let title = person
                    .assignment
                    .title
                    .clone()
                    .unwrap_or_else(|| person.assignment.name.clone());
                out.push_str(&format!("- **{}**: ", title));
                if let Some(body) = &person.assignment.body {
                    if let Some(desc) = &body.description {
                        out.push_str(desc);
                    }
                }
                out.push('\n');
            }
            out.push('\n');
        }

        // Write relations summary
        if !relations.is_empty() {
            out.push_str("## Key Relationships\n\n");
            for rel in relations.iter().take(10) {
                let from = rel.from.as_string();
                let to = rel.to.as_string();
                let label = rel.label.as_ref().or(rel.description.as_ref());
                match label {
                    Some(l) => out.push_str(&format!("- {} → {}: {}\n", from, to, l)),
                    None => out.push_str(&format!("- {} → {}\n", from, to)),
                }
            }
            if relations.len() > 10 {
                out.push_str(&format!(
                    "\n... and {} more relationships\n",
                    relations.len() - 10
                ));
            }
            out.push('\n');
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_language::Parser;

    #[test]
    fn export_empty_program_returns_empty_string() {
        let program = Program::default();
        let exporter = ContextExporter::new("general");
        assert!(exporter.export(&program).is_empty());
    }

    #[test]
    fn unknown_template_uses_general_header() {
        let program = Parser::new("test.sruja".to_string())
            .parse("S = system \"X\" {}")
            .expect("parse");
        let exporter = ContextExporter::new("custom_template");
        let out = exporter.export(&program);
        assert!(
            out.starts_with("# System Architecture Context"),
            "unknown template should fall back to general"
        );
    }
}
