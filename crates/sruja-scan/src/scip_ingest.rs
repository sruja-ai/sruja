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
                        kind: EdgeKind::Calls,
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
