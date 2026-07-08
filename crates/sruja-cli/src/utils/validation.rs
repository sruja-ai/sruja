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

/// Validates that a value is within a specified range.
///
/// Returns `Ok(value)` if the value is within the range, or `Err(String)` with an
/// error message if it is outside the range.
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    name: &str,
) -> Result<T, String> {
    if value < min || value > max {
        Err(format!(
            "{} must be between {} and {}, got {}",
            name, min, max, value
        ))
    } else {
        Ok(value)
    }
}

/// Validates that a collection is not empty.
///
/// Returns `Ok(collection)` if the collection has at least one element, or `Err(String)`
/// with an error message if it is empty.
pub fn validate_not_empty_collection<T>(collection: &[T], name: &str) -> Result<(), String> {
    if collection.is_empty() {
        Err(format!("{} must not be empty", name))
    } else {
        Ok(())
    }
}

/// Validates that a percentage is between 0 and 100.
///
/// Returns `Ok(percentage)` if valid, or `Err(String)` with an error message.
pub fn validate_percentage(percentage: f64, name: &str) -> Result<f64, String> {
    validate_range(percentage, 0.0, 100.0, name)
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

    #[test]
    fn test_validate_range_within_bounds() {
        assert!(validate_range(5, 1, 10, "value").is_ok());
    }

    #[test]
    fn test_validate_range_at_min() {
        assert!(validate_range(1, 1, 10, "value").is_ok());
    }

    #[test]
    fn test_validate_range_at_max() {
        assert!(validate_range(10, 1, 10, "value").is_ok());
    }

    #[test]
    fn test_validate_range_below_min() {
        assert!(validate_range(0, 1, 10, "value").is_err());
    }

    #[test]
    fn test_validate_range_above_max() {
        assert!(validate_range(11, 1, 10, "value").is_err());
    }

    #[test]
    fn test_validate_not_empty_collection_with_elements() {
        assert!(validate_not_empty_collection(&[1, 2, 3], "list").is_ok());
    }

    #[test]
    fn test_validate_not_empty_collection_empty() {
        assert!(validate_not_empty_collection(&[], "list").is_err());
    }

    #[test]
    fn test_validate_percentage_valid() {
        assert!(validate_percentage(50.0, "progress").is_ok());
    }

    #[test]
    fn test_validate_percentage_at_min() {
        assert!(validate_percentage(0.0, "progress").is_ok());
    }

    #[test]
    fn test_validate_percentage_at_max() {
        assert!(validate_percentage(100.0, "progress").is_ok());
    }

    #[test]
    fn test_validate_percentage_below_min() {
        assert!(validate_percentage(-1.0, "progress").is_err());
    }

    #[test]
    fn test_validate_percentage_above_max() {
        assert!(validate_percentage(101.0, "progress").is_err());
    }
}
