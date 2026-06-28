use std::collections::HashMap;

use super::config::AreaDef;

/// Splits gaps and bugs by configured project area so parallel agents
/// edit non-overlapping files.
#[derive(Debug, Clone)]
pub struct AreaPartitioner {
    areas: Vec<AreaDef>,
}

impl AreaPartitioner {
    pub fn new(areas: Vec<AreaDef>) -> Self {
        Self { areas }
    }

    /// Assign a file path to an area name.
    pub fn assign(&self, file_path: &str) -> &str {
        for area in &self.areas {
            for pattern in &area.patterns {
                if glob_match(pattern, file_path) {
                    return &area.name;
                }
            }
        }
        // Fallback to first area or "unknown"
        self.areas.first().map(|a| a.name.as_str()).unwrap_or("unknown")
    }

    /// Split a list of items with an area field into per-area buckets.
    pub fn split_by_area<'a, T: AreaAware>(
        &'a self,
        items: &'a [T],
    ) -> HashMap<&'a str, Vec<&'a T>> {
        let mut map: HashMap<&str, Vec<&T>> = HashMap::new();
        for item in items {
            let area = self.assign(item.area_hint());
            map.entry(area).or_default().push(item);
        }
        map
    }
}

/// Trait for items that know which area they belong to.
pub trait AreaAware {
    fn area_hint(&self) -> &str;
}

/// Simple glob-style pattern matching (`**/*`, `*`, `?`).
/// Does NOT support `{}` or `[]` groups — keeps it lightweight.
fn glob_match(pattern: &str, path: &str) -> bool {
    if pattern == "**/*" || pattern == "*" {
        return true;
    }

    let pat_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();

    glob_match_parts(&pat_parts, &path_parts)
}

fn glob_match_parts(pat: &[&str], path: &[&str]) -> bool {
    match (pat.first(), path.first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        // ** at the start: match any number of path segments
        (Some(&"**"), _) => {
            let rest = &pat[1..];
            if rest.is_empty() {
                return true;
            }
            // Try matching rest of pattern against progressively shorter paths
            for i in 0..=path.len() {
                if glob_match_parts(rest, &path[i..]) {
                    return true;
                }
            }
            false
        }
        (Some(p), Some(fp)) => {
            if simple_glob_match(p, fp) {
                glob_match_parts(&pat[1..], &path[1..])
            } else {
                false
            }
        }
        (Some(&"*"), _) => path.len() == 1,
        (Some(_), _) => false,
    }
}

fn simple_glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') && !pattern.contains('?') {
        return pattern == text;
    }

    // Convert simple glob to regex-like matching
    let pat_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    simple_glob_recursive(&pat_chars, &text_chars)
}

fn simple_glob_recursive(pat: &[char], text: &[char]) -> bool {
    match (pat.first(), text.first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some(&'*'), _) => {
            let rest = &pat[1..];
            if rest.is_empty() {
                return true; // trailing * matches everything
            }
            // Try matching rest against progressively shorter text suffixes
            for i in 0..=text.len() {
                if simple_glob_recursive(rest, &text[i..]) {
                    return true;
                }
            }
            false
        }
        (Some(&'?'), Some(_)) => simple_glob_recursive(&pat[1..], &text[1..]),
        (Some(p), Some(t)) if p == t => simple_glob_recursive(&pat[1..], &text[1..]),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign_to_area() {
        let areas = vec![
            AreaDef { name: "core".into(), patterns: vec!["crates/core/**".into()] },
            AreaDef { name: "api".into(), patterns: vec!["crates/api/**".into()] },
        ];
        let p = AreaPartitioner::new(areas);
        assert_eq!(p.assign("crates/core/src/lib.rs"), "core");
        assert_eq!(p.assign("crates/api/src/handler.rs"), "api");
    }

    #[test]
    fn test_assign_fallback_to_first() {
        let areas = vec![
            AreaDef { name: "all".into(), patterns: vec!["**/*".into()] },
        ];
        let p = AreaPartitioner::new(areas);
        assert_eq!(p.assign("some/random/file.rs"), "all");
    }

    #[test]
    fn test_glob_match_star() {
        assert!(glob_match("*.rs", "lib.rs"));
        assert!(!glob_match("*.rs", "lib.js"));
    }

    #[test]
    fn test_glob_match_double_star() {
        assert!(glob_match("**", "anything/goes/here.rs"));
        assert!(glob_match("crates/**/src", "crates/core/src"));
        assert!(!glob_match("crates/**/src", "crates/core/lib"));
    }

    #[test]
    fn test_glob_match_question() {
        assert!(glob_match("src/??.rs", "src/ab.rs"));
        assert!(!glob_match("src/??.rs", "src/abc.rs"));
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("src/lib.rs", "src/lib.rs"));
        assert!(!glob_match("src/lib.rs", "src/main.rs"));
    }
}
