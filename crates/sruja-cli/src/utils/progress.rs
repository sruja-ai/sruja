use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Returns a default progress spinner for short operations.
pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.blue} {msg}")
            .expect("valid template"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Returns a progress bar for long operations with a counter.
pub fn progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} {bar:40.cyan/blue} {pos}/{len} ({eta})")
            .expect("valid template")
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    pb.set_message(message.to_string());
    pb
}

/// Finishes a progress bar with a success message.
pub fn finish_success(pb: &ProgressBar, message: &str) {
    pb.finish_with_message(format!("{} {}", console::style("✓").green(), message));
}

/// Finishes a progress bar with an error message.
pub fn finish_error(pb: &ProgressBar, message: &str) {
    pb.finish_with_message(format!("{} {}", console::style("✗").red(), message));
}
