//! Repository map generator for LLM context.
//!
//! Generates a tree-like structure of the repository annotated with
//! Tree-sitter signatures (structs, traits, functions) ranked by importance.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::graph::{Graph, Node};
use crate::tree_sitter::{detect_language, parse_file, DefinitionKind, ParsedFile, ScanConfig};

#[derive(Debug, Clone)]
pub struct RepoMapOptions {
    pub max_files: usize,
    pub max_tokens: usize,
    pub include_signatures: bool,
}

impl Default for RepoMapOptions {
    fn default() -> Self {
        Self {
            max_files: 100,
            max_tokens: 5000,
            include_signatures: true,
        }
    }
}

#[derive(Debug, Clone)]
struct FileRank {
    path: String,
    score: f64,
    parsed: Option<ParsedFile>,
}

pub fn generate_repomap(
    repo_root: &Path,
    options: &RepoMapOptions,
) -> Result<String, crate::ScanError> {
    let config = ScanConfig::default();
    let repo_canon = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());

    let collected = collect_files(repo_root, &config);

    let parsed_files: Vec<(String, ParsedFile)> = collected
        .iter()
        .filter_map(|(path, content)| {
            let rel_path = path
                .strip_prefix(&repo_canon)
                .ok()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());

            detect_language(path)
                .and_then(|lang| parse_file(path, content, lang))
                .map(|parsed| (rel_path, parsed))
        })
        .collect();

    let import_graph = build_import_graph(&parsed_files);
    let rankings = pagerank(&import_graph, parsed_files.len());

    let mut file_ranks: Vec<FileRank> = parsed_files
        .iter()
        .map(|(path, parsed)| {
            let score = rankings.get(path).copied().unwrap_or(0.0);
            FileRank {
                path: path.clone(),
                score,
                parsed: Some(parsed.clone()),
            }
        })
        .collect();

    file_ranks.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    file_ranks.truncate(options.max_files);

    let mut output = String::new();
    output.push_str("# Repository Map\n\n");
    output.push_str(&format!(
        "Top {} files ranked by importance (PageRank)\n\n",
        file_ranks.len()
    ));

    let dir_tree = build_directory_tree(&file_ranks);
    render_tree(&mut output, &dir_tree, &file_ranks, options);

    Ok(output)
}

fn collect_files(repo_root: &Path, config: &ScanConfig) -> Vec<(std::path::PathBuf, String)> {
    let walker = crate::tree_sitter::build_walker_internal(repo_root, config);
    let mut out = Vec::new();

    for entry in walker {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if detect_language(path).is_some() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if content.len() <= config.max_file_size {
                    out.push((path.to_path_buf(), content));
                }
            }
        }
    }

    out
}

fn build_import_graph(files: &[(String, ParsedFile)]) -> HashMap<String, Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    let path_to_canonical: HashMap<String, String> = files
        .iter()
        .map(|(path, _): &(String, ParsedFile)| {
            let canonical = path.replace('\\', "/");
            (canonical.clone(), canonical)
        })
        .collect();

    for (path, parsed) in files {
        let source = path.replace('\\', "/");
        let mut targets: Vec<String> = Vec::new();

        for import in &parsed.imports {
            if let Some(resolved) = resolve_import(import, &path_to_canonical, path) {
                targets.push(resolved);
            }
        }

        graph.insert(source, targets);
    }

    graph
}

fn resolve_import(
    import: &str,
    files: &HashMap<String, String>,
    source_path: &str,
) -> Option<String> {
    let import_clean = import.trim_start_matches('.').trim_start_matches('/');

    for canonical in files.keys() {
        if canonical.contains(import_clean) || canonical.ends_with(&format!("/{}", import_clean)) {
            return Some(canonical.clone());
        }
    }

    if import.starts_with('.') {
        let source_dir = source_path
            .rfind('/')
            .map(|i| &source_path[..i])
            .unwrap_or("");
        let resolved = format!("{}/{}", source_dir, import_clean);
        let resolved = resolved.replace("//", "/");

        for canonical in files.keys() {
            if canonical.starts_with(&resolved) || canonical == &resolved {
                return Some(canonical.clone());
            }
        }
    }

    None
}

fn pagerank(graph: &HashMap<String, Vec<String>>, node_count: usize) -> HashMap<String, f64> {
    if node_count == 0 {
        return HashMap::new();
    }

    let damping = 0.85;
    let iterations = 20;

    let all_nodes: HashSet<&String> = graph.keys().chain(graph.values().flatten()).collect();
    let n = all_nodes.len().max(1);

    let mut scores: HashMap<&str, f64> = all_nodes
        .iter()
        .map(|&k: &&String| (k.as_str(), 1.0 / n as f64))
        .collect();

    let mut incoming: HashMap<&str, Vec<&str>> = HashMap::new();
    for (source, targets) in graph {
        for target in targets {
            incoming
                .entry(target.as_str())
                .or_default()
                .push(source.as_str());
        }
    }

    for _ in 0..iterations {
        let mut new_scores: HashMap<&str, f64> = HashMap::new();

        for &node in &all_nodes {
            let mut score = (1.0 - damping) / n as f64;

            if let Some(predecessors) = incoming.get(node.as_str()) {
                for &pred in predecessors {
                    let pred_score = scores.get(pred).copied().unwrap_or(0.0);
                    let out_degree = graph
                        .get(pred)
                        .map(|v: &Vec<String>| v.len().max(1) as f64)
                        .unwrap_or(1.0);
                    score += damping * pred_score / out_degree;
                }
            }

            new_scores.insert(node.as_str(), score);
        }

        scores = new_scores;
    }

    scores
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect()
}

#[derive(Debug, Clone)]
struct DirNode {
    #[allow(dead_code)]
    name: String,
    files: Vec<String>,
    children: HashMap<String, DirNode>,
}

fn build_directory_tree(file_ranks: &[FileRank]) -> DirNode {
    let mut root = DirNode {
        name: String::new(),
        files: Vec::new(),
        children: HashMap::new(),
    };

    for file in file_ranks {
        let parts: Vec<&str> = file.path.split('/').collect();
        insert_into_tree(&mut root, &parts, 0);
    }

    root
}

fn insert_into_tree(node: &mut DirNode, parts: &[&str], depth: usize) {
    if depth >= parts.len() {
        return;
    }

    if depth == parts.len() - 1 {
        node.files.push(parts[depth].to_string());
        return;
    }

    let child = node
        .children
        .entry(parts[depth].to_string())
        .or_insert_with(|| DirNode {
            name: parts[depth].to_string(),
            files: Vec::new(),
            children: HashMap::new(),
        });

    insert_into_tree(child, parts, depth + 1);
}

fn render_tree(
    output: &mut String,
    node: &DirNode,
    file_ranks: &[FileRank],
    options: &RepoMapOptions,
) {
    render_tree_recursive(output, node, file_ranks, options, "", true);
}

fn render_tree_recursive(
    output: &mut String,
    node: &DirNode,
    file_ranks: &[FileRank],
    options: &RepoMapOptions,
    prefix: &str,
    is_last: bool,
) {
    let mut dirs: Vec<_> = node.children.keys().cloned().collect();
    dirs.sort();

    for (i, dir_name) in dirs.iter().enumerate() {
        let last = i == dirs.len() - 1 && node.files.is_empty();
        let connector = if is_last { "└── " } else { "├── " };
        let extension = if last { "    " } else { "│   " };

        output.push_str(&format!("{}{}{}/\n", prefix, connector, dir_name));

        if let Some(child) = node.children.get(dir_name) {
            render_tree_recursive(
                output,
                child,
                file_ranks,
                options,
                &format!("{}{}", prefix, extension),
                last,
            );
        }
    }

    let mut files: Vec<_> = node.files.iter().collect();
    files.sort();

    for (i, file_name) in files.iter().enumerate() {
        let last = i == files.len() - 1;
        let connector = if last { "└── " } else { "├── " };

        output.push_str(&format!("{}{}{}\n", prefix, connector, file_name));

        if options.include_signatures {
            if let Some(file_rank) = file_ranks
                .iter()
                .find(|f| f.path.ends_with(file_name.as_str()))
            {
                if let Some(ref parsed) = file_rank.parsed {
                    if !parsed.definitions.is_empty() {
                        let extension = if last { "    " } else { "│   " };
                        let sig_prefix = format!("{}{}    ", prefix, extension);

                        for def in &parsed.definitions {
                            let kind_str = match def.kind {
                                DefinitionKind::Function => "fn",
                                DefinitionKind::Class => "class",
                                DefinitionKind::Interface => "interface",
                                DefinitionKind::Struct => "struct",
                                DefinitionKind::Enum => "enum",
                                DefinitionKind::Constant => "const",
                                DefinitionKind::Variable => "var",
                            };
                            output.push_str(&format!(
                                "{}{} {} (L{})\n",
                                sig_prefix, kind_str, def.name, def.line
                            ));
                        }
                    }
                }
            }
        }
    }
}

pub fn generate_repomap_from_graph(
    repo_root: &Path,
    graph: &Graph,
    options: &RepoMapOptions,
) -> Result<String, crate::ScanError> {
    let config = ScanConfig::default();
    let repo_canon = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());

    let node_rankings = pagerank_from_graph(graph);

    let mut file_nodes: Vec<(&Node, f64)> = graph
        .nodes
        .iter()
        .filter_map(|node| {
            if node.path.is_some() {
                let score = node_rankings.get(&node.id).copied().unwrap_or(0.0);
                Some((node, score))
            } else {
                None
            }
        })
        .collect();

    file_nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    file_nodes.truncate(options.max_files);

    let collected = collect_files(repo_root, &config);
    let parsed_map: HashMap<String, ParsedFile> = collected
        .iter()
        .filter_map(|(path, content)| {
            let rel_path = path
                .strip_prefix(&repo_canon)
                .ok()
                .map(|p| p.to_string_lossy().to_string())?;
            detect_language(path)
                .and_then(|lang| parse_file(path, content, lang))
                .map(|parsed| (rel_path, parsed))
        })
        .collect();

    let mut file_ranks: Vec<FileRank> = file_nodes
        .iter()
        .filter_map(|(node, score)| {
            let path = node.path.as_ref()?;
            let rel_path = path.replace('\\', "/");
            let parsed = parsed_map.get(&rel_path).cloned();

            Some(FileRank {
                path: rel_path,
                score: *score,
                parsed,
            })
        })
        .collect();

    file_ranks.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    file_ranks.truncate(options.max_files);

    let mut output = String::new();
    output.push_str("# Repository Map\n\n");
    output.push_str(&format!(
        "Top {} files ranked by importance (PageRank)\n\n",
        file_ranks.len()
    ));

    let dir_tree = build_directory_tree(&file_ranks);
    render_tree(&mut output, &dir_tree, &file_ranks, options);

    Ok(output)
}

fn pagerank_from_graph(graph: &Graph) -> HashMap<String, f64> {
    let n = graph.nodes.len();
    if n == 0 {
        return HashMap::new();
    }

    let damping = 0.85;
    let iterations = 20;

    let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
    let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();

    for edge in &graph.edges {
        outgoing
            .entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
        incoming
            .entry(edge.target.clone())
            .or_default()
            .push(edge.source.clone());
    }

    let n_f64 = n as f64;
    let mut scores: HashMap<String, f64> = graph
        .nodes
        .iter()
        .map(|n| (n.id.clone(), 1.0 / n_f64))
        .collect();

    for _ in 0..iterations {
        let mut new_scores: HashMap<String, f64> = HashMap::new();

        for node in &graph.nodes {
            let mut score = (1.0 - damping) / n as f64;

            if let Some(predecessors) = incoming.get(&node.id) {
                for pred in predecessors {
                    let pred_score = scores.get(pred).copied().unwrap_or(0.0);
                    let out_degree = outgoing
                        .get(pred)
                        .map(|v| v.len().max(1) as f64)
                        .unwrap_or(1.0);
                    score += damping * pred_score / out_degree;
                }
            }

            new_scores.insert(node.id.clone(), score);
        }

        scores = new_scores;
    }

    scores
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repomap_options_default() {
        let opts = RepoMapOptions::default();
        assert_eq!(opts.max_files, 100);
        assert_eq!(opts.max_tokens, 5000);
        assert!(opts.include_signatures);
    }

    #[test]
    fn test_pagerank_empty() {
        let graph: HashMap<String, Vec<String>> = HashMap::new();
        let rankings = pagerank(&graph, 0);
        assert!(rankings.is_empty());
    }

    #[test]
    fn test_pagerank_single_node() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("a.rs".to_string(), vec![]);
        let rankings = pagerank(&graph, 1);
        assert!(!rankings.is_empty());
    }
}
