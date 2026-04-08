use console::{style as console_style, StyledObject};

pub fn style<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    console_style(text)
}

/// Returns a red styled object for error messages.
pub fn error<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    console_style(text).red().bold()
}

/// Returns a yellow styled object for warning messages.
pub fn warning<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    console_style(text).yellow()
}

/// Returns a green styled object for success messages.
pub fn success<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    console_style(text).green().bold()
}

/// Returns a blue styled object for info messages.
pub fn info<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    console_style(text).cyan()
}

/// Returns a dimmed styled object for secondary information.
pub fn dim<T: std::fmt::Display>(text: T) -> StyledObject<T> {
    console_style(text).dim()
}

/// Prints a themed header.
pub fn print_header(title: &str) {
    println!("{}", console_style(title).bold().underlined());
}
