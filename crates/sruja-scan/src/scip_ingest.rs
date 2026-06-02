use crate::graph::{Edge, EdgeEvidence, EdgeKind, Graph};
use protobuf::Message;
use scip::types::Index;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// In-memory representation of a SCIP index for architectural analysis.
pub struct ScipIndex {
    pub documents: Vec<DocumentInfo>,
}

pub struct DocumentInfo {
    pub relative_path: String,
    pub occurrences: Vec<OccurrenceInfo>,
}

pub struct OccurrenceInfo {
    pub symbol: String,
    pub line: i32,
}

impl ScipIndex {
    /// Load a SCIP index from a binary .scip file.
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let index = Index::parse_from_bytes(&buffer[..])?;

        let mut documents = Vec::new();
        for doc in &index.documents {
            let mut occurrences = Vec::new();
            for occ in &doc.occurrences {
                if !occ.symbol.is_empty() {
                    occurrences.push(OccurrenceInfo {
                        symbol: occ.symbol.clone(),
                        line: occ.range.first().copied().unwrap_or(0),
                    });
                }
            }

            documents.push(DocumentInfo {
                relative_path: doc.relative_path.clone(),
                occurrences,
            });
        }

        Ok(ScipIndex { documents })
    }
}

/// Main entry point for SCIP enrichment during scanning.
pub fn enrich_with_scip(repo_root: &Path) -> Result<Graph, Box<dyn Error>> {
    let scip_path = repo_root.join("index.scip");
    if !scip_path.exists() {
        return Err("No index.scip found at repo root".into());
    }

    let index = ScipIndex::load_from_file(&scip_path)?;
    let nodes = Vec::new();
    let mut edges = Vec::new();

    for doc in &index.documents {
        let file_id = doc.relative_path.replace(['/', '\\', '.'], "_");

        for occ in &doc.occurrences {
            if !occ.symbol.is_empty() && is_interesting_symbol(&occ.symbol) {
                let target_id = resolve_symbol_to_id(&occ.symbol);
                if target_id != file_id {
                    edges.push(Edge {
                        source: file_id.clone(),
                        target: target_id,
                        kind: EdgeKind::new(EdgeKind::CALLS),
                        evidence: vec![EdgeEvidence {
                            rule: "scip_reference".to_string(),
                            file: Some(doc.relative_path.clone()),
                            line: Some(occ.line as u32 + 1), // SCIP is 0-indexed
                            detail: Some(format!("SCIP resolved symbol: {}", occ.symbol)),
                        }],
                        confidence: Default::default(),
                    });
                }
            }
        }
    }

    Ok(Graph {
        metadata: {
            let mut m = HashMap::new();
            m.insert("scip.enriched".to_string(), "true".to_string());
            m
        },
        nodes,
        edges,
        incidents: Vec::new(),
        confidence: Some(90),
    })
}

fn resolve_symbol_to_id(symbol: &str) -> String {
    if let Some(path_part) = symbol.split(' ').next_back() {
        if let Some(file_part) = path_part.split('/').next() {
            if file_part.contains('.') {
                return file_part.replace(['/', '\\', '.'], "_");
            }
        }
    }
    symbol.replace(['/', '\\', '.', ' '], "_")
}

fn is_interesting_symbol(symbol: &str) -> bool {
    !symbol.starts_with("local ") && !symbol.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        let mut dir = std::env::temp_dir();
        let uniq = format!(
            "{}-{}-{}",
            prefix,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        dir.push(uniq);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn interesting_symbol_filters_local_and_empty() {
        assert!(is_interesting_symbol("scip-java foo/bar"));
        assert!(!is_interesting_symbol(""));
        assert!(!is_interesting_symbol("local foo"));
    }

    #[test]
    fn resolve_symbol_to_id_is_deterministic_and_sanitizes() {
        let id1 = resolve_symbol_to_id("scip-typescript npm://pkg/foo bar/baz.ts");
        let id2 = resolve_symbol_to_id("scip-typescript npm://pkg/foo bar/baz.ts");
        assert_eq!(id1, id2);
        assert!(!id1.contains('/'));
        assert!(!id1.contains('.'));
        assert!(!id1.contains(' '));
    }

    #[test]
    fn resolve_symbol_to_id_uses_filename_when_path_segment_has_extension() {
        let id = resolve_symbol_to_id("scip-java maven/pkg Main.java");
        assert_eq!(id, "Main_java");
    }

    #[test]
    fn enrich_with_scip_errors_when_index_missing() {
        let dir = make_temp_dir("sruja-scip-missing");
        let err = enrich_with_scip(&dir).expect_err("expected missing index.scip error");
        assert!(err.to_string().contains("No index.scip"));
        // Best-effort cleanup (avoid failing the test if deletion fails).
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_symbol_to_id_handles_simple_symbol() {
        let id = resolve_symbol_to_id("my_symbol");
        assert_eq!(id, "my_symbol");
    }

    #[test]
    fn resolve_symbol_to_id_handles_dotted_path() {
        let id = resolve_symbol_to_id("com.example.MyClass");
        assert_eq!(id, "com_example_MyClass");
    }

    #[test]
    fn resolve_symbol_to_id_handles_slashed_path() {
        let id = resolve_symbol_to_id("src/main/java/MyClass");
        assert_eq!(id, "src_main_java_MyClass");
    }

    #[test]
    fn resolve_symbol_to_id_handles_scip_symbol_format() {
        let id = resolve_symbol_to_id("scip-java com.example MyClass");
        assert_eq!(id, "scip-java_com_example_MyClass");
    }

    #[test]
    fn resolve_symbol_to_id_handles_mixed_separators() {
        let id = resolve_symbol_to_id("com.example/my.module.Class");
        assert_eq!(id, "com_example");
    }

    #[test]
    fn resolve_symbol_to_id_handles_backslash() {
        let id = resolve_symbol_to_id("src\\main\\MyClass");
        assert_eq!(id, "src_main_MyClass");
    }

    #[test]
    fn scip_index_struct_creation() {
        let doc = DocumentInfo {
            relative_path: "src/main.rs".to_string(),
            occurrences: vec![
                OccurrenceInfo {
                    symbol: "com.example.MyClass".to_string(),
                    line: 10,
                },
                OccurrenceInfo {
                    symbol: "local_var".to_string(),
                    line: 15,
                },
            ],
        };

        assert_eq!(doc.relative_path, "src/main.rs");
        assert_eq!(doc.occurrences.len(), 2);
        assert_eq!(doc.occurrences[0].line, 10);
        assert_eq!(doc.occurrences[1].symbol, "local_var");
    }

    #[test]
    fn scip_index_multiple_documents() {
        let docs = vec![
            DocumentInfo {
                relative_path: "file1.rs".to_string(),
                occurrences: vec![],
            },
            DocumentInfo {
                relative_path: "file2.rs".to_string(),
                occurrences: vec![],
            },
        ];

        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].relative_path, "file1.rs");
        assert_eq!(docs[1].relative_path, "file2.rs");
    }

    #[test]
    fn resolve_symbol_to_id_empty_after_split() {
        let id = resolve_symbol_to_id(" ");
        assert_eq!(id, "_");
    }

    #[test]
    fn is_interesting_symbol_with_whitespace() {
        assert!(is_interesting_symbol("  "));
        assert!(is_interesting_symbol("a"));
        assert!(is_interesting_symbol("local"));
        assert!(!is_interesting_symbol("local "));
        assert!(!is_interesting_symbol("local foo"));
    }

    #[test]
    fn document_info_with_empty_occurrences() {
        let doc = DocumentInfo {
            relative_path: "empty.rs".to_string(),
            occurrences: Vec::new(),
        };

        assert!(doc.occurrences.is_empty());
    }

    #[test]
    fn occurrence_info_creation() {
        let occ = OccurrenceInfo {
            symbol: "test".to_string(),
            line: 42,
        };

        assert_eq!(occ.symbol, "test");
        assert_eq!(occ.line, 42);
    }

    #[test]
    fn resolve_symbol_to_id_preserves_underscores() {
        let id = resolve_symbol_to_id("my_symbol_name");
        assert_eq!(id, "my_symbol_name");
    }
}
