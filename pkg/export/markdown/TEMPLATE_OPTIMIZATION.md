# Markdown Template Optimization: text/template vs html/template

## Change Summary

Switched markdown exporter from `html/template` to `text/template` for better performance and appropriateness.

## Why This Change?

### Markdown Doesn't Need XSS Protection

1. **Markdown is Plain Text**
   - Markdown files are plain text, not HTML
   - No HTML context that requires escaping
   - No JavaScript execution in markdown files
   - Safe by design

2. **Performance Improvement**
   - `text/template` is simpler and faster
   - No context-aware escaping overhead
   - More appropriate for text-based output

3. **Correct Tool for the Job**
   - `html/template`: Use for HTML output (needs XSS protection)
   - `text/template`: Use for plain text output (markdown, JSON, etc.)

## Files Changed

1. `template.go` - Changed import from `html/template` to `text/template`
2. `templates_simple.go` - Changed import from `html/template` to `text/template`

## Compatibility

✅ **No Breaking Changes**
- `text/template` and `html/template` have identical APIs
- Template syntax is the same
- Template functions work identically
- All tests pass without modification

## Security Considerations

**No security impact:**
- Markdown output is plain text
- No HTML context that could be exploited
- No JavaScript execution
- Safe to use `text/template` for markdown generation

**For HTML output:**
- HTML exporter still correctly uses `html/template`
- XSS protection remains in place for HTML exports
- Best practice maintained

## Test Results

✅ All tests passing:
- Basic system export
- Mermaid diagram generation
- TOC generation
- Scenario sequence diagrams
- Complex real-world examples

## Recommendations

### ✅ Current Implementation (Correct)

| Export Format | Template Package | Reason |
|---------------|-----------------|--------|
| HTML | `html/template` | Needs XSS protection |
| Markdown | `text/template` | Plain text, no XSS risk |

### Best Practices

1. **HTML Output** → Use `html/template` (automatic XSS protection)
2. **Text Output** → Use `text/template` (markdown, JSON, plain text)
3. **Match the tool to the output format**

## References

- [Go text/template documentation](https://pkg.go.dev/text/template)
- [Go html/template documentation](https://pkg.go.dev/html/template)
- HTML exporter analysis: `pkg/export/html/TEMPLATE_ANALYSIS.md`












