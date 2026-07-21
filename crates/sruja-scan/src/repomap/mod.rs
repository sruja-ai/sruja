//! Repository map generator for LLM context.
//!
//! Generates a tree-like structure of the repository annotated with
//! Tree-sitter signatures (structs, traits, functions) ranked by importance.

mod types;
mod imports;
mod render;
mod analyze;

pub use types::RepoMapOptions;
pub(crate) use types::*;
pub(crate) use imports::*;
pub(crate) use render::*;
pub(crate) use analyze::*;
pub(crate) use crate::tree_sitter::ParsedFile;

use std::collections::HashMap;
use std::path::Path;

use crate::graph::Graph;
use crate::tree_sitter::{detect_language, parse_file, ScanConfig};

pub fn generate_repomap(
    repo_root: &Path,
    options: &RepoMapOptions,
) -> Result<String, crate::ScanError> {
    let config = ScanConfig::default();
    let repo_canon = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());

    let (parsed_files, mut diagnostics) = collect_parsed_files(repo_root, &config, &repo_canon);

    let (import_graph, unresolved_imports_by_file) = build_import_graph(&parsed_files);
    diagnostics.unresolved_imports_by_file = unresolved_imports_by_file;
    let rankings = pagerank(&import_graph);

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

    file_ranks.sort_by(|a, b| match b.score.partial_cmp(&a.score) {
        Some(std::cmp::Ordering::Equal) | None => a.path.cmp(&b.path),
        Some(o) => o,
    });
    file_ranks.truncate(options.max_files);

    let mut output = String::new();
    let mut budget = TokenBudget::new(options.max_tokens);

    budget.push_str(&mut output, "# Repository Map\n\n");
    budget.push_str(
        &mut output,
        &format!(
            "Top {} files ranked by importance (PageRank)\n\n",
            file_ranks.len()
        ),
    );

    render_diagnostics(&mut output, &mut budget, &diagnostics, &import_graph);

    let dir_tree = build_directory_tree(&file_ranks);
    render_tree(&mut output, &mut budget, &dir_tree, &file_ranks, options);
    budget.finish(&mut output);

    Ok(output)
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

    let centralities = crate::graph::centrality::compute_all_centrality(graph);
    let mut repo_prefixes = vec![
        repo_root
            .to_string_lossy()
            .replace('\\', "/")
            .trim_end_matches('/')
            .to_string(),
        repo_canon
            .to_string_lossy()
            .replace('\\', "/")
            .trim_end_matches('/')
            .to_string(),
    ];
    repo_prefixes.sort_by_key(|p| std::cmp::Reverse(p.len()));
    repo_prefixes.dedup();

    let (parsed_files, mut diagnostics) = collect_parsed_files(repo_root, &config, &repo_canon);
    let parsed_map: HashMap<String, ParsedFile> = parsed_files.iter().cloned().collect();

    let mut best_by_path: HashMap<String, f64> = HashMap::new();
    for node in &graph.nodes {
        let Some(ref path) = node.path else {
            continue;
        };
        let rel_path = normalize_repo_rel(&repo_prefixes, path);
        if rel_path.is_empty() {
            continue;
        }
        let score = if let Some(c) = centralities.get(&node.id) {
            (c.pagerank * 0.4) + (c.betweenness_centrality * 0.4) + (c.degree_centrality * 0.2)
        } else {
            0.0
        };
        best_by_path
            .entry(rel_path)
            .and_modify(|s| *s = s.max(score))
            .or_insert(score);
    }

    let mut file_ranks: Vec<FileRank> = best_by_path
        .into_iter()
        .map(|(path, score)| {
            let parsed = parsed_map.get(&path).cloned();
            FileRank {
                path,
                score,
                parsed,
            }
        })
        .collect();

    file_ranks.sort_by(|a, b| match b.score.partial_cmp(&a.score) {
        Some(std::cmp::Ordering::Equal) | None => a.path.cmp(&b.path),
        Some(o) => o,
    });
    file_ranks.truncate(options.max_files);

    let (import_graph, unresolved_imports_by_file) = build_import_graph(&parsed_files);
    diagnostics.unresolved_imports_by_file = unresolved_imports_by_file;

    let mut output = String::new();
    let mut budget = TokenBudget::new(options.max_tokens);
    budget.push_str(&mut output, "# Repository Map\n\n");
    budget.push_str(
        &mut output,
        &format!(
            "Top {} files ranked by importance (PageRank)\n\n",
            file_ranks.len()
        ),
    );

    render_diagnostics(&mut output, &mut budget, &diagnostics, &import_graph);

    let dir_tree = build_directory_tree(&file_ranks);
    render_tree(&mut output, &mut budget, &dir_tree, &file_ranks, options);
    budget.finish(&mut output);

    Ok(output)
}

fn collect_parsed_files(
    repo_root: &Path,
    config: &ScanConfig,
    repo_canon: &Path,
) -> (Vec<(String, ParsedFile)>, RepoMapDiagnostics) {
    let walker = crate::tree_sitter::build_walker_internal(repo_root, config);
    let mut out: Vec<(String, ParsedFile)> = Vec::new();
    let mut diagnostics = RepoMapDiagnostics::default();

    for entry in walker {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(lang) = detect_language(path) else {
            continue;
        };
        diagnostics.language_files_seen += 1;

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                diagnostics
                    .read_failed
                    .push(rel_path(repo_root, repo_canon, path));
                continue;
            }
        };

        if content.len() > config.max_file_size {
            diagnostics
                .skipped_large
                .push(rel_path(repo_root, repo_canon, path));
            continue;
        }

        diagnostics.collected_files += 1;

        let rel = rel_path(repo_root, repo_canon, path);
        match parse_file(path, &content, lang) {
            Some(parsed) => out.push((rel, parsed)),
            None => diagnostics.parse_failed.push(rel),
        }
    }

    diagnostics.parse_failed.sort();
    diagnostics.parse_failed.dedup();

    (out, diagnostics)
}

fn build_directory_tree(file_ranks: &[FileRank]) -> DirNode {
    let mut root = DirNode {
        name: String::new(),
        files: Vec::new(),
        children: HashMap::new(),
    };

    for file in file_ranks {
        let parts: Vec<&str> = file.path.split('/').collect();
        insert_into_tree(&mut root, &parts, 0, &file.path);
    }

    root
}

fn insert_into_tree(node: &mut DirNode, parts: &[&str], depth: usize, full_path: &str) {
    if depth >= parts.len() {
        return;
    }

    if depth == parts.len() - 1 {
        node.files.push(full_path.to_string());
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

    insert_into_tree(child, parts, depth + 1, full_path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree_sitter::ParsedFile;
    use std::collections::{HashMap, HashSet};
    use tempfile::tempdir;

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
        let rankings = pagerank(&graph);
        assert!(rankings.is_empty());
    }

    #[test]
    fn test_pagerank_single_node() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("a.rs".to_string(), vec![]);
        let rankings = pagerank(&graph);
        assert!(!rankings.is_empty());
    }

    #[test]
    fn pagerank_is_deterministic_across_insertion_orders() {
        let mut g1: HashMap<String, Vec<String>> = HashMap::new();
        g1.insert(
            "a.rs".to_string(),
            vec!["b.rs".to_string(), "c.rs".to_string()],
        );
        g1.insert("b.rs".to_string(), vec!["c.rs".to_string()]);

        let mut g2: HashMap<String, Vec<String>> = HashMap::new();
        g2.insert("b.rs".to_string(), vec!["c.rs".to_string()]);
        g2.insert(
            "a.rs".to_string(),
            vec!["c.rs".to_string(), "b.rs".to_string()],
        );

        let r1 = pagerank(&g1);
        let r2 = pagerank(&g2);
        assert_eq!(r1, r2);
    }

    #[test]
    fn resolve_import_matches_by_substring() {
        let files: HashSet<String> = ["src/foo/bar.rs".to_string(), "src/baz/qux.rs".to_string()]
            .into_iter()
            .collect();
        let local_roots = local_roots_from_files(&files);

        let resolved = resolve_import("foo/bar", &files, "src/main.rs", &local_roots);
        assert_eq!(resolved.as_deref(), Some("src/foo/bar.rs"));
    }

    #[test]
    fn resolve_import_prefers_relative_resolution_for_dot_imports() {
        let files: HashSet<String> = ["src/a/mod.rs".to_string(), "src/b/util.rs".to_string()]
            .into_iter()
            .collect();
        let local_roots = local_roots_from_files(&files);

        let resolved = resolve_import("../b/util", &files, "src/a/mod.rs", &local_roots);
        assert_eq!(resolved.as_deref(), Some("src/b/util.rs"));
    }

    #[test]
    fn build_import_graph_resolves_imports_into_canonical_paths() {
        let files = vec![
            (
                "src/a/mod.rs".to_string(),
                ParsedFile {
                    name: "mod".to_string(),
                    path: "src/a/mod.rs".to_string(),
                    imports: vec!["../b/util".to_string()],
                    exports: Vec::new(),
                    definitions: Vec::new(),
                },
            ),
            (
                "src/b/util.rs".to_string(),
                ParsedFile {
                    name: "util".to_string(),
                    path: "src/b/util.rs".to_string(),
                    imports: Vec::new(),
                    exports: Vec::new(),
                    definitions: Vec::new(),
                },
            ),
        ];

        let (graph, _) = build_import_graph(&files);
        let targets = graph.get("src/a/mod.rs").expect("source present");
        assert_eq!(targets, &vec!["src/b/util.rs".to_string()]);
    }

    #[test]
    fn build_directory_tree_groups_files_by_folders() {
        let file_ranks = vec![
            FileRank {
                path: "src/a/mod.rs".to_string(),
                score: 1.0,
                parsed: None,
            },
            FileRank {
                path: "src/b/util.rs".to_string(),
                score: 1.0,
                parsed: None,
            },
        ];

        let tree = build_directory_tree(&file_ranks);
        assert!(tree.children.contains_key("src"));

        let src = tree.children.get("src").expect("src dir");
        assert!(src.children.contains_key("a"));
        assert!(src.children.contains_key("b"));
    }

    #[test]
    fn build_directory_tree_keeps_full_paths() {
        let file_ranks = vec![
            FileRank {
                path: "src/a/mod.rs".to_string(),
                score: 1.0,
                parsed: None,
            },
            FileRank {
                path: "src/b/mod.rs".to_string(),
                score: 1.0,
                parsed: None,
            },
        ];
        let tree = build_directory_tree(&file_ranks);
        let src = tree.children.get("src").expect("src dir");
        let a = src.children.get("a").expect("a dir");
        let b = src.children.get("b").expect("b dir");
        assert_eq!(a.files, vec!["src/a/mod.rs".to_string()]);
        assert_eq!(b.files, vec!["src/b/mod.rs".to_string()]);
    }

    #[test]
    fn generate_repomap_respects_token_budget() {
        let dir = tempdir().expect("tempdir");
        let repo_root = dir.path();
        std::fs::create_dir_all(repo_root.join("src")).expect("mkdir");
        std::fs::write(
            repo_root.join("src/main.rs"),
            "fn a() {}\nfn b() {}\nfn c() {}\nfn d() {}\nfn e() {}\nfn f() {}\n",
        )
        .expect("write file");

        let opts = RepoMapOptions {
            max_files: 100,
            max_tokens: 60,
            include_signatures: true,
        };
        let out = generate_repomap(repo_root, &opts).expect("repomap");
        assert!(out.contains("# Repository Map"));
        assert!(out.len() < 800);
        assert!(out.contains("[truncated]"));
    }
}
