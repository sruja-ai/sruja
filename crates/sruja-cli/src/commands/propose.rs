use super::{parse_sruja_file, CliError};
use sruja_diagnostics::format_diagnostic;
use sruja_diff::{Proposal, ProposalChange, ProposalStatus};
use sruja_engine::Validator;
use sruja_intent::IntentModel;
use sruja_scan::scan_repo;
use std::collections::HashSet;
use std::path::Path;

pub async fn propose_create(
    repo_root: &str,
    description: &str,
    workflow_id: Option<String>,
    add_elements: Vec<String>,      // format: "id:kind:label[:tech]"
    add_relationships: Vec<String>, // format: "source->target[:label]"
    remove_elements: Vec<String>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let sruja_file = repo_path.join("repo.sruja");

    let format = OutputFormat::parse(format)?;

    let (_baseline_content, baseline_program) = if sruja_file.exists() {
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

    let has_baseline = sruja_file.exists();
    let baseline_elements = collect_program_elements(&baseline_program);
    let baseline_relationships = collect_program_relationships(&baseline_program);

    if !has_baseline && !remove_elements.is_empty() {
        return Err(CliError::validation(
            "Cannot remove elements: repo.sruja does not exist yet.".to_string(),
        ));
    }

    let mut add_element_specs: Vec<(String, String, String, Option<String>)> = Vec::new();
    let mut add_element_ids: HashSet<String> = HashSet::new();
    for spec in add_elements {
        let parsed = parse_add_element_spec(&spec)?;
        let id = parsed.0.clone();
        if baseline_elements.contains(&id) {
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
        add_element_specs.push(parsed);
    }

    let mut remove_element_ids: HashSet<String> = HashSet::new();
    let mut remove_element_list: Vec<String> = Vec::new();
    for id in remove_elements {
        let id = id.trim().to_string();
        if id.is_empty() {
            return Err(CliError::validation(
                "remove-elements contains an empty id".to_string(),
            ));
        }
        if has_baseline && !baseline_elements.contains(&id) {
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

    for (id, kind, label, tech) in &add_element_specs {
        proposal.changes.push(ProposalChange::AddElement {
            id: id.clone(),
            kind: kind.clone(),
            label: label.clone(),
            technology: tech.clone(),
            parent: None,
            description: None,
        });
    }

    for id in &remove_element_list {
        proposal.changes.push(ProposalChange::RemoveElement {
            id: id.clone(),
            reason: None,
        });
    }

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

        if remove_element_ids.contains(&source) || remove_element_ids.contains(&target) {
            return Err(CliError::validation(format!(
                "Relationship '{} -> {}' references an element removed in this proposal",
                source, target
            )));
        }

        let source_known = baseline_elements.contains(&source) || add_element_ids.contains(&source);
        let target_known = baseline_elements.contains(&target) || add_element_ids.contains(&target);
        if !source_known || !target_known {
            return Err(CliError::validation(format!(
                "Relationship '{} -> {}' references unknown element(s). Ensure both ends exist in repo.sruja or are added in this proposal.",
                source, target
            )));
        }

        proposal.changes.push(ProposalChange::AddRelationship {
            source,
            target,
            label,
            kind: None,
        });
    }

    if proposal.changes.is_empty() {
        return Err(CliError::validation(
            "Proposal has no changes. Pass --add-elements / --add-relationships / --remove-elements."
                .to_string(),
        ));
    }

    // 2. Validate against current graph + tribal knowledge
    let graph = scan_repo(repo_path)?;
    let intent_model = IntentModel::default(); // Simplified for now

    let validation = proposal.validate(&graph, &intent_model);
    proposal.status = if validation.is_valid {
        ProposalStatus::Pending
    } else {
        ProposalStatus::Draft
    };

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
            println!();
            println!("Wrote: {}", file_path.display());
            println!("Next: sruja propose approve {}", proposal.id);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&proposal)?);
            eprintln!("Wrote {}", file_path.display());
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
        if let Some((src, tgt)) = parse_relationship_line(trimmed) {
            if src == source && tgt == target {
                let end = find_line_end_inclusive(text, line_start);
                matches.push((line_start, end));
            }
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
    write_ops_jsonl(repo_path, proposal)?;

    match format {
        OutputFormat::Text => {
            println!(
                "Proposal '{}' approved and merged into repo.sruja",
                proposal_id
            );
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": "proposal_approve/v1",
                    "proposal_id": proposal_id,
                    "status": "approved",
                    "repo_sruja": "repo.sruja",
                }))?
            );
        }
    }

    crate::commands::context_events::record_proposal_merge(repo_path, proposal_id);

    Ok(())
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
    let label = parts
        .get(1)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Ok((source, target, label))
}

fn collect_program_elements(program: &sruja_language::ast::Program) -> HashSet<String> {
    use sruja_language::ast::TopLevelItem;
    let mut out: HashSet<String> = HashSet::new();
    for item in &program.items {
        if let TopLevelItem::ElementDef(def) = item {
            out.insert(def.assignment.name.clone());
        }
    }
    out
}

fn collect_program_relationships(
    program: &sruja_language::ast::Program,
) -> HashSet<(String, String)> {
    use sruja_language::ast::TopLevelItem;
    let mut out: HashSet<(String, String)> = HashSet::new();
    for item in &program.items {
        if let TopLevelItem::Relation(rel) = item {
            out.insert((rel.from.as_string(), rel.to.as_string()));
        }
    }
    out
}

fn parse_relationship_line(line: &str) -> Option<(String, String)> {
    let s = line.trim();
    if s.is_empty() {
        return None;
    }
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'-' {
        i += 1;
    }
    if i == 0 {
        return None;
    }
    let src = s[..i].to_string();
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i + 1 >= bytes.len() || bytes[i] != b'-' || bytes[i + 1] != b'>' {
        return None;
    }
    i += 2;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let start_t = i;
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'"' {
        i += 1;
    }
    if start_t == i {
        return None;
    }
    let tgt = s[start_t..i].to_string();
    Some((src, tgt))
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
    fn relationship_line_parser_is_strict() {
        assert_eq!(
            parse_relationship_line(r#"A -> B "label""#),
            Some(("A".to_string(), "B".to_string()))
        );
        assert_eq!(parse_relationship_line(r#"X Y -> Z"#), None);
    }
}
