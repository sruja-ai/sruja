//! Curation logic for agentic memory.

use super::types::{
    CurationReport, LearningEntry, LearningKind, LowUtilityEntry, MergeSuggestion, StaleEntry,
};
use super::AgenticMemory;

/// Entries with many retrievals but low post-retrieval success (deletion candidates).
pub fn low_utility_entries(
    memory: &AgenticMemory,
    min_retrievals: u32,
    max_utility_ratio: f64,
) -> Vec<&LearningEntry> {
    memory
        .learnings
        .iter()
        .filter(|e| {
            e.retrieval_count >= min_retrievals
                && e.task_total_after > 0
                && e.utility_ratio().is_some_and(|r| r < max_utility_ratio)
        })
        .collect()
}

/// Builds a curation report for `sruja agent curate`.
pub fn curation_report(memory: &AgenticMemory) -> CurationReport {
    let low_utility = low_utility_entries(memory, 2, 0.4)
        .into_iter()
        .map(|e| LowUtilityEntry {
            id: e.id.clone(),
            retrieval_count: e.retrieval_count,
            task_total_after: e.task_total_after,
            utility_ratio: e.utility_ratio(),
            context: e.context.clone(),
        })
        .collect();

    let stale_threshold = 0.15_f64;
    let stale_entries: Vec<StaleEntry> = memory
        .learnings
        .iter()
        .filter(|e| {
            let score = e.decay_score();
            score < stale_threshold && e.age_days() > 30
        })
        .map(|e| StaleEntry {
            id: e.id.clone(),
            age_days: e.age_days(),
            decay_score: e.decay_score(),
            retrieval_count: e.retrieval_count,
            context: e.context.clone(),
        })
        .collect();

    let mut merge_suggestions = Vec::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

    for entry in &memory.learnings {
        if visited.contains(&entry.id) {
            continue;
        }
        let cluster = memory.find_cluster(&entry.id);
        if cluster.len() < 2 {
            continue;
        }
        for e in &cluster {
            visited.insert(e.id.clone());
        }
        let ids: Vec<String> = cluster.iter().map(|e| e.id.clone()).collect();
        let mut tag_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for e in &cluster {
            for t in &e.tags {
                *tag_counts.entry(t.as_str()).or_default() += 1;
            }
        }
        let shared_tags: Vec<String> = tag_counts
            .into_iter()
            .filter(|(_, c)| *c >= 2)
            .map(|(t, _)| t.to_string())
            .collect();
        merge_suggestions.push(MergeSuggestion {
            entry_ids: ids,
            shared_tags,
            cluster_size: cluster.len(),
        });
    }

    CurationReport {
        total_entries: memory.learnings.len(),
        low_utility,
        merge_suggestions,
        stale_entries,
    }
}

/// Archives entries that have decayed below the staleness threshold.
///
/// Returns the archived entries. Invariant entries are never archived.
pub fn auto_archive_stale(
    memory: &mut AgenticMemory,
    decay_threshold: f64,
    min_age_days: i64,
) -> Vec<LearningEntry> {
    let to_archive: Vec<String> = memory
        .learnings
        .iter()
        .filter(|e| {
            e.decay_score() < decay_threshold
                && e.age_days() > min_age_days
                && e.kind != Some(LearningKind::Invariant)
        })
        .map(|e| e.id.clone())
        .collect();

    let mut archived = Vec::new();
    for id in &to_archive {
        if let Ok(entry) = memory.delete_learning(id) {
            archived.push(entry);
        }
    }
    archived
}

/// Merges multiple entries into one, preserving links and utility counters.
pub fn merge_learnings(
    memory: &mut AgenticMemory,
    ids: &[String],
    mut merged: LearningEntry,
) -> Result<String, super::types::MemoryError> {
    use super::types::MemoryError;

    if ids.len() < 2 {
        return Err(MemoryError::InvalidIds(
            "merge requires at least two entry ids".to_string(),
        ));
    }

    let unique: std::collections::HashSet<&str> = ids.iter().map(|s| s.as_str()).collect();
    if unique.len() != ids.len() {
        return Err(MemoryError::InvalidIds(
            "duplicate ids in merge request".to_string(),
        ));
    }

    let mut to_remove: Vec<usize> = Vec::new();
    let mut tags = std::collections::HashSet::new();
    let mut elements = std::collections::HashSet::new();
    let mut evidence = std::collections::HashSet::new();
    let mut related = std::collections::HashSet::new();
    let mut retrieval_count: u32 = 0;

    for (idx, entry) in memory.learnings.iter().enumerate() {
        if ids.contains(&entry.id) {
            to_remove.push(idx);
            tags.extend(entry.tags.iter().cloned());
            elements.extend(entry.affected_elements.iter().cloned());
            evidence.extend(entry.evidence_refs.iter().cloned());
            related.extend(entry.related_ids.iter().cloned());
            retrieval_count = retrieval_count.saturating_add(entry.retrieval_count);
            // Task outcomes reset on merge — merged text is a new editorial artifact.
        }
    }

    if to_remove.len() != ids.len() {
        let missing: Vec<_> = ids
            .iter()
            .filter(|id| !memory.learnings.iter().any(|e| e.id == **id))
            .cloned()
            .collect();
        return Err(MemoryError::NotFound(missing.join(", ")));
    }

    for id in ids {
        related.remove(id.as_str());
    }

    if merged.id.is_empty() {
        merged.id = super::types::generate_entry_id();
    }
    let new_id = merged.id.clone();

    merged.tags = if merged.tags.is_empty() {
        tags.into_iter().collect()
    } else {
        let mut t: std::collections::HashSet<_> = merged.tags.into_iter().collect();
        t.extend(tags);
        t.into_iter().collect()
    };
    if merged.affected_elements.is_empty() {
        merged.affected_elements = elements.into_iter().collect();
    } else {
        let mut e: std::collections::HashSet<_> = merged.affected_elements.into_iter().collect();
        e.extend(elements);
        merged.affected_elements = e.into_iter().collect();
    }
    if merged.evidence_refs.is_empty() {
        merged.evidence_refs = evidence.into_iter().collect();
    } else {
        let mut e: std::collections::HashSet<_> = merged.evidence_refs.into_iter().collect();
        e.extend(evidence);
        merged.evidence_refs = e.into_iter().collect();
    }
    merged.related_ids = related.into_iter().collect();
    merged.retrieval_count = retrieval_count;
    merged.task_success_after = 0;
    merged.task_total_after = 0;

    for idx in to_remove.into_iter().rev() {
        memory.learnings.remove(idx);
    }

    for entry in &mut memory.learnings {
        for old_id in ids {
            if let Some(pos) = entry.related_ids.iter().position(|r| r == old_id) {
                entry.related_ids[pos] = new_id.clone();
            }
        }
        entry
            .related_ids
            .retain(|rid| rid != &new_id || entry.id == new_id);
    }

    memory.add_learning(merged);
    Ok(new_id)
}
