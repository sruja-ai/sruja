use std::path::Path;

use super::types::{
    FixReport, FixStatus, PipelineResult, Scorecard,
    StageResult,
};

/// Live dashboard writer that produces a `LIVE.md` file after every pipeline
/// stage, giving the user real-time visibility into pipeline progress.
#[derive(Debug, Clone)]
pub struct LiveReport {
    report_dir: std::path::PathBuf,
    current_text: String,
}

impl LiveReport {
    pub fn new(repo: &Path) -> Self {
        Self {
            report_dir: repo.join(".sruja").join("pipeline"),
            current_text: String::new(),
        }
    }

    /// Initialize the live report with a header.
    pub fn init(&mut self, cycle: usize, max_cycles: usize, focus: Option<&str>) {
        let lines = [
            "# Pipeline — Live Dashboard".to_string(),
            String::new(),
            format!(
                "Cycle {cycle}/{max_cycles} · Focus: {}",
                focus.unwrap_or("(full project)")
            ),
            String::new(),
            "| Stage | Status | Result |".to_string(),
            "|-------|--------|--------|".to_string(),
        ];
        self.current_text = lines.join("\n");
        let _ = self.write();
    }

    /// Update the live report after a stage completes.
    pub fn update(&mut self, stage: &StageResult, artifact_name: &str, duration_ms: u64) {
        let status_icon = if stage.success { "✅" } else { "❌" };
        let line = format!(
            "| {} | {status_icon} done ({duration_ms}ms) | {artifact_name} |",
            stage.stage_id
        );
        self.current_text.push_str(&format!("\n{line}"));
        let _ = self.write();
    }

    /// Add a section to the report (e.g., gaps list, bugs list).
    pub fn add_section(&mut self, title: &str, content: &str) {
        self.current_text
            .push_str(&format!("\n\n## {title}\n{content}"));
        let _ = self.write();
    }

    /// Append the scorecard to the report.
    pub fn add_scorecard(&mut self, scorecard: &Scorecard) {
        let section = format!(
            "\n\n## Scorecard\n\
             | Dimension | Score |\n\
             |-----------|-------|\n\
             | Functional correctness | {}/5 |\n\
             | Code quality | {}/5 |\n\
             | Test coverage | {}/5 |\n\
             | UX quality | {}/5 |\n\
             | Cost efficiency | {}/5 |\n\
             | **Overall** | **{:.1}/5** |\n\
             \nSummary: {}\n",
            scorecard.functional_correctness,
            scorecard.code_quality,
            scorecard.test_coverage,
            scorecard.ux_quality,
            scorecard.cost_efficiency,
            scorecard.total,
            scorecard.summary,
        );
        self.current_text.push_str(&section);
        let _ = self.write();
    }

    /// Append the final result summary.
    pub fn add_result(&mut self, result: &PipelineResult) {
        let section = format!(
            "\n\n## Result\n\
             - Cycles: {}\n\
             - Converged: {}\n\
             - Reason: {}\n\
             - Lessons recorded: {}\n\
             - Stages executed: {}\n",
            result.cycles,
            result.converged,
            result.reason,
            result.lessons_recorded,
            result.stages.len(),
        );
        self.current_text.push_str(&section);
        let _ = self.write();
    }

    /// Write the current report to disk.
    pub fn write(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.report_dir)?;
        let path = self.report_dir.join("LIVE.md");
        std::fs::write(&path, &self.current_text)
    }

    /// Gap report summary.
    pub fn gap_summary(gaps: &[crate::pipeline::types::Gap]) -> String {
        if gaps.is_empty() {
            return "No gaps found.".to_string();
        }
        let mut lines = vec!["| # | Gap | Severity |".to_string(), "|---|-----|----------|".to_string()];
        for (i, g) in gaps.iter().enumerate() {
            lines.push(format!("| {} | {} | {} |", i + 1, g.description, g.severity));
        }
        lines.join("\n")
    }

    /// Bug report summary.
    pub fn bug_summary(bugs: &[crate::pipeline::types::Bug]) -> String {
        if bugs.is_empty() {
            return "No bugs found.".to_string();
        }
        let mut lines = vec!["| # | Bug | Severity |".to_string(), "|---|-----|----------|".to_string()];
        for (i, b) in bugs.iter().enumerate() {
            lines.push(format!("| {} | {} | {} |", i + 1, b.description, b.severity));
        }
        lines.join("\n")
    }

    /// Fix report summary.
    pub fn fix_summary(fixes: &[FixReport]) -> String {
        if fixes.is_empty() {
            return "No fixes applied.".to_string();
        }
        let mut lines = vec!["| Bug | Status | Files |".to_string(), "|-----|--------|-------|".to_string()];
        for f in fixes {
            let status = match f.status {
                FixStatus::Resolved => "✅ resolved",
                FixStatus::Failed => "❌ failed",
                FixStatus::Blocked => "🔒 blocked",
            };
            lines.push(format!(
                "| {} | {status} | {} |",
                f.bug_id,
                f.modified_files.join(", ")
            ));
        }
        lines.join("\n")
    }
}
