use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// A simple progress tracker that allows incrementing a counter and reading its value.
#[allow(dead_code)]
pub struct ProgressTracker {
    current: AtomicUsize,
    total: usize,
    start_time: Instant,
}

#[allow(dead_code)]
impl ProgressTracker {
    /// Creates a new tracker with `total` as the expected maximum.
    pub fn new(total: usize) -> Self {
        Self {
            current: AtomicUsize::new(0),
            total,
            start_time: Instant::now(),
        }
    }

    /// Increments the counter by 1 and returns the new value.
    pub fn increment(&self) -> usize {
        self.current.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Returns the current value.
    pub fn current(&self) -> usize {
        self.current.load(Ordering::Relaxed)
    }

    /// Returns the total (maximum expected value).
    pub fn total(&self) -> usize {
        self.total
    }

    /// Returns the progress as a percentage (0.0 to 100.0).
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        let current = self.current() as f64;
        let total = self.total as f64;
        (current / total * 100.0).min(100.0)
    }

    /// Returns the elapsed time since the tracker was created.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Estimates the remaining time based on current progress.
    /// Returns None if no progress has been made yet.
    pub fn eta(&self) -> Option<Duration> {
        let current = self.current();
        if current == 0 {
            return None;
        }
        let elapsed = self.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        let rate = current as f64 / elapsed_secs;
        let remaining = self.total - current;
        let remaining_secs = remaining as f64 / rate;
        Some(Duration::from_secs_f64(remaining_secs))
    }

    /// Returns a human-readable ETA string.
    /// Returns "N/A" if no progress has been made yet.
    pub fn eta_string(&self) -> String {
        match self.eta() {
            Some(duration) => {
                let secs = duration.as_secs();
                if secs < 60 {
                    format!("{}s", secs)
                } else if secs < 3600 {
                    format!("{}m {}s", secs / 60, secs % 60)
                } else {
                    format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
                }
            }
            None => "N/A".to_string(),
        }
    }

    /// Returns a human-readable progress string.
    /// Example: "50% (5/10) ETA: 30s"
    pub fn progress_string(&self) -> String {
        format!(
            "{:.0}% ({}/{}) ETA: {}",
            self.percentage(),
            self.current(),
            self.total,
            self.eta_string()
        )
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_creates_progress_bar_with_message() {
        let pb = spinner("working");
        pb.finish_and_clear();
    }

    #[test]
    fn test_progress_tracker_percentage() {
        let tracker = ProgressTracker::new(10);
        assert_eq!(tracker.percentage(), 0.0);
        tracker.increment();
        assert_eq!(tracker.percentage(), 10.0);
        tracker.increment();
        assert_eq!(tracker.percentage(), 20.0);
    }

    #[test]
    fn test_progress_tracker_percentage_with_zero_total() {
        let tracker = ProgressTracker::new(0);
        assert_eq!(tracker.percentage(), 100.0);
    }

    #[test]
    fn test_progress_tracker_eta_with_no_progress() {
        let tracker = ProgressTracker::new(10);
        assert!(tracker.eta().is_none());
        assert_eq!(tracker.eta_string(), "N/A");
    }

    #[test]
    fn test_progress_tracker_progress_string() {
        let tracker = ProgressTracker::new(10);
        tracker.increment();
        let progress = tracker.progress_string();
        assert!(progress.contains("10%"));
        assert!(progress.contains("1/10"));
        assert!(progress.contains("ETA:"));
    }
}
