//! Tree-sitter based code parsing for architecture extraction.
//!
//! This module parses source code files using Tree-sitter grammars to extract:
//! - Module/package structure from file paths
//! - Import statements (dependencies)
//! - Export statements (public interfaces)
//! - Function and class definitions (components)

pub mod classifier;
pub mod config;
pub mod detector;
pub mod languages;
mod scan;

pub use config::ScanConfig;
pub use detector::{detect_language, Language};
pub use languages::{Definition, DefinitionKind, ParsedFile};
pub use scan::{build_walker_internal, parse_file, scan_with_tree_sitter};

#[cfg(test)]
mod scan_diagnostics_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn walker_honors_srujaignore() {
        let dir = tempdir().expect("tempdir");
        let repo_root = dir.path();
        std::fs::create_dir_all(repo_root.join("src")).expect("mkdir src");
        std::fs::create_dir_all(repo_root.join("ignored")).expect("mkdir ignored");
        std::fs::write(repo_root.join(".srujaignore"), "ignored\n").expect("write .srujaignore");
        std::fs::write(repo_root.join("src/main.rs"), "fn main() {}\n").expect("write keep");
        std::fs::write(repo_root.join("ignored/bad.rs"), "fn bad() {}\n").expect("write ignored");

        let config = ScanConfig::default();
        let walker = build_walker_internal(repo_root, &config);
        let mut files: Vec<String> = Vec::new();
        for entry in walker {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if path.is_file() {
                files.push(
                    path.strip_prefix(repo_root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }

        assert!(files.contains(&"src/main.rs".to_string()));
        assert!(!files.iter().any(|p| p.contains("ignored/bad.rs")));
    }

    #[test]
    fn scan_attaches_basic_diagnostics_metadata() {
        let dir = tempdir().expect("tempdir");
        let repo_root = dir.path();
        std::fs::create_dir_all(repo_root.join("src")).expect("mkdir src");
        std::fs::write(repo_root.join("src/main.rs"), "fn main() {}\n").expect("write file");

        let config = ScanConfig::default();
        let graph = scan_with_tree_sitter(repo_root, &config).expect("scan");

        assert!(graph.metadata.contains_key("scan.language_files_seen"));
        assert!(graph.metadata.contains_key("scan.collected_files"));
        assert!(graph.metadata.contains_key("scan.parsed_files"));
        assert!(graph.metadata.contains_key("scan.read_failed"));
        assert!(graph.metadata.contains_key("scan.skipped_large"));
        assert!(graph.metadata.contains_key("scan.parse_failed"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let config = ScanConfig::default();
        let graph = scan_with_tree_sitter(dir.path(), &config).unwrap();
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn test_scan_typescript_file() {
        let dir = tempfile::tempdir().unwrap();
        let ts_file = dir.path().join("service.ts");
        std::fs::write(
            &ts_file,
            r#"
import { db } from './database';
import { User } from './models';

export class UserService {
    getUser(id: string) {
        return db.find(id);
    }
}
"#,
        )
        .unwrap();

        let config = ScanConfig::default();
        let graph = scan_with_tree_sitter(dir.path(), &config).unwrap();

        assert!(!graph.nodes.is_empty(), "Should have at least one node");
    }

    #[test]
    fn test_scan_java_files() {
        let dir = tempfile::tempdir().unwrap();
        let main_file = dir.path().join("src").join("Main.java");
        let helper_file = dir.path().join("src").join("util").join("Helper.java");
        std::fs::create_dir_all(helper_file.parent().unwrap()).unwrap();
        std::fs::write(
            &main_file,
            r#"
package com.example;

import com.example.util.Helper;

public class Main {
    public static void main(String[] args) {
        Helper.help();
    }
}
"#,
        )
        .unwrap();
        std::fs::write(
            &helper_file,
            r#"
package com.example.util;

public class Helper {
    public static int help() {
        return 1;
    }
}
"#,
        )
        .unwrap();

        let config = ScanConfig::default();
        let graph = scan_with_tree_sitter(dir.path(), &config).unwrap();

        assert!(!graph.nodes.is_empty(), "Should have nodes");
        assert!(
            graph.nodes.iter().any(|n| n.id.contains("Main_java")),
            "Should include Main.java node"
        );
    }

    #[test]
    fn test_incremental_scan() {
        let dir = tempfile::tempdir().unwrap();
        let ts_file = dir.path().join("service.ts");
        std::fs::write(
            &ts_file,
            r#"
import { db } from './database';
export class UserService {
    getUser(id: string) { return db.find(id); }
}
"#,
        )
        .unwrap();

        let config = ScanConfig {
            incremental: true,
            ..ScanConfig::default()
        };

        // First scan - builds cache
        let graph1 = scan_with_tree_sitter(dir.path(), &config).unwrap();
        assert!(!graph1.nodes.is_empty());
        assert!(dir
            .path()
            .join(".sruja")
            .join("scan_manifest.json")
            .exists());
        assert!(dir.path().join(".sruja").join("ast_cache.json").exists());

        // Second scan - unchanged, should hit cache
        let graph2 = scan_with_tree_sitter(dir.path(), &config).unwrap();
        assert_eq!(graph1.nodes.len(), graph2.nodes.len());

        // Modify file - should invalidate cache and update successfully
        std::fs::write(
            &ts_file,
            r#"
import { db } from './database';
import { logger } from './logger';
export class UserService {
    getUser(id: string) { logger.info(id); return db.find(id); }
}
"#,
        )
        .unwrap();

        let graph3 = scan_with_tree_sitter(dir.path(), &config).unwrap();
        assert!(!graph3.nodes.is_empty());
    }
}
