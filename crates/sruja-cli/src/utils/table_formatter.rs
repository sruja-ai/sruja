pub struct TableFormatter {
    max_width: usize,
}

impl TableFormatter {
    pub fn new(max_width: usize) -> Self {
        Self { max_width }
    }

    pub fn format_dashboard(&self, title: &str, blocks: Vec<(String, String)>) -> String {
        let mut output = String::new();

        let border = "═".repeat(self.max_width.saturating_sub(1).max(1));
        output.push_str(&format!("╔{}╗\n", border));

        let title_centered = format!(" {} ", title);
        let title_padding_left = (self.max_width.saturating_sub(title_centered.len()) / 2).max(1);
        let title_padding_right = self
            .max_width
            .saturating_sub(title_centered.len() + title_padding_left + 1);
        output.push_str(&format!(
            "║{}{}{}║\n",
            " ".repeat(title_padding_left),
            title_centered,
            " ".repeat(title_padding_right)
        ));
        output.push_str(&format!("╠{}╣\n", border));

        for (i, (header, content)) in blocks.iter().enumerate() {
            if i > 0 {
                output.push_str(&format!(
                    "╟{}╢\n",
                    "─".repeat(self.max_width.saturating_sub(1))
                ));
            }
            output.push_str(&format!(
                "║ {} ║\n",
                crate::utils::colors::style(header).bold()
            ));

            for line in content.lines() {
                let trimmed = line.trim_end();
                let visible_len = console::strip_ansi_codes(trimmed).len();
                let line_padding = self.max_width.saturating_sub(visible_len + 3);
                output.push_str(&format!("║  {}{}\n", trimmed, " ".repeat(line_padding)));
            }
        }

        output.push_str(&format!("╚{}╝\n", border));
        output
    }

    pub fn format_sparkline(scores: &[u8]) -> String {
        const BARS: &[char] = &[' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        scores
            .iter()
            .map(|&s| {
                let idx = ((s as f32 / 100.0) * (BARS.len() - 1) as f32).round() as usize;
                BARS[idx]
            })
            .collect()
    }

    pub fn detect_width() -> usize {
        use std::env;

        if let Some(width) = env::var("COLUMNS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
        {
            return width.clamp(60, 120);
        }

        80
    }

    pub fn auto() -> Self {
        Self::new(Self::detect_width())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_width_default() {
        std::env::remove_var("COLUMNS");
        let width = TableFormatter::detect_width();
        assert_eq!(width, 80);
    }

    #[test]
    fn test_auto_creates_formatter() {
        let formatter = TableFormatter::auto();
        assert!(formatter.max_width >= 60);
        assert!(formatter.max_width <= 120);
    }

    #[test]
    fn test_format_dashboard() {
        let formatter = TableFormatter::new(60);
        let result = formatter.format_dashboard(
            "Title",
            vec![("Header".to_string(), "Content here".to_string())],
        );
        assert!(result.contains("Title"));
        assert!(result.contains("Content here"));
        assert!(result.contains("╔"));
        assert!(result.contains("╚"));
    }

    #[test]
    fn test_format_sparkline() {
        let result = TableFormatter::format_sparkline(&[0, 50, 100]);
        assert_eq!(result.chars().count(), 3);
        assert_eq!(result.chars().next(), Some(' '));
    }
}
