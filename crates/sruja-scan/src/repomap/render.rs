use std::collections::HashMap;
use std::path::Path;

use crate::tree_sitter::DefinitionKind;

use super::analyze::{fan_in_out, find_dependency_cycles, format_fanout};
use super::types::{
    DirNode, FileRank, RepoMapDiagnostics, RepoMapOptions, TokenBudget,
};

pub(crate) fn render_tree(
    output: &mut String,
    budget: &mut TokenBudget,
    node: &DirNode,
    file_ranks: &[FileRank],
    options: &RepoMapOptions,
) {
    let file_rank_by_path: HashMap<&str, &FileRank> =
        file_ranks.iter().map(|f| (f.path.as_str(), f)).collect();
    render_tree_recursive(output, budget, node, &file_rank_by_path, options, "", true);
}

fn render_tree_recursive(
    output: &mut String,
    budget: &mut TokenBudget,
    node: &DirNode,
    file_rank_by_path: &HashMap<&str, &FileRank>,
    options: &RepoMapOptions,
    prefix: &str,
    is_last: bool,
) {
    let mut dirs: Vec<_> = node.children.keys().cloned().collect();
    dirs.sort();

    for (i, dir_name) in dirs.iter().enumerate() {
        if budget.truncated {
            return;
        }
        let last = i == dirs.len() - 1 && node.files.is_empty();
        let connector = if is_last { "└── " } else { "├── " };
        let extension = if last { "    " } else { "│   " };

        let ok = budget.push_str(output, &format!("{}{}{}/\n", prefix, connector, dir_name));
        if !ok {
            return;
        }

        if let Some(child) = node.children.get(dir_name) {
            render_tree_recursive(
                output,
                budget,
                child,
                file_rank_by_path,
                options,
                &format!("{}{}", prefix, extension),
                last,
            );
        }
    }

    let mut files: Vec<_> = node.files.iter().collect();
    files.sort();

    for (i, file_name) in files.iter().enumerate() {
        if budget.truncated {
            return;
        }
        let last = i == files.len() - 1;
        let connector = if last { "└── " } else { "├── " };
        let display_name = file_name.rsplit('/').next().unwrap_or(file_name.as_str());

        let ok = budget.push_str(
            output,
            &format!("{}{}{}\n", prefix, connector, display_name),
        );
        if !ok {
            return;
        }

        if options.include_signatures {
            if let Some(file_rank) = file_rank_by_path.get(file_name.as_str()).copied() {
                if let Some(ref parsed) = file_rank.parsed {
                    if !parsed.definitions.is_empty() {
                        let extension = if last { "    " } else { "│   " };
                        let sig_prefix = format!("{}{}    ", prefix, extension);

                        for def in parsed.definitions.iter().take(12) {
                            let kind_str = match def.kind {
                                DefinitionKind::Function => "fn",
                                DefinitionKind::Class => "class",
                                DefinitionKind::Interface => "interface",
                                DefinitionKind::Struct => "struct",
                                DefinitionKind::Enum => "enum",
                                DefinitionKind::Constant => "const",
                                DefinitionKind::Variable => "var",
                            };
                            let ok = budget.push_str(
                                output,
                                &format!(
                                    "{}{} {} (L{})\n",
                                    sig_prefix, kind_str, def.name, def.line
                                ),
                            );
                            if !ok {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn render_diagnostics(
    output: &mut String,
    budget: &mut TokenBudget,
    diagnostics: &RepoMapDiagnostics,
    import_graph: &HashMap<String, Vec<String>>,
) {
    let mut lines: Vec<String> = Vec::new();
    let parsed_files = import_graph.len();
    let total_edges: usize = import_graph.values().map(|t| t.len()).sum();
    let unresolved_total: usize = diagnostics.unresolved_imports_by_file.values().sum();

    lines.push(format!(
        "- Parsed files: {} (collected: {}; supported: {}).",
        parsed_files, diagnostics.collected_files, diagnostics.language_files_seen
    ));
    lines.push(format!(
        "- Dependency edges: {} (unresolved imports: {}).",
        total_edges, unresolved_total
    ));

    if diagnostics.language_files_seen > diagnostics.collected_files {
        let skipped = diagnostics
            .language_files_seen
            .saturating_sub(diagnostics.collected_files);
        if skipped > 0 {
            lines.push(format!(
                "- Skipped {} language files (read failed or too large).",
                skipped
            ));
        }
    }

    if !diagnostics.skipped_large.is_empty() {
        lines.push(format!(
            "- Skipped {} large files (examples: {}).",
            diagnostics.skipped_large.len(),
            diagnostics
                .skipped_large
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if !diagnostics.read_failed.is_empty() {
        lines.push(format!(
            "- Failed to read {} files (examples: {}).",
            diagnostics.read_failed.len(),
            diagnostics
                .read_failed
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if !diagnostics.parse_failed.is_empty() {
        lines.push(format!(
            "- Tree-sitter parsing failed for {} files (examples: {}).",
            diagnostics.parse_failed.len(),
            diagnostics
                .parse_failed
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let cycles = find_dependency_cycles(import_graph, 3);
    if !cycles.is_empty() {
        lines.push(format!("- Dependency cycles: {}.", cycles.len()));
        let mut cycle_examples: Vec<String> = Vec::new();
        for cycle in cycles {
            if cycle.len() >= 2 {
                let mut chain = cycle.iter().take(4).cloned().collect::<Vec<_>>();
                if let Some(first) = cycle.first().cloned() {
                    chain.push(first);
                }
                cycle_examples.push(chain.join(" -> "));
            }
        }
        lines.push(format!(
            "- Dependency cycles detected (examples: {}).",
            cycle_examples
                .into_iter()
                .take(2)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    } else {
        lines.push("- Dependency cycles: none detected.".to_string());
    }

    if !diagnostics.unresolved_imports_by_file.is_empty() {
        let mut unresolved: Vec<(&String, &usize)> =
            diagnostics.unresolved_imports_by_file.iter().collect();
        unresolved.sort_by(|a, b| b.1.cmp(a.1));
        let top: Vec<String> = unresolved
            .into_iter()
            .filter(|(_, c)| **c >= 5)
            .take(3)
            .map(|(p, c)| format!("{} ({})", p, c))
            .collect();
        if !top.is_empty() {
            lines.push(format!(
                "- Many imports could not be resolved (top: {}).",
                top.join(", ")
            ));
        }
    }

    let (fan_out, fan_in) = fan_in_out(import_graph);
    if let Some(line) = format_fanout("High fan-out", &fan_out) {
        lines.push(line);
    }
    if let Some(line) = format_fanout("High fan-in", &fan_in) {
        lines.push(line);
    }

    let ok = budget.push_str(output, "## Static Findings\n");
    if !ok {
        return;
    }
    let ok = budget.push_str(
        output,
        "(concise signals; intended for humans + LLM review)\n\n",
    );
    if !ok {
        return;
    }

    for line in lines.into_iter().take(8) {
        let ok = budget.push_str(output, &format!("{}\n", line));
        if !ok {
            return;
        }
    }

    let _ = budget.push_str(output, "\n");
}

pub(crate) fn rel_path(repo_root: &Path, repo_canon: &Path, path: &Path) -> String {
    let rel = path
        .strip_prefix(repo_canon)
        .or_else(|_| path.strip_prefix(repo_root))
        .unwrap_or(path);
    rel.to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string()
}

pub(crate) fn normalize_repo_rel(repo_prefixes: &[String], raw_path: &str) -> String {
    let normalized = raw_path.replace('\\', "/");
    let mut trimmed: &str = normalized.as_str();
    for prefix in repo_prefixes {
        if prefix.is_empty() {
            continue;
        }
        if trimmed.starts_with(prefix) {
            trimmed = trimmed.strip_prefix(prefix).unwrap_or(trimmed);
            break;
        }
    }
    trimmed
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string()
}
