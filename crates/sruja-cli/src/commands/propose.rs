use super::{parse_sruja_file, CliError};
use sruja_diagnostics::format_diagnostic;
use sruja_diff::{Proposal, ProposalChange, ProposalStatus};
use sruja_engine::Validator;
use sruja_intent::{IntentContext, IntentModel};
use sruja_scan::scan_repo;
use std::path::Path;

pub async fn propose_create(
    repo_root: &str,
    description: &str,
    workflow_id: Option<String>,
    add_elements: Vec<String>,      // format: "id:kind:label[:tech]"
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
    proposal.workflow_id = workflow_id;

    for spec in add_elements {
        let parts: Vec<&str> = spec.split(':').collect();
        if parts.len() < 3 {
            return Err(CliError::validation(format!(
                "Invalid element spec: {}. Expected id:kind:label[:tech]",
                spec
            )));
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
            return Err(CliError::validation(format!(
                "Invalid relationship spec: {}. Expected source->target[:label]",
                spec
            )));
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
        proposal
            .changes
            .push(ProposalChange::RemoveElement { id, reason: None });
    }

    // 2. Validate against current graph + tribal knowledge
    let graph = scan_repo(repo_path)?;
    let _intent_context = IntentContext::new();
    let intent_model = IntentModel::default(); // Simplified for now

    let validation = proposal.validate(&graph, &intent_model);

    // 3. Save
    let file_path = proposal
        .save(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    println!("═══════════════════════════════════════════════════════════════");
    println!(
        "📋 Proposal Created: {} ({:?})",
        proposal.id, proposal.status
    );
    println!("═══════════════════════════════════════════════════════════════");
    println!("Description: {}", proposal.description);
    println!();
    println!("── Changes ──────────────────────────────────────────────────");
    for change in &proposal.changes {
        match change {
            ProposalChange::AddElement {
                id, kind, label, ..
            } => println!("  + {} ({} \"{}\")", id, kind, label),
            ProposalChange::AddRelationship {
                source,
                target,
                label,
                ..
            } => {
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
    println!(
        "Run 'sruja propose approve {}' to merge into repo.sruja",
        proposal.id
    );

    Ok(())
}

pub async fn propose_list(repo_root: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let proposals = Proposal::load_all(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    if proposals.is_empty() {
        println!("No proposals found.");
        return Ok(());
    }

    println!("{:<10} {:<10} {:<30}", "ID", "Status", "Description");
    println!("{}", "-".repeat(50));
    for p in proposals {
        println!(
            "{:<10} {:<10} {:<30}",
            p.id,
            format!("{:?}", p.status),
            p.description
        );
    }

    Ok(())
}

fn validate_updated_architecture(
    repo_path: &Path,
    dsl: &str,
) -> Result<sruja_language::ast::Program, CliError> {
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

    Ok(program)
}

fn ensure_trailing_newline(mut s: String) -> String {
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

fn escape_dsl_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn find_line_start(text: &str, idx: usize) -> usize {
    text[..idx].rfind('\n').map(|p| p + 1).unwrap_or(0)
}

fn find_line_end_inclusive(text: &str, idx: usize) -> usize {
    text[idx..]
        .find('\n')
        .map(|p| idx + p + 1)
        .unwrap_or_else(|| text.len())
}

fn find_element_assignment_offset(text: &str, id: &str) -> Option<usize> {
    let mut offset = 0usize;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(id) {
            if (rest.starts_with('=') || rest.starts_with(char::is_whitespace))
                && trimmed.contains('=')
            {
                return Some(offset + (line.len() - trimmed.len()));
            }
        }
        offset += line.len() + 1;
    }
    None
}

fn find_matching_brace_end(text: &str, open_brace_idx: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate().skip(open_brace_idx) {
        let c = b as char;
        if in_string {
            if escape {
                escape = false;
                continue;
            }
            if c == '\\' {
                escape = true;
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            continue;
        }
        if c == '{' {
            depth += 1;
            continue;
        }
        if c == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(i + 1);
            }
        }
    }
    None
}

fn remove_element_block(text: &mut String, id: &str) -> Result<(), CliError> {
    let Some(assign_idx) = find_element_assignment_offset(text, id) else {
        return Err(CliError::validation(format!(
            "Element '{}' not found in repo.sruja",
            id
        )));
    };
    let start = find_line_start(text, assign_idx);
    let line_end = find_line_end_inclusive(text, assign_idx);
    let after_eq = text[assign_idx..line_end].find('{').map(|p| assign_idx + p);
    if let Some(open) = after_eq {
        let Some(end_block) = find_matching_brace_end(text, open) else {
            return Err(CliError::validation(format!(
                "Failed to find end of element '{}' block",
                id
            )));
        };
        let end = find_line_end_inclusive(text, end_block);
        text.replace_range(start..end, "");
        return Ok(());
    }
    text.replace_range(start..line_end, "");
    Ok(())
}

fn modify_element_kind(text: &mut String, id: &str, new_kind: &str) -> Result<(), CliError> {
    let Some(assign_idx) = find_element_assignment_offset(text, id) else {
        return Err(CliError::validation(format!(
            "Element '{}' not found in repo.sruja",
            id
        )));
    };
    let line_end = find_line_end_inclusive(text, assign_idx);
    let line = &text[assign_idx..line_end];
    let eq_pos = line.find('=').ok_or_else(|| {
        CliError::validation(format!("Failed to parse element assignment for '{}'", id))
    })?;
    let mut i = eq_pos + 1;
    let bytes = line.as_bytes();
    while i < line.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let start_kind = i;
    while i < line.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'"' && bytes[i] != b'{'
    {
        i += 1;
    }
    let end_kind = i;
    let abs_start = assign_idx + start_kind;
    let abs_end = assign_idx + end_kind;
    text.replace_range(abs_start..abs_end, new_kind);
    Ok(())
}

fn modify_element_property(
    text: &mut String,
    id: &str,
    key: &str,
    value: &str,
) -> Result<(), CliError> {
    let Some(assign_idx) = find_element_assignment_offset(text, id) else {
        return Err(CliError::validation(format!(
            "Element '{}' not found in repo.sruja",
            id
        )));
    };
    let start_line = find_line_start(text, assign_idx);
    let open = text[assign_idx..]
        .find('{')
        .map(|p| assign_idx + p)
        .ok_or_else(|| CliError::validation(format!("Element '{}' has no body block", id)))?;
    let Some(end_block) = find_matching_brace_end(text, open) else {
        return Err(CliError::validation(format!(
            "Failed to find end of element '{}' block",
            id
        )));
    };
    let block = &text[start_line..end_block];
    let mut cursor = start_line;
    for line in block.lines() {
        let line_start = cursor;
        let line_end = find_line_end_inclusive(text, line_start);
        let trimmed = line.trim_start();
        if trimmed.starts_with(key) && trimmed[key.len()..].trim_start().starts_with('"') {
            let indent_len = line.len().saturating_sub(trimmed.len());
            let indent = &line[..indent_len];
            let new_line = format!("{}{} \"{}\"\n", indent, key, escape_dsl_string(value));
            text.replace_range(line_start..line_end, &new_line);
            return Ok(());
        }
        cursor = line_end;
    }
    let insert_at = find_line_end_inclusive(text, open);
    let new_line = format!("  {} \"{}\"\n", key, escape_dsl_string(value));
    text.insert_str(insert_at, &new_line);
    Ok(())
}

fn remove_relationship_line(text: &mut String, source: &str, target: &str) -> Result<(), CliError> {
    let mut matches: Vec<(usize, usize)> = Vec::new();
    let mut offset = 0usize;
    for line in text.lines() {
        let line_start = offset;
        let line_end = offset + line.len();
        let trimmed = line.trim_start();
        let is_match = trimmed.starts_with(source)
            && trimmed.contains("->")
            && trimmed.contains(target)
            && trimmed[source.len()..].contains("->");
        if is_match {
            let end = find_line_end_inclusive(text, line_start);
            matches.push((line_start, end));
        }
        offset = line_end + 1;
    }

    if matches.is_empty() {
        return Err(CliError::validation(format!(
            "Relationship '{} -> {}' not found in repo.sruja",
            source, target
        )));
    }

    matches.sort_by(|a, b| b.0.cmp(&a.0));
    for (start, end) in matches {
        text.replace_range(start..end, "");
    }
    Ok(())
}

fn add_relationship(text: &mut String, source: &str, target: &str, label: Option<&str>) {
    let needle = format!("{} -> {}", source, target);
    if text.lines().any(|l| l.trim_start().starts_with(&needle)) {
        return;
    }
    let mut s = ensure_trailing_newline(std::mem::take(text));
    if !s.ends_with("\n\n") {
        s.push('\n');
    }
    if let Some(l) = label {
        s.push_str(&format!(
            "{} -> {} \"{}\"\n",
            source,
            target,
            escape_dsl_string(l)
        ));
    } else {
        s.push_str(&format!("{} -> {}\n", source, target));
    }
    *text = s;
}

fn add_element(
    text: &mut String,
    id: &str,
    kind: &str,
    label: &str,
    technology: Option<&str>,
    description: Option<&str>,
) {
    if find_element_assignment_offset(text, id).is_some() {
        return;
    }
    let mut s = ensure_trailing_newline(std::mem::take(text));
    if !s.ends_with("\n\n") {
        s.push('\n');
    }
    let title = escape_dsl_string(label);
    if technology.is_none() && description.is_none() {
        s.push_str(&format!("{id} = {kind} \"{title}\"\n"));
        *text = s;
        return;
    }
    s.push_str(&format!("{id} = {kind} \"{title}\" {{\n"));
    if let Some(t) = technology {
        s.push_str(&format!("  technology \"{}\"\n", escape_dsl_string(t)));
    }
    if let Some(d) = description {
        s.push_str(&format!("  description \"{}\"\n", escape_dsl_string(d)));
    }
    s.push_str("}\n");
    *text = s;
}

fn apply_changes_to_dsl(before: &str, changes: &[ProposalChange]) -> Result<String, CliError> {
    let mut text = before.to_string();
    for ch in changes {
        match ch {
            ProposalChange::AddElement {
                id,
                kind,
                label,
                technology,
                description,
                ..
            } => add_element(
                &mut text,
                id,
                kind,
                label,
                technology.as_deref(),
                description.as_deref(),
            ),
            ProposalChange::RemoveElement { id, .. } => remove_element_block(&mut text, id)?,
            ProposalChange::ModifyElement {
                id,
                field,
                new_value,
                ..
            } => match field.as_str() {
                "kind" => modify_element_kind(&mut text, id, new_value)?,
                "technology" => modify_element_property(&mut text, id, "technology", new_value)?,
                "description" => modify_element_property(&mut text, id, "description", new_value)?,
                other => {
                    return Err(CliError::validation(format!(
                        "Unsupported ModifyElement field '{}'",
                        other
                    )))
                }
            },
            ProposalChange::AddRelationship {
                source,
                target,
                label,
                ..
            } => {
                add_relationship(&mut text, source, target, label.as_deref());
            }
            ProposalChange::RemoveRelationship { source, target, .. } => {
                remove_relationship_line(&mut text, source, target)?;
            }
        }
    }
    Ok(text)
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
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let mut proposals = Proposal::load_all(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    let proposal = proposals
        .iter_mut()
        .find(|p| p.id == proposal_id)
        .ok_or_else(|| CliError::validation(format!("Proposal '{}' not found", proposal_id)))?;

    if proposal.status == ProposalStatus::Approved {
        println!("Proposal '{}' is already approved.", proposal_id);
        return Ok(());
    }

    let sruja_file = repo_path.join("repo.sruja");
    let before_content = if sruja_file.exists() {
        std::fs::read_to_string(&sruja_file)?
    } else {
        String::new()
    };
    let updated_dsl = apply_changes_to_dsl(&before_content, &proposal.changes)?;

    let _validated_program = validate_updated_architecture(repo_path, &updated_dsl)?;

    if dry_run {
        let before_lines = before_content.lines().count();
        let after_lines = updated_dsl.lines().count();
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
    write_ops_jsonl(repo_path, proposal)?;

    println!(
        "Proposal '{}' approved and merged into repo.sruja",
        proposal_id
    );

    crate::commands::context_events::record_proposal_merge(repo_path, proposal_id);

    Ok(())
}
