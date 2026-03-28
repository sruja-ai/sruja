use sruja_diagnostics::Diagnostic;

/// Enrich diagnostics with a source snippet and caret indicator.
///
/// This is a CLI-only enhancement (no file IO): callers pass the file content.
/// We merge the generated snippet with any existing `Diagnostic.context` lines
/// (those are preserved as `note: ...` lines).
pub fn enrich_diagnostics_with_source(content: &str, diagnostics: &mut [Diagnostic]) {
    // `lines()` drops trailing empty final line, which is fine for diagnostics display.
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return;
    }

    for diag in diagnostics {
        let line_1_indexed = diag.location.line;
        if line_1_indexed == 0 {
            continue;
        }

        let idx = (line_1_indexed.saturating_sub(1)) as usize;
        if idx >= lines.len() {
            continue;
        }

        // Width for pretty alignment of line numbers (show +/-1 lines).
        let max_line = (line_1_indexed + 1).to_string();
        let width = max_line.len().max(2);

        let mut ctx: Vec<String> = Vec::with_capacity(6);

        // Previous line
        if idx > 0 {
            ctx.push(format!(
                "{:>width$} | {}",
                line_1_indexed - 1,
                lines[idx - 1],
                width = width
            ));
        }

        // Current line
        ctx.push(format!(
            "{:>width$} | {}",
            line_1_indexed,
            lines[idx],
            width = width
        ));

        // Caret line (best-effort: column is 1-indexed and counts characters approximately)
        let col_1_indexed = diag.location.column.max(1) as usize;
        let caret_pos = col_1_indexed.saturating_sub(1);
        let pad_width = " ".repeat(width);
        let caret_spaces = " ".repeat(caret_pos.min(500)); // cap to avoid absurd spacing
        ctx.push(format!("{} | {}^", pad_width, caret_spaces));

        // Next line
        if idx + 1 < lines.len() {
            ctx.push(format!(
                "{:>width$} | {}",
                line_1_indexed + 1,
                lines[idx + 1],
                width = width
            ));
        }

        // Preserve any existing context as notes (dedup conservatively).
        let old_context = std::mem::take(&mut diag.context);
        for line in old_context {
            if line.trim().is_empty() {
                continue;
            }
            let note = format!("note: {}", line);
            if !ctx.iter().any(|existing| existing == &note) {
                ctx.push(note);
            }
        }

        diag.context = ctx;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_diagnostics::{codes, Severity, SourceLocation};

    fn create_diagnostic_at(line: u32, column: u32, context: Vec<String>) -> Diagnostic {
        Diagnostic::new(
            codes::CODE_UNDEFINED_REF,
            Severity::Error,
            "Undefined ref",
            SourceLocation::new("test.sruja".to_string(), line, column),
        )
        .with_context(context)
    }

    #[test]
    fn test_enrich_diagnostics_with_source_adds_snippet_and_caret_and_preserves_notes() {
        let content = "first line\nsecond line\nthird line\n";
        let mut diagnostics = vec![create_diagnostic_at(
            2,
            4,
            vec!["existing context".to_string(), "".to_string()],
        )];

        enrich_diagnostics_with_source(content, &mut diagnostics);

        let diag = &diagnostics[0];
        assert!(diag.context.iter().any(|l| l.contains("| second line")));
        assert!(diag.context.iter().any(|l| l.contains("^")));
        assert!(diag.context.iter().any(|l| l == "note: existing context"));
    }

    #[test]
    fn test_enrich_diagnostics_with_source_ignores_zero_line() {
        let content = "line\n";
        let mut diagnostics = vec![create_diagnostic_at(0, 1, vec!["x".to_string()])];
        enrich_diagnostics_with_source(content, &mut diagnostics);
        assert_eq!(diagnostics[0].context, vec!["x".to_string()]);
    }

    #[test]
    fn test_enrich_diagnostics_with_source_ignores_out_of_range_line() {
        let content = "line\n";
        let mut diagnostics = vec![create_diagnostic_at(10, 1, vec!["x".to_string()])];
        enrich_diagnostics_with_source(content, &mut diagnostics);
        assert_eq!(diagnostics[0].context, vec!["x".to_string()]);
    }
}
