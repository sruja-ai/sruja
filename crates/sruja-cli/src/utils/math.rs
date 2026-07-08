/// Adds two numbers together.
///
/// # Examples
///
/// ```
/// assert_eq!(super::math::add(2, 3), 5);
/// assert_eq!(super::math::add(-1, 1), 0);
/// ```
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_positive() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-1, -2), -3);
    }

    #[test]
    fn test_add_zero() {
        assert_eq!(add(5, 0), 5);
    }

    #[test]
    fn test_add_mixed() {
        assert_eq!(add(-3, 7), 4);
    }
}
