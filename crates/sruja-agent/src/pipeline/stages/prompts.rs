/// Load a role prompt from a markdown file, with optional YAML frontmatter.
///
/// If `path` is Some and the file exists, the prompt content is read and
/// frontmatter is stripped. If the file doesn't exist or `path` is None,
/// an error is returned — the user must provide prompt files for their
/// pipeline stages.
///
/// Frontmatter format (stripped before returning the prompt body):
/// ```markdown
/// ---
/// mode: full
/// color: "#3b82f6"
/// ---
///
/// Prompt body starts here...
/// ```
pub fn load_role_prompt(path: Option<&std::path::Path>) -> Result<String, String> {
    let path = path.ok_or_else(|| "No prompt file configured for this stage".to_string())?;
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read prompt file {}: {e}", path.display()))?;
    Ok(strip_frontmatter(&content))
}

/// Strip YAML frontmatter (delimited by `---` ... `---`) from a markdown file.
/// Returns everything after the closing `---`. If no frontmatter is detected,
/// returns the content as-is.
pub fn strip_frontmatter(content: &str) -> String {
    let trimmed = content.trim();
    if let Some(rest) = trimmed.strip_prefix("---") {
        if let Some(end) = rest.find("\n---") {
            // end is position of \n in \n---, so skip 4 chars: \n - - -
            let body = rest[end + 4..].trim();
            return body.to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_frontmatter() {
        let content = "---\nmode: full\ncolor: \"#000\"\n---\n\nThis is the prompt body.";
        assert_eq!(strip_frontmatter(content), "This is the prompt body.");
    }

    #[test]
    fn test_strip_frontmatter_no_frontmatter() {
        let content = "Just a plain prompt.";
        assert_eq!(strip_frontmatter(content), "Just a plain prompt.");
    }

    #[test]
    fn test_strip_frontmatter_empty_after() {
        let content = "---\nkey: val\n---";
        assert_eq!(strip_frontmatter(content), "");
    }
}
