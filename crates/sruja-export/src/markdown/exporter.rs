//! Markdown exporter implementation (initial parity pass)
//!
//! Ports Go markdown exporter structure. Generates structured markdown
//! documents with sections for systems, persons, requirements, ADRs, etc.
//! When `include_mermaid_diagrams` is true, embeds Mermaid code blocks so
//! diagrams render in GitHub, VS Code, and other Markdown viewers.

use std::collections::HashMap;

use sruja_language::{collect_elements, ElementKind, Program, TopLevelItem};

use crate::mermaid::exporter::{MermaidConfig, MermaidExporter};
use crate::mermaid::scenario_to_sequence_diagram;

use super::options::MarkdownOptions;

/// C4 container-level kinds: container, database, queue, datastore (L2 nodes under a system).
fn is_container_level(kind: &ElementKind) -> bool {
    matches!(
        kind,
        ElementKind::Container
            | ElementKind::Database
            | ElementKind::Queue
            | ElementKind::DataStore
    )
}

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

        // Systems with FQN for L2/L3 diagram targeting
        let systems_with_fqn: Vec<(String, &sruja_language::ElementDef)> = elements
            .iter()
            .filter(|(_, e)| e.assignment.kind == ElementKind::System)
            .map(|(fqn, e)| (fqn.clone(), e))
            .collect();
        let systems: Vec<_> = systems_with_fqn.iter().map(|(_, e)| *e).collect();
        let persons: Vec<_> = elements
            .values()
            .filter(|e| e.assignment.kind == ElementKind::Person)
            .collect();

        let mut requirements: Vec<_> = Vec::new();
        let mut adrs: Vec<_> = Vec::new();
        let mut scenario_items: Vec<(String, String, &[sruja_language::ScenarioStep])> = Vec::new();

        for item in &program.items {
            match item {
                TopLevelItem::Requirement(r) => requirements.push(r),
                TopLevelItem::Adr(a) => adrs.push(a),
                TopLevelItem::Scenario(s) => {
                    scenario_items.push((s.id.clone(), s.title.clone(), s.steps.as_slice()));
                }
                TopLevelItem::Flow(f) => {
                    scenario_items.push((f.id.clone(), f.title.clone(), f.steps.as_slice()));
                }
                _ => {}
            }
        }

        // Write TOC
        if self.options.include_toc {
            self.write_toc(
                &mut out,
                &systems,
                &persons,
                &requirements,
                &adrs,
                !scenario_items.is_empty(),
            );
        }

        // Write overview
        if self.options.include_overview {
            self.write_overview(&mut out, program);
        }

        // Write systems (with L2 per system, L3 per container)
        if self.options.include_systems {
            self.write_systems(&mut out, program, &elements, &systems_with_fqn);
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

        // Write scenarios (with Mermaid sequence diagrams)
        if self.options.include_scenarios {
            self.write_scenarios(&mut out, &scenario_items);
        }

        out
    }

    fn write_toc(
        &self,
        out: &mut String,
        _systems: &[&sruja_language::ElementDef],
        _persons: &[&sruja_language::ElementDef],
        _requirements: &[&sruja_language::Requirement],
        _adrs: &[&sruja_language::Adr],
        has_scenarios: bool,
    ) {
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
        if has_scenarios {
            out.push_str("- [Scenarios](#scenarios)\n");
        }
        out.push('\n');
    }

    fn write_overview(&self, out: &mut String, program: &Program) {
        out.push_str("## Overview\n\n");

        if self.options.include_mermaid_diagrams {
            let config = MermaidConfig {
                view_level: 1,
                target_id: None,
                ..self.options.mermaid_config.clone()
            };
            let mermaid = MermaidExporter::new(config).export(program);
            if !mermaid.is_empty() {
                out.push_str("### Context diagram (L1)\n\n");
                out.push_str("```mermaid\n");
                out.push_str(&mermaid);
                if !mermaid.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("```\n\n");
            }
        }

        out.push_str(
            "_Architecture overview (descriptions) will be populated from overview blocks._\n\n",
        );
    }

    fn write_systems(
        &self,
        out: &mut String,
        program: &Program,
        elements: &HashMap<String, sruja_language::ElementDef>,
        systems_with_fqn: &[(String, &sruja_language::ElementDef)],
    ) {
        if systems_with_fqn.is_empty() {
            return;
        }
        out.push_str("## Systems\n\n");
        for (sys_fqn, sys) in systems_with_fqn {
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
                    out.push_str(&format!("**Technology:** {}\n\n", tech));
                }
            }
            // L2: system + its containers
            if self.options.include_mermaid_diagrams {
                let config = MermaidConfig {
                    view_level: 2,
                    target_id: Some(sys_fqn.clone()),
                    ..self.options.mermaid_config.clone()
                };
                let mermaid = MermaidExporter::new(config).export(program);
                if !mermaid.is_empty() {
                    out.push_str("#### Container diagram (L2)\n\n");
                    out.push_str("```mermaid\n");
                    out.push_str(&mermaid);
                    if !mermaid.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push_str("```\n\n");
                }
            }
            // Containers of this system (direct children that are container-level)
            let sys_segment_count = sys_fqn.split('.').count();
            let mut containers: Vec<_> = elements
                .iter()
                .filter(|(fqn, e)| {
                    fqn.starts_with(&format!("{}.", sys_fqn))
                        && fqn.split('.').count() == sys_segment_count + 1
                        && is_container_level(&e.assignment.kind)
                })
                .map(|(fqn, e)| (fqn.clone(), e))
                .collect();
            containers.sort_by(|a, b| a.0.cmp(&b.0));
            for (container_fqn, container_elem) in containers {
                let container_title = container_elem
                    .assignment
                    .title
                    .clone()
                    .unwrap_or_else(|| container_elem.assignment.name.clone());
                out.push_str(&format!("##### {}\n\n", container_title));
                if let Some(body) = &container_elem.assignment.body {
                    if let Some(desc) = &body.description {
                        out.push_str(&format!("{}\n\n", desc));
                    }
                    if let Some(tech) = &body.technology {
                        out.push_str(&format!("**Technology:** {}\n\n", tech));
                    }
                }
                // L3: container + its components
                if self.options.include_mermaid_diagrams {
                    let config = MermaidConfig {
                        view_level: 3,
                        target_id: Some(container_fqn),
                        ..self.options.mermaid_config.clone()
                    };
                    let mermaid = MermaidExporter::new(config).export(program);
                    if !mermaid.is_empty() {
                        out.push_str("###### Component diagram (L3)\n\n");
                        out.push_str("```mermaid\n");
                        out.push_str(&mermaid);
                        if !mermaid.ends_with('\n') {
                            out.push('\n');
                        }
                        out.push_str("```\n\n");
                    }
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
            let title = person
                .assignment
                .title
                .clone()
                .unwrap_or_else(|| person.assignment.name.clone());
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

    fn write_scenarios(
        &self,
        out: &mut String,
        scenario_items: &[(String, String, &[sruja_language::ScenarioStep])],
    ) {
        if scenario_items.is_empty() {
            return;
        }
        out.push_str("## Scenarios\n\n");
        for (id, title, steps) in scenario_items {
            out.push_str(&format!("### {}\n\n", title));
            if self.options.include_mermaid_diagrams && !steps.is_empty() {
                let seq = scenario_to_sequence_diagram(id, title, steps);
                if !seq.is_empty() {
                    out.push_str("```mermaid\n");
                    out.push_str(&seq);
                    if !seq.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push_str("```\n\n");
                }
            }
        }
    }
}
