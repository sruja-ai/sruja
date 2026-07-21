//! Self-diagnosis capabilities for the agent.
//!
//! When the agent encounters failures, this module analyzes the failure patterns
//! and suggests recovery strategies. It learns from past failures to improve
//! future performance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{ErrorClass, StepResult, StepStatus};

/// Diagnosis of a failure, including root cause and suggested recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureDiagnosis {
    /// The error class detected.
    pub error_class: ErrorClass,
    /// Human-readable description of the root cause.
    pub root_cause: String,
    /// Suggested recovery strategy.
    pub recovery_strategy: RecoveryStrategy,
    /// Confidence in the diagnosis (0.0 - 1.0).
    pub confidence: f64,
    /// Files or modules involved in the failure.
    pub affected_areas: Vec<String>,
    /// Specific suggestions for fixing the issue.
    pub suggestions: Vec<String>,
}

/// Recovery strategies the agent can employ.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RecoveryStrategy {
    /// Retry the same approach (transient error).
    Retry,
    /// Try a completely different approach.
    DifferentApproach,
    /// Focus on a specific file or area.
    TargetedFix { file: String },
    /// Run diagnostics first (cargo check, tests, etc.).
    DiagnoseFirst,
    /// Simplify the task (break into smaller pieces).
    Simplify,
    /// Ask for human help.
    Escalate,
    /// Skip this step and continue.
    Skip,
}

/// Pattern of failures seen across iterations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    /// The error class that keeps recurring.
    pub error_class: ErrorClass,
    /// How many times this pattern has occurred.
    pub count: usize,
    /// The iterations where this occurred.
    pub iterations: Vec<usize>,
    /// Whether the pattern is getting worse (more frequent).
    pub worsening: bool,
}

/// Self-diagnosis engine that analyzes failures and suggests recovery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelfDiagnosis {
    /// History of failure patterns seen.
    pub patterns: Vec<FailurePattern>,
    /// Past diagnoses and their outcomes.
    pub history: Vec<DiagnosisOutcome>,
    /// Statistics on error classes.
    pub error_stats: HashMap<ErrorClass, usize>,
}

/// Outcome of a diagnosis (did the recovery work?).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisOutcome {
    /// The original diagnosis.
    pub diagnosis: FailureDiagnosis,
    /// Whether the recovery was successful.
    pub successful: bool,
    /// The iteration where this was tried.
    pub iteration: usize,
}

impl SelfDiagnosis {
    /// Create a new self-diagnosis engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze step results and produce a diagnosis.
    pub fn diagnose(&mut self, results: &[StepResult], iteration: usize) -> Option<FailureDiagnosis> {
        // Find failed steps
        let failed_steps: Vec<&StepResult> = results
            .iter()
            .filter(|r| r.status == StepStatus::Failed)
            .collect();

        if failed_steps.is_empty() {
            return None;
        }

        // Classify the error
        let error_class = self.classify_failures(&failed_steps);
        
        // Update statistics
        *self.error_stats.entry(error_class).or_insert(0) += 1;

        // Update failure patterns
        self.update_patterns(error_class, iteration);

        // Generate diagnosis based on error class and history
        let diagnosis = self.generate_diagnosis(error_class, &failed_steps, iteration);

        Some(diagnosis)
    }

    /// Classify failures from step results.
    fn classify_failures(&self, failed_steps: &[&StepResult]) -> ErrorClass {
        // Check tool signals for specific error patterns
        for step in failed_steps {
            for signal in &step.tool_signals {
                if !signal.ok {
                    // Check the output for error patterns
                    let output_lower = step.output.to_lowercase();
                    
                    if output_lower.contains("error[e0") || output_lower.contains("error:") {
                        if output_lower.contains("mismatched types") || output_lower.contains("trait bound") {
                            return ErrorClass::Type;
                        }
                        return ErrorClass::Compilation;
                    }
                    
                    if output_lower.contains("test ... failed") || output_lower.contains("assertion failed") {
                        return ErrorClass::Test;
                    }
                    
                    if output_lower.contains("panicked") || output_lower.contains("unwrap on none") {
                        return ErrorClass::Runtime;
                    }
                    
                    if output_lower.contains("warning") {
                        return ErrorClass::Lint;
                    }
                }
            }
        }

        // Check for convergence issues
        let non_converged = failed_steps.iter().filter(|s| !s.converged).count();
        if non_converged > failed_steps.len() / 2 {
            return ErrorClass::Other; // Convergence issue
        }

        ErrorClass::Other
    }

    /// Update failure patterns based on new error.
    fn update_patterns(&mut self, error_class: ErrorClass, iteration: usize) {
        // Find existing pattern or create new one
        let pattern = self.patterns.iter_mut().find(|p| p.error_class == error_class);
        
        if let Some(pattern) = pattern {
            pattern.count += 1;
            pattern.iterations.push(iteration);
            // Check if worsening (occurred in last 2 iterations)
            pattern.worsening = pattern.iterations.len() >= 2
                && iteration.saturating_sub(*pattern.iterations.last().unwrap_or(&0)) <= 2;
        } else {
            self.patterns.push(FailurePattern {
                error_class,
                count: 1,
                iterations: vec![iteration],
                worsening: false,
            });
        }
    }

    /// Generate a diagnosis based on error class and history.
    fn generate_diagnosis(
        &self,
        error_class: ErrorClass,
        failed_steps: &[&StepResult],
        _iteration: usize,
    ) -> FailureDiagnosis {
        let pattern = self.patterns.iter().find(|p| p.error_class == error_class);
        let is_recurring = pattern.map_or(false, |p| p.count > 1);
        let is_worsening = pattern.map_or(false, |p| p.worsening);

        // Base diagnosis on error class
        let (root_cause, recovery_strategy, suggestions) = match error_class {
            ErrorClass::Compilation => {
                let cause = if is_recurring {
                    "Recurring compilation errors - likely a systematic issue"
                } else {
                    "Compilation error in the code"
                };
                let strategy = if is_worsening {
                    RecoveryStrategy::DiagnoseFirst
                } else {
                    RecoveryStrategy::Retry
                };
                let suggestions = vec![
                    "Run cargo check to see all errors".to_string(),
                    "Check imports and type annotations".to_string(),
                    "Verify the file was saved correctly".to_string(),
                ];
                (cause, strategy, suggestions)
            }
            ErrorClass::Type => {
                let cause = "Type mismatch or trait bound issue";
                let strategy = RecoveryStrategy::TargetedFix {
                    file: self.extract_primary_file(failed_steps),
                };
                let suggestions = vec![
                    "Check function signatures match".to_string(),
                    "Verify trait implementations".to_string(),
                    "Look for lifetime annotation issues".to_string(),
                ];
                (cause, strategy, suggestions)
            }
            ErrorClass::Test => {
                let cause = if is_recurring {
                    "Test failures indicate logic issues"
                } else {
                    "Test assertion failed"
                };
                let strategy = RecoveryStrategy::DifferentApproach;
                let suggestions = vec![
                    "Review test expectations vs implementation".to_string(),
                    "Check edge cases in test data".to_string(),
                    "Verify acceptance criteria are correct".to_string(),
                ];
                (cause, strategy, suggestions)
            }
            ErrorClass::Runtime => {
                let cause = "Runtime panic or error";
                let strategy = RecoveryStrategy::TargetedFix {
                    file: self.extract_primary_file(failed_steps),
                };
                let suggestions = vec![
                    "Check for unwrap() on None values".to_string(),
                    "Verify array bounds".to_string(),
                    "Add proper error handling".to_string(),
                ];
                (cause, strategy, suggestions)
            }
            ErrorClass::Lint => {
                let cause = "Lint warnings or style issues";
                let strategy = RecoveryStrategy::Retry;
                let suggestions = vec![
                    "Run cargo clippy --fix".to_string(),
                    "Check formatting with cargo fmt".to_string(),
                ];
                (cause, strategy, suggestions)
            }
            ErrorClass::Architecture => {
                let cause = "Architectural boundary violation";
                let strategy = RecoveryStrategy::Escalate;
                let suggestions = vec![
                    "Review architecture constraints".to_string(),
                    "Check module dependencies".to_string(),
                ];
                (cause, strategy, suggestions)
            }
            ErrorClass::SpecGap => {
                let cause = "Acceptance criteria not addressed";
                let strategy = RecoveryStrategy::Simplify;
                let suggestions = vec![
                    "Review all acceptance criteria".to_string(),
                    "Break task into smaller pieces".to_string(),
                ];
                (cause, strategy, suggestions)
            }
            ErrorClass::Other => {
                let cause = if is_recurring {
                    "Recurring unknown failures - may need different approach"
                } else {
                    "Unknown failure occurred"
                };
                let strategy = if is_worsening {
                    RecoveryStrategy::Simplify
                } else {
                    RecoveryStrategy::Retry
                };
                let suggestions = vec![
                    "Try a simpler approach".to_string(),
                    "Check recent changes".to_string(),
                ];
                (cause, strategy, suggestions)
            }
        };

        // Calculate confidence based on history
        let confidence = if is_recurring {
            0.9 // High confidence for recurring patterns
        } else if is_worsening {
            0.8 // High confidence for worsening patterns
        } else {
            0.6 // Medium confidence for new patterns
        };

        // Extract affected areas
        let affected_areas = self.extract_affected_areas(failed_steps);

        FailureDiagnosis {
            error_class,
            root_cause: root_cause.to_string(),
            recovery_strategy,
            confidence,
            affected_areas,
            suggestions,
        }
    }

    /// Extract the primary file involved in failures.
    fn extract_primary_file(&self, failed_steps: &[&StepResult]) -> String {
        // Look for file paths in the output
        for step in failed_steps {
            for line in step.output.lines() {
                if line.contains("src/") && (line.contains(".rs") || line.contains(".ts")) {
                    // Extract file path
                    if let Some(start) = line.find("src/") {
                        if let Some(end) = line[start..].find(|c: char| c.is_whitespace() || c == ':') {
                            return line[start..start + end].to_string();
                        }
                    }
                }
            }
        }
        "unknown".to_string()
    }

    /// Extract affected areas from failed steps.
    fn extract_affected_areas(&self, failed_steps: &[&StepResult]) -> Vec<String> {
        let mut areas = Vec::new();
        
        for step in failed_steps {
            // Check tool signals for affected files
            for signal in &step.tool_signals {
                if signal.tool.contains("file") || signal.tool.contains("edit") {
                    // Try to extract file path from output
                    if let Some(file) = self.extract_file_from_output(&step.output) {
                        if !areas.contains(&file) {
                            areas.push(file);
                        }
                    }
                }
            }
        }

        areas
    }

    /// Extract file path from output text.
    fn extract_file_from_output(&self, output: &str) -> Option<String> {
        for line in output.lines() {
            if line.contains("src/") && line.contains(".rs") {
                if let Some(start) = line.find("src/") {
                    if let Some(end) = line[start..].find(|c: char| c.is_whitespace() || c == ':') {
                        return Some(line[start..start + end].to_string());
                    }
                }
            }
        }
        None
    }

    /// Record the outcome of a diagnosis (for learning).
    pub fn record_outcome(&mut self, diagnosis: FailureDiagnosis, successful: bool, iteration: usize) {
        self.history.push(DiagnosisOutcome {
            diagnosis,
            successful,
            iteration,
        });
    }

    /// Get statistics on past diagnoses.
    pub fn get_stats(&self) -> DiagnosisStats {
        let total = self.history.len();
        let successful = self.history.iter().filter(|o| o.successful).count();
        let failed = total - successful;

        let by_strategy: HashMap<RecoveryStrategy, (usize, usize)> = self.history.iter().fold(
            HashMap::new(),
            |mut acc, outcome| {
                let entry = acc
                    .entry(outcome.diagnosis.recovery_strategy.clone())
                    .or_insert((0, 0));
                if outcome.successful {
                    entry.0 += 1;
                } else {
                    entry.1 += 1;
                }
                acc
            },
        );

        DiagnosisStats {
            total_diagnoses: total,
            successful_recoveries: successful,
            failed_recoveries: failed,
            success_rate: if total > 0 {
                successful as f64 / total as f64
            } else {
                0.0
            },
            by_strategy,
        }
    }

    /// Format failure context for LLM-powered deep analysis.
    ///
    /// This produces a structured prompt that an LLM can use to provide
    /// deeper insights into complex failures that heuristic diagnosis
    /// cannot adequately explain.
    pub fn format_for_llm_analysis(
        &mut self,
        failed_steps: &[&StepResult],
        goal: &str,
        plan_description: &str,
    ) -> String {
        let mut out = String::from("## Failure Analysis Request\n\n");
        
        out.push_str(&format!("**Goal:** {}\n\n", goal));
        out.push_str(&format!("**Plan:** {}\n\n", plan_description));
        
        // Include failed step outputs
        out.push_str("## Failed Steps\n\n");
        for (i, step) in failed_steps.iter().enumerate() {
            out.push_str(&format!("### Step {}: {}\n", i + 1, step.subtask_id));
            out.push_str(&format!("**Status:** {:?}\n", step.status));
            out.push_str(&format!("**Converged:** {}\n", step.converged));
            out.push_str("**Output:**\n```\n");
            // Truncate output to avoid overwhelming the LLM
            let truncated: String = step.output.chars().take(2000).collect();
            out.push_str(&truncated);
            if step.output.len() > 2000 {
                out.push_str("\n... (truncated)");
            }
            out.push_str("\n```\n\n");
            
            // Include tool signals
            if !step.tool_signals.is_empty() {
                out.push_str("**Tool calls:**\n");
                for signal in &step.tool_signals {
                    out.push_str(&format!(
                        "- {} (ok: {}, elapsed: {}ms)\n",
                        signal.tool, signal.ok, signal.elapsed_ms
                    ));
                }
                out.push_str("\n");
            }
        }
        
        // Include heuristic diagnosis if available
        let step_results: Vec<StepResult> = failed_steps.iter().map(|s| (*s).clone()).collect();
        if let Some(diagnosis) = self.diagnose(&step_results, 0) {
            out.push_str("## Heuristic Diagnosis\n\n");
            out.push_str(&format!("**Error class:** {:?}\n", diagnosis.error_class));
            out.push_str(&format!("**Root cause:** {}\n", diagnosis.root_cause));
            out.push_str(&format!("**Confidence:** {:.0}%\n", diagnosis.confidence * 100.0));
            out.push_str(&format!("**Suggested recovery:** {:?}\n\n", diagnosis.recovery_strategy));
        }
        
        // Include pattern history
        if !self.patterns.is_empty() {
            out.push_str("## Historical Patterns\n\n");
            for pattern in &self.patterns {
                out.push_str(&format!(
                    "- {:?}: {} occurrences (iterations: {:?}, trending {})\n",
                    pattern.error_class,
                    pattern.count,
                    pattern.iterations,
                    if pattern.worsening { "up" } else { "stable" }
                ));
            }
            out.push_str("\n");
        }
        
        out.push_str("## Analysis Request\n\n");
        out.push_str("Please analyze this failure and provide:\n");
        out.push_str("1. Root cause analysis (be specific about what went wrong)\n");
        out.push_str("2. Why the heuristic diagnosis may be wrong or incomplete\n");
        out.push_str("3. A specific, actionable recovery strategy\n");
        out.push_str("4. How to prevent this type of failure in the future\n");
        
        out
    }
}

/// Statistics on diagnosis outcomes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisStats {
    pub total_diagnoses: usize,
    pub successful_recoveries: usize,
    pub failed_recoveries: usize,
    pub success_rate: f64,
    pub by_strategy: HashMap<RecoveryStrategy, (usize, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::ToolSignal;
    use crate::tool::ToolSource;

    fn create_failed_step(output: &str) -> StepResult {
        StepResult {
            subtask_id: "test".into(),
            status: StepStatus::Failed,
            output: output.to_string(),
            usage: Default::default(),
            tool_signals: vec![ToolSignal {
                tool: "shell".into(),
                ok: false,
                empty: false,
                elapsed_ms: 100,
                source: ToolSource::Builtin,
            }],
            converged: true,
        }
    }

    #[test]
    fn test_compilation_error_diagnosis() {
        let mut diagnosis = SelfDiagnosis::new();
        let results = vec![create_failed_step(
            "error[E0308]: mismatched types\nexpected `String`, found `i32`",
        )];

        let result = diagnosis.diagnose(&results, 1);
        assert!(result.is_some());

        let diag = result.unwrap();
        // mismatched types is classified as Type error, not generic Compilation
        assert!(
            diag.error_class == ErrorClass::Compilation || diag.error_class == ErrorClass::Type,
            "Expected Compilation or Type, got {:?}",
            diag.error_class
        );
        assert!(diag.suggestions.len() > 0);
    }

    #[test]
    fn test_recurring_pattern_detection() {
        let mut diagnosis = SelfDiagnosis::new();
        let results = vec![create_failed_step("error[E0308]: mismatched types")];

        // First occurrence
        diagnosis.diagnose(&results, 1);
        
        // Second occurrence - should detect pattern
        let result = diagnosis.diagnose(&results, 2);
        assert!(result.is_some());

        let diag = result.unwrap();
        assert!(diag.confidence > 0.7); // Higher confidence for recurring
    }

    #[test]
    fn test_stats_tracking() {
        let mut diagnosis = SelfDiagnosis::new();
        let diag = FailureDiagnosis {
            error_class: ErrorClass::Compilation,
            root_cause: "test".into(),
            recovery_strategy: RecoveryStrategy::Retry,
            confidence: 0.8,
            affected_areas: vec![],
            suggestions: vec![],
        };

        diagnosis.record_outcome(diag.clone(), true, 1);
        diagnosis.record_outcome(diag, false, 2);

        let stats = diagnosis.get_stats();
        assert_eq!(stats.total_diagnoses, 2);
        assert_eq!(stats.successful_recoveries, 1);
        assert_eq!(stats.failed_recoveries, 1);
    }
}
