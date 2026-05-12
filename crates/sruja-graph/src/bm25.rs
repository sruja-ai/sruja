//! BM25 sparse retrieval over ingested external context documents.
//!
//! Provides a lightweight, local-first inverted index with BM25 scoring for
//! exact-match retrieval over documents in `.sruja/context/`. This guarantees
//! recall for specific terms, acronyms, and identifiers that substring matching
//! or embedding-based search may miss.
//!
//! BM25 parameters use the standard Lucene/Elasticsearch defaults (k1=1.2, b=0.75).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

const BM25_K1: f64 = 1.2;
const BM25_B: f64 = 0.75;

/// A single indexed document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDoc {
    pub path: String,
    pub title: String,
    pub category: String,
    pub word_count: usize,
    /// Linked architecture element IDs from YAML front-matter.
    #[serde(default)]
    pub linked_elements: Vec<String>,
}

/// Inverted index entry: maps a term to the documents containing it.
#[derive(Debug, Clone, Default)]
struct PostingList {
    entries: Vec<Posting>,
}

#[derive(Debug, Clone)]
struct Posting {
    doc_idx: usize,
    term_freq: u32,
}

/// BM25-scored search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bm25Hit {
    pub path: String,
    pub title: String,
    pub category: String,
    pub score: f64,
    pub matched_terms: Vec<String>,
    pub excerpt: String,
    pub linked_elements: Vec<String>,
}

/// Sparse inverted index over `.sruja/context/` documents.
///
/// Built on-demand via [`SparseIndex::build`]. Index construction is O(total_words)
/// across all context documents — typically sub-millisecond for < 100 documents.
/// No cross-call caching is performed; each `build()` re-reads the filesystem.
/// For repos with > 500 context documents, consider caching the index on disk
/// and invalidating on directory mtime changes.
pub struct SparseIndex {
    docs: Vec<IndexedDoc>,
    doc_contents: Vec<String>,
    index: HashMap<String, PostingList>,
    avg_doc_len: f64,
}

impl SparseIndex {
    /// Builds a BM25 index from all documents in `.sruja/context/`.
    pub fn build(repo_path: &Path) -> Self {
        let context_dir = repo_path.join(".sruja").join("context");
        let mut docs = Vec::new();
        let mut doc_contents = Vec::new();

        if context_dir.exists() && context_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&context_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if !matches!(ext.as_str(), "md" | "yaml" | "yml" | "json" | "txt") {
                        continue;
                    }

                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let name = path
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                        let category = crate::context_score::detect_context_category(
                            &name.to_lowercase(),
                            &ext,
                        );
                        let (body, linked_elements) = strip_front_matter(&content);
                        let word_count = body.split_whitespace().count();

                        let title = extract_title(&body).unwrap_or_else(|| name.clone());

                        docs.push(IndexedDoc {
                            path: path.to_string_lossy().to_string(),
                            title,
                            category,
                            word_count,
                            linked_elements,
                        });
                        doc_contents.push(body);
                    }
                }
            }
        }

        let total_words: usize = docs.iter().map(|d| d.word_count).sum();
        let avg_doc_len = if docs.is_empty() {
            1.0
        } else {
            total_words as f64 / docs.len() as f64
        };

        let mut index: HashMap<String, PostingList> = HashMap::new();
        for (doc_idx, content) in doc_contents.iter().enumerate() {
            let mut term_freqs: HashMap<String, u32> = HashMap::new();
            for token in tokenize(content) {
                *term_freqs.entry(token).or_insert(0) += 1;
            }
            for (term, freq) in term_freqs {
                index.entry(term).or_default().entries.push(Posting {
                    doc_idx,
                    term_freq: freq,
                });
            }
        }

        Self {
            docs,
            doc_contents,
            index,
            avg_doc_len,
        }
    }

    /// Searches the index for documents matching the query.
    ///
    /// Returns results sorted by BM25 score (descending), capped at `max_results`.
    pub fn search(&self, query: &str, max_results: usize) -> Vec<Bm25Hit> {
        if self.docs.is_empty() {
            return Vec::new();
        }

        let query_terms: Vec<String> = tokenize(query);
        let n = self.docs.len() as f64;

        let mut scores: HashMap<usize, (f64, Vec<String>)> = HashMap::new();

        for term in &query_terms {
            let Some(postings) = self.index.get(term) else {
                continue;
            };

            let df = postings.entries.len() as f64;
            let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();

            for posting in &postings.entries {
                let doc_len = self.docs[posting.doc_idx].word_count as f64;
                let tf = posting.term_freq as f64;
                let numerator = tf * (BM25_K1 + 1.0);
                let denominator =
                    tf + BM25_K1 * (1.0 - BM25_B + BM25_B * (doc_len / self.avg_doc_len));
                let bm25_term = idf * (numerator / denominator);

                let entry = scores
                    .entry(posting.doc_idx)
                    .or_insert_with(|| (0.0, Vec::new()));
                entry.0 += bm25_term;
                if !entry.1.contains(term) {
                    entry.1.push(term.clone());
                }
            }
        }

        let mut results: Vec<(usize, f64, Vec<String>)> = scores
            .into_iter()
            .map(|(idx, (score, terms))| (idx, score, terms))
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(max_results);

        results
            .into_iter()
            .map(|(idx, score, matched_terms)| {
                let doc = &self.docs[idx];
                let excerpt = extract_relevant_excerpt(
                    &self.doc_contents[idx],
                    matched_terms.first().map(|s| s.as_str()).unwrap_or(""),
                    200,
                );
                Bm25Hit {
                    path: doc.path.clone(),
                    title: doc.title.clone(),
                    category: doc.category.clone(),
                    score,
                    matched_terms,
                    excerpt,
                    linked_elements: doc.linked_elements.clone(),
                }
            })
            .collect()
    }

    /// Returns the number of indexed documents.
    pub fn doc_count(&self) -> usize {
        self.docs.len()
    }

    /// Returns the total number of unique terms in the index.
    pub fn term_count(&self) -> usize {
        self.index.len()
    }
}

/// Tokenizes text into lowercase alphanumeric terms.
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|w| w.len() >= 2)
        .map(|w| w.to_lowercase())
        .collect()
}

/// Strips YAML front-matter and returns the body and any linked elements.
fn strip_front_matter(content: &str) -> (String, Vec<String>) {
    let mut linked = Vec::new();

    if !content.starts_with("---") {
        return (content.to_string(), linked);
    }

    if let Some(end) = content[3..].find("---") {
        let front_matter = &content[3..3 + end];
        let body = &content[3 + end + 3..];

        for line in front_matter.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("elements:") {
                let rest = trimmed.strip_prefix("elements:").unwrap_or("").trim();
                let cleaned = rest.trim_start_matches('[').trim_end_matches(']');
                for elem in cleaned.split(',') {
                    let e = elem.trim().trim_matches('"').trim_matches('\'').to_string();
                    if !e.is_empty() {
                        linked.push(e);
                    }
                }
            }
        }

        return (body.to_string(), linked);
    }

    (content.to_string(), linked)
}

fn extract_title(body: &str) -> Option<String> {
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            return Some(heading.trim().to_string());
        }
    }
    None
}

fn extract_relevant_excerpt(content: &str, term: &str, max_len: usize) -> String {
    let content_lower = content.to_lowercase();
    let term_lower = term.to_lowercase();

    if let Some(pos) = content_lower.find(&term_lower) {
        let start = pos.saturating_sub(40);
        let end = (pos + term.len() + 120).min(content.len());
        let snippet = &content[start..end];
        let trimmed = snippet.trim();
        if trimmed.len() > max_len {
            format!("{}...", &trimmed[..max_len])
        } else {
            trimmed.to_string()
        }
    } else {
        let first_line = content.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        if first_line.len() > max_len {
            format!("{}...", &first_line[..max_len])
        } else {
            first_line.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_context_dir(dir: &Path) -> std::path::PathBuf {
        let ctx = dir.join(".sruja").join("context");
        std::fs::create_dir_all(&ctx).unwrap();
        ctx
    }

    #[test]
    fn build_empty_index() {
        let tmp = tempdir().unwrap();
        let idx = SparseIndex::build(tmp.path());
        assert_eq!(idx.doc_count(), 0);
        assert_eq!(idx.term_count(), 0);
    }

    #[test]
    fn build_and_search_basic() {
        let tmp = tempdir().unwrap();
        let ctx = setup_context_dir(tmp.path());
        std::fs::write(
            ctx.join("adr-001.md"),
            "# Use PostgreSQL\nWe chose PostgreSQL for ACID compliance and reliability.",
        )
        .unwrap();
        std::fs::write(
            ctx.join("adr-002.md"),
            "# Use Redis for Caching\nRedis provides sub-millisecond latency for hot paths.",
        )
        .unwrap();

        let idx = SparseIndex::build(tmp.path());
        assert_eq!(idx.doc_count(), 2);

        let results = idx.search("PostgreSQL ACID", 5);
        assert!(!results.is_empty());
        assert!(results[0].title.contains("PostgreSQL"));
        assert!(results[0].score > 0.0);
    }

    #[test]
    fn search_returns_no_results_for_missing_terms() {
        let tmp = tempdir().unwrap();
        let ctx = setup_context_dir(tmp.path());
        std::fs::write(ctx.join("note.md"), "# Some notes\nNothing relevant here.").unwrap();

        let idx = SparseIndex::build(tmp.path());
        let results = idx.search("xyznonexistent", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn front_matter_element_linking() {
        let tmp = tempdir().unwrap();
        let ctx = setup_context_dir(tmp.path());
        std::fs::write(
            ctx.join("adr-003.md"),
            "---\nelements: [Auth.Handler, Database.Users]\ncategory: adr\n---\n# JWT Authentication\nWe use JWT for stateless auth.",
        )
        .unwrap();

        let idx = SparseIndex::build(tmp.path());
        assert_eq!(idx.doc_count(), 1);

        let results = idx.search("JWT authentication", 5);
        assert!(!results.is_empty());
        assert!(results[0]
            .linked_elements
            .contains(&"Auth.Handler".to_string()));
        assert!(results[0]
            .linked_elements
            .contains(&"Database.Users".to_string()));
    }

    #[test]
    fn bm25_ranks_relevant_higher() {
        let tmp = tempdir().unwrap();
        let ctx = setup_context_dir(tmp.path());
        std::fs::write(
            ctx.join("specific.md"),
            "# Kubernetes Deployment\nKubernetes orchestrates our containers with health checks and auto-scaling. Kubernetes pods run in the production cluster.",
        )
        .unwrap();
        std::fs::write(
            ctx.join("general.md"),
            "# General Architecture\nOur system uses microservices deployed on various platforms with monitoring and logging.",
        )
        .unwrap();

        let idx = SparseIndex::build(tmp.path());
        let results = idx.search("kubernetes deployment", 5);
        assert!(!results.is_empty());
        assert!(
            results[0].title.contains("Kubernetes"),
            "Doc mentioning kubernetes should rank first, got: {}",
            results[0].title
        );
    }

    #[test]
    fn tokenize_handles_special_chars() {
        let tokens = tokenize("API.Routes -> Database.Users (HTTP/JSON)");
        assert!(tokens.contains(&"api".to_string()));
        assert!(tokens.contains(&"routes".to_string()));
        assert!(tokens.contains(&"database".to_string()));
        assert!(tokens.contains(&"http".to_string()));
        assert!(tokens.contains(&"json".to_string()));
    }

    #[test]
    fn strip_front_matter_extracts_elements() {
        let content = "---\nelements: [Auth, Database.Users]\n---\n# Title\nBody text.";
        let (body, elements) = strip_front_matter(content);
        assert_eq!(elements, vec!["Auth", "Database.Users"]);
        assert!(body.contains("Title"));
        assert!(!body.contains("elements:"));
    }

    #[test]
    fn strip_front_matter_no_front_matter() {
        let content = "# Just a document\nWith content.";
        let (body, elements) = strip_front_matter(content);
        assert_eq!(body, content);
        assert!(elements.is_empty());
    }
}
