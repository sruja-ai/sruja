use std::collections::HashMap;

use crate::tree_sitter::ParsedFile;

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
pub(crate) struct RepoMapDiagnostics {
    pub(crate) language_files_seen: usize,
    pub(crate) collected_files: usize,
    pub(crate) read_failed: Vec<String>,
    pub(crate) skipped_large: Vec<String>,
    pub(crate) parse_failed: Vec<String>,
    pub(crate) unresolved_imports_by_file: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct FileRank {
    pub(crate) path: String,
    pub(crate) score: f64,
    pub(crate) parsed: Option<ParsedFile>,
}

#[derive(Debug, Clone)]
pub(crate) struct DirNode {
    #[allow(dead_code)]
    pub(crate) name: String,
    pub(crate) files: Vec<String>,
    pub(crate) children: HashMap<String, DirNode>,
}

#[derive(Debug, Clone)]
pub(crate) struct TokenBudget {
    max_tokens: usize,
    used_tokens: usize,
    pub(crate) truncated: bool,
}

impl TokenBudget {
    pub(crate) fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens: max_tokens.max(1),
            used_tokens: 0,
            truncated: false,
        }
    }

    fn estimate_tokens(s: &str) -> usize {
        s.len().div_ceil(4)
    }

    pub(crate) fn push_str(&mut self, out: &mut String, s: &str) -> bool {
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

    pub(crate) fn finish(&mut self, out: &mut String) {
        if self.truncated {
            out.push_str("\n[truncated]\n");
        }
    }
}

pub(crate) type FanoutList = Vec<(String, usize)>;
