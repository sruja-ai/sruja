//! Markdown exporter implementation (initial parity pass)
//!
//! Ports Go markdown exporter structure. Generates structured markdown
//! documents with sections for systems, persons, requirements, ADRs, etc.

use sruja_language::{collect_elements, ElementKind, Program};

use super::options::MarkdownOptions;

pub struct MarkdownExporter {
    pub options: MarkdownOptions,
}

impl MarkdownExporter {
    pub fn new(options: MarkdownOptions) -> Self {
        Self { options }
    }

    pub fn export(&self, program: &Program) -> String {
        if program.items.is_empty() {
            return String::new();
        }

        let (elements, _relations) = collect_elements(program);
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

        let mut requirements: Vec<_> = Vec::new();
        let mut adrs: Vec<_> = Vec::new();
        let mut scenarios: Vec<_> = Vec::new();

        for item in &program.items {
            match item {
                sruja_language::TopLevelItem::Requirement(r) => requirements.push(r),
                sruja_language::TopLevelItem::Adr(a) => adrs.push(a),
                sruja_language::TopLevelItem::Scenario(s) => scenarios.push(s),
                sruja_language::TopLevelItem::Flow(f) => scenarios.push(&f.id), // Treat flow as scenario
                _ => {}
            }
        }

        // Write TOC
        if self.options.include_toc {
            self.write_toc(&mut out, &systems, &persons, &requirements, &adrs);
        }

        // Write overview
        if self.options.include_overview {
            self.write_overview(&mut out, program);
        }

        // Write systems
        if self.options.include_systems {
            self.write_systems(&mut out, &systems);
        }

        // Write persons
        if self.options.include_persons {
            self.write_persons(&mut out, &persons);
        }

        // Write requirements
        if self.options.include_requirements {
            self.write_requirements(&mut out, &requirements);
        }

        // Write ADRs
        if self.options.include_adrs {
            self.write_adrs(&mut out, &adrs);
        }

        // Write scenarios
        if self.options.include_scenarios {
            self.write_scenarios(&mut out, &scenarios);
        }

        out
    }

    fn write_toc(&self, out: &mut String, _systems: &[&sruja_language::ElementDef], _persons: &[&sruja_language::ElementDef], _requirements: &[&sruja_language::Requirement], _adrs: &[&sruja_language::Adr]) {
        out.push_str("## Table of Contents\n\n");
        if self.options.include_overview {
            out.push_str("- [Overview](#overview)\n");
        }
        if self.options.include_systems {
            out.push_str("- [Systems](#systems)\n");
        }
        if self.options.include_persons {
            out.push_str("- [Persons](#persons)\n");
        }
        if self.options.include_requirements {
            out.push_str("- [Requirements](#requirements)\n");
        }
        if self.options.include_adrs {
            out.push_str("- [ADRs](#adrs)\n");
        }
        out.push('\n');
    }

    fn write_overview(&self, out: &mut String, _program: &Program) {
        out.push_str("## Overview\n\n");
        out.push_str("_Architecture overview will be populated from overview blocks._\n\n");
    }

    fn write_systems(&self, out: &mut String, systems: &[&sruja_language::ElementDef]) {
        if systems.is_empty() {
            return;
        }
        out.push_str("## Systems\n\n");
        for sys in systems {
            let title = sys.assignment.title.clone().unwrap_or_else(|| sys.assignment.name.clone());
            out.push_str(&format!("### {}\n\n", title));
            if let Some(body) = &sys.assignment.body {
                if let Some(desc) = &body.description {
                    out.push_str(&format!("{}\n\n", desc));
                }
                if let Some(tech) = &body.technology {
                    out.push_str(&format!("**Technology:** {}\n\n", tech));
                }
            }
        }
    }

    fn write_persons(&self, out: &mut String, persons: &[&sruja_language::ElementDef]) {
        if persons.is_empty() {
            return;
        }
        out.push_str("## Persons\n\n");
        for person in persons {
            let title = person.assignment.title.clone().unwrap_or_else(|| person.assignment.name.clone());
            out.push_str(&format!("### {}\n\n", title));
            if let Some(body) = &person.assignment.body {
                if let Some(desc) = &body.description {
                    out.push_str(&format!("{}\n\n", desc));
                }
            }
        }
    }

    fn write_requirements(&self, out: &mut String, requirements: &[&sruja_language::Requirement]) {
        if requirements.is_empty() {
            return;
        }
        out.push_str("## Requirements\n\n");
        for req in requirements {
            out.push_str(&format!("### {}\n\n", req.title));
            out.push_str(&format!("**Type:** {}\n\n", req.r#type));
            if let Some(desc) = &req.description {
                out.push_str(&format!("{}\n\n", desc));
            }
        }
    }

    fn write_adrs(&self, out: &mut String, adrs: &[&sruja_language::Adr]) {
        if adrs.is_empty() {
            return;
        }
        out.push_str("## Architecture Decision Records\n\n");
        for adr in adrs {
            out.push_str(&format!("### {}\n\n", adr.title));
            if let Some(status) = &adr.status {
                out.push_str(&format!("**Status:** {}\n\n", status));
            }
            if let Some(context) = &adr.context {
                out.push_str(&format!("**Context:** {}\n\n", context));
            }
            if let Some(decision) = &adr.decision {
                out.push_str(&format!("**Decision:** {}\n\n", decision));
            }
            if let Some(consequences) = &adr.consequences {
                out.push_str(&format!("**Consequences:** {}\n\n", consequences));
            }
        }
    }

    fn write_scenarios(&self, out: &mut String, _scenarios: &[&str]) {
        // TODO: Implement scenario writing
        out.push_str("## Scenarios\n\n");
        out.push_str("_Scenarios section will be populated._\n\n");
    }
}
