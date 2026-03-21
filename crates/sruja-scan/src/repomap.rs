//! Repository map generator for LLM context.
//!
//! Generates a tree-like structure of the repository annotated with
//! Tree-sitter signatures (structs, traits, functions) ranked by importance.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::graph::Graph;
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

#[derive(Debug, Default, Clone)]
struct RepoMapDiagnostics {
    language_files_seen: usize,
    collected_files: usize,
    read_failed: Vec<String>,
    skipped_large: Vec<String>,
    parse_failed: Vec<String>,
    unresolved_imports_by_file: HashMap<String, usize>,
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

fn build_import_graph(
    files: &[(String, ParsedFile)],
) -> (HashMap<String, Vec<String>>, HashMap<String, usize>) {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut unresolved_imports_by_file: HashMap<String, usize> = HashMap::new();

    let file_set: HashSet<String> = files
        .iter()
        .map(|(path, _): &(String, ParsedFile)| path.replace('\\', "/"))
        .collect();
    let local_roots = local_roots_from_files(&file_set);

    for (path, parsed) in files {
        let source = path.replace('\\', "/");
        let mut targets: Vec<String> = Vec::new();

        for import in &parsed.imports {
            if let Some(resolved) = resolve_import(import, &file_set, path, &local_roots) {
                targets.push(resolved);
            } else if should_count_unresolved(import, &local_roots) {
                *unresolved_imports_by_file
                    .entry(source.clone())
                    .or_insert(0) += 1;
            }
        }

        graph.insert(source, targets);
    }

    (graph, unresolved_imports_by_file)
}

fn resolve_import(
    import: &str,
    files: &HashSet<String>,
    source_path: &str,
    local_roots: &HashSet<String>,
) -> Option<String> {
    let import = import.trim().trim_matches('"').trim_matches('\'');
    if import.is_empty() {
        return None;
    }

    let mut candidates: Vec<String> = Vec::new();

    if import.starts_with('.') {
        candidates.extend(relative_candidates(source_path, import));
    }

    if import.contains("::") {
        candidates.extend(rust_candidates(source_path, import, files, local_roots));
    }

    if import.contains('.') && !import.starts_with('.') {
        candidates.extend(python_candidates(import));
    }
    if !import.contains('.') && local_roots.contains(import) {
        candidates.extend(python_candidates(import));
    }

    if import.contains('/') && !import.starts_with('.') {
        candidates.extend(path_candidates(import));
    }

    for cand in candidates {
        if let Some(found) = find_existing(files, &cand) {
            return Some(found);
        }
    }

    None
}

fn local_roots_from_files(files: &HashSet<String>) -> HashSet<String> {
    let mut roots: HashSet<String> = HashSet::new();
    for p in files {
        let p = p.replace('\\', "/");
        if let Some(first) = p.split('/').next() {
            if !first.is_empty() {
                roots.insert(first.to_string());
            }
        }
        if let Some(rest) = p.strip_prefix("crates/") {
            if let Some(crate_name) = rest.split('/').next() {
                if !crate_name.is_empty() {
                    roots.insert(crate_name.to_string());
                }
            }
        }
        if let Some(idx) = p.find("/src/") {
            let after = &p[idx + "/src/".len()..];
            if let Some(pkg) = after.split('/').next() {
                if !pkg.is_empty() {
                    roots.insert(pkg.to_string());
                }
            }
        }
    }
    roots
}

fn should_count_unresolved(import: &str, local_roots: &HashSet<String>) -> bool {
    let import = import.trim().trim_matches('"').trim_matches('\'');
    if import.is_empty() {
        return false;
    }
    if import.starts_with('.') || import.starts_with('/') {
        return true;
    }
    if import.contains('/') {
        return true;
    }
    if import.contains("::") {
        let first = import.split("::").next().unwrap_or("");
        if matches!(first, "std" | "core" | "alloc") {
            return false;
        }
        if matches!(first, "crate" | "self" | "super") {
            return true;
        }
        return local_roots.contains(first);
    }
    if import.contains('.') {
        let first = import.split('.').next().unwrap_or("");
        return local_roots.contains(first);
    }
    false
}

fn find_existing(files: &HashSet<String>, candidate: &str) -> Option<String> {
    let candidate = candidate.replace('\\', "/");
    if files.contains(&candidate) {
        return Some(candidate);
    }
    let suffix = format!("/{}", candidate);
    for p in files {
        if p.ends_with(&suffix) {
            return Some(p.clone());
        }
    }
    None
}

fn relative_candidates(source_path: &str, import: &str) -> Vec<String> {
    let source_dir = source_path
        .rfind('/')
        .map(|i| &source_path[..i])
        .unwrap_or("");
    let normalized = if import.starts_with('.')
        && !import.starts_with("./")
        && !import.starts_with("../")
        && !import.contains('/')
    {
        let dots = import.chars().take_while(|c| *c == '.').count();
        let up = dots.saturating_sub(1);
        let mut base = std::path::Path::new(source_dir).to_path_buf();
        for _ in 0..up {
            base.pop();
        }
        let rest = import[dots..].trim_start_matches('.');
        let rest = rest.replace('.', "/");
        let base = normalize_components(&base);
        if base.is_empty() {
            rest
        } else if rest.is_empty() {
            base
        } else {
            format!("{}/{}", base, rest)
        }
    } else {
        let base = std::path::Path::new(source_dir);
        let joined = base.join(import);
        normalize_components(&joined)
    };

    let mut out = Vec::new();
    out.push(normalized.clone());
    out.push(format!("{}.rs", normalized));
    out.push(format!("{}/mod.rs", normalized));
    out.push(format!("{}.py", normalized));
    out.push(format!("{}/__init__.py", normalized));
    out.push(format!("{}.js", normalized));
    out.push(format!("{}.ts", normalized));
    out.push(format!("{}/index.js", normalized));
    out.push(format!("{}/index.ts", normalized));
    out
}

fn normalize_components(path: &std::path::Path) -> String {
    use std::path::Component;

    let mut parts: Vec<String> = Vec::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop();
            }
            Component::Normal(s) => {
                parts.push(s.to_string_lossy().to_string());
            }
            Component::RootDir | Component::Prefix(_) => {}
        }
    }
    parts.join("/")
}

fn python_candidates(import: &str) -> Vec<String> {
    let path = import.replace('.', "/");
    vec![
        format!("{}.py", path),
        format!("{}/__init__.py", path),
        format!("src/{}.py", path),
        format!("src/{}/__init__.py", path),
    ]
}

fn path_candidates(import: &str) -> Vec<String> {
    let import = import.trim_start_matches('/');
    vec![
        import.to_string(),
        format!("{}.rs", import),
        format!("{}/mod.rs", import),
        format!("{}.py", import),
        format!("{}/__init__.py", import),
        format!("{}.js", import),
        format!("{}.ts", import),
        format!("{}/index.js", import),
        format!("{}/index.ts", import),
    ]
}

fn rust_candidates(
    source_path: &str,
    import: &str,
    files: &HashSet<String>,
    local_roots: &HashSet<String>,
) -> Vec<String> {
    let parts: Vec<&str> = import.split("::").filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return Vec::new();
    }

    let first = parts[0];
    if !matches!(first, "crate" | "self" | "super" | "std" | "core" | "alloc")
        && !local_roots.contains(first)
    {
        return Vec::new();
    }

    let source_dir = source_path
        .rfind('/')
        .map(|i| &source_path[..i])
        .unwrap_or("");

    let crate_root = if let Some(idx) = source_path.find("/src/") {
        let prefix = &source_path[..idx + "/src".len()];
        prefix.to_string()
    } else {
        "src".to_string()
    };

    let (base_dir, rest): (String, &[&str]) = match first {
        "crate" => (crate_root.clone(), &parts[1..]),
        "self" => (source_dir.to_string(), &parts[1..]),
        "super" => {
            let parent = std::path::Path::new(source_dir)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("");
            (parent.to_string(), &parts[1..])
        }
        _ => {
            if files
                .iter()
                .any(|p| p.starts_with(&format!("crates/{}/", first)))
            {
                (format!("crates/{}/src", first), &parts[1..])
            } else {
                (String::new(), &parts[1..])
            }
        }
    };

    let mut out = Vec::new();
    let mut push_paths = |segs: &[&str]| {
        if segs.is_empty() {
            return;
        }
        let joined = segs.join("/");
        if base_dir.is_empty() {
            out.push(format!("{}.rs", joined));
            out.push(format!("{}/mod.rs", joined));
            out.push(format!("src/{}.rs", joined));
            out.push(format!("src/{}/mod.rs", joined));
            return;
        }
        out.push(format!("{}/{}.rs", base_dir, joined));
        out.push(format!("{}/{}/mod.rs", base_dir, joined));
        out.push(format!("{}.rs", joined));
        out.push(format!("{}/mod.rs", joined));
    };

    push_paths(rest);
    if rest.len() >= 2 {
        push_paths(&rest[..rest.len() - 1]);
    }

    out
}

fn pagerank(graph: &HashMap<String, Vec<String>>) -> HashMap<String, f64> {
    if graph.is_empty() {
        return HashMap::new();
    }

    let damping = 0.85;
    let iterations = 20;

    let mut nodes: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (source, targets) in graph {
        nodes.insert(source.clone());
        for target in targets {
            nodes.insert(target.clone());
        }
    }
    let nodes: Vec<String> = nodes.into_iter().collect();
    let n = nodes.len().max(1);

    let mut scores: HashMap<String, f64> =
        nodes.iter().map(|k| (k.clone(), 1.0 / n as f64)).collect();

    let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
    for (source, targets) in graph {
        for target in targets {
            incoming
                .entry(target.clone())
                .or_default()
                .push(source.clone());
        }
    }
    for preds in incoming.values_mut() {
        preds.sort();
        preds.dedup();
    }

    for _ in 0..iterations {
        let mut new_scores: HashMap<String, f64> = HashMap::new();
        for node in &nodes {
            let mut score = (1.0 - damping) / n as f64;

            if let Some(predecessors) = incoming.get(node) {
                for pred in predecessors {
                    let pred_score = scores.get(pred).copied().unwrap_or(0.0);
                    let out_degree = graph
                        .get(pred)
                        .map(|v: &Vec<String>| v.len().max(1) as f64)
                        .unwrap_or(1.0);
                    score += damping * pred_score / out_degree;
                }
            }

            new_scores.insert(node.clone(), score);
        }

        scores = new_scores;
    }

    scores
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

fn render_tree(
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
        let score = node_rankings.get(&node.id).copied().unwrap_or(0.0);
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

fn rel_path(repo_root: &Path, repo_canon: &Path, path: &Path) -> String {
    let rel = path
        .strip_prefix(repo_canon)
        .or_else(|_| path.strip_prefix(repo_root))
        .unwrap_or(path);
    rel.to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string()
}

fn normalize_repo_rel(repo_prefixes: &[String], raw_path: &str) -> String {
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

#[derive(Debug, Clone)]
struct TokenBudget {
    max_tokens: usize,
    used_tokens: usize,
    truncated: bool,
}

impl TokenBudget {
    fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens: max_tokens.max(1),
            used_tokens: 0,
            truncated: false,
        }
    }

    fn estimate_tokens(s: &str) -> usize {
        s.len().div_ceil(4)
    }

    fn push_str(&mut self, out: &mut String, s: &str) -> bool {
        if self.truncated {
            return false;
        }
        let t = Self::estimate_tokens(s);
        if self.used_tokens.saturating_add(t) > self.max_tokens {
            self.truncated = true;
            return false;
        }
        out.push_str(s);
        self.used_tokens = self.used_tokens.saturating_add(t);
        true
    }

    fn finish(&mut self, out: &mut String) {
        if self.truncated {
            out.push_str("\n[truncated]\n");
        }
    }
}

fn render_diagnostics(
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

fn fan_in_out(import_graph: &HashMap<String, Vec<String>>) -> (FanoutList, FanoutList) {
    let mut out_counts: HashMap<&str, usize> = HashMap::new();
    let mut in_counts: HashMap<&str, usize> = HashMap::new();
    for (src, targets) in import_graph {
        out_counts.insert(src.as_str(), targets.len());
        for tgt in targets {
            *in_counts.entry(tgt.as_str()).or_insert(0) += 1;
        }
    }
    let mut fan_out: Vec<(String, usize)> = out_counts
        .into_iter()
        .filter(|(_, c)| *c >= 15)
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    fan_out.sort_by_key(|item| std::cmp::Reverse(item.1));

    let mut fan_in: Vec<(String, usize)> = in_counts
        .into_iter()
        .filter(|(_, c)| *c >= 15)
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    fan_in.sort_by_key(|item| std::cmp::Reverse(item.1));

    (fan_out, fan_in)
}

type FanoutList = Vec<(String, usize)>;

fn format_fanout(label: &str, items: &[(String, usize)]) -> Option<String> {
    let top: Vec<String> = items
        .iter()
        .take(3)
        .map(|(p, c)| format!("{} ({})", p, c))
        .collect();
    if top.is_empty() {
        None
    } else {
        Some(format!("- {} modules (top: {}).", label, top.join(", ")))
    }
}

fn find_dependency_cycles(
    graph: &HashMap<String, Vec<String>>,
    max_cycles: usize,
) -> Vec<Vec<String>> {
    struct Tarjan<'a> {
        graph: &'a HashMap<String, Vec<String>>,
        index: usize,
        stack: Vec<String>,
        on_stack: HashSet<String>,
        indices: HashMap<String, usize>,
        lowlink: HashMap<String, usize>,
        sccs: Vec<Vec<String>>,
    }

    impl<'a> Tarjan<'a> {
        fn new(graph: &'a HashMap<String, Vec<String>>) -> Self {
            Self {
                graph,
                index: 0,
                stack: Vec::new(),
                on_stack: HashSet::new(),
                indices: HashMap::new(),
                lowlink: HashMap::new(),
                sccs: Vec::new(),
            }
        }

        fn strongconnect(&mut self, v: String) {
            self.indices.insert(v.clone(), self.index);
            self.lowlink.insert(v.clone(), self.index);
            self.index += 1;
            self.stack.push(v.clone());
            self.on_stack.insert(v.clone());

            if let Some(targets) = self.graph.get(&v) {
                for w in targets {
                    if !self.indices.contains_key(w) {
                        self.strongconnect(w.clone());
                        let v_low = *self.lowlink.get(&v).unwrap_or(&0);
                        let w_low = *self.lowlink.get(w).unwrap_or(&0);
                        self.lowlink.insert(v.clone(), v_low.min(w_low));
                    } else if self.on_stack.contains(w) {
                        let v_low = *self.lowlink.get(&v).unwrap_or(&0);
                        let w_idx = *self.indices.get(w).unwrap_or(&0);
                        self.lowlink.insert(v.clone(), v_low.min(w_idx));
                    }
                }
            }

            let v_idx = *self.indices.get(&v).unwrap_or(&0);
            let v_low = *self.lowlink.get(&v).unwrap_or(&0);
            if v_low == v_idx {
                let mut scc: Vec<String> = Vec::new();
                while let Some(w) = self.stack.pop() {
                    self.on_stack.remove(&w);
                    scc.push(w.clone());
                    if w == v {
                        break;
                    }
                }
                if scc.len() >= 2 {
                    self.sccs.push(scc);
                }
            }
        }
    }

    let mut nodes: HashSet<String> = HashSet::new();
    for (src, targets) in graph {
        nodes.insert(src.clone());
        for t in targets {
            nodes.insert(t.clone());
        }
    }

    let mut nodes_vec: Vec<String> = nodes.into_iter().collect();
    nodes_vec.sort();
    let mut tarjan = Tarjan::new(graph);
    for v in nodes_vec {
        if !tarjan.indices.contains_key(&v) {
            tarjan.strongconnect(v);
        }
    }

    let mut sccs = tarjan.sccs;
    sccs.sort_by_key(|scc| std::cmp::Reverse(scc.len()));
    sccs.truncate(max_cycles);
    sccs
}

#[cfg(test)]
mod tests {
    use super::*;
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
