//! Domain vocabulary extraction from component text.
//!
//! Extracts tokens (identifiers, domain terms) for vocabulary graphs and
//! similarity analysis. Deterministic, no API required.

use std::collections::{HashMap, HashSet};

/// A term and its frequency within a component or corpus.
#[derive(Debug, Clone)]
pub struct TermCount {
    pub term: String,
    pub count: usize,
}

/// Extracted vocabulary for a single component.
#[derive(Debug, Clone)]
pub struct ComponentVocabulary {
    /// Component identifier (e.g. node id or path).
    pub component_id: String,
    /// Terms with their frequency.
    pub terms: Vec<TermCount>,
}

/// Extractor configuration.
#[derive(Debug, Clone)]
pub struct ExtractorConfig {
    /// Minimum term length (characters).
    pub min_term_len: usize,
    /// Maximum term length (chars); longer terms are split or truncated.
    pub max_term_len: usize,
    /// Minimum frequency to include a term in corpus stats.
    pub min_frequency: usize,
    /// Stopwords to exclude (lowercased).
    pub stopwords: HashSet<String>,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        let stopwords: HashSet<String> = [
            "the", "a", "an", "of", "to", "in", "for", "on", "with", "at",
            "by", "from", "is", "are", "was", "were", "be", "been", "and",
            "or", "but", "if", "then", "else", "it", "its", "as", "this",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        Self {
            min_term_len: 2,
            max_term_len: 50,
            min_frequency: 1,
            stopwords,
        }
    }
}

/// Extract domain vocabulary from component texts.
#[derive(Debug, Clone)]
pub struct VocabularyExtractor {
    config: ExtractorConfig,
}

impl VocabularyExtractor {
    pub fn new(config: ExtractorConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(ExtractorConfig::default())
    }

    /// Extract terms from a single text (component label, description, etc.).
    pub fn extract_from_text(&self, text: &str) -> Vec<TermCount> {
        let tokens = self.tokenize(text);
        let mut counts: HashMap<String, usize> = HashMap::new();
        for t in tokens {
            if self.config.stopwords.contains(&t) {
                continue;
            }
            if t.len() >= self.config.min_term_len && t.len() <= self.config.max_term_len {
                *counts.entry(t).or_insert(0) += 1;
            }
        }
        let mut out: Vec<TermCount> = counts
            .into_iter()
            .map(|(term, count)| TermCount { term, count })
            .collect();
        out.sort_by(|a, b| b.count.cmp(&a.count));
        out
    }

    /// Extract vocabulary for multiple components.
    pub fn extract_components(&self, components: &[(&str, &str)]) -> Vec<ComponentVocabulary> {
        components
            .iter()
            .map(|(id, text)| {
                let terms = self.extract_from_text(text);
                ComponentVocabulary {
                    component_id: (*id).to_string(),
                    terms,
                }
            })
            .collect()
    }

    /// Tokenize text into lowercase alphanumeric segments.
    /// Splits on non-alphanumeric (underscore preserved as part of identifier).
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect()
    }

    /// Corpus-wide term frequency across all components.
    pub fn corpus_frequency(
        &self,
        component_vocs: &[ComponentVocabulary],
    ) -> HashMap<String, usize> {
        let mut freq: HashMap<String, usize> = HashMap::new();
        for cv in component_vocs {
            for tc in &cv.terms {
                *freq.entry(tc.term.clone()).or_insert(0) += tc.count;
            }
        }
        freq.retain(|_, v| *v >= self.config.min_frequency);
        freq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple() {
        let ex = VocabularyExtractor::with_defaults();
        let t = ex.tokenize("OrderService processOrder");
        assert!(t.contains(&"orderservice".to_string()));
        assert!(t.contains(&"processorder".to_string()));
    }

    #[test]
    fn extract_from_text() {
        let ex = VocabularyExtractor::with_defaults();
        let tc = ex.extract_from_text("OrderService processOrder order cart");
        assert!(!tc.is_empty());
    }

    #[test]
    fn extract_components() {
        let ex = VocabularyExtractor::with_defaults();
        let comps = vec![
            ("order-service", "OrderService processOrder cart"),
            ("payment-service", "PaymentService processPayment fee"),
        ];
        let vocs = ex.extract_components(&comps);
        assert_eq!(vocs.len(), 2);
        assert_eq!(vocs[0].component_id, "order-service");
    }
}
