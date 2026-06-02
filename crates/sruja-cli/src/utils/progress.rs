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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_creates_progress_bar_with_message() {
        let pb = spinner("working");
        pb.finish_and_clear();
    }
}
