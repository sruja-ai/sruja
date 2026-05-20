use super::{parse_sruja_file, CliError};
use sruja_diagnostics::format_diagnostic;
use sruja_diff::{Proposal, ProposalChange, ProposalStatus};
use sruja_engine::Validator;
use sruja_export::DslPrinter;
use sruja_intent::IntentModel;
use sruja_scan::scan_repo;
use std::collections::HashSet;
use std::path::Path;

pub struct ProposeCreateRequest {
    pub description: String,
    pub workflow_id: Option<String>,
    pub add_elements: Vec<String>,
    pub add_relationships: Vec<String>,
    pub remove_elements: Vec<String>,
    pub remove_relationships: Vec<String>,
    pub format: String,
}

struct AddElementSpec {
    id: String,
    kind: String,
    label: String,
    technology: Option<String>,
    description: Option<String>,
}

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

pub async fn propose_list(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let format = OutputFormat::parse(format)?;
    let proposals = Proposal::load_all(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    if proposals.is_empty() {
        match format {
            OutputFormat::Text => println!("No proposals found."),
            OutputFormat::Json => println!("[]"),
        }
        return Ok(());
    }

    let mut proposals = proposals;
    proposals.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    match format {
        OutputFormat::Text => {
            println!(
                "{:<12} {:<10} {:<24} Description",
                "ID", "Status", "Created"
            );
            println!("{}", "-".repeat(80));
            for p in proposals {
                println!(
                    "{:<12} {:<10} {:<24} {}",
                    p.id,
                    format!("{:?}", p.status),
                    p.created_at,
                    p.description
                );
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&proposals)?);
        }
    }

    Ok(())
}

fn validate_updated_architecture(repo_path: &Path, dsl: &str) -> Result<(), CliError> {
    let parser = sruja_language::Parser::new("repo.sruja.working".to_string());
    let program = parser.parse(dsl).map_err(|diags| {
        CliError::parse_with_diagnostics("repo.sruja.working".to_string(), diags)
    })?;
    let validator = Validator::with_default_rules();
    let mut diagnostics = validator.validate_sync(&program);
    crate::modules::validation::enrich_diagnostics_with_source(dsl, &mut diagnostics);
    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
        .count();
    if errors > 0 {
        for diag in diagnostics
            .iter()
            .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
            .take(20)
        {
            eprintln!("{}", format_diagnostic(diag));
        }
        return Err(CliError::validation(format!(
            "Proposed architecture fails lint with {} errors",
            errors
        )));
    }

    let actual_graph = scan_repo(repo_path)?;
    let proposed_graph = sruja_diff::program_to_graph(&program);
    let diff = sruja_diff::compare_graphs(&actual_graph, &proposed_graph);
    let error_violations = diff
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Error))
        .count();
    if error_violations > 0 {
        for v in diff
            .violations
            .iter()
            .filter(|v| matches!(v.severity, sruja_diff::Severity::Error))
            .take(20)
        {
            let rule = v.rule_id.as_deref().unwrap_or("-");
            let loc = v.location.as_deref().unwrap_or("-");
            eprintln!("{} {} {}", rule, loc, v.message);
        }
        return Err(CliError::validation(format!(
            "Proposed architecture fails drift checks with {} error violations",
            error_violations
        )));
    }

    Ok(())
}

fn write_ops_jsonl(repo_path: &Path, proposal: &Proposal) -> Result<(), CliError> {
    let dir = repo_path.join(".sruja").join("proposals");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.ops.jsonl", proposal.id));
    let mut out = String::new();
    for ch in &proposal.changes {
        out.push_str(&serde_json::to_string(ch)?);
        out.push('\n');
    }
    std::fs::write(path, out)?;
    Ok(())
}

pub async fn propose_approve(
    repo_root: &str,
    proposal_id: &str,
    dry_run: bool,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let format = OutputFormat::parse(format)?;
    let mut proposals = Proposal::load_all(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    let proposal = proposals
        .iter_mut()
        .find(|p| p.id == proposal_id)
        .ok_or_else(|| CliError::validation(format!("Proposal '{}' not found", proposal_id)))?;

    if proposal.status == ProposalStatus::Approved {
        match format {
            OutputFormat::Text => println!("Proposal '{}' is already approved.", proposal_id),
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "schema_version": "proposal_approve/v1",
                        "proposal_id": proposal_id,
                        "status": "already_approved",
                    }))?
                );
            }
        }
        return Ok(());
    }

    let sruja_file = repo_path.join("repo.sruja");
    let (before_content, baseline_program) = if sruja_file.exists() {
        parse_sruja_file(&sruja_file)?
    } else {
        (String::new(), sruja_language::ast::Program::default())
    };

    let updated_program = proposal
        .apply(&baseline_program)
        .map_err(|e| CliError::validation(e.to_string()))?;
    let updated_dsl = DslPrinter::new().print(&updated_program);

    validate_updated_architecture(repo_path, &updated_dsl)?;

    if dry_run {
        let before_lines = before_content.lines().count();
        let after_lines = updated_dsl.lines().count();
        match format {
            OutputFormat::Text => {
                println!("Proposal: {}", proposal.id);
                println!("Status before: {:?}", proposal.status);
                if let Some(w) = proposal.workflow_id.as_deref() {
                    println!("Workflow: {}", w);
                }
                println!("repo.sruja lines: {} -> {}", before_lines, after_lines);
                println!();
                println!("Changes:");
                for ch in &proposal.changes {
                    match ch {
                        ProposalChange::AddElement {
                            id, kind, label, ..
                        } => println!("  + {} = {} \"{}\"", id, kind, label),
                        ProposalChange::RemoveElement { id, .. } => println!("  - {}", id),
                        ProposalChange::ModifyElement { id, field, .. } => {
                            println!("  ~ {} ({})", id, field)
                        }
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
                        ProposalChange::RemoveRelationship { source, target, .. } => {
                            println!("  - {} -> {}", source, target)
                        }
                    }
                }
                println!();
                println!("Dry run: no files were written.");
            }
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "schema_version": "proposal_approve_dry_run/v1",
                        "proposal_id": proposal.id,
                        "workflow_id": proposal.workflow_id,
                        "status_before": format!("{:?}", proposal.status),
                        "lines_before": before_lines,
                        "lines_after": after_lines,
                        "changes": proposal.changes,
                        "note": "No files were written. Re-run without --dry-run to merge into repo.sruja.",
                    }))?
                );
            }
        }
        return Ok(());
    }

    let working = repo_path.join("repo.sruja.working");
    std::fs::write(&working, &updated_dsl)?;
    if sruja_file.exists() {
        let backup = repo_path.join("repo.sruja.bak");
        if backup.exists() {
            let _ = std::fs::remove_file(&backup);
        }
        std::fs::rename(&sruja_file, &backup)?;
        if let Err(e) = std::fs::rename(&working, &sruja_file) {
            let _ = std::fs::rename(&backup, &sruja_file);
            return Err(CliError::Io(e));
        }
        let _ = std::fs::remove_file(&backup);
    } else {
        std::fs::rename(&working, &sruja_file)?;
    }

    proposal.status = ProposalStatus::Approved;
    proposal
        .save(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
    let ops_path = repo_path
        .join(".sruja")
        .join("proposals")
        .join(format!("{}.ops.jsonl", proposal.id));
    write_ops_jsonl(repo_path, proposal)?;

    match format {
        OutputFormat::Text => {
            println!(
                "Proposal '{}' approved and merged into repo.sruja",
                proposal_id
            );
            println!("Wrote: {}", ops_path.display());
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "proposal_approve/v1",
                    "proposal_id": proposal_id,
                    "status": "approved",
                    "repo_sruja": "repo.sruja",
                    "ops_jsonl": ops_path,
                }))?
            );
        }
    }

    crate::commands::context_events::record_proposal_merge(repo_path, proposal_id);

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
struct ProposalLintSummary {
    error_count: usize,
    errors: Vec<String>,
}

fn lint_proposal_against_program(
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    fn parse(s: &str) -> Result<Self, CliError> {
        match s.trim() {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => Err(CliError::validation(format!(
                "Invalid format '{}'. Expected 'text' or 'json'.",
                other
            ))),
        }
    }
}

fn split_escaped(input: &str, sep: char) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut escape = false;
    for ch in input.chars() {
        if escape {
            cur.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == sep {
            parts.push(cur);
            cur = String::new();
            continue;
        }
        cur.push(ch);
    }
    if escape {
        cur.push('\\');
    }
    parts.push(cur);
    parts
}

fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

fn is_valid_qualified_ident(s: &str) -> bool {
    if s.trim().is_empty() {
        return false;
    }
    s.split('.').all(is_valid_identifier)
}

fn root_ident(s: &str) -> &str {
    s.split('.').next().unwrap_or(s)
}

fn parse_add_element_spec(
    spec: &str,
) -> Result<(String, String, String, Option<String>), CliError> {
    let parts = split_escaped(spec, ':');
    if parts.len() < 3 || parts.len() > 4 {
        return Err(CliError::validation(format!(
            "Invalid element spec '{}'. Expected id:kind:label[:tech]. Use \\: to escape ':' in fields.",
            spec
        )));
    }
    let id = parts[0].trim().to_string();
    let kind = parts[1].trim().to_string();
    let label = parts[2].trim().to_string();
    if id.is_empty() || kind.is_empty() || label.is_empty() {
        return Err(CliError::validation(format!(
            "Invalid element spec '{}'. id, kind, and label must be non-empty.",
            spec
        )));
    }
    if !is_valid_identifier(&id) {
        return Err(CliError::validation(format!(
            "Invalid element spec '{}'. id '{}' must be an identifier (letters, digits, _, -).",
            spec, id
        )));
    }
    if !is_valid_identifier(&kind) {
        return Err(CliError::validation(format!(
            "Invalid element spec '{}'. kind '{}' must be an identifier (letters, digits, _, -).",
            spec, kind
        )));
    }
    let tech = parts
        .get(3)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Ok((id, kind, label, tech))
}

fn parse_add_relationship_spec(spec: &str) -> Result<(String, String, Option<String>), CliError> {
    let (left, right) = spec.split_once("->").ok_or_else(|| {
        CliError::validation(format!(
            "Invalid relationship spec '{}'. Expected source->target[:label].",
            spec
        ))
    })?;
    let source = left.trim().to_string();
    let rest = right.trim();
    if source.is_empty() || rest.is_empty() {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. Expected source->target[:label].",
            spec
        )));
    }
    if !is_valid_qualified_ident(&source) {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. source '{}' must be a qualified identifier (e.g., A or System.Container).",
            spec, source
        )));
    }
    let parts = split_escaped(rest, ':');
    if parts.len() > 2 {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. Expected source->target[:label]. Use \\: to escape ':' in label.",
            spec
        )));
    }
    let target = parts[0].trim().to_string();
    if target.is_empty() {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. Target must be non-empty.",
            spec
        )));
    }
    if !is_valid_qualified_ident(&target) {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. target '{}' must be a qualified identifier (e.g., B or System.Container).",
            spec, target
        )));
    }
    let label = parts
        .get(1)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Ok((source, target, label))
}

fn parse_remove_relationship_spec(spec: &str) -> Result<(String, String), CliError> {
    let (left, right) = spec.split_once("->").ok_or_else(|| {
        CliError::validation(format!(
            "Invalid relationship spec '{}'. Expected source->target.",
            spec
        ))
    })?;
    let source = left.trim().to_string();
    let target = right.trim().to_string();
    if source.is_empty() || target.is_empty() {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. Expected source->target.",
            spec
        )));
    }
    if !is_valid_qualified_ident(&source) {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. source '{}' must be a qualified identifier (e.g., A or System.Container).",
            spec, source
        )));
    }
    if !is_valid_qualified_ident(&target) {
        return Err(CliError::validation(format!(
            "Invalid relationship spec '{}'. target '{}' must be a qualified identifier (e.g., B or System.Container).",
            spec, target
        )));
    }
    Ok((source, target))
}

fn is_under_root(fqn: &str, root: &str) -> bool {
    fqn == root || fqn.starts_with(&format!("{}.", root))
}

fn kind_requires_description_str(kind_lc: &str) -> bool {
    matches!(
        kind_lc,
        "container" | "component" | "database" | "datastore" | "data_store"
    )
}

fn kind_requires_technology_str(kind_lc: &str) -> bool {
    matches!(
        kind_lc,
        "container" | "database" | "datastore" | "data_store"
    )
}

fn derive_title(description: &str) -> String {
    let first_line = description.lines().next().unwrap_or("").trim();
    if first_line.is_empty() {
        return "Architecture Change".to_string();
    }
    let mut out = first_line.to_string();
    if out.len() > 64 {
        out.truncate(64);
    }
    out
}

fn new_short_id() -> String {
    let id = uuid::Uuid::new_v4().simple().to_string();
    id.chars().take(12).collect()
}

fn new_short_id_unique(repo_path: &Path) -> Result<String, CliError> {
    let proposals_dir = repo_path.join(".sruja").join("proposals");
    for _ in 0..10 {
        let id = new_short_id();
        let candidate = proposals_dir.join(format!("{}.json", id));
        if !candidate.exists() {
            return Ok(id);
        }
    }
    Err(CliError::validation(
        "Failed to generate a unique proposal id after multiple attempts.".to_string(),
    ))
}

fn collect_program_top_level_elements(program: &sruja_language::ast::Program) -> HashSet<String> {
    use sruja_language::ast::TopLevelItem;
    let mut out: HashSet<String> = HashSet::new();
    for item in &program.items {
        if let TopLevelItem::ElementDef(def) = item {
            out.insert(def.assignment.name.clone());
        }
    }
    out
}

fn collect_program_element_fqns(program: &sruja_language::ast::Program) -> HashSet<String> {
    let (elements, _relations) = sruja_language::collect_elements(program);
    elements.keys().cloned().collect()
}

fn collect_program_relationships(
    program: &sruja_language::ast::Program,
) -> HashSet<(String, String)> {
    let mut out: HashSet<(String, String)> = HashSet::new();
    for rel in sruja_language::collect_all_relations(program) {
        out.insert((rel.from.as_string(), rel.to.as_string()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_spec_parses_with_escaped_colon() {
        let (id, kind, label, tech) =
            parse_add_element_spec(r#"A:system:Payments\: Core:Go"#).unwrap();
        assert_eq!(id, "A");
        assert_eq!(kind, "system");
        assert_eq!(label, "Payments: Core");
        assert_eq!(tech.as_deref(), Some("Go"));
    }

    #[test]
    fn relationship_spec_parses_with_label() {
        let (s, t, l) = parse_add_relationship_spec("A -> B:uses api").unwrap();
        assert_eq!(s, "A");
        assert_eq!(t, "B");
        assert_eq!(l.as_deref(), Some("uses api"));
    }

    #[test]
    fn identifier_validation_matches_dsl_identifier_shape() {
        assert!(is_valid_identifier("A"));
        assert!(is_valid_identifier("A_b"));
        assert!(is_valid_identifier("a-b"));
        assert!(!is_valid_identifier("1A"));
        assert!(!is_valid_identifier("A.B"));
        assert!(!is_valid_identifier(""));
    }

    #[test]
    fn qualified_identifier_validation_accepts_dot_paths() {
        assert!(is_valid_qualified_ident("A"));
        assert!(is_valid_qualified_ident("System.Container"));
        assert!(is_valid_qualified_ident("A_b.C-d"));
        assert!(!is_valid_qualified_ident("A..B"));
        assert!(!is_valid_qualified_ident(".A"));
        assert!(!is_valid_qualified_ident("A."));
        assert!(!is_valid_qualified_ident(""));
    }
}
