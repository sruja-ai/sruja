/// Validates that a string is not empty.
///
/// Returns `Ok(&str)` if the string is non-empty, or `Err(&str)` with an
/// error message if it is empty or consists only of whitespace.
pub fn validate_not_empty(input: &str) -> Result<&str, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Err("Input must not be empty".to_string())
    } else {
        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_empty_string_succeeds() {
        assert!(validate_not_empty("hello").is_ok());
    }

    #[test]
    fn test_empty_string_fails() {
        assert!(validate_not_empty("").is_err());
    }

    #[test]
    fn test_whitespace_only_fails() {
        assert!(validate_not_empty("   ").is_err());
    }

    #[test]
    fn test_preserves_original_value() {
        let input = "  hello  ";
        assert_eq!(validate_not_empty(input).unwrap(), input);
    }
}
