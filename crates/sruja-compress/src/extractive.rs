//! Deterministic extractive prose compressor — no model weights, always on.
//!
//! Ports Headroom's TextCrusher: split into segments, score each by recency +
//! BM25 relevance to the query + structural salience, suppress near-duplicates
//! via word-shingle overlap, keep the top segments (in original order) up to the
//! target ratio. Output is extractive: kept text is verbatim — no rewrite.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::{
    count_tokens, restore_kept, BackendId, CompressContext, CompressError, Compressed,
    TextCompressor,
};

#[derive(Debug, Clone)]
pub struct TextCrusherConfig {
    pub target_ratio: f64,
    pub min_segments_for_crush: usize,
    pub min_segment_chars: usize,
    pub near_dup_threshold: f64,
    pub w_recency: f64,
    pub w_relevance: f64,
    pub w_salience: f64,
}

impl Default for TextCrusherConfig {
    fn default() -> Self {
        Self {
            target_ratio: 0.5,
            min_segments_for_crush: 6,
            min_segment_chars: 24,
            near_dup_threshold: 0.6,
            w_recency: 0.25,
            w_relevance: 0.5,
            w_salience: 0.25,
        }
    }
}

/// Lightweight BM25 over the segment corpus, scored against the query. Bounded
/// to ~[0,1] for blending with recency/salience. Swap for sruja's shared
/// relevance scorer at the integration seam.
struct BM25 {
    docs: Vec<Vec<String>>,
    df: HashMap<String, usize>,
    avgdl: f64,
    n: usize,
}

impl BM25 {
    fn new(docs: Vec<Vec<String>>) -> Self {
        let n = docs.len();
        let mut df: HashMap<String, usize> = HashMap::new();
        for d in &docs {
            for t in d.iter().collect::<HashSet<_>>() {
                *df.entry(t.clone()).or_default() += 1;
            }
        }
        let avgdl = if n == 0 {
            0.0
        } else {
            docs.iter().map(|d| d.len()).sum::<usize>() as f64 / n as f64
        };
        Self { docs, df, avgdl, n }
    }

    fn score(&self, idx: usize, query: &[String]) -> f64 {
        if self.n == 0 || self.avgdl == 0.0 {
            return 0.0;
        }
        let doc = &self.docs[idx];
        let k1 = 1.5f64;
        let b = 0.75f64;
        let mut s = 0.0;
        let qset: HashSet<&String> = query.iter().collect();
        let seen: HashSet<&String> = doc.iter().collect();
        for term in seen {
            if !qset.contains(term) {
                continue;
            }
            let df = self.df.get(term).copied().unwrap_or(0) as f64;
            let idf = (((self.n as f64 - df) + 0.5) / (df + 0.5) + 1.0).ln();
            let tf = doc.iter().filter(|t| *t == term).count() as f64;
            let denom = tf + k1 * (1.0 - b + b * doc.len() as f64 / self.avgdl);
            s += idf * (tf * (k1 + 1.0)) / denom;
        }
        (s / (s + 1.0)).tanh()
    }
}

const KEYWORDS: [&str; 10] = [
    "error",
    "exception",
    "failed",
    "failure",
    "fail",
    "warning",
    "traceback",
    "assert",
    "todo",
    "fixme",
];

pub struct TextCrusher {
    config: TextCrusherConfig,
}

impl Default for TextCrusher {
    fn default() -> Self {
        Self::new(TextCrusherConfig::default())
    }
}

impl TextCrusher {
    pub fn new(config: TextCrusherConfig) -> Self {
        Self { config }
    }

    fn passthrough(&self, content: &str) -> Compressed {
        let toks = count_tokens(content);
        Compressed {
            text: content.to_string(),
            original_tokens: toks,
            compressed_tokens: toks,
            backend: BackendId::TextCrusher,
            ccr_handle: None,
        }
    }
}

impl TextCompressor for TextCrusher {
    fn backend(&self) -> BackendId {
        BackendId::TextCrusher
    }

    fn compress(
        &self,
        content: &str,
        ctx: &CompressContext<'_>,
    ) -> Result<Compressed, CompressError> {
        let cfg = &self.config;
        let ratio = ctx
            .target_ratio
            .unwrap_or(cfg.target_ratio)
            .clamp(0.05, 1.0);

        let segments = split_segments(content);
        if segments.len() < cfg.min_segments_for_crush {
            return Ok(self.passthrough(content));
        }

        let n = segments.len();
        let total_chars: usize = segments.iter().map(|s| s.len()).sum();
        let target_chars = ((total_chars as f64 * ratio) as usize).max(1);

        let query_tokens: Vec<String> = match ctx.query {
            Some(q) => tokenize(q),
            None => Vec::new(),
        };

        let seg_tokens: Vec<Vec<String>> = segments.iter().map(|s| tokenize(s)).collect();
        let scorer = BM25::new(seg_tokens.clone());

        let mut scores = vec![0.0f64; n];
        for i in 0..n {
            let recency = (i as f64 + 1.0) / n as f64;
            let rel = if query_tokens.is_empty() {
                0.0
            } else {
                scorer.score(i, &query_tokens)
            };
            let words: Vec<&str> = segments[i].split_whitespace().collect();
            let salient = words.iter().filter(|w| is_salient(w)).count();
            let salience = salient as f64 / (words.len() as f64 + 1.0);
            let mut score =
                cfg.w_recency * recency + cfg.w_relevance * rel + cfg.w_salience * salience;
            if segments[i].len() < cfg.min_segment_chars {
                score *= 0.25;
            }
            scores[i] = score;
        }

        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| {
            scores[b]
                .partial_cmp(&scores[a])
                .unwrap_or(Ordering::Equal)
                .then(a.cmp(&b))
        });

        let mut kept = vec![false; n];
        let mut seen: HashSet<String> = HashSet::new();
        let mut kept_chars = 0usize;
        let mut kept_count = 0usize;
        for &i in &order {
            if kept_chars >= target_chars {
                break;
            }
            let sh = shingles(&seg_tokens[i], 3);
            if !sh.is_empty() {
                let covered =
                    sh.iter().filter(|s| seen.contains(*s)).count() as f64 / sh.len() as f64;
                if covered >= cfg.near_dup_threshold {
                    continue;
                }
            }
            kept[i] = true;
            kept_count += 1;
            for s in sh {
                seen.insert(s);
            }
            kept_chars += segments[i].len();
        }

        if kept_count == 0 {
            return Ok(self.passthrough(content));
        }

        let compressed: String = (0..n)
            .filter(|&i| kept[i])
            .map(|i| segments[i].as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let compressed = restore_kept(content, &compressed, &ctx.keep);

        let orig_tok = count_tokens(content);
        let comp_tok = count_tokens(&compressed);

        Ok(Compressed {
            text: compressed,
            original_tokens: orig_tok,
            compressed_tokens: comp_tok,
            backend: BackendId::TextCrusher,
            ccr_handle: None,
        })
    }
}

fn split_segments(text: &str) -> Vec<String> {
    let mut segs = Vec::new();
    for line in text.split('\n') {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut cur = String::new();
        let mut prev_term = false;
        for c in trimmed.chars() {
            if prev_term && c.is_whitespace() {
                let s = cur.trim();
                if !s.is_empty() {
                    segs.push(s.to_string());
                }
                cur.clear();
                prev_term = false;
                continue;
            }
            cur.push(c);
            prev_term = matches!(c, '.' | '!' | '?');
        }
        let s = cur.trim();
        if !s.is_empty() {
            segs.push(s.to_string());
        }
    }
    segs
}

fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() || c == '_' {
            for lc in c.to_lowercase() {
                cur.push(lc);
            }
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn shingles(words: &[String], k: usize) -> HashSet<String> {
    let mut set = HashSet::new();
    if words.is_empty() {
        return set;
    }
    if words.len() < k {
        for size in 1..=words.len() {
            for w in words.windows(size) {
                set.insert(w.join("\u{1}"));
            }
        }
        return set;
    }
    for w in words.windows(k) {
        set.insert(w.join("\u{1}"));
    }
    set
}

fn is_salient(word: &str) -> bool {
    if word.chars().any(|c| c.is_ascii_digit()) {
        return true;
    }
    let lower = word
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase();
    if KEYWORDS.contains(&lower.as_str()) {
        return true;
    }
    let alpha: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
    if alpha.len() >= 2 && alpha.iter().all(|c| c.is_uppercase()) {
        return true;
    }
    if let Some(dot) = word.find('.') {
        let a = &word[..dot];
        let b = &word[dot + 1..];
        if !a.is_empty()
            && !b.is_empty()
            && a.chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_')
            && b.chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_')
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(n: usize) -> String {
        (0..n)
            .map(|i| format!("Sentence number {i} describes a distinct topic {i} in some detail."))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn extractive_and_compresses() {
        let content = doc(40);
        let r = TextCrusher::default()
            .compress(&content, &CompressContext::default())
            .unwrap();
        assert!(r.compressed_tokens < r.original_tokens);
        let orig: HashSet<&str> = content.split_whitespace().collect();
        assert!(r.text.split_whitespace().all(|w| orig.contains(w)));
    }

    #[test]
    fn deterministic() {
        let content = doc(40);
        let tc = TextCrusher::default();
        assert_eq!(
            tc.compress(&content, &CompressContext::default())
                .unwrap()
                .text,
            tc.compress(&content, &CompressContext::default())
                .unwrap()
                .text
        );
    }

    #[test]
    fn passthrough_when_small() {
        let r = TextCrusher::default()
            .compress("one. two. three.", &CompressContext::default())
            .unwrap();
        assert_eq!(r.savings(), 0.0);
    }

    #[test]
    fn respects_keep_policy() {
        let content = "Error: something went wrong.\nfile: src/main.rs\nFixme: todo.\nOrdinary line one.\nOrdinary line two.";
        let mut ctx = CompressContext::default();
        ctx.keep = crate::KeepPolicy::for_tool_output();
        let r = TextCrusher::default().compress(content, &ctx).unwrap();
        assert!(r.text.contains("Error:"));
        assert!(r.text.contains("Fixme:"));
    }
}
