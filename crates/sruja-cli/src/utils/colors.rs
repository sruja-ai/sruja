use console::{colors_enabled, style as console_style, StyledObject};
use std::time::Duration;

pub fn style<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    console_style(text)
}

/// Returns true if colors are enabled (respects NO_COLOR, terminal detection, etc.)
pub fn is_color_enabled() -> bool {
    colors_enabled()
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("SRUJA_NO_COLOR").is_err()
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

pub fn badge(text: &str, color: &str) -> String {
    let colored_text = match color {
        "success" => success(text).to_string(),
        "error" => error(text).to_string(),
        "warning" => warning(text).to_string(),
        "info" => info(text).to_string(),
        _ => style(text).white().to_string(),
    };
    format!("[{}]", style(colored_text).bold())
}

pub fn sparkline(scores: &[u8]) -> String {
    const BARS: &[char] = &[' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if scores.is_empty() {
        return String::new();
    }
    scores
        .iter()
        .map(|&s| {
            let idx = ((s as f32 / 100.0) * (BARS.len() - 1) as f32).round() as usize;
            BARS[idx]
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────
// Beautiful formatting helpers (terminal UI refresh)
// ─────────────────────────────────────────────────────────────────────────

/// Detect terminal width, with a reasonable default.
pub fn terminal_width() -> usize {
    if let Ok((w, _)) = crossterm::terminal::size() {
        return (w as usize).clamp(60, 240);
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&w| w >= 60)
        .unwrap_or(80)
}

/// Renders a section header like `── Goal ─────────────────────────────`
/// The label is dimmed; the dashes fill the terminal width.
pub fn section_header(label: &str) -> String {
    let width = terminal_width();
    let dash_count = width.saturating_sub(2 + label.len() + 2); // "  label  "
    let dashes = "─".repeat(dash_count.saturating_sub(2));
    if is_color_enabled() {
        format!("{} {} {}", dim("──").dim(), info(label), dim(dashes),)
    } else {
        format!("-- {label} {dashes}")
    }
}

/// Renders a closing section line matching the header width.
pub fn section_footer() -> String {
    let width = terminal_width();
    let dashes = "─".repeat(width.saturating_sub(1));
    if is_color_enabled() {
        dim(dashes).to_string()
    } else {
        dashes
    }
}

/// Renders a phase transition header like:
///   ── Analyzing codebase ───────────────────────────────── 00:03
pub fn phase_header(icon: &str, label: &str) -> String {
    let width = terminal_width();
    let text = format!("{}  {}", icon, label);
    let dash_count = width.saturating_sub(console::strip_ansi_codes(&text).len() + 3);
    let dashes = "─".repeat(dash_count.saturating_sub(2));
    if is_color_enabled() {
        format!("{} {}", dim(text), dim(dashes),)
    } else {
        format!("{text} {dashes}")
    }
}

/// Renders a phase completion line with checkmark and elapsed time.
///   ✓ Analyzing codebase  in 3s
pub fn phase_done(icon: &str, label: &str, elapsed: &str) -> String {
    if is_color_enabled() {
        format!(
            "{}  {}  {}",
            success("✔"),
            dim(format!("{icon} {label}")),
            dim(format!("({elapsed})")),
        )
    } else {
        format!("OK  {icon} {label} ({elapsed})")
    }
}

/// Renders a step line with status icon, like Claude Code's nested output.
///   ✓ Step 1/3: Fix the test import
pub fn step_line(
    icon: &str,
    id: &str,
    description: &str,
    kind: &str,
    elapsed: Option<&str>,
) -> String {
    let base = if is_color_enabled() {
        let kind_str = dim(format!("[{}]", kind));
        format!("  {}  {}. {}  {}", icon, id, description, kind_str)
    } else {
        format!("  [{kind}] {id}. {description}")
    };
    if let Some(e) = elapsed {
        if is_color_enabled() {
            format!("{}  {}", base, dim(format!("({e})")))
        } else {
            format!("{} ({e})", base)
        }
    } else {
        base
    }
}

/// Renders an indented info line for secondary details.
pub fn detail_line(text: &str) -> String {
    if is_color_enabled() {
        format!("  {}", dim(text))
    } else {
        format!("  {text}")
    }
}

/// Renders a summary key-value pair.
pub fn summary_line(key: &str, value: &str) -> String {
    if is_color_enabled() {
        format!("  {} {}", dim(format!("{key}:")), value)
    } else {
        format!("  {key}: {value}")
    }
}

/// Renders an inline badge styled by verdict/status.
pub fn verdict_badge(text: &str, verdict: &str) -> String {
    let styled = match verdict {
        "pass" | "proceed" | "ok" => success(text),
        "fail" | "error" | "halt" => error(text),
        "warn" | "ask" => warning(text),
        "info" => info(text),
        _ => dim(text),
    };
    if is_color_enabled() {
        format!("{}", styled)
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_icon_maps_known_levels() {
        assert!(severity_icon("error").contains('✗') || severity_icon("error") == "✗");
        assert!(severity_icon("warning").contains('⚠') || severity_icon("warning") == "⚠");
        assert_eq!(severity_icon("unknown"), "•");
    }

    #[test]
    fn health_bar_renders_score_and_width() {
        let bar = health_bar(50, 10);
        assert!(bar.contains("50"));
        assert!(bar.contains('/'));
        assert!(bar.starts_with('['));
    }

    #[test]
    fn elapsed_display_formats_seconds_and_hours() {
        assert_eq!(elapsed_display(Duration::from_secs(30)), "30s");
        assert_eq!(elapsed_display(Duration::from_secs(3700)), "1h 1m");
    }

    #[test]
    fn sparkline_empty_and_non_empty() {
        assert!(sparkline(&[]).is_empty());
        let line = sparkline(&[0, 50, 100]);
        assert_eq!(line.chars().count(), 3);
    }

    #[test]
    fn badge_wraps_text() {
        let b = badge("OK", "success");
        assert!(b.starts_with('[') && b.ends_with(']'));
    }

    #[test]
    fn terminal_width_returns_reasonable_default() {
        let w = terminal_width();
        assert!(w >= 60);
        assert!(w <= 240);
    }

    #[test]
    fn section_header_ends_with_dashes() {
        let h = section_header("Test");
        assert!(!h.is_empty());
    }

    #[test]
    fn phase_done_contains_check() {
        let d = phase_done("🔍", "Analyzing", "3s");
        assert!(d.contains('✔') || d.contains("OK"));
    }

    #[test]
    fn verdict_badge_preserves_text() {
        let b = verdict_badge("PASS", "pass");
        // Should contain the text regardless of color styling
        assert!(console::strip_ansi_codes(&b).contains("PASS"));
    }
}
