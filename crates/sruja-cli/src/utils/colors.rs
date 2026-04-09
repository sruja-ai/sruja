use console::{style as console_style, StyledObject, colors_enabled};
use std::time::Duration;

pub fn style<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    console_style(text)
}

/// Returns true if colors are enabled (respects NO_COLOR, terminal detection, etc.)
pub fn is_color_enabled() -> bool {
    colors_enabled() && std::env::var("NO_COLOR").is_err() && std::env::var("SRUJA_NO_COLOR").is_err()
}

/// Returns a red styled object for error messages.
pub fn error<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    if is_color_enabled() {
        console_style(text).red().bold()
    } else {
        console_style(text)
    }
}

/// Returns a yellow styled object for warning messages.
pub fn warning<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    if is_color_enabled() {
        console_style(text).yellow()
    } else {
        console_style(text)
    }
}

/// Returns a green styled object for success messages.
pub fn success<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    if is_color_enabled() {
        console_style(text).green().bold()
    } else {
        console_style(text)
    }
}

/// Returns a blue styled object for info messages.
pub fn info<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    if is_color_enabled() {
        console_style(text).cyan()
    } else {
        console_style(text)
    }
}

/// Returns a dimmed styled object for secondary information.
pub fn dim<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    if is_color_enabled() {
        console_style(text).dim()
    } else {
        console_style(text)
    }
}

/// Prints a themed header.
pub fn print_header(title: &str) {
    if is_color_enabled() {
        println!("{}", console_style(title).bold().underlined());
    } else {
        println!("{}", title);
        println!("{}", "=".repeat(title.len()));
    }
}

/// Returns a severity icon.
pub fn severity_icon(severity: &str) -> String {
    match severity {
        "error" | "Error" => error("✗").to_string(),
        "warning" | "Warning" => warning("⚠").to_string(),
        "info" | "Info" | "notice" => info("ℹ").to_string(),
        _ => "•".to_string(),
    }
}

/// Renders a health bar: [████░░░░░░] 42/100
pub fn health_bar(score: u8, width: usize) -> String {
    let filled_len = (score as f32 / 100.0 * width as f32).round() as usize;
    let empty_len = width.saturating_sub(filled_len);
    
    let bar_color = if score >= 90 {
        success("█")
    } else if score >= 70 {
        info("█")
    } else if score >= 40 {
        warning("█")
    } else {
        error("█")
    };

    let filled = bar_color.to_string().repeat(filled_len);
    let empty = dim("░").to_string().repeat(empty_len);
    
    let score_styled = if score >= 90 {
        success(score)
    } else if score >= 70 {
        info(score)
    } else if score >= 40 {
        warning(score)
    } else {
        error(score)
    };

    format!("[{}{}] {}/100", filled, empty, score_styled)
}

/// Formats a duration or timestamp for human display ("2h 5m ago", "12s", etc.)
pub fn elapsed_display(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}
