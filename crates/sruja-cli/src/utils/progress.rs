use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// A simple progress tracker that allows incrementing a counter and reading its value.
pub struct ProgressTracker {
    current: AtomicUsize,
    total: usize,
}

impl ProgressTracker {
    /// Creates a new tracker with `total` as the expected maximum.
    pub fn new(total: usize) -> Self {
        Self {
            current: AtomicUsize::new(0),
            total,
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
}
