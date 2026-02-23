//! Vocabulary relationship graph: terms to components.

use std::collections::HashMap;

/// Maps terms to components that use them, and vice versa.
#[derive(Debug, Clone, Default)]
pub struct VocabularyGraph {
    /// term -> set of component ids
    term_to_components: HashMap<String, Vec<String>>,
    /// component -> set of terms
    component_to_terms: HashMap<String, Vec<String>>,
}

impl VocabularyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_component_vocabularies(
        vocabs: &[super::ComponentVocabulary],
    ) -> Self {
        let mut term_to_components: HashMap<String, Vec<String>> = HashMap::new();
        let mut component_to_terms: HashMap<String, Vec<String>> = HashMap::new();

        for cv in vocabs {
            let mut terms: Vec<String> = cv.terms.iter().map(|t| t.term.clone()).collect();
            terms.sort();
            terms.dedup();
            component_to_terms.insert(cv.component_id.clone(), terms.clone());

            for t in &terms {
                term_to_components
                    .entry(t.clone())
                    .or_default()
                    .push(cv.component_id.clone());
            }
        }

        for v in term_to_components.values_mut() {
            v.sort();
            v.dedup();
        }

        Self {
            term_to_components,
            component_to_terms,
        }
    }

    pub fn components_using_term(&self, term: &str) -> &[String] {
        self.term_to_components
            .get(term)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn terms_for_component(&self, component_id: &str) -> &[String] {
        self.component_to_terms
            .get(component_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn shared_terms(&self, a: &str, b: &str) -> Vec<String> {
        let ta: std::collections::HashSet<_> = self
            .terms_for_component(a)
            .iter()
            .map(|s| s.as_str())
            .collect();
        self.terms_for_component(b)
            .iter()
            .filter(|t| ta.contains(t.as_str()))
            .cloned()
            .collect()
    }

    pub fn term_count(&self) -> usize {
        self.term_to_components.len()
    }

    pub fn component_count(&self) -> usize {
        self.component_to_terms.len()
    }
}
