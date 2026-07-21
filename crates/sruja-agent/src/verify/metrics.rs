//! Cross-iteration test metrics tracking and regression detection.
//!
//! Stores per-iteration [`TestSuiteResult`] snapshots and computes:
//! - Pass rate progression across iterations
//! - Regression detection (tests that were passing but now fail)
//! - Convergence decisions based on [`ConvergenceConfig`]

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::manifest::{ConvergenceConfig, ConvergenceMode};
use crate::verify::test_parser::{TestFailure, TestSuiteResult};

/// A single iteration's test metrics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationSnapshot {
    /// 1-based iteration index.
    pub iteration: usize,
    /// Parsed test suite result for this iteration.
    pub suite: TestSuiteResult,
    /// Set of test names that failed in this iteration.
    pub failed_test_names: HashSet<String>,
    /// Set of test names that failed in the PREVIOUS iteration (for regression detection).
    pub previous_failed_test_names: HashSet<String>,
    /// Tests that regressed: were passing last iteration, now failing.
    pub regressions: Vec<TestFailure>,
    /// Tests that recovered: were failing last iteration, now passing.
    pub recoveries: Vec<String>,
    /// Timestamp (ISO 8601).
    pub timestamp: String,
}

/// Tracks test metrics across loop iterations and drives convergence decisions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricsTracker {
    /// Per-iteration snapshots, ordered by iteration number.
    pub snapshots: Vec<IterationSnapshot>,
}

impl MetricsTracker {
    /// Record a new iteration's test results and detect regressions.
    pub fn record(&mut self, iteration: usize, suite: TestSuiteResult) -> &IterationSnapshot {
        let failed_names: HashSet<String> = suite.failures.iter().map(|f| f.name.clone()).collect();

        // Get previous iteration's failures for regression detection
        let prev_failed: HashSet<String> = self
            .snapshots
            .last()
            .map(|s| s.failed_test_names.clone())
            .unwrap_or_default();

        // Regressions: failed now but not before
        let mut regressions = Vec::new();
        for failure in &suite.failures {
            if !prev_failed.contains(&failure.name) && !prev_failed.is_empty() {
                let mut f = failure.clone();
                f.is_regression = true;
                regressions.push(f);
            }
        }

        // Recoveries: failed before but passing now
        let current_passed: HashSet<String> = if suite.total > 0 {
            // All test names that are NOT in the failures set
            // We don't have the full list of test names, so we use the previous
            // failures set minus current failures as recoveries
            prev_failed.difference(&failed_names).cloned().collect()
        } else {
            HashSet::new()
        };
        let recoveries: Vec<String> = current_passed.into_iter().collect();

        let snapshot = IterationSnapshot {
            iteration,
            suite,
            failed_test_names: failed_names,
            previous_failed_test_names: prev_failed,
            regressions,
            recoveries,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.snapshots.push(snapshot);
        self.snapshots.last().unwrap()
    }

    /// Get the latest pass rate (0.0–1.0).
    pub fn current_pass_rate(&self) -> f64 {
        self.snapshots
            .last()
            .map(|s| s.suite.pass_rate)
            .unwrap_or(0.0)
    }

    /// Get the pass rate from the previous iteration.
    pub fn previous_pass_rate(&self) -> f64 {
        if self.snapshots.len() >= 2 {
            self.snapshots[self.snapshots.len() - 2].suite.pass_rate
        } else {
            0.0
        }
    }

    /// Compute the improvement in pass rate since the previous iteration.
    pub fn pass_rate_delta(&self) -> f64 {
        self.current_pass_rate() - self.previous_pass_rate()
    }

    /// Check if there are any regressions in the latest iteration.
    pub fn has_regressions(&self) -> bool {
        self.snapshots
            .last()
            .map(|s| !s.regressions.is_empty())
            .unwrap_or(false)
    }

    /// Get regressions from the latest iteration.
    pub fn latest_regressions(&self) -> &[TestFailure] {
        self.snapshots
            .last()
            .map(|s| s.regressions.as_slice())
            .unwrap_or(&[])
    }

    /// Evaluate convergence based on the configured mode.
    ///
    /// Returns `Some(reason)` if the loop should stop, `None` if it should continue.
    pub fn check_convergence(&self, config: &ConvergenceConfig) -> Option<String> {
        if self.snapshots.is_empty() {
            return None;
        }

        match config.mode {
            ConvergenceMode::PassRateOnly => self.check_pass_rate_only(config),
            ConvergenceMode::MetricDriven => self.check_metric_driven(config),
            ConvergenceMode::NoImprovement => self.check_no_improvement(config),
        }
    }

    fn check_pass_rate_only(&self, config: &ConvergenceConfig) -> Option<String> {
        let pass_rate = self.current_pass_rate();
        if pass_rate >= config.min_pass_rate {
            Some(format!(
                "pass rate {pass_rate:.1}% >= target {:.1}%",
                config.min_pass_rate * 100.0
            ))
        } else {
            None
        }
    }

    fn check_metric_driven(&self, config: &ConvergenceConfig) -> Option<String> {
        // If no custom metrics defined, fall back to pass_rate_only
        if config.metrics.is_empty() {
            return self.check_pass_rate_only(config);
        }

        let mut all_met = true;
        let mut details = Vec::new();

        for (name, target) in &config.metrics {
            let value = self.compute_metric(name);
            let met = value >= target.target;
            if !met {
                all_met = false;
            }
            details.push(format!(
                "{name}={value:.3} (target={:.3}, weight={:.1}, {})",
                target.target,
                target.weight,
                if met { "met" } else { "NOT met" }
            ));
        }

        if all_met {
            Some(format!("all metrics met: {}", details.join(", ")))
        } else {
            None
        }
    }

    fn check_no_improvement(&self, config: &ConvergenceConfig) -> Option<String> {
        if self.snapshots.len() < 2 {
            return None;
        }

        let delta = self.pass_rate_delta().abs();
        if delta < config.convergence_threshold {
            Some(format!(
                "pass rate improvement ({delta:.4}) below threshold ({:.4})",
                config.convergence_threshold
            ))
        } else {
            None
        }
    }

    /// Compute a named metric value from the latest snapshot.
    fn compute_metric(&self, name: &str) -> f64 {
        match name {
            "pass_rate" => self.current_pass_rate(),
            "coverage_delta" => {
                // Coverage delta: how much the pass rate improved since iteration 1
                let baseline = self
                    .snapshots
                    .first()
                    .map(|s| s.suite.pass_rate)
                    .unwrap_or(0.0);
                self.current_pass_rate() - baseline
            }
            _ => 0.0,
        }
    }
}

/// Persistence for metrics across crash/resume cycles.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoopState {
    /// Goal statement.
    pub goal: String,
    /// All iteration snapshots.
    pub snapshots: Vec<IterationSnapshot>,
    /// Final convergence reason, if terminated.
    pub convergence_reason: Option<String>,
    /// Timestamp of last update.
    pub last_updated: String,
}

impl LoopState {
    /// Load from `.sruja/loop-state.json`. Returns default if file doesn't exist.
    pub fn load(repo: &Path) -> Self {
        let path = repo.join(".sruja/loop-state.json");
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save to `.sruja/loop-state.json`.
    pub fn save(&self, repo: &Path) -> std::io::Result<()> {
        let dir = repo.join(".sruja");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("loop-state.json");
        let json = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(path, json)
    }

    /// Build from a [`MetricsTracker`].
    pub fn from_tracker(tracker: &MetricsTracker, goal: &str) -> Self {
        Self {
            goal: goal.to_string(),
            snapshots: tracker.snapshots.clone(),
            convergence_reason: None,
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::MetricTarget;
    use crate::verify::test_parser::{TestFormat, TestSuiteResult};

    fn make_suite(passed: usize, failed: usize, failure_names: &[&str]) -> TestSuiteResult {
        let total = passed + failed;
        TestSuiteResult {
            total,
            passed,
            failed,
            skipped: 0,
            failures: failure_names
                .iter()
                .map(|n| TestFailure {
                    name: n.to_string(),
                    file: None,
                    error: "error".into(),
                    is_regression: false,
                })
                .collect(),
            pass_rate: if total > 0 {
                passed as f64 / total as f64
            } else {
                1.0
            },
            duration_ms: None,
            format: TestFormat::CargoTest,
        }
    }

    #[test]
    fn tracks_pass_rate_across_iterations() {
        let mut tracker = MetricsTracker::default();
        tracker.record(1, make_suite(7, 3, &["a", "b", "c"]));
        assert!((tracker.current_pass_rate() - 0.7).abs() < 0.01);

        tracker.record(2, make_suite(8, 2, &["a", "b"]));
        assert!((tracker.current_pass_rate() - 0.8).abs() < 0.01);
        assert!((tracker.pass_rate_delta() - 0.1).abs() < 0.01);
    }

    #[test]
    fn detects_regressions() {
        let mut tracker = MetricsTracker::default();
        tracker.record(1, make_suite(8, 2, &["test_a", "test_b"]));

        // test_c is new failure -> regression
        tracker.record(2, make_suite(8, 2, &["test_a", "test_c"]));

        let snap = tracker.snapshots.last().unwrap();
        assert_eq!(snap.regressions.len(), 1);
        assert_eq!(snap.regressions[0].name, "test_c");
        assert!(snap.regressions[0].is_regression);
    }

    #[test]
    fn detects_recoveries() {
        let mut tracker = MetricsTracker::default();
        tracker.record(1, make_suite(8, 2, &["test_a", "test_b"]));

        // test_b recovered, test_a still failing
        tracker.record(2, make_suite(9, 1, &["test_a"]));

        let snap = tracker.snapshots.last().unwrap();
        assert!(snap.recoveries.contains(&"test_b".to_string()));
    }

    #[test]
    fn convergence_pass_rate_only() {
        let mut tracker = MetricsTracker::default();
        let config = ConvergenceConfig {
            mode: ConvergenceMode::PassRateOnly,
            min_pass_rate: 0.9,
            ..Default::default()
        };

        tracker.record(1, make_suite(7, 3, &["a", "b", "c"]));
        assert!(tracker.check_convergence(&config).is_none());

        tracker.record(2, make_suite(9, 1, &["a"]));
        assert!(tracker.check_convergence(&config).is_some());
    }

    #[test]
    fn convergence_no_improvement() {
        let mut tracker = MetricsTracker::default();
        let config = ConvergenceConfig {
            mode: ConvergenceMode::NoImprovement,
            convergence_threshold: 0.05,
            ..Default::default()
        };

        tracker.record(1, make_suite(8, 2, &["a", "b"]));
        assert!(tracker.check_convergence(&config).is_none());

        // Same pass rate — no improvement
        tracker.record(2, make_suite(8, 2, &["a", "b"]));
        assert!(tracker.check_convergence(&config).is_some());
    }

    #[test]
    fn convergence_metric_driven() {
        let mut tracker = MetricsTracker::default();
        let mut metrics = std::collections::HashMap::new();
        metrics.insert(
            "pass_rate".to_string(),
            MetricTarget {
                target: 0.95,
                weight: 1.0,
            },
        );
        let config = ConvergenceConfig {
            mode: ConvergenceMode::MetricDriven,
            metrics,
            ..Default::default()
        };

        tracker.record(1, make_suite(8, 2, &["a", "b"]));
        assert!(tracker.check_convergence(&config).is_none());

        tracker.record(2, make_suite(19, 1, &["a"]));
        assert!(tracker.check_convergence(&config).is_some());
    }

    #[test]
    fn loop_state_persistence() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();

        let mut tracker = MetricsTracker::default();
        tracker.record(1, make_suite(8, 2, &["a", "b"]));
        tracker.record(2, make_suite(10, 0, &[]));

        let state = LoopState::from_tracker(&tracker, "fix tests");
        state.save(repo).unwrap();

        let loaded = LoopState::load(repo);
        assert_eq!(loaded.snapshots.len(), 2);
        assert_eq!(loaded.goal, "fix tests");
        assert!((loaded.snapshots[1].suite.pass_rate - 1.0).abs() < f64::EPSILON);
    }
}
