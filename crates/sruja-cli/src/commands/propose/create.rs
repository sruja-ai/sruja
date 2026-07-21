use super::CliError;
use super::types::{AddElementSpec, OutputFormat, ProposalLintSummary, ProposeCreateRequest};
use super::spec::{
    is_valid_identifier, kind_requires_description_str, kind_requires_technology_str,
    parse_add_element_spec, parse_add_relationship_spec, parse_remove_relationship_spec,
    root_ident, is_under_root,
};
use super::collect::{
    collect_program_top_level_elements, collect_program_element_fqns,
    collect_program_relationships, collect_program_defined_kinds, normalize_element_kind,
};
use crate::commands::parse_sruja_file;
use super::{derive_title, new_short_id_unique};
use sruja_diff::{Proposal, ProposalChange, ProposalStatus};
use sruja_engine::Validator;
use sruja_export::DslPrinter;
use sruja_intent::IntentModel;
use sruja_scan::scan_repo;
use sruja_diagnostics::format_diagnostic;
use std::collections::HashSet;
use std::path::Path;

pub async fn propose_create(
    repo_root: &str,
    request: ProposeCreateRequest,
) -> Result<(), CliError> {
    let ProposeCreateRequest {
        description,
        workflow_id,
        add_elements,
        add_relationships,
        remove_elements,
        remove_relationships,
        format,
    } = request;

    let repo_path = Path::new(repo_root);
    let sruja_file = repo_path.join("repo.sruja");

    let format = OutputFormat::parse(&format)?;

    let (_baseline_content, baseline_program) = if sruja_file.exists() {
        parse_sruja_file(&sruja_file)?
    } else {
        (String::new(), sruja_language::ast::Program::default())
    };

    let title = derive_title(&description);
    let proposal_id = new_short_id_unique(repo_path)?;
    let mut proposal = Proposal::new(proposal_id, title, description);
    proposal.workflow_id = workflow_id;

    let has_baseline = sruja_file.exists();
    let baseline_top_level_elements = collect_program_top_level_elements(&baseline_program);
    let baseline_element_fqns = collect_program_element_fqns(&baseline_program);
    let baseline_relationships = collect_program_relationships(&baseline_program);
    let baseline_defined_kinds = collect_program_defined_kinds(&baseline_program);

    if !has_baseline && !remove_elements.is_empty() {
        return Err(CliError::validation(
            "Cannot remove elements: repo.sruja does not exist yet.".to_string(),
        ));
    }
    if !has_baseline && !remove_relationships.is_empty() {
        return Err(CliError::validation(
            "Cannot remove relationships: repo.sruja does not exist yet.".to_string(),
        ));
    }

    let mut add_element_specs: Vec<AddElementSpec> = Vec::new();
    let mut add_element_ids: HashSet<String> = HashSet::new();
    for spec in add_elements {
        let parsed = parse_add_element_spec(&spec)?;
        let (id, kind, label, tech) = parsed;
        let id = id.clone();
        let kind = normalize_element_kind(&kind, has_baseline, &baseline_defined_kinds)?;
        let kind_lc = kind.trim().to_lowercase();
        if kind_requires_technology_str(&kind_lc) && tech.is_none() {
            return Err(CliError::validation(format!(
                "Element '{}' of kind '{}' is missing required technology. Use id:kind:label:tech",
                id, kind
            )));
        }
        if baseline_top_level_elements.contains(&id) {
            return Err(CliError::validation(format!(
                "Element '{}' already exists in repo.sruja",
                id
            )));
        }
        if !add_element_ids.insert(id.clone()) {
            return Err(CliError::validation(format!(
                "Element '{}' is specified more than once in --add-elements",
                id
            )));
        }
        let description = if kind_requires_description_str(&kind_lc) {
            Some(label.clone())
        } else {
            None
        };
        add_element_specs.push(AddElementSpec {
            id,
            kind,
            label,
            technology: tech,
            description,
        });
    }
    add_element_specs.sort_by(|a, b| a.id.cmp(&b.id));

    let mut remove_element_ids: HashSet<String> = HashSet::new();
    let mut remove_element_list: Vec<String> = Vec::new();
    for id in remove_elements {
        let id = id.trim().to_string();
        if id.is_empty() {
            return Err(CliError::validation(
                "remove-elements contains an empty id".to_string(),
            ));
        }
        if !is_valid_identifier(&id) {
            return Err(CliError::validation(format!(
                "Invalid remove-elements id '{}'. Expected an identifier (letters, digits, _, -).",
                id
            )));
        }
        if has_baseline && !baseline_top_level_elements.contains(&id) {
            return Err(CliError::validation(format!(
                "Element '{}' not found in repo.sruja",
                id
            )));
        }
        if !remove_element_ids.insert(id.clone()) {
            continue;
        }
        if add_element_ids.contains(&id) {
            return Err(CliError::validation(format!(
                "Element '{}' is both added and removed in the same proposal",
                id
            )));
        }
        remove_element_list.push(id);
    }
    remove_element_list.sort();

    for spec in &add_element_specs {
        proposal.changes.push(ProposalChange::AddElement {
            id: spec.id.clone(),
            kind: spec.kind.clone(),
            label: spec.label.clone(),
            technology: spec.technology.clone(),
            parent: None,
            description: spec.description.clone(),
        });
    }

    for id in &remove_element_list {
        proposal.changes.push(ProposalChange::RemoveElement {
            id: id.clone(),
            reason: None,
        });
    }

    let mut add_relationship_pairs: Vec<(String, String, Option<String>)> = Vec::new();
    let mut added_relationships: HashSet<(String, String)> = HashSet::new();
    for spec in add_relationships {
        let (source, target, label) = parse_add_relationship_spec(&spec)?;
        if baseline_relationships.contains(&(source.clone(), target.clone())) {
            return Err(CliError::validation(format!(
                "Relationship '{} -> {}' already exists in repo.sruja",
                source, target
            )));
        }
        if !added_relationships.insert((source.clone(), target.clone())) {
            continue;
        }

        if remove_element_ids.contains(root_ident(&source))
            || remove_element_ids.contains(root_ident(&target))
        {
            return Err(CliError::validation(format!(
                "Relationship '{} -> {}' references an element removed in this proposal",
                source, target
            )));
        }

        let source_known =
            baseline_element_fqns.contains(&source) || add_element_ids.contains(&source);
        let target_known =
            baseline_element_fqns.contains(&target) || add_element_ids.contains(&target);
        if !source_known || !target_known {
            return Err(CliError::validation(format!(
                "Relationship '{} -> {}' references unknown element(s). Ensure both ends exist in repo.sruja or are added in this proposal.",
                source, target
            )));
        }

        add_relationship_pairs.push((source, target, label));
    }
    add_relationship_pairs
        .sort_by(|a, b| (a.0.as_str(), a.1.as_str()).cmp(&(b.0.as_str(), b.1.as_str())));
    for (source, target, label) in add_relationship_pairs {
        proposal.changes.push(ProposalChange::AddRelationship {
            source,
            target,
            label,
            kind: None,
        });
    }

    let mut remove_relationship_set: HashSet<(String, String)> = HashSet::new();
    for spec in remove_relationships {
        let (source, target) = parse_remove_relationship_spec(&spec)?;
        if !baseline_relationships.contains(&(source.clone(), target.clone())) {
            return Err(CliError::validation(format!(
                "Relationship '{} -> {}' not found in repo.sruja",
                source, target
            )));
        }
        remove_relationship_set.insert((source, target));
    }

    if has_baseline && !remove_element_ids.is_empty() {
        for (source, target) in &baseline_relationships {
            let touches_removed = remove_element_ids
                .iter()
                .any(|id| is_under_root(source, id) || is_under_root(target, id));
            if touches_removed {
                remove_relationship_set.insert((source.clone(), target.clone()));
            }
        }
    }

    for (source, target) in &remove_relationship_set {
        if added_relationships.contains(&(source.clone(), target.clone())) {
            return Err(CliError::validation(format!(
                "Relationship '{} -> {}' is both added and removed in the same proposal",
                source, target
            )));
        }
    }

    let mut remove_relationship_list: Vec<(String, String)> =
        remove_relationship_set.into_iter().collect();
    remove_relationship_list
        .sort_by(|a, b| (a.0.as_str(), a.1.as_str()).cmp(&(b.0.as_str(), b.1.as_str())));
    for (source, target) in remove_relationship_list {
        proposal.changes.push(ProposalChange::RemoveRelationship {
            source,
            target,
            reason: None,
        });
    }

    if proposal.changes.is_empty() {
        return Err(CliError::validation(
            "Proposal has no changes. Pass --add-elements / --add-relationships / --remove-elements / --remove-relationships."
                .to_string(),
        ));
    }

    // 2. Validate against current graph + tribal knowledge
    let graph = scan_repo(repo_path)?;
    let intent_model = IntentModel::default();

    let validation = proposal.validate(&graph, &intent_model);
    let lint = lint_proposal_against_program(&proposal, &baseline_program)?;
    proposal.validation = Some(validation.clone());
    proposal.status = if validation.is_valid {
        ProposalStatus::Pending
    } else {
        ProposalStatus::Draft
    };
    if lint.error_count > 0 {
        proposal.status = ProposalStatus::Draft;
    }

    // 3. Save
    let file_path = proposal
        .save(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    match format {
        OutputFormat::Text => {
            println!("Proposal: {}", proposal.id);
            println!("Status: {:?}", proposal.status);
            if let Some(w) = proposal.workflow_id.as_deref() {
                println!("Workflow: {}", w);
            }
            println!("Description: {}", proposal.description);
            println!();
            println!("Changes:");
            for change in &proposal.changes {
                match change {
                    ProposalChange::AddElement {
                        id, kind, label, ..
                    } => println!("  + {} = {} \"{}\"", id, kind, label),
                    ProposalChange::AddRelationship {
                        source,
                        target,
                        label,
                        ..
                    } => {
                        if let Some(l) = label.as_deref() {
                            println!("  + {} -> {} \"{}\"", source, target, l);
                        } else {
                            println!("  + {} -> {}", source, target);
                        }
                    }
                    ProposalChange::RemoveElement { id, .. } => println!("  - {}", id),
                    ProposalChange::RemoveRelationship { source, target, .. } => {
                        println!("  - {} -> {}", source, target)
                    }
                    ProposalChange::ModifyElement { id, field, .. } => {
                        println!("  ~ {} ({})", id, field)
                    }
                }
            }
            println!();
            println!("Validation:");
            println!("  is_valid: {}", validation.is_valid);
            println!(
                "  policy_violations: {}",
                validation.policy_violations.len()
            );
            println!("  tribal_warnings: {}", validation.tribal_warnings.len());
            println!("  suggestions: {}", validation.suggestions.len());
            println!("  blast_radius: {}", validation.blast_radius.len());
            if !validation.policy_violations.is_empty() {
                println!();
                println!("Policy violations:");
                for v in &validation.policy_violations {
                    println!("  {}", v);
                }
            }
            if !validation.tribal_warnings.is_empty() {
                println!();
                println!("Tribal warnings:");
                for w in &validation.tribal_warnings {
                    println!("  {}", w);
                }
            }
            if !validation.suggestions.is_empty() {
                println!();
                println!("Suggestions:");
                for s in &validation.suggestions {
                    println!("  {}", s);
                }
            }
            if lint.error_count > 0 {
                println!();
                println!("Lint errors (showing up to 20):");
                for diag in lint.errors.iter().take(20) {
                    println!("  {}", diag);
                }
            }
            println!();
            println!("Wrote: {}", file_path.display());
            println!("Next: sruja propose approve {}", proposal.id);
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "proposal_create/v1",
                    "proposal": proposal,
                    "validation": validation,
                    "lint": {
                        "error_count": lint.error_count,
                        "errors": lint.errors,
                    },
                    "wrote": file_path,
                }))?
            );
        }
    }

    Ok(())
}

pub(super) fn lint_proposal_against_program(
    proposal: &Proposal,
    baseline_program: &sruja_language::ast::Program,
) -> Result<ProposalLintSummary, CliError> {
    let updated_program = proposal
        .apply(baseline_program)
        .map_err(|e| CliError::validation(e.to_string()))?;
    let updated_dsl = DslPrinter::new().print(&updated_program);

    let parser = sruja_language::Parser::new("repo.sruja.proposal".to_string());
    let program = parser.parse(&updated_dsl).map_err(|diags| {
        CliError::parse_with_diagnostics("repo.sruja.proposal".to_string(), diags)
    })?;
    let validator = Validator::with_default_rules();
    let mut diagnostics = validator.validate_sync(&program);
    crate::modules::validation::enrich_diagnostics_with_source(&updated_dsl, &mut diagnostics);

    let mut errors: Vec<String> = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
        .map(format_diagnostic)
        .collect();
    errors.truncate(20);

    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
        .count();

    Ok(ProposalLintSummary {
        error_count,
        errors,
    })
}
