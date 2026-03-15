#[allow(dead_code)]
pub struct TableFormatter {
    max_width: usize,
}

#[allow(dead_code)]
impl TableFormatter {
    pub fn new(max_width: usize) -> Self {
        Self { max_width }
    }

    pub fn wrap_text(&self, text: &str) -> Vec<String> {
        if self.max_width <= 4 {
            return vec![text.to_string()];
        }

        let wrap_width = self.max_width.saturating_sub(4);
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= wrap_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    pub fn format_section(&self, header: &str, content: &[(&str, &str)]) -> String {
        let mut output = String::new();

        let border = "─".repeat(self.max_width.saturating_sub(1).max(1));
        output.push_str(&format!("┌{}\n", border));

        let header_padding = self.max_width.saturating_sub(header.len() + 3);
        output.push_str(&format!("│ {}{}\n", header, " ".repeat(header_padding)));
        output.push_str(&format!("├{}\n", border));

        for (key, value) in content {
            let full_text = format!("{}: {}", key, value);
            let wrapped = self.wrap_text(&full_text);
            for line in wrapped {
                let line_padding = self.max_width.saturating_sub(line.len() + 3);
                output.push_str(&format!("│ {}{}\n", line, " ".repeat(line_padding)));
            }
        }

        output.push_str(&format!("└{}\n", border));
        output
    }

    pub fn format_list_section(&self, header: &str, items: &[String]) -> String {
        let mut output = String::new();

        let border = "─".repeat(self.max_width.saturating_sub(1).max(1));
        output.push_str(&format!("┌{}\n", border));

        let header_padding = self.max_width.saturating_sub(header.len() + 3);
        output.push_str(&format!("│ {}{}\n", header, " ".repeat(header_padding)));
        output.push_str(&format!("├{}\n", border));

        for item in items {
            let wrapped = self.wrap_text(&format!("• {}", item));
            for line in wrapped {
                let line_padding = self.max_width.saturating_sub(line.len() + 3);
                output.push_str(&format!("│ {}{}\n", line, " ".repeat(line_padding)));
            }
        }

        output.push_str(&format!("└{}\n", border));
        output
    }

    pub fn format_key_value_pairs(&self, pairs: &[(&str, &str)]) -> String {
        let mut output = String::new();

        for (key, value) in pairs {
            let full_text = format!("{}: {}", key, value);
            let wrapped = self.wrap_text(&full_text);

            for (i, line) in wrapped.iter().enumerate() {
                if i == 0 {
                    let line_padding = self.max_width.saturating_sub(line.len() + 3);
                    output.push_str(&format!("│ {}{}\n", line, " ".repeat(line_padding)));
                } else {
                    let indented = format!("  {}", line);
                    let line_padding = self.max_width.saturating_sub(indented.len() + 3);
                    output.push_str(&format!("│ {}{}\n", indented, " ".repeat(line_padding)));
                }
            }
        }

        output
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
    fn test_wrap_text_short_text() {
        let formatter = TableFormatter::new(40);
        let result = formatter.wrap_text("Hello world");
        assert_eq!(result, vec!["Hello world"]);
    }

    #[test]
    fn test_wrap_text_long_text() {
        let formatter = TableFormatter::new(20);
        let result = formatter.wrap_text("This is a very long text that should wrap");
        assert!(result.len() > 1);
    }

    #[test]
    fn test_wrap_text_empty() {
        let formatter = TableFormatter::new(40);
        let result = formatter.wrap_text("");
        assert_eq!(result, vec![""]);
    }

    #[test]
    fn test_wrap_text_small_width() {
        let formatter = TableFormatter::new(4);
        let result = formatter.wrap_text("Hello world");
        assert_eq!(result, vec!["Hello world"]);
    }

    #[test]
    fn test_format_section() {
        let formatter = TableFormatter::new(40);
        let result = formatter.format_section("Header", &[("Key", "Value")]);
        assert!(result.contains("Header"));
        assert!(result.contains("Key: Value"));
        assert!(result.contains("┌"));
        assert!(result.contains("└"));
    }

    #[test]
    fn test_format_section_empty_content() {
        let formatter = TableFormatter::new(40);
        let result = formatter.format_section("Header", &[]);
        assert!(result.contains("Header"));
    }

    #[test]
    fn test_format_list_section() {
        let formatter = TableFormatter::new(40);
        let result =
            formatter.format_list_section("Items", &["Item 1".to_string(), "Item 2".to_string()]);
        assert!(result.contains("Items"));
        assert!(result.contains("Item 1"));
        assert!(result.contains("Item 2"));
    }

    #[test]
    fn test_format_list_section_empty() {
        let formatter = TableFormatter::new(40);
        let result = formatter.format_list_section("Items", &[]);
        assert!(result.contains("Items"));
    }

    #[test]
    fn test_format_key_value_pairs() {
        let formatter = TableFormatter::new(40);
        let result = formatter.format_key_value_pairs(&[("Key1", "Value1"), ("Key2", "Value2")]);
        assert!(result.contains("Key1: Value1"));
        assert!(result.contains("Key2: Value2"));
    }

    #[test]
    fn test_format_key_value_pairs_empty() {
        let formatter = TableFormatter::new(40);
        let result = formatter.format_key_value_pairs(&[]);
        assert!(result.is_empty());
    }

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
    fn test_wrap_text_preserves_words() {
        let formatter = TableFormatter::new(20);
        let result = formatter.wrap_text("word1 word2 word3");
        for line in &result {
            assert!(!line.contains("word1word2"));
        }
    }
}
