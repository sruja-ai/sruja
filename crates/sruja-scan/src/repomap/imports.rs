use std::collections::{HashMap, HashSet};

use crate::tree_sitter::ParsedFile;

pub(crate) fn build_import_graph(
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

pub(crate) fn resolve_import(
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

pub(crate) fn local_roots_from_files(files: &HashSet<String>) -> HashSet<String> {
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
