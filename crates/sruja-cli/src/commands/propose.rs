use std::path::Path;
use sruja_diff::{Proposal, ProposalChange, ProposalStatus};
use super::{CliError, parse_sruja_file};
use sruja_scan::scan_repo;
use sruja_intent::{IntentContext, IntentModel};

pub async fn propose_create(
    repo_root: &str,
    description: &str,
    add_elements: Vec<String>, // format: "id:kind:label[:tech]"
    add_relationships: Vec<String>, // format: "source->target[:label]"
    remove_elements: Vec<String>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let sruja_file = repo_path.join("repo.sruja");
    
    // 1. Load baseline to validate existing elements
    let (_content, _program) = if sruja_file.exists() {
        parse_sruja_file(&sruja_file)?
    } else {
        (String::new(), sruja_language::ast::Program::default())
    };

    let mut proposal = Proposal::new(
        uuid::Uuid::new_v4().to_string()[..8].to_string(),
        "Architecture Change".to_string(),
        description.to_string(),
    );

    for spec in add_elements {
        let parts: Vec<&str> = spec.split(':').collect();
        if parts.len() < 3 {
            return Err(CliError::validation(format!("Invalid element spec: {}. Expected id:kind:label[:tech]", spec)));
        }
        proposal.changes.push(ProposalChange::AddElement {
            id: parts[0].to_string(),
            kind: parts[1].to_string(),
            label: parts[2].to_string(),
            technology: parts.get(3).map(|s| s.to_string()),
            parent: None,
            description: None,
        });
    }

    for spec in add_relationships {
        let parts: Vec<&str> = spec.split("->").collect();
        if parts.len() < 2 {
            return Err(CliError::validation(format!("Invalid relationship spec: {}. Expected source->target[:label]", spec)));
        }
        let target_parts: Vec<&str> = parts[1].split(':').collect();
        proposal.changes.push(ProposalChange::AddRelationship {
            source: parts[0].to_string(),
            target: target_parts[0].to_string(),
            label: target_parts.get(1).map(|s| s.to_string()),
            kind: None,
        });
    }

    for id in remove_elements {
        proposal.changes.push(ProposalChange::RemoveElement {
            id,
            reason: None,
        });
    }

    // 2. Validate against current graph + tribal knowledge
    let graph = scan_repo(repo_path)?;
    let _intent_context = IntentContext::new();
    let intent_model = IntentModel::default(); // Simplified for now
    
    let validation = proposal.validate(&graph, &intent_model);

    // 3. Save
    let file_path = proposal.save(repo_path).map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    println!("═══════════════════════════════════════════════════════════════");
    println!("📋 Proposal Created: {} ({:?})", proposal.id, proposal.status);
    println!("═══════════════════════════════════════════════════════════════");
    println!("Description: {}", proposal.description);
    println!();
    println!("── Changes ──────────────────────────────────────────────────");
    for change in &proposal.changes {
        match change {
            ProposalChange::AddElement { id, kind, label, .. } => println!("  + {} ({} \"{}\")", id, kind, label),
            ProposalChange::AddRelationship { source, target, label, .. } => {
                let l = label.as_deref().unwrap_or("");
                println!("  + {} -> {} \"{}\"", source, target, l);
            }
            ProposalChange::RemoveElement { id, .. } => println!("  - {}", id),
            _ => {}
        }
    }
    
    if !validation.tribal_warnings.is_empty() {
        println!();
        println!("── Tribal Warnings ──────────────────────────────────────────");
        for warning in &validation.tribal_warnings {
            println!("  {}", warning);
        }
    }

    println!();
    println!("Saved to: {}", file_path.display());
    println!("Run 'sruja propose approve {}' to merge into repo.sruja", proposal.id);
    
    Ok(())
}

pub async fn propose_list(repo_root: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let proposals = Proposal::load_all(repo_path).map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
    
    if proposals.is_empty() {
        println!("No proposals found.");
        return Ok(());
    }

    println!("{:<10} {:<10} {:<30}", "ID", "Status", "Description");
    println!("{}", "-".repeat(50));
    for p in proposals {
        println!("{:<10} {:<10} {:<30}", p.id, format!("{:?}", p.status), p.description);
    }
    
    Ok(())
}

pub async fn propose_approve(repo_root: &str, proposal_id: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let mut proposals = Proposal::load_all(repo_path).map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
    
    let proposal = proposals.iter_mut().find(|p| p.id == proposal_id)
        .ok_or_else(|| CliError::validation(format!("Proposal '{}' not found", proposal_id)))?;

    if proposal.status == ProposalStatus::Approved {
        println!("Proposal '{}' is already approved.", proposal_id);
        return Ok(());
    }

    let sruja_file = repo_path.join("repo.sruja");
    let (_content, program) = if sruja_file.exists() {
        parse_sruja_file(&sruja_file)?
    } else {
        (String::new(), sruja_language::ast::Program::default())
    };

    // Apply change to AST
    let updated_program = proposal.apply(&program).map_err(|e| CliError::validation(e.to_string()))?;

    // Export back to DSL
    let printer = sruja_export::DslPrinter::new();
    let updated_dsl = printer.print(&updated_program);

    std::fs::write(&sruja_file, updated_dsl)?;
    
    proposal.status = ProposalStatus::Approved;
    proposal.save(repo_path).map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    println!("Proposal '{}' approved and merged into repo.sruja", proposal_id);
    
    Ok(())
}
