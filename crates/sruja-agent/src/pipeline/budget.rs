use serde::{Deserialize, Serialize};

/// Budgets and convergence thresholds for the pipeline.
///
/// Every limit is configurable from `.sruja/pipeline.toml` — zero hardcoded
/// stage names or specific role limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineBudgets {
    /// Maximum outer pipeline cycles (default: 3).
    #[serde(default = "default_max_cycles")]
    pub max_cycles: usize,
    /// Maximum analyzer passes per cycle (default: 1).
    #[serde(default = "default_analyzer_passes")]
    pub max_analyzer_passes: usize,
    /// Maximum prober passes per cycle (default: 2).
    #[serde(default = "default_prober_passes")]
    pub max_prober_passes: usize,
    /// Maximum fixer attempts per bug per cycle (default: 3).
    #[serde(default = "default_fixer_attempts")]
    pub max_fixer_attempts_per_bug: usize,
    /// Score threshold for convergence (default: 4.0, on 0-5 scale).
    #[serde(default = "default_score_threshold")]
    pub convergence_score_threshold: f64,
    /// Minimum score improvement to avoid plateau detection (default: 0.3).
    #[serde(default = "default_min_improvement")]
    pub min_improvement_threshold: f64,
}

fn default_max_cycles() -> usize { 3 }
fn default_analyzer_passes() -> usize { 1 }
fn default_prober_passes() -> usize { 2 }
fn default_fixer_attempts() -> usize { 3 }
fn default_score_threshold() -> f64 { 4.0 }
fn default_min_improvement() -> f64 { 0.3 }

impl Default for PipelineBudgets {
    fn default() -> Self {
        Self {
            max_cycles: default_max_cycles(),
            max_analyzer_passes: default_analyzer_passes(),
            max_prober_passes: default_prober_passes(),
            max_fixer_attempts_per_bug: default_fixer_attempts(),
            convergence_score_threshold: default_score_threshold(),
            min_improvement_threshold: default_min_improvement(),
        }
    }
}

/// Runtime budget tracker — mutable counters, not config.
#[derive(Debug, Clone)]
pub struct BudgetTracker {
    pub budgets: PipelineBudgets,
    pub cycle: usize,
    pub analyzer_passes: usize,
    pub prober_passes: usize,
    pub fixer_attempts: std::collections::HashMap<String, usize>,
    pub previous_score: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ConvergenceResult {
    pub converged: bool,
    pub reason: String,
}

impl BudgetTracker {
    pub fn new(budgets: PipelineBudgets) -> Self {
        Self {
            budgets,
            cycle: 0,
            analyzer_passes: 0,
            prober_passes: 0,
            fixer_attempts: std::collections::HashMap::new(),
            previous_score: None,
        }
    }

    pub fn can_run_analyzer(&self) -> bool {
        self.analyzer_passes < self.budgets.max_analyzer_passes
    }

    pub fn record_analyzer_pass(&mut self) {
        self.analyzer_passes += 1;
    }

    pub fn can_run_prober(&self) -> bool {
        self.prober_passes < self.budgets.max_prober_passes
    }

    pub fn record_prober_pass(&mut self) {
        self.prober_passes += 1;
    }

    pub fn can_attempt_fix(&self, bug_id: &str) -> bool {
        let attempts = self.fixer_attempts.get(bug_id).copied().unwrap_or(0);
        attempts < self.budgets.max_fixer_attempts_per_bug
    }

    pub fn record_fix_attempt(&mut self, bug_id: &str) {
        *self.fixer_attempts.entry(bug_id.to_string()).or_insert(0) += 1;
    }

    /// Check convergence and return reason.
    pub fn check_convergence(
        &mut self,
        current_score: Option<f64>,
        has_blocking_bugs: bool,
        budget_remaining: bool,
    ) -> ConvergenceResult {
        let Some(score) = current_score else {
            return ConvergenceResult {
                converged: false,
                reason: "no scorecard produced yet".into(),
            };
        };

        if !has_blocking_bugs && score >= self.budgets.convergence_score_threshold {
            return ConvergenceResult {
                converged: true,
                reason: format!(
                    "goals met — score {score:.1} >= {}, zero blocking bugs",
                    self.budgets.convergence_score_threshold,
                ),
            };
        }

        if !budget_remaining {
            return ConvergenceResult {
                converged: false,
                reason: format!(
                    "budget exhausted — score {score:.1}, max cycles reached",
                ),
            };
        }

        // Plateau detection
        if let Some(prev) = self.previous_score {
            if score - prev < self.budgets.min_improvement_threshold {
                return ConvergenceResult {
                    converged: true,
                    reason: format!(
                        "plateaued — score {score:.1} vs previous {prev:.1} (improvement < {})",
                        self.budgets.min_improvement_threshold,
                    ),
                };
            }
        }

        ConvergenceResult {
            converged: false,
            reason: format!("score {score:.1} below {threshold:.1}",
                threshold = self.budgets.convergence_score_threshold,
            ),
        }
    }

    pub fn start_new_cycle(&mut self) {
        self.cycle += 1;
        self.analyzer_passes = 0;
        self.prober_passes = 0;
        self.fixer_attempts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_budget() {
        let budgets = PipelineBudgets {
            max_analyzer_passes: 2,
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(budgets);
        assert!(tracker.can_run_analyzer());
        tracker.record_analyzer_pass();
        assert!(tracker.can_run_analyzer());
        tracker.record_analyzer_pass();
        assert!(!tracker.can_run_analyzer());
    }

    #[test]
    fn test_fixer_budget_per_bug() {
        let budgets = PipelineBudgets {
            max_fixer_attempts_per_bug: 2,
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(budgets);
        assert!(tracker.can_attempt_fix("bug-1"));
        tracker.record_fix_attempt("bug-1");
        assert!(tracker.can_attempt_fix("bug-1"));
        tracker.record_fix_attempt("bug-1");
        assert!(!tracker.can_attempt_fix("bug-1"));
        // Different bug unaffected
        assert!(tracker.can_attempt_fix("bug-2"));
    }

    #[test]
    fn test_convergence_goals_met() {
        let mut tracker = BudgetTracker::new(PipelineBudgets::default());
        let r = tracker.check_convergence(Some(4.5), false, true);
        assert!(r.converged);
        assert!(r.reason.contains("goals met"));
    }

    #[test]
    fn test_convergence_blocking_bugs() {
        let mut tracker = BudgetTracker::new(PipelineBudgets::default());
        let r = tracker.check_convergence(Some(4.5), true, true);
        assert!(!r.converged);
    }

    #[test]
    fn test_convergence_budget_exhausted() {
        let mut tracker = BudgetTracker::new(PipelineBudgets::default());
        let r = tracker.check_convergence(Some(2.0), false, false);
        assert!(!r.converged);
        assert!(r.reason.contains("budget exhausted"));
    }

    #[test]
    fn test_convergence_plateau() {
        // Need convergence_threshold > current score so plateau check fires
        let budgets = PipelineBudgets {
            convergence_score_threshold: 5.0,
            min_improvement_threshold: 0.5,
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(budgets);
        tracker.previous_score = Some(4.0);
        let r = tracker.check_convergence(Some(4.2), false, true);
        assert!(r.converged);
        assert!(r.reason.contains("plateaued"));
    }

    #[test]
    fn test_convergence_improving() {
        let budgets = PipelineBudgets {
            min_improvement_threshold: 0.5,
            convergence_score_threshold: 5.0,
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(budgets);
        tracker.previous_score = Some(3.0);
        let r = tracker.check_convergence(Some(4.0), false, true);
        assert!(!r.converged);
        assert!(r.reason.contains("below"));
    }

    #[test]
    fn test_convergence_no_score() {
        let mut tracker = BudgetTracker::new(PipelineBudgets::default());
        let r = tracker.check_convergence(None, false, true);
        assert!(!r.converged);
    }
}
