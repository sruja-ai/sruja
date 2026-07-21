use super::CliError;
use crate::commands::parse_sruja_file;
use super::types::OutputFormat;
use sruja_diff::{Proposal, ProposalChange, ProposalStatus};
use sruja_engine::Validator;
use sruja_export::DslPrinter;
use sruja_scan::scan_repo;
use sruja_diagnostics::format_diagnostic;
use std::path::Path;

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
