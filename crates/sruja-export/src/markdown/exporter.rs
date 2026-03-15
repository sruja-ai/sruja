//! Markdown exporter implementation (initial parity pass)
//!
//! Ports Go markdown exporter structure. Generates structured markdown
//! documents with sections for systems, persons, requirements, ADRs, etc.
//! When `include_mermaid_diagrams` is true, embeds Mermaid code blocks so
//! diagrams render in GitHub, VS Code, and other Markdown viewers.

use std::collections::HashMap;

use sruja_language::{
    collect_elements, ConstraintsBlock, ConventionsBlock, ElementKind, OverviewBlock, Policy,
    Program, TopLevelItem,
};

use crate::mermaid::exporter::{MermaidConfig, MermaidExporter};
use crate::mermaid::feedback_loops::{causal_loop_to_diagram, feedback_loop_to_diagram};
use crate::mermaid::scenario_to_sequence_diagram;
use crate::mermaid::views::{collect_views, resolve_view, ResolvedView};

use super::escape::{escape_heading, escape_inline};
use super::options::MarkdownOptions;

/// Write element metadata as Markdown key-value lines when options.include_metadata is true.
fn write_element_metadata_if(
    out: &mut String,
    include_metadata: bool,
    metadata: &[sruja_language::MetaEntry],
) {
    if !include_metadata || metadata.is_empty() {
        return;
    }
    for entry in metadata {
        if let Some(ref v) = entry.value {
            if !v.is_empty() {
                out.push_str(&format!(
                    "**{}:** {}\n\n",
                    escape_inline(&entry.key),
                    escape_inline(v)
                ));
            }
        }
    }
}

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

        let (elements, relations) = collect_elements(program);

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
        let mut policies: Vec<&Policy> = Vec::new();
        let mut scenario_items: Vec<(
            String,
            String,
            Option<String>,
            &[sruja_language::ScenarioStep],
        )> = Vec::new();
        let mut feedback_loops: Vec<_> = Vec::new();
        let mut causal_loops: Vec<_> = Vec::new();
        let mut overview_block: Option<&OverviewBlock> = None;
        let mut constraints_block: Option<&ConstraintsBlock> = None;
        let mut conventions_block: Option<&ConventionsBlock> = None;
        let mut deployments: Vec<&sruja_language::DeploymentNode> = Vec::new();

        for item in &program.items {
            match item {
                TopLevelItem::Requirement(r) => requirements.push(r),
                TopLevelItem::Adr(a) => adrs.push(a),
                TopLevelItem::Policy(p) => policies.push(p),
                TopLevelItem::Overview(o) => overview_block = Some(o),
                TopLevelItem::Constraints(c) => constraints_block = Some(c),
                TopLevelItem::Conventions(c) => conventions_block = Some(c),
                TopLevelItem::Deployment(d) => deployments.push(d),
                TopLevelItem::Scenario(s) => {
                    scenario_items.push((
                        s.id.clone(),
                        s.title.clone(),
                        s.description.clone(),
                        s.steps.as_slice(),
                    ));
                }
                TopLevelItem::Flow(f) => {
                    scenario_items.push((
                        f.id.clone(),
                        f.title.clone(),
                        f.description.clone(),
                        f.steps.as_slice(),
                    ));
                }
                TopLevelItem::FeedbackLoop(fl) => feedback_loops.push(fl),
                TopLevelItem::CausalLoop(cl) => causal_loops.push(cl),
                _ => {}
            }
        }

        // Document title (professional architecture docs start with a clear title)
        let title = self
            .options
            .document_title
            .clone()
            .or_else(|| {
                overview_block.and_then(|o| {
                    o.summary
                        .as_ref()
                        .map(|s| s.lines().next().unwrap_or(s).trim().to_string())
                })
            })
            .unwrap_or_else(|| "Architecture Overview".to_string());

        // View-driven: single named view
        if self.options.use_views {
            if let Some(ref name) = self.options.view_name {
                let views = collect_views(program);
                if let Some(view) = views.iter().find(|v| v.id == *name) {
                    let resolved = resolve_view(view, &elements, &relations);
                    return self.export_single_view(&title, &resolved);
                }
                // View not found: fall back to full document below
            }
        }

        let mut out = String::new();
        out.push_str(&format!("# {}\n\n", escape_heading(&title)));

        // Table of contents
        if self.options.include_toc {
            let has_custom_views = self.options.use_views
                && self.options.include_all_views
                && !collect_views(program).is_empty();
            let has_relations = self.options.include_relations && !relations.is_empty();
            self.write_toc(
                &mut out,
                &systems,
                &persons,
                &requirements,
                &adrs,
                &policies,
                constraints_block.is_some(),
                conventions_block.is_some(),
                !scenario_items.is_empty(),
                !feedback_loops.is_empty(),
                !causal_loops.is_empty(),
                self.options.include_deployments && !deployments.is_empty(),
                has_relations,
                has_custom_views,
            );
        }

        // 1. Introduction & context (arc42-aligned)
        if self.options.include_overview {
            self.write_overview(&mut out, program, overview_block);
        }

        if self.options.include_persons {
            self.write_stakeholders(&mut out, &persons);
        }

        // 2. Building blocks (C4 systems / containers / components)
        if self.options.include_systems {
            self.write_systems(&mut out, program, &elements, &systems_with_fqn);
        }

        // 3. Deployment view
        if self.options.include_deployments && !deployments.is_empty() {
            self.write_deployments(&mut out, &deployments);
        }

        // Optional relations list
        if self.options.include_relations && !relations.is_empty() {
            self.write_relations(&mut out, &relations);
        }

        // 4. Runtime view (scenarios and flows)
        if self.options.include_scenarios {
            self.write_scenarios(&mut out, &scenario_items);
        }

        // 5. Requirements & architectural decisions
        if self.options.include_requirements {
            self.write_requirements(&mut out, &requirements);
        }
        if self.options.include_adrs {
            self.write_adrs(&mut out, &adrs);
        }

        // 6. Governance (policies, constraints, conventions)
        if !policies.is_empty() {
            self.write_policies(&mut out, &policies);
        }
        if let Some(c) = constraints_block {
            self.write_constraints(&mut out, c);
        }
        if let Some(c) = conventions_block {
            self.write_conventions(&mut out, c);
        }

        // 7. Analysis (feedback and causal loops)
        if !feedback_loops.is_empty() {
            self.write_feedback_loops(&mut out, &feedback_loops);
        }
        if !causal_loops.is_empty() {
            self.write_causal_loops(&mut out, &causal_loops);
        }

        // Glossary and Recommendations (stub sections when options set)
        if self.options.include_glossary {
            out.push_str("## Glossary\n\n_To be populated._\n\n");
        }
        if self.options.include_recommendations {
            out.push_str("## Recommendations\n\n_To be populated._\n\n");
        }

        // 8. Custom views (when use_views and include_all_views)
        if self.options.use_views && self.options.include_all_views {
            self.write_custom_views_section(&mut out, program, &elements, &relations);
        }

        out
    }

    /// Emit a reduced document for a single resolved view: title, optional TOC, view title/description, one Mermaid diagram, optional element list.
    fn export_single_view(&self, title: &str, resolved: &ResolvedView) -> String {
        let mut out = String::new();
        out.push_str(&format!("# {}\n\n", escape_heading(title)));
        if self.options.include_toc {
            out.push_str("- [View](#view)\n\n");
        }
        out.push_str("## View\n\n");
        out.push_str(&format!(
            "### {}\n\n",
            escape_heading(&resolved.title)
        ));
        if let Some(ref desc) = resolved.description {
            if !desc.is_empty() {
                out.push_str(&format!("{}\n\n", escape_inline(desc)));
            }
        }
        if self.options.include_mermaid_diagrams && !resolved.elements.is_empty() {
            let mermaid =
                MermaidExporter::new(self.options.mermaid_config.clone())
                    .export_from_resolved_view(resolved);
            if !mermaid.is_empty() {
                out.push_str("```mermaid\n");
                out.push_str(&mermaid);
                if !mermaid.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("```\n\n");
            }
        }
        if self.options.include_metadata && !resolved.elements.is_empty() {
            out.push_str("#### Elements in this view\n\n");
            let mut fqns: Vec<_> = resolved.elements.keys().collect();
            fqns.sort();
            for fqn in fqns {
                let elem = &resolved.elements[fqn];
                let label = elem
                    .assignment
                    .title
                    .clone()
                    .unwrap_or_else(|| elem.assignment.name.clone());
                out.push_str(&format!("- {} ({})\n", escape_inline(&label), fqn));
            }
            out.push('\n');
        }
        out
    }

    /// Append a "Custom views" section with one subsection per defined view (title, description, Mermaid).
    fn write_custom_views_section(
        &self,
        out: &mut String,
        program: &Program,
        elements: &HashMap<String, sruja_language::ElementDef>,
        relations: &[sruja_language::Relation],
    ) {
        let views = collect_views(program);
        if views.is_empty() {
            return;
        }
        out.push_str("## Custom views\n\n");
        for view in &views {
            let resolved = resolve_view(view, elements, relations);
            if resolved.elements.is_empty() {
                continue;
            }
            out.push_str(&format!(
                "### {}\n\n",
                escape_heading(&resolved.title)
            ));
            if let Some(ref desc) = resolved.description {
                if !desc.is_empty() {
                    out.push_str(&format!("{}\n\n", escape_inline(desc)));
                }
            }
            if self.options.include_mermaid_diagrams {
                let mermaid = MermaidExporter::new(self.options.mermaid_config.clone())
                    .export_from_resolved_view(&resolved);
                if !mermaid.is_empty() {
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

    #[allow(clippy::too_many_arguments)]
    fn write_toc(
        &self,
        out: &mut String,
        _systems: &[&sruja_language::ElementDef],
        _persons: &[&sruja_language::ElementDef],
        _requirements: &[&sruja_language::Requirement],
        _adrs: &[&sruja_language::Adr],
        policies: &[&Policy],
        has_constraints: bool,
        has_conventions: bool,
        has_scenarios: bool,
        has_feedback_loops: bool,
        has_causal_loops: bool,
        has_deployments: bool,
        has_relations: bool,
        has_custom_views: bool,
    ) {
        out.push_str("## Table of Contents\n\n");
        if self.options.include_overview {
            out.push_str("- [Overview](#overview)\n");
        }
        if self.options.include_persons {
            out.push_str("- [Stakeholders](#stakeholders)\n");
        }
        if self.options.include_systems {
            out.push_str("- [Systems](#systems)\n");
        }
        if has_deployments {
            out.push_str("- [Deployments](#deployments)\n");
        }
        if has_relations {
            out.push_str("- [Relations](#relations)\n");
        }
        if has_scenarios {
            out.push_str("- [Scenarios](#scenarios)\n");
        }
        if self.options.include_requirements {
            out.push_str("- [Requirements](#requirements)\n");
        }
        if self.options.include_adrs {
            out.push_str("- [Architecture Decision Records](#architecture-decision-records)\n");
        }
        if !policies.is_empty() {
            out.push_str("- [Policies](#policies)\n");
        }
        if has_constraints {
            out.push_str("- [Constraints](#constraints)\n");
        }
        if has_conventions {
            out.push_str("- [Conventions](#conventions)\n");
        }
        if has_feedback_loops {
            out.push_str("- [Feedback Loops](#feedback-loops)\n");
        }
        if has_causal_loops {
            out.push_str("- [Causal Loops](#causal-loops)\n");
        }
        if self.options.include_glossary {
            out.push_str("- [Glossary](#glossary)\n");
        }
        if self.options.include_recommendations {
            out.push_str("- [Recommendations](#recommendations)\n");
        }
        if has_custom_views {
            out.push_str("- [Custom views](#custom-views)\n");
        }
        out.push('\n');
    }

    fn write_overview(
        &self,
        out: &mut String,
        program: &Program,
        overview_block: Option<&OverviewBlock>,
    ) {
        out.push_str("## Overview\n\n");

        if let Some(ov) = overview_block {
            if let Some(summary) = &ov.summary {
                out.push_str(&format!("{}\n\n", escape_inline(summary)));
            }
            if let Some(audience) = &ov.audience {
                out.push_str(&format!("**Audience:** {}\n\n", escape_inline(audience)));
            }
            if let Some(scope) = &ov.scope {
                out.push_str(&format!("**Scope:** {}\n\n", escape_inline(scope)));
            }
            if !ov.goals.is_empty() {
                out.push_str("**Goals:**\n\n");
                for g in &ov.goals {
                    out.push_str(&format!("- {}\n", escape_inline(g)));
                }
                out.push('\n');
            }
            if !ov.non_goals.is_empty() {
                out.push_str("**Non-goals:**\n\n");
                for ng in &ov.non_goals {
                    out.push_str(&format!("- {}\n", escape_inline(ng)));
                }
                out.push('\n');
            }
            if !ov.risks.is_empty() {
                out.push_str("**Risks:**\n\n");
                for r in &ov.risks {
                    out.push_str(&format!("- {}\n", escape_inline(r)));
                }
                out.push('\n');
            }
        } else {
            out.push_str(
                "_Add an `overview { summary \"...\"; audience \"...\"; scope \"...\" }` block to populate this section._\n\n",
            );
        }

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
            out.push_str(&format!("### {}\n\n", escape_heading(&title)));
            if let Some(body) = &sys.assignment.body {
                if let Some(desc) = &body.description {
                    out.push_str(&format!("{}\n\n", escape_inline(desc)));
                }
                if let Some(tech) = &body.technology {
                    out.push_str(&format!("**Technology:** {}\n\n", escape_inline(tech)));
                }
                write_element_metadata_if(out, self.options.include_metadata, &body.metadata);
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
                out.push_str(&format!("##### {}\n\n", escape_heading(&container_title)));
                if let Some(body) = &container_elem.assignment.body {
                    if let Some(desc) = &body.description {
                        out.push_str(&format!("{}\n\n", escape_inline(desc)));
                    }
                    if let Some(tech) = &body.technology {
                        out.push_str(&format!("**Technology:** {}\n\n", escape_inline(tech)));
                    }
                    write_element_metadata_if(out, self.options.include_metadata, &body.metadata);
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

    fn write_stakeholders(&self, out: &mut String, persons: &[&sruja_language::ElementDef]) {
        if persons.is_empty() {
            return;
        }
        out.push_str("## Stakeholders\n\n");
        for person in persons {
            let title = person
                .assignment
                .title
                .clone()
                .unwrap_or_else(|| person.assignment.name.clone());
            out.push_str(&format!("### {}\n\n", escape_heading(&title)));
            if let Some(body) = &person.assignment.body {
                if let Some(desc) = &body.description {
                    out.push_str(&format!("{}\n\n", escape_inline(desc)));
                }
                write_element_metadata_if(out, self.options.include_metadata, &body.metadata);
            }
        }
    }

    fn write_requirements(&self, out: &mut String, requirements: &[&sruja_language::Requirement]) {
        if requirements.is_empty() {
            return;
        }
        out.push_str("## Requirements\n\n");
        for req in requirements {
            out.push_str(&format!("### {}\n\n", escape_heading(&req.title)));
            if req.id != req.title && !req.id.is_empty() {
                out.push_str(&format!("**ID:** {}\n\n", escape_inline(&req.id)));
            }
            out.push_str(&format!("**Type:** {}\n\n", escape_inline(&req.r#type)));
            if let Some(desc) = &req.description {
                out.push_str(&format!("{}\n\n", escape_inline(desc)));
            }
            if !req.tags.is_empty() {
                out.push_str(&format!(
                    "**Tags:** {}\n\n",
                    escape_inline(&req.tags.join(", "))
                ));
            }
        }
    }

    fn write_adrs(&self, out: &mut String, adrs: &[&sruja_language::Adr]) {
        if adrs.is_empty() {
            return;
        }
        out.push_str("## Architecture Decision Records\n\n");
        for adr in adrs {
            out.push_str(&format!("### {}\n\n", escape_heading(&adr.title)));
            if !adr.id.is_empty() {
                out.push_str(&format!("**ID:** {}\n\n", escape_inline(&adr.id)));
            }
            if let Some(status) = &adr.status {
                out.push_str(&format!("**Status:** {}\n\n", escape_inline(status)));
            }
            if let Some(context) = &adr.context {
                out.push_str(&format!("**Context:** {}\n\n", escape_inline(context)));
            }
            if let Some(decision) = &adr.decision {
                out.push_str(&format!("**Decision:** {}\n\n", escape_inline(decision)));
            }
            if let Some(consequences) = &adr.consequences {
                out.push_str(&format!(
                    "**Consequences:** {}\n\n",
                    escape_inline(consequences)
                ));
            }
        }
    }

    fn write_policies(&self, out: &mut String, policies: &[&Policy]) {
        if policies.is_empty() {
            return;
        }
        out.push_str("## Policies\n\n");
        for policy in policies {
            out.push_str(&format!("### {}\n\n", escape_heading(&policy.title)));
            if !policy.id.is_empty() {
                out.push_str(&format!("**ID:** {}\n\n", escape_inline(&policy.id)));
            }
            out.push_str(&format!(
                "**Category:** {}\n\n",
                escape_inline(&policy.category)
            ));
            out.push_str(&format!(
                "**Enforcement:** {}\n\n",
                escape_inline(&policy.enforcement)
            ));
            if let Some(desc) = &policy.description {
                out.push_str(&format!("{}\n\n", escape_inline(desc)));
            }
        }
    }

    fn write_constraints(&self, out: &mut String, constraints: &ConstraintsBlock) {
        if constraints.entries.is_empty() {
            return;
        }
        out.push_str("## Constraints\n\n");
        for entry in &constraints.entries {
            out.push_str(&format!("- {}\n", escape_inline(&entry.value)));
        }
        out.push('\n');
    }

    fn write_conventions(&self, out: &mut String, conventions: &ConventionsBlock) {
        if conventions.entries.is_empty() {
            return;
        }
        out.push_str("## Conventions\n\n");
        for entry in &conventions.entries {
            out.push_str(&format!("- {}\n", escape_inline(&entry.value)));
        }
        out.push('\n');
    }

    fn write_deployments(&self, out: &mut String, deployments: &[&sruja_language::DeploymentNode]) {
        if deployments.is_empty() {
            return;
        }
        out.push_str("## Deployments\n\n");
        for dep in deployments {
            self.write_deployment_node(out, dep, 0);
        }
    }

    fn write_deployment_node(
        &self,
        out: &mut String,
        node: &sruja_language::DeploymentNode,
        depth: usize,
    ) {
        let heading = "#".repeat((3 + depth).min(6));
        let title = node.label.as_deref().unwrap_or(node.id.as_str());
        out.push_str(&format!("{} {}\n\n", heading, escape_heading(title)));
        if let Some(tech) = &node.technology {
            out.push_str(&format!("**Technology:** {}\n\n", escape_inline(tech)));
        }
        if !node.children.is_empty() {
            for child in &node.children {
                self.write_deployment_node(out, child, depth + 1);
            }
        }
    }

    fn write_relations(&self, out: &mut String, relations: &[sruja_language::Relation]) {
        if relations.is_empty() {
            return;
        }
        out.push_str("## Relations\n\n");
        for rel in relations {
            let from_s = rel.from.as_string();
            let to_s = rel.to.as_string();
            let label = rel.label.as_deref().unwrap_or("");
            out.push_str(&format!(
                "- {} → {} \"{}\"\n",
                escape_inline(&from_s),
                escape_inline(&to_s),
                escape_inline(label)
            ));
        }
        out.push('\n');
    }

    fn write_scenarios(
        &self,
        out: &mut String,
        scenario_items: &[(
            String,
            String,
            Option<String>,
            &[sruja_language::ScenarioStep],
        )],
    ) {
        if scenario_items.is_empty() {
            return;
        }
        out.push_str("## Scenarios\n\n");
        for (id, title, description, steps) in scenario_items {
            out.push_str(&format!("### {}\n\n", escape_heading(title)));
            if let Some(ref desc) = description {
                if !desc.is_empty() {
                    out.push_str(&format!("{}\n\n", escape_inline(desc)));
                }
            }
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

    fn write_feedback_loops(
        &self,
        out: &mut String,
        feedback_loops: &[&sruja_language::FeedbackLoop],
    ) {
        if feedback_loops.is_empty() {
            return;
        }
        out.push_str("## Feedback Loops\n\n");
        for fl in feedback_loops {
            out.push_str(&format!("### {}\n\n", escape_heading(&fl.title)));
            if !fl.id.is_empty() {
                out.push_str(&format!("**ID:** {}\n\n", escape_inline(&fl.id)));
            }
            if let Some(ref loop_id) = fl.loop_id {
                out.push_str(&format!("**Loop ID:** {}\n\n", escape_inline(loop_id)));
            }
            out.push_str(&format!(
                "**Type:** {} ({})\n\n",
                escape_inline(&fl.loop_type.to_string()),
                fl.loop_type.to_symbol()
            ));
            if let Some(ref description) = fl.description {
                out.push_str(&format!(
                    "**Description:** {}\n\n",
                    escape_inline(description)
                ));
            }
            if self.options.include_mermaid_diagrams && !fl.relationships.is_empty() {
                let diagram = feedback_loop_to_diagram(fl);
                out.push_str("```mermaid\n");
                out.push_str(&diagram);
                if !diagram.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("```\n\n");
            }
        }
    }

    fn write_causal_loops(&self, out: &mut String, causal_loops: &[&sruja_language::CausalLoop]) {
        if causal_loops.is_empty() {
            return;
        }
        out.push_str("## Causal Loops\n\n");
        for cl in causal_loops {
            out.push_str(&format!("### {}\n\n", escape_heading(&cl.title)));
            if !cl.id.is_empty() {
                out.push_str(&format!("**ID:** {}\n\n", escape_inline(&cl.id)));
            }
            if let Some(ref loop_id) = cl.loop_id {
                out.push_str(&format!("**Loop ID:** {}\n\n", escape_inline(loop_id)));
            }
            out.push_str(&format!(
                "**Type:** {} ({})\n\n",
                escape_inline(&cl.loop_type.to_string()),
                cl.loop_type.to_symbol()
            ));
            if let Some(ref description) = cl.description {
                out.push_str(&format!(
                    "**Description:** {}\n\n",
                    escape_inline(description)
                ));
            }
            if !cl.variables.is_empty() {
                out.push_str("**Variables:**\n\n");
                for v in &cl.variables {
                    let label = v.label.as_deref().unwrap_or(v.id.as_str());
                    out.push_str(&format!(
                        "- {} ({})\n",
                        escape_inline(&v.id),
                        escape_inline(label)
                    ));
                }
                out.push('\n');
            }
            if self.options.include_mermaid_diagrams && !cl.relationships.is_empty() {
                let diagram = causal_loop_to_diagram(cl);
                out.push_str("```mermaid\n");
                out.push_str(&diagram);
                if !diagram.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("```\n\n");
            }
        }
    }
}
