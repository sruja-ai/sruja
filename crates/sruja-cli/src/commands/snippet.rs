use serde::Serialize;
use std::path::Path;

use crate::commands::CliError;
use sruja_scan::Graph;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CodeSnippet {
    pub symbol_id: Option<String>,
    pub file_path: String,
    pub line: usize,
    pub end_line: usize,
    pub source: String,
    pub language: String,
}

fn safe_slice_lines<'a>(lines: &'a [&'a str], start: usize, end: usize) -> &'a [&'a str] {
    let start_idx = start.saturating_sub(1).min(lines.len());
    let end_idx = end.min(lines.len()).max(start_idx);
    &lines[start_idx..end_idx]
}

pub fn fetch_code_snippet(
    repo_root: &str,
    graph: &Graph,
    symbol_id: Option<&str>,
    file_path: Option<&str>,
    line: Option<usize>,
    end_line: Option<usize>,
) -> Result<Vec<CodeSnippet>, CliError> {
    let mut snippets = Vec::new();

    if let Some(sym_id) = symbol_id {
        if let Some(node) = graph.nodes.iter().find(|n| n.id == sym_id) {
            let path = node
                .path
                .as_ref()
                .ok_or_else(|| CliError::validation(format!("Symbol {} has no path", sym_id)))?;
            let file_abs = Path::new(repo_root).join(path);

            let line_str = node.metadata.get("line").ok_or_else(|| {
                CliError::validation(format!("Symbol {} has no line metadata", sym_id))
            })?;
            let start = line_str.parse::<usize>().map_err(|e| {
                CliError::validation(format!("Invalid line in metadata for {}: {}", sym_id, e))
            })?;

            let end_line_str = node.metadata.get("end_line").ok_or_else(|| {
                CliError::validation(format!("Symbol {} has no end_line metadata", sym_id))
            })?;
            let end = end_line_str.parse::<usize>().map_err(|e| {
                CliError::validation(format!(
                    "Invalid end_line in metadata for {}: {}",
                    sym_id, e
                ))
            })?;

            let content = std::fs::read_to_string(&file_abs).map_err(|e| {
                CliError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Failed to read {}: {}", file_abs.display(), e),
                ))
            })?;

            let lines: Vec<&str> = content.lines().collect();
            let slice = safe_slice_lines(&lines, start, end);

            let language = node
                .metadata
                .get("language")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            snippets.push(CodeSnippet {
                symbol_id: Some(sym_id.to_string()),
                file_path: path.clone(),
                line: start,
                end_line: end,
                source: slice.join("\n"),
                language,
            });
        } else {
            return Err(CliError::validation(format!("Symbol {} not found", sym_id)));
        }
    } else if let Some(fp) = file_path {
        let file_abs = if Path::new(fp).is_absolute() {
            fp.to_string()
        } else {
            Path::new(repo_root).join(fp).to_string_lossy().to_string()
        };

        let content = std::fs::read_to_string(&file_abs).map_err(|e| {
            CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to read {}: {}", file_abs, e),
            ))
        })?;

        let all_symbols: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| {
                n.path.as_ref().map(|p| p.contains(fp)).unwrap_or(false)
                    && !n.id.is_empty()
                    && n.id.contains(':')
            })
            .collect();

        if all_symbols.is_empty() {
            let start = line.unwrap_or(1);
            let end = end_line.unwrap_or_else(|| start + 50);
            let lines: Vec<&str> = content.lines().collect();
            let slice = safe_slice_lines(&lines, start, end);

            snippets.push(CodeSnippet {
                symbol_id: None,
                file_path: fp.to_string(),
                line: start,
                end_line: end,
                source: slice.join("\n"),
                language: "unknown".to_string(),
            });
        } else {
            for node in all_symbols {
                let line_str = node.metadata.get("line").map(|s| s.as_str()).unwrap_or("1");
                let start = line_str.parse::<usize>().unwrap_or(1);
                let end_line_str = node
                    .metadata
                    .get("end_line")
                    .map(|s| s.as_str())
                    .unwrap_or("60");
                let end = end_line_str.parse::<usize>().unwrap_or(start + 60);

                let lines: Vec<&str> = content.lines().collect();
                let slice = safe_slice_lines(&lines, start, end);

                let language = node
                    .metadata
                    .get("language")
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());

                snippets.push(CodeSnippet {
                    symbol_id: Some(node.id.clone()),
                    file_path: fp.to_string(),
                    line: start,
                    end_line: end,
                    source: slice.join("\n"),
                    language,
                });
            }
        }
    } else {
        return Err(CliError::validation(
            "Either symbol_id or file_path must be provided",
        ));
    }

    Ok(snippets)
}
