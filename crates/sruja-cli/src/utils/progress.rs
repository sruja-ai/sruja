use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    let tick_strings = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&tick_strings)
            .template("{spinner:.blue} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner().tick_strings(&tick_strings)),
    );
    pb.set_message(message.to_string());
    pb
}

/// Multi-step progress indicator for sequential operations.
///
/// Displays as: `⠋ [2/4] Scanning manifests... (12s)`
#[allow(dead_code)]
pub struct MultiStepProgress {
    steps: Vec<String>,
    current: usize,
    pb: ProgressBar,
    start: std::time::Instant,
}

#[allow(dead_code)]
impl MultiStepProgress {
    /// Create a new multi-step progress with the given step labels.
    pub fn new(steps: Vec<&str>) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        let tick_strings = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&tick_strings)
                .template("{spinner:.blue} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_spinner().tick_strings(&tick_strings)),
        );

        let total = steps.len();
        let step_labels: Vec<String> = steps.iter().map(|s| s.to_string()).collect();

        let progress = Self {
            steps: step_labels,
            current: 0,
            pb,
            start: std::time::Instant::now(),
        };

        if total > 0 {
            progress.update_display();
        }

        progress
    }

    /// Advance to the next step with an optional custom message override.
    pub fn advance(&mut self, message: Option<&str>) {
        self.current += 1;
        if let Some(msg) = message {
            // Temporarily override the step label
            if self.current <= self.steps.len() {
                self.steps[self.current - 1] = msg.to_string();
            }
        }
        self.update_display();
    }

    /// Finish with a success message.
    pub fn finish_success(self, message: &str) {
        let elapsed = self.start.elapsed();
        let elapsed_str = format_elapsed(elapsed);
        self.pb
            .finish_with_message(format!("{} ✓ ({})", message, elapsed_str));
    }

    /// Finish with a failure message.
    pub fn finish_failure(self, message: &str) {
        let elapsed = self.start.elapsed();
        let elapsed_str = format_elapsed(elapsed);
        self.pb
            .abandon_with_message(format!("{} ✗ ({})", message, elapsed_str));
    }

    /// Get the current step index (0-based).
    pub fn current_step(&self) -> usize {
        self.current
    }

    fn update_display(&self) {
        let total = self.steps.len();
        let current = self.current.min(total);
        let elapsed = self.start.elapsed();
        let elapsed_str = format_elapsed(elapsed);

        let step_label = if current < total {
            &self.steps[current]
        } else {
            "done"
        };

        self.pb.set_message(format!(
            "[{}/{}] {} ({})",
            current + 1,
            total,
            step_label,
            elapsed_str
        ));
    }
}

#[allow(dead_code)]
fn format_elapsed(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_elapsed_seconds_and_minutes() {
        assert_eq!(format_elapsed(Duration::from_secs(5)), "5s");
        assert_eq!(format_elapsed(Duration::from_secs(125)), "2m 5s");
    }

    #[test]
    fn multi_step_progress_advances_and_finishes() {
        let mut progress = MultiStepProgress::new(vec!["scan", "merge"]);
        assert_eq!(progress.current_step(), 0);
        progress.advance(None);
        assert_eq!(progress.current_step(), 1);
        progress.finish_success("done");
    }

    #[test]
    fn spinner_creates_progress_bar_with_message() {
        let pb = spinner("working");
        pb.finish_and_clear();
    }
}
