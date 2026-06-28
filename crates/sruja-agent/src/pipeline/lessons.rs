use std::collections::HashMap;

use super::types::{Lesson, PipelineRole};

/// Per-role lesson store with cap enforcement and prompt injection.
///
/// Lessons are recorded when a reviewer (Confirmer, Auditor) rejects work.
/// They're injected into the worker's system prompt on the next cycle as
/// "## Lessons from previous cycles".
///
/// Persisted as `LearningEntry`s with tag `"pipeline_lesson"` in the
/// existing `.sruja/agent_memory.json`.
#[derive(Debug, Clone)]
pub struct LessonStore {
    lessons: HashMap<PipelineRole, Vec<Lesson>>,
    max_per_role: usize,
}

impl LessonStore {
    pub fn new(max_per_role: usize) -> Self {
        Self {
            lessons: HashMap::new(),
            max_per_role,
        }
    }

    /// Record a lesson from a reviewer rejection.
    pub fn record(&mut self, lesson: Lesson) {
        let role = lesson.role;
        let lessons = self.lessons.entry(role).or_default();
        lessons.push(lesson);
        // Cap at max_per_role — keep most recent
        if lessons.len() > self.max_per_role {
            let excess = lessons.len() - self.max_per_role;
            lessons.drain(..excess);
        }
    }

    /// Get lessons for a specific role.
    pub fn for_role(&self, role: PipelineRole) -> &[Lesson] {
        self.lessons.get(&role).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Format lessons for prompt injection as markdown.
    pub fn format_for_prompt(&self, role: PipelineRole) -> String {
        let lessons = self.for_role(role);
        if lessons.is_empty() {
            return String::new();
        }
        let mut lines = vec!["## Lessons from previous cycles".to_string()];
        for (i, l) in lessons.iter().enumerate() {
            lines.push(format!("{}. {} → {}", i + 1, l.what_wrong, l.correction));
        }
        lines.join("\n")
    }

    /// Total lessons across all roles.
    pub fn total(&self) -> usize {
        self.lessons.values().map(|v| v.len()).sum()
    }

    /// Count of lessons per role.
    pub fn count_by_role(&self) -> Vec<(PipelineRole, usize)> {
        let mut counts: Vec<_> = self
            .lessons
            .iter()
            .map(|(r, v)| (*r, v.len()))
            .collect();
        counts.sort_by_key(|(r, _)| *r as u8);
        counts
    }

    /// Build from existing LearningEntry-like data stored in agent memory.
    /// Expects entries with tag "pipeline_lesson".
    pub fn from_memory_entries(
        entries: &[crate::memory::LearningEntry],
        max_per_role: usize,
    ) -> Self {
        let mut store = Self::new(max_per_role);
        for entry in entries {
            if !entry.tags.iter().any(|t| t == "pipeline_lesson") {
                continue;
            }
            let role = entry
                .tags
                .iter()
                .find_map(|t| t.strip_prefix("role:"))
                .and_then(|r| match r {
                    "analyzer" => Some(PipelineRole::Analyzer),
                    "prober" => Some(PipelineRole::Prober),
                    "confirmer" => Some(PipelineRole::Confirmer),
                    "fixer" => Some(PipelineRole::Fixer),
                    "auditor" => Some(PipelineRole::Auditor),
                    "retester" => Some(PipelineRole::ReTester),
                    "judge" => Some(PipelineRole::Judge),
                    _ => None,
                })
                .unwrap_or(PipelineRole::Confirmer);

            store.record(Lesson {
                id: entry.id.clone(),
                role,
                cycle: 0, // extracted from tags if needed
                what_wrong: entry.context.clone(),
                correction: entry.guardrail_advice.clone(),
            });
        }
        store
    }

    /// Flatten lessons into LearningEntries for persistence.
    pub fn to_learning_entries(&self) -> Vec<crate::memory::LearningEntry> {
        use chrono::Utc;

        let mut entries = Vec::new();
        for (role, lessons) in &self.lessons {
            for lesson in lessons {
                entries.push(crate::memory::LearningEntry {
                    id: lesson.id.clone(),
                    kind: None,
                    timestamp: Utc::now(),
                    run_id: None,
                    repo: None,
                    selector: None,
                    context: lesson.what_wrong.clone(),
                    hypothesis: format!("role:{} cycle:{}", role, lesson.cycle),
                    outcome: crate::ExperimentOutcome::Failed,
                    reason: None,
                    guardrail_advice: lesson.correction.clone(),
                    affected_elements: vec![],
                    evidence_refs: vec![],
                    confidence: None,
                    tags: vec!["pipeline_lesson".to_string(), format!("role:{}", role)],
                    hitl_kind: None,
                    related_ids: vec![],
                    retrieval_count: 0,
                    task_success_after: 0,
                    task_total_after: 0,
                });
            }
        }
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_format() {
        let mut store = LessonStore::new(15);
        store.record(Lesson {
            id: "l1".into(),
            role: PipelineRole::Prober,
            cycle: 1,
            what_wrong: "Filed bug without reading guard clause at line 52".into(),
            correction: "Trace through .get() calls before filing".into(),
        });
        let formatted = store.format_for_prompt(PipelineRole::Prober);
        assert!(formatted.contains("Filed bug without reading guard clause"));
        assert!(formatted.contains("Trace through .get() calls"));
    }

    #[test]
    fn test_empty_format() {
        let store = LessonStore::new(15);
        assert_eq!(store.format_for_prompt(PipelineRole::Fixer), "");
    }

    #[test]
    fn test_cap_per_role() {
        let mut store = LessonStore::new(3);
        for i in 0..10 {
            store.record(Lesson {
                id: format!("l{i}"),
                role: PipelineRole::Fixer,
                cycle: 1,
                what_wrong: format!("mistake {i}"),
                correction: "fix it".into(),
            });
        }
        assert_eq!(store.for_role(PipelineRole::Fixer).len(), 3);
        // Keeps most recent
        assert!(store.for_role(PipelineRole::Fixer)[0].id.contains('7'));
    }

    #[test]
    fn test_role_isolation() {
        let mut store = LessonStore::new(15);
        store.record(Lesson {
            id: "l1".into(),
            role: PipelineRole::Prober,
            cycle: 1,
            what_wrong: "prober mistake".into(),
            correction: "c".into(),
        });
        store.record(Lesson {
            id: "l2".into(),
            role: PipelineRole::Fixer,
            cycle: 1,
            what_wrong: "fixer mistake".into(),
            correction: "c".into(),
        });
        let prober_fmt = store.format_for_prompt(PipelineRole::Prober);
        assert!(prober_fmt.contains("prober mistake"));
        assert!(!prober_fmt.contains("fixer mistake"));
    }

    #[test]
    fn test_total_and_count() {
        let mut store = LessonStore::new(15);
        for i in 0..3 {
            store.record(Lesson {
                id: format!("l{i}"),
                role: PipelineRole::Prober,
                cycle: 1,
                what_wrong: "m".into(),
                correction: "c".into(),
            });
        }
        for i in 0..2 {
            store.record(Lesson {
                id: format!("l{i}"),
                role: PipelineRole::Fixer,
                cycle: 1,
                what_wrong: "m".into(),
                correction: "c".into(),
            });
        }
        assert_eq!(store.total(), 5);
        let counts = store.count_by_role();
        assert_eq!(counts.len(), 2);
    }
}
