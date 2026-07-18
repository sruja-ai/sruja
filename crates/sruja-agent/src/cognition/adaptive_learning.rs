//! Adaptive learning from error and success patterns.
//!
//! This module tracks both error and success patterns across multiple agent runs
//! and provides insights for improving future performance. It learns which types
//! of errors are common and which approaches work well in specific codebases.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::ErrorClass;

/// Pattern of errors seen in a specific codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseErrorProfile {
    /// The repository path.
    pub repo_path: String,
    /// Error class frequency distribution.
    pub error_distribution: HashMap<ErrorClass, usize>,
    /// Total errors seen.
    pub total_errors: usize,
    /// Common error patterns with their frequency.
    pub patterns: Vec<ErrorPattern>,
    /// Last updated timestamp.
    pub last_updated: String,
}

/// A specific error pattern with context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    /// The error class.
    pub error_class: ErrorClass,
    /// Description of the pattern.
    pub description: String,
    /// How many times this pattern has occurred.
    pub frequency: usize,
    /// Suggested pre-emptive action.
    pub pre_emptive_action: Option<String>,
    /// Whether this pattern is increasing in frequency.
    pub trending_up: bool,
}

/// A success pattern observed in a codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessPattern {
    /// What worked well.
    pub description: String,
    /// The approach that succeeded.
    pub approach: String,
    /// How many times this approach has succeeded.
    pub frequency: usize,
    /// The task complexity this works for.
    pub task_complexity: Option<String>,
    /// Files or areas where this approach works.
    pub applicable_areas: Vec<String>,
    /// When this pattern was last observed.
    pub last_observed: String,
}

/// Outcome of an agent run for learning purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunOutcome {
    /// The goal statement.
    pub goal: String,
    /// Whether the run converged successfully.
    pub converged: bool,
    /// Number of iterations taken.
    pub iterations: usize,
    /// The approach description (from plan or step descriptions).
    pub approach: String,
    /// Files that were modified.
    pub modified_files: Vec<String>,
    /// Task complexity classification.
    pub complexity: String,
    /// Errors encountered (if any).
    pub errors: Vec<ErrorClass>,
    /// Timestamp.
    pub timestamp: String,
}

/// Adaptive learner that tracks error and success patterns and suggests improvements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdaptiveLearner {
    /// Profiles for different codebases.
    pub profiles: HashMap<String, CodebaseErrorProfile>,
    /// Global error statistics.
    pub global_stats: GlobalErrorStats,
    /// Learning insights discovered.
    pub insights: Vec<LearningInsight>,
    /// Success patterns observed.
    pub success_patterns: Vec<SuccessPattern>,
    /// Run history for learning from outcomes.
    pub run_history: Vec<RunOutcome>,
    /// Total successful runs.
    pub successful_runs: usize,
    /// Total failed runs.
    pub failed_runs: usize,
}

/// Global error statistics across all codebases.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalErrorStats {
    /// Total errors across all codebases.
    pub total_errors: usize,
    /// Error distribution across all codebases.
    pub error_distribution: HashMap<ErrorClass, usize>,
    /// Most common error patterns.
    pub common_patterns: Vec<String>,
}

/// An insight learned from error pattern analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningInsight {
    /// The insight description.
    pub description: String,
    /// The error pattern that led to this insight.
    pub trigger_pattern: ErrorPattern,
    /// Suggested action based on this insight.
    pub suggested_action: String,
    /// Confidence in this insight (0.0 - 1.0).
    pub confidence: f64,
    /// When this insight was discovered.
    pub discovered_at: String,
}

impl AdaptiveLearner {
    /// Create a new adaptive learner.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an error occurrence for a specific codebase.
    pub fn record_error(
        &mut self,
        repo_path: &str,
        error_class: ErrorClass,
        description: &str,
    ) {
        // Get pre-emptive action before mutable borrow
        let pre_emptive_action = self.suggest_pre_emptive_action(error_class);

        // Update codebase profile
        let profile = self
            .profiles
            .entry(repo_path.to_string())
            .or_insert_with(|| CodebaseErrorProfile {
                repo_path: repo_path.to_string(),
                error_distribution: HashMap::new(),
                total_errors: 0,
                patterns: Vec::new(),
                last_updated: chrono::Utc::now().to_rfc3339(),
            });

        *profile
            .error_distribution
            .entry(error_class)
            .or_insert(0) += 1;
        profile.total_errors += 1;
        profile.last_updated = chrono::Utc::now().to_rfc3339();

        // Update or create pattern
        let pattern = profile
            .patterns
            .iter_mut()
            .find(|p| p.error_class == error_class && p.description == description);

        if let Some(pattern) = pattern {
            pattern.frequency += 1;
            // Check if trending up (occurred in last 3 recordings)
            pattern.trending_up = pattern.frequency >= 3;
        } else {
            profile.patterns.push(ErrorPattern {
                error_class,
                description: description.to_string(),
                frequency: 1,
                pre_emptive_action,
                trending_up: false,
            });
        }

        // Update global stats
        *self
            .global_stats
            .error_distribution
            .entry(error_class)
            .or_insert(0) += 1;
        self.global_stats.total_errors += 1;

        // Generate insights if patterns are significant
        self.generate_insights(repo_path, error_class);
    }

    /// Suggest a pre-emptive action for an error class.
    fn suggest_pre_emptive_action(&self, error_class: ErrorClass) -> Option<String> {
        match error_class {
            ErrorClass::Compilation => Some("Run cargo check before making changes".to_string()),
            ErrorClass::Type => Some("Check type annotations first".to_string()),
            ErrorClass::Test => Some("Verify test expectations".to_string()),
            ErrorClass::Runtime => Some("Check for unwrap/None patterns".to_string()),
            ErrorClass::Lint => Some("Run cargo clippy --fix".to_string()),
            ErrorClass::Architecture => Some("Check architecture constraints".to_string()),
            ErrorClass::SpecGap => Some("Review acceptance criteria".to_string()),
            ErrorClass::Other => None,
        }
    }

    /// Generate insights based on error patterns.
    fn generate_insights(&mut self, repo_path: &str, error_class: ErrorClass) {
        let profile = match self.profiles.get(repo_path) {
            Some(p) => p,
            None => return,
        };

        let error_count = profile
            .error_distribution
            .get(&error_class)
            .copied()
            .unwrap_or(0);

        // Generate insight if error is frequent
        if error_count >= 3 {
            let insight = LearningInsight {
                description: format!(
                    "Frequent {:?} errors in {}",
                    error_class, repo_path
                ),
                trigger_pattern: ErrorPattern {
                    error_class,
                    description: format!("Recurring {:?} errors", error_class),
                    frequency: error_count,
                    pre_emptive_action: self.suggest_pre_emptive_action(error_class),
                    trending_up: true,
                },
                suggested_action: self
                    .suggest_pre_emptive_action(error_class)
                    .unwrap_or_else(|| "Investigate root cause".to_string()),
                confidence: (error_count as f64 / 10.0).min(0.95),
                discovered_at: chrono::Utc::now().to_rfc3339(),
            };

            // Only add if not already discovered
            if !self.insights.iter().any(|i| {
                i.trigger_pattern.error_class == error_class
                    && i.description.contains(repo_path)
            }) {
                self.insights.push(insight);
            }
        }
    }

    /// Get pre-emptive actions for a specific codebase.
    pub fn get_pre_emptive_actions(&self, repo_path: &str) -> Vec<String> {
        let profile = match self.profiles.get(repo_path) {
            Some(p) => p,
            None => return Vec::new(),
        };

        profile
            .patterns
            .iter()
            .filter(|p| p.frequency >= 2)
            .filter_map(|p| p.pre_emptive_action.clone())
            .collect()
    }

    /// Get insights for a specific codebase.
    pub fn get_insights_for_repo(&self, repo_path: &str) -> Vec<&LearningInsight> {
        self.insights
            .iter()
            .filter(|i| i.description.contains(repo_path))
            .collect()
    }

    /// Get the most common error patterns across all codebases.
    pub fn get_common_patterns(&self) -> Vec<(ErrorClass, usize)> {
        let mut patterns: Vec<(ErrorClass, usize)> = self
            .global_stats
            .error_distribution
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect();
        patterns.sort_by(|a, b| b.1.cmp(&a.1));
        patterns
    }

    /// Save the adaptive learner state to `.sruja/adaptive_learning.json`
    /// relative to the given repository root.
    pub fn save_to_path(&self, repo: &Path) -> std::io::Result<()> {
        let dir = repo.join(".sruja");
        fs::create_dir_all(&dir)?;
        let file_path = dir.join("adaptive_learning.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(file_path, json)
    }

    /// Load an adaptive learner state from `.sruja/adaptive_learning.json`
    /// relative to the given repository root. Returns `Ok(None)` if the file
    /// does not exist.
    pub fn load_from_path(repo: &Path) -> std::io::Result<Option<Self>> {
        let file_path = repo.join(".sruja").join("adaptive_learning.json");
        if !file_path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&file_path)?;
        let learner: Self = serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Some(learner))
    }

    /// Format all collected insights as a string suitable for injection into
    /// an LLM prompt. Returns an empty string when no insights exist.
    pub fn format_insights_for_prompt(&self) -> String {
        let has_insights = !self.insights.is_empty();
        let has_success = !self.success_patterns.is_empty();
        
        if !has_insights && !has_success {
            return String::new();
        }

        let mut out = String::new();
        
        // Error insights
        if has_insights {
            out.push_str("## Error Pattern Insights\n\n");
            for (i, insight) in self.insights.iter().enumerate() {
                out.push_str(&format!(
                    "{}. {} (confidence: {:.0}%)\n   Trigger: {:?} error — \"{}\" (seen {} times)\n   Suggested action: {}\n\n",
                    i + 1,
                    insight.description,
                    insight.confidence * 100.0,
                    insight.trigger_pattern.error_class,
                    insight.trigger_pattern.description,
                    insight.trigger_pattern.frequency,
                    insight.suggested_action,
                ));
            }
        }
        
        // Success patterns
        if has_success {
            out.push_str("## What Works Well\n\n");
            for pattern in &self.success_patterns {
                out.push_str(&format!(
                    "- {} (used {} times)\n  Approach: {}\n  Best for: {}\n\n",
                    pattern.description,
                    pattern.frequency,
                    pattern.approach,
                    pattern.applicable_areas.join(", ")
                ));
            }
        }
        
        out
    }

    /// Record a successful run outcome.
    pub fn record_success(
        &mut self,
        _repo_path: &str,
        goal: &str,
        approach: &str,
        modified_files: Vec<String>,
        complexity: &str,
        iterations: usize,
    ) {
        self.successful_runs += 1;
        
        let outcome = RunOutcome {
            goal: goal.to_string(),
            converged: true,
            iterations,
            approach: approach.to_string(),
            modified_files: modified_files.clone(),
            complexity: complexity.to_string(),
            errors: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.run_history.push(outcome);
        
        // Update or create success pattern
        let pattern = self.success_patterns.iter_mut().find(|p| p.approach == approach);
        if let Some(pattern) = pattern {
            pattern.frequency += 1;
            pattern.last_observed = chrono::Utc::now().to_rfc3339();
            // Add new applicable areas
            for file in &modified_files {
                if !pattern.applicable_areas.contains(file) {
                    pattern.applicable_areas.push(file.clone());
                }
            }
        } else {
            self.success_patterns.push(SuccessPattern {
                description: format!("Successful approach: {}", approach),
                approach: approach.to_string(),
                frequency: 1,
                task_complexity: Some(complexity.to_string()),
                applicable_areas: modified_files,
                last_observed: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    /// Record a failed run outcome.
    pub fn record_run_failure(
        &mut self,
        _repo_path: &str,
        goal: &str,
        approach: &str,
        errors: Vec<ErrorClass>,
        complexity: &str,
        iterations: usize,
    ) {
        self.failed_runs += 1;
        
        let outcome = RunOutcome {
            goal: goal.to_string(),
            converged: false,
            iterations,
            approach: approach.to_string(),
            modified_files: Vec::new(),
            complexity: complexity.to_string(),
            errors,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.run_history.push(outcome);
    }

    /// Get success patterns applicable to a specific complexity level.
    pub fn get_success_patterns_for_complexity(&self, complexity: &str) -> Vec<&SuccessPattern> {
        self.success_patterns
            .iter()
            .filter(|p| {
                p.task_complexity.as_deref() == Some(complexity) || p.task_complexity.is_none()
            })
            .collect()
    }

    /// Get the overall success rate.
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_runs + self.failed_runs;
        if total == 0 {
            0.0
        } else {
            self.successful_runs as f64 / total as f64
        }
    }

    /// Format success patterns for prompt injection.
    pub fn format_success_patterns_for_prompt(&self, complexity: &str) -> String {
        let patterns = self.get_success_patterns_for_complexity(complexity);
        if patterns.is_empty() {
            return String::new();
        }

        let mut out = String::from("## Approaches That Work Well\n\n");
        for pattern in patterns {
            out.push_str(&format!(
                "- {} (success rate: {:.0}%, used {} times)\n  Approach: {}\n\n",
                pattern.description,
                self.success_rate() * 100.0,
                pattern.frequency,
                pattern.approach,
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_error() {
        let mut learner = AdaptiveLearner::new();
        learner.record_error("/repo", ErrorClass::Compilation, "missing import");

        let profile = learner.profiles.get("/repo").unwrap();
        assert_eq!(profile.total_errors, 1);
        assert_eq!(
            profile.error_distribution.get(&ErrorClass::Compilation),
            Some(&1)
        );
    }

    #[test]
    fn test_insight_generation() {
        let mut learner = AdaptiveLearner::new();
        
        // Record 3 errors to trigger insight generation
        for _ in 0..3 {
            learner.record_error("/repo", ErrorClass::Compilation, "missing import");
        }

        let insights = learner.get_insights_for_repo("/repo");
        assert_eq!(insights.len(), 1);
        assert!(insights[0].description.contains("Compilation"));
    }

    #[test]
    fn test_pre_emptive_actions() {
        let mut learner = AdaptiveLearner::new();
        
        // Record errors to trigger pre-emptive actions
        for _ in 0..2 {
            learner.record_error("/repo", ErrorClass::Compilation, "missing import");
        }

        let actions = learner.get_pre_emptive_actions("/repo");
        assert!(!actions.is_empty());
        assert!(actions[0].contains("cargo check"));
    }

    #[test]
    fn test_record_success() {
        let mut learner = AdaptiveLearner::new();
        learner.record_success(
            "/repo",
            "add health endpoint",
            "read file, add endpoint, test",
            vec!["src/api.rs".to_string()],
            "simple",
            2,
        );

        assert_eq!(learner.successful_runs, 1);
        assert_eq!(learner.success_patterns.len(), 1);
        assert_eq!(learner.run_history.len(), 1);
    }

    #[test]
    fn test_success_rate() {
        let mut learner = AdaptiveLearner::new();
        learner.record_success("/repo", "goal", "approach", vec![], "simple", 1);
        learner.record_run_failure("/repo", "goal2", "approach2", vec![ErrorClass::Compilation], "simple", 1);

        let rate = learner.success_rate();
        assert!((rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_success_patterns_for_complexity() {
        let mut learner = AdaptiveLearner::new();
        learner.record_success(
            "/repo",
            "goal",
            "approach1",
            vec![],
            "simple",
            1,
        );
        learner.record_success(
            "/repo",
            "goal2",
            "approach2",
            vec![],
            "complex",
            2,
        );

        let simple_patterns = learner.get_success_patterns_for_complexity("simple");
        assert_eq!(simple_patterns.len(), 1);
        assert_eq!(simple_patterns[0].approach, "approach1");
    }
}