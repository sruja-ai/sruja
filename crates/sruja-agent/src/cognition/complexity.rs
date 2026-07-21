use serde::{Deserialize, Serialize};

/// Heuristic task complexity, determined from the goal statement and scope.
///
/// Controls prompt selection, TDD enforcement, tool-call budgets, and whether
/// post-loop artifacts (decision record, runbook) are generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskComplexity {
    /// One-line change: comment, typo, rename, format, whitespace.
    /// Skips TDD, skips post-loop artifacts, uses a minimal plan prompt.
    Trivial,
    /// Small change: 1-2 files, no architecture impact.
    /// Full review but lightweight planning.
    #[default]
    Simple,
    /// Multi-file change or moderate refactoring.
    /// Full TDD pipeline, full review.
    Moderate,
    /// Architecture-level: migration, system redesign, new module.
    /// Full pipeline, max iterations.
    Complex,
    /// Research/analysis: comprehend IS the output. No code changes produced.
    /// Pipeline: [Comprehend]. Recovery: Fail. Hard-capped at 1 iteration.
    Research,
}

impl TaskComplexity {
    /// Whether TDD should be enforced for this complexity level.
    pub fn enforce_tdd(self) -> bool {
        !matches!(self, TaskComplexity::Trivial | TaskComplexity::Research)
    }

    /// Whether post-loop artifacts (decision record, runbook) should be generated.
    pub fn generate_artifacts(self) -> bool {
        !matches!(self, TaskComplexity::Trivial)
    }

    /// Effective max tool iterations for this complexity level.
    pub fn max_tool_iterations(self, configured: usize) -> usize {
        match self {
            TaskComplexity::Trivial => configured.min(7),
            TaskComplexity::Simple => configured.min(7),
            TaskComplexity::Research => configured.min(10),
            _ => configured,
        }
    }
}

/// Uses keyword heuristics + scope (file/element count). Deterministic —
/// no LLM call — so it adds zero latency.
pub fn classify_task_complexity(
    goal: &str,
    target_files: &[String],
    target_elements: &[String],
) -> TaskComplexity {
    let goal_lower = goal.to_lowercase();
    let file_count = target_files.len();
    let element_count = target_elements.len();

    // Research heuristics: detect analysis/review-only goals BEFORE Complex so
    // "explain the migration system" → Research, not Complex (explaining is
    // research even about a complex topic).
    {
        let trimmed = goal_lower.trim();
        let starts_with_how = trimmed.starts_with("how to") || trimmed.starts_with("how do");
        let has_implementation_keywords = {
            let words: std::collections::HashSet<&str> = goal_lower.split_whitespace().collect();
            const IMPL_WORDS: &[&str] = &[
                "add",
                "create",
                "implement",
                "write",
                "edit",
                "fix",
                "refactor",
                "migrate",
                "delete",
                "remove",
                "modify",
            ];
            IMPL_WORDS.iter().any(|k| words.contains(k))
        };

        let is_question = trimmed.ends_with('?');
        let is_exploratory_prefix = [
            "what",
            "why",
            "explain",
            "analyze",
            "describe",
            "investigate",
            "evaluate",
            "review",
            "research",
        ]
        .iter()
        .any(|prefix| {
            let p = format!("{} ", prefix);
            goal_lower.starts_with(&p) || goal_lower == *prefix
        });

        if !starts_with_how
            && !has_implementation_keywords
            && (is_question || is_exploratory_prefix)
        {
            return TaskComplexity::Research;
        }
    }

    // Complex keywords: architecture-level work.
    let complex_keywords = [
        "migrate",
        "migration",
        "architecture",
        "redesign",
        "restructure",
        "system-wide",
        "overhaul",
    ];
    let is_complex_keyword = complex_keywords.iter().any(|k| goal_lower.contains(k));
    if is_complex_keyword || element_count >= 3 || file_count >= 5 {
        return TaskComplexity::Complex;
    }

    // Trivial keywords: cosmetic / single-token changes.
    let trivial_keywords = [
        "comment",
        "doc comment",
        "add a comment",
        "add comment",
        "typo",
        "spelling",
        "whitespace",
        "reformat",
        "add a blank line",
        "add newline",
    ];
    let is_trivial_keyword = trivial_keywords.iter().any(|k| goal_lower.contains(k));
    if is_trivial_keyword && file_count <= 1 && element_count == 0 {
        return TaskComplexity::Trivial;
    }

    // Rename is trivial when scoped to one file.
    if goal_lower.contains("rename") && file_count <= 1 {
        return TaskComplexity::Trivial;
    }

    // Simple: small scope, no architecture keywords.
    if file_count <= 2 && element_count <= 1 {
        return TaskComplexity::Simple;
    }

    TaskComplexity::Moderate
}
