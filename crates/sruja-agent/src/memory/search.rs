//! Search and tag extraction for agentic memory.

use super::types::LearningEntry;
use super::AgenticMemory;

/// Finds learning entries relevant to a specific architectural element ID.
///
/// Relevancy is determined by:
/// 1. Direct match in `affected_elements`.
/// 2. Parent match (element_id starts with an affected element).
/// 3. String match in the `context` field.
pub fn find_relevant<'a>(memory: &'a AgenticMemory, element_id: &str) -> Vec<&'a LearningEntry> {
    memory
        .learnings
        .iter()
        .filter(|l| l.is_relevant_to(element_id))
        .collect()
}

/// Returns all entries in the same thematic cluster as the given entry ID.
///
/// Performs a transitive walk through `related_ids` links, returning
/// the full connected component -- analogous to opening a Zettelkasten "box."
pub fn find_cluster<'a>(memory: &'a AgenticMemory, entry_id: &str) -> Vec<&'a LearningEntry> {
    let index_map: std::collections::HashMap<&str, usize> = memory
        .learnings
        .iter()
        .enumerate()
        .map(|(i, e)| (e.id.as_str(), i))
        .collect();

    let Some(&start) = index_map.get(entry_id) else {
        return Vec::new();
    };

    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start);
    visited.insert(start);

    while let Some(idx) = queue.pop_front() {
        for related_id in &memory.learnings[idx].related_ids {
            if let Some(&ri) = index_map.get(related_id.as_str()) {
                if visited.insert(ri) {
                    queue.push_back(ri);
                }
            }
        }
    }

    visited.iter().map(|&i| &memory.learnings[i]).collect()
}

/// Returns all distinct thematic tags across all entries.
pub fn all_tags(memory: &AgenticMemory) -> Vec<String> {
    let mut tags: std::collections::HashSet<String> = std::collections::HashSet::new();
    for entry in &memory.learnings {
        for tag in &entry.tags {
            tags.insert(tag.clone());
        }
    }
    let mut sorted: Vec<String> = tags.into_iter().collect();
    sorted.sort();
    sorted
}

/// Returns all entries matching a given tag.
pub fn find_by_tag<'a>(memory: &'a AgenticMemory, tag: &str) -> Vec<&'a LearningEntry> {
    let tag_lower = tag.to_lowercase();
    memory
        .learnings
        .iter()
        .filter(|e| e.tags.iter().any(|t| t.to_lowercase() == tag_lower))
        .collect()
}

/// Extracts thematic tags from an entry's textual fields.
///
/// Tags are normalized, deduplicated keywords drawn from the context,
/// hypothesis, and guardrail text. Short common words are filtered out.
pub fn extract_tags(entry: &LearningEntry) -> Vec<String> {
    let combined = format!(
        "{} {} {}",
        entry.context, entry.hypothesis, entry.guardrail_advice
    );
    let stop_words: std::collections::HashSet<&str> = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "shall", "can",
        "need", "must", "to", "of", "in", "for", "on", "with", "at", "by", "from", "as", "into",
        "through", "during", "before", "after", "above", "below", "between", "out", "off", "over",
        "under", "again", "further", "then", "once", "and", "but", "or", "nor", "not", "no", "so",
        "if", "it", "its", "this", "that", "these", "those", "all", "each", "every", "both", "few",
        "more", "most", "other", "some", "such", "only", "same", "than", "too", "very", "just",
        "don", "now", "also", "use", "using", "used", "via",
    ]
    .into_iter()
    .collect();

    let mut seen = std::collections::HashSet::new();
    let mut tags = Vec::new();

    for word in combined.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_') {
        let w = word.to_lowercase();
        if w.len() >= 4 && !stop_words.contains(w.as_str()) && seen.insert(w.clone()) {
            tags.push(w);
        }
    }

    tags.truncate(12);
    tags
}

/// Finds indices of existing entries related to a new entry.
///
/// Relatedness is determined by shared affected elements, overlapping tags,
/// or matching context keywords -- implementing Zettelkasten's association logic.
pub fn find_related_indices(memory: &AgenticMemory, new_entry: &LearningEntry) -> Vec<usize> {
    let new_tags: std::collections::HashSet<&str> = new_entry.tags.iter().map(|s| s.as_str()).collect();
    let new_elements: std::collections::HashSet<&str> = new_entry
        .affected_elements
        .iter()
        .map(|s| s.as_str())
        .collect();

    let mut scored: Vec<(usize, u32)> = Vec::new();

    for (idx, existing) in memory.learnings.iter().enumerate() {
        let mut score: u32 = 0;

        let shared_elements = existing
            .affected_elements
            .iter()
            .filter(|e| {
                new_elements.contains(e.as_str())
                    || new_elements.iter().any(|ne| {
                        ne.starts_with(&format!("{}.", e)) || e.starts_with(&format!("{}.", ne))
                    })
            })
            .count();
        score += (shared_elements as u32) * 3;

        let shared_tags = existing
            .tags
            .iter()
            .filter(|t| new_tags.contains(t.as_str()))
            .count();
        score += (shared_tags as u32) * 2;

        let ctx_lower = existing.context.to_lowercase();
        let new_ctx_lower = new_entry.context.to_lowercase();
        let ctx_words: Vec<&str> = new_ctx_lower
            .split_whitespace()
            .filter(|w| w.len() >= 4)
            .collect();
        let ctx_overlap = ctx_words.iter().filter(|w| ctx_lower.contains(*w)).count();
        score += ctx_overlap as u32;

        if score >= 2 {
            scored.push((idx, score));
        }
    }

    scored.sort_by_key(|item| std::cmp::Reverse(item.1));
    scored.truncate(5);
    scored.into_iter().map(|(idx, _)| idx).collect()
}
