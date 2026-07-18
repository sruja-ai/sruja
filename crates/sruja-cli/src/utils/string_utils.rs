/// Truncate a string to `max_len` characters, appending `"…"` if truncated.
///
/// If the string already fits within `max_len`, it is returned unchanged.
/// The ellipsis is counted toward the limit: a 10-char limit means up to 9
/// visible characters + `"…"`.
///
/// # Examples
///
/// ```
/// use sruja_cli::utils::string_utils::truncate_with_ellipsis;
///
/// assert_eq!(truncate_with_ellipsis("hello", 10), "hello");
/// assert_eq!(truncate_with_ellipsis("hello world", 8), "hello w…");
/// assert_eq!(truncate_with_ellipsis("hi", 3), "hi");
/// ```
#[allow(dead_code)]
pub fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }
    if s.chars().count() <= max_len {
        return s.to_string();
    }
    if max_len == 1 {
        return "…".to_string();
    }
    let truncated: String = s.chars().take(max_len - 1).collect();
    format!("{truncated}…")
}

/// Convert a `snake_case` (or `SCREAMING_SNAKE_CASE`) identifier into
/// Title Case.
///
/// Each segment is capitalised and segments are joined with spaces.
///
/// # Examples
///
/// ```
/// use sruja_cli::utils::string_utils::snake_to_title_case;
///
/// assert_eq!(snake_to_title_case("hello_world"), "Hello World");
/// assert_eq!(snake_to_title_case("FOO_BAR_BAZ"), "Foo Bar Baz");
/// assert_eq!(snake_to_title_case("already"), "Already");
/// ```
#[allow(dead_code)]
pub fn snake_to_title_case(s: &str) -> String {
    s.split('_')
        .filter(|seg| !seg.is_empty())
        .map(|seg| {
            let mut chars = seg.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    let rest: String = chars.collect::<String>().to_lowercase();
                    format!("{upper}{rest}")
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Count the number of whitespace-separated words in a string.
///
/// Consecutive whitespace is treated as a single separator and leading /
/// trailing whitespace is ignored.
///
/// # Examples
///
/// ```
/// use sruja_cli::utils::string_utils::count_words;
///
/// assert_eq!(count_words("hello world"), 2);
/// assert_eq!(count_words("  spaces   everywhere  "), 2);
/// assert_eq!(count_words(""), 0);
/// assert_eq!(count_words("single"), 1);
/// ```
#[allow(dead_code)]
pub fn count_words(s: &str) -> usize {
    s.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── truncate_with_ellipsis ──────────────────────────────────────

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate_with_ellipsis("abc", 5), "abc");
    }

    #[test]
    fn truncate_exact_length_unchanged() {
        assert_eq!(truncate_with_ellipsis("abcde", 5), "abcde");
    }

    #[test]
    fn truncate_longer_string_adds_ellipsis() {
        assert_eq!(truncate_with_ellipsis("abcdef", 5), "abcd…");
    }

    #[test]
    fn truncate_one_char_limit() {
        assert_eq!(truncate_with_ellipsis("abc", 1), "…");
    }

    #[test]
    fn truncate_zero_limit() {
        assert_eq!(truncate_with_ellipsis("anything", 0), "");
    }

    #[test]
    fn truncate_empty_string() {
        assert_eq!(truncate_with_ellipsis("", 5), "");
    }

    #[test]
    fn truncate_utf8_multibyte_chars() {
        // Each é is 2 bytes; ensure we don't panic on byte-boundary splits
        // and the ellipsis is appended.
        let s = "é".repeat(10); // 10 chars, 20 bytes
        let result = truncate_with_ellipsis(&s, 10);
        // 10 chars fits exactly, so no truncation needed
        assert_eq!(result, s);

        let result2 = truncate_with_ellipsis(&s, 5);
        // Should truncate to 4 chars + ellipsis
        assert!(result2.ends_with('…'));
        assert_eq!(result2.chars().count(), 5);
    }

    // ── snake_to_title_case ─────────────────────────────────────────

    #[test]
    fn title_case_simple() {
        assert_eq!(snake_to_title_case("hello_world"), "Hello World");
    }

    #[test]
    fn title_case_single_word() {
        assert_eq!(snake_to_title_case("hello"), "Hello");
    }

    #[test]
    fn title_case_screaming_snake() {
        assert_eq!(snake_to_title_case("FOO_BAR_BAZ"), "Foo Bar Baz");
    }

    #[test]
    fn title_case_leading_trailing_underscores() {
        assert_eq!(snake_to_title_case("_hello_"), "Hello");
    }

    #[test]
    fn title_case_empty_string() {
        assert_eq!(snake_to_title_case(""), "");
    }

    #[test]
    fn title_case_multiple_consecutive_underscores() {
        assert_eq!(snake_to_title_case("a__b__c"), "A B C");
    }

    // ── count_words ─────────────────────────────────────────────────

    #[test]
    fn words_normal() {
        assert_eq!(count_words("hello world"), 2);
    }

    #[test]
    fn words_empty() {
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn words_single() {
        assert_eq!(count_words("single"), 1);
    }

    #[test]
    fn words_extra_whitespace() {
        assert_eq!(count_words("  spaces   everywhere  "), 2);
    }

    #[test]
    fn words_tabs_and_newlines() {
        assert_eq!(count_words("line1\nline2\tline3"), 3);
    }
}
