use crate::graph::{Graph, Node, NodeKind};
use crate::ScanError;
use std::collections::HashMap;
use std::path::Path;

pub fn discover_docs_and_assets(repo_root: &Path) -> Result<Graph, ScanError> {
    let repo_canon = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());
    let mut graph = Graph::new();
    let walker = ignore::WalkBuilder::new(repo_root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .add_custom_ignore_filename(".srujaignore")
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if !crate::is_safe_path(path, &repo_canon) {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

        if path.components().any(|c| c.as_os_str() == ".sruja") {
            continue;
        }

        let rel_path = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
            .replace('\\', "/");

        if ext == "md" || ext == "markdown" || ext == "txt" {
            let mut label = file_name.to_string();
            let mut description = String::new();

            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("# ") {
                        label = trimmed.trim_start_matches("# ").trim().to_string();
                        break;
                    } else if !trimmed.is_empty() && description.is_empty() {
                        description = trimmed.to_string();
                    }
                }
            }

            let mut metadata = HashMap::new();
            metadata.insert("source_type".to_string(), "documentation".to_string());
            if !description.is_empty() {
                metadata.insert("description".to_string(), description);
            }

            let id = format!("doc:{}", rel_path.replace("/", "_"));
            let node = Node {
                id,
                kind: NodeKind::new("Doc"),
                label,
                path: Some(rel_path),
                metadata,
                ..Default::default()
            };
            graph.nodes.push(node);
        } else if ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "svg" || ext == "gif" {
            let mut metadata = HashMap::new();
            metadata.insert("source_type".to_string(), "asset".to_string());

            let id = format!("asset:{}", rel_path.replace("/", "_"));
            let node = Node {
                id,
                kind: NodeKind::new("Asset"),
                label: file_name.to_string(),
                path: Some(rel_path),
                metadata,
                ..Default::default()
            };
            graph.nodes.push(node);
        }
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_discover_docs_and_assets() {
        let dir = tempdir().unwrap();

        let md_file = dir.path().join("README.md");
        fs::write(&md_file, "# Project Title\nThis is a cool project.\n").unwrap();

        let png_file = dir.path().join("logo.png");
        fs::write(&png_file, b"fake_png_bytes").unwrap();

        let graph = discover_docs_and_assets(dir.path()).unwrap();
        assert_eq!(graph.nodes.len(), 2);

        let doc_node = graph
            .nodes
            .iter()
            .find(|n| n.id.starts_with("doc:"))
            .unwrap();
        assert_eq!(doc_node.kind, "Doc");
        assert_eq!(doc_node.label, "Project Title");
        assert_eq!(doc_node.path.as_deref(), Some("README.md"));

        let asset_node = graph
            .nodes
            .iter()
            .find(|n| n.id.starts_with("asset:"))
            .unwrap();
        assert_eq!(asset_node.kind, "Asset");
        assert_eq!(asset_node.label, "logo.png");
        assert_eq!(asset_node.path.as_deref(), Some("logo.png"));
    }
}
