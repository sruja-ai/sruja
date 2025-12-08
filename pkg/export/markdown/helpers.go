// pkg/export/markdown/helpers.go
// Package markdown provides helper functions.
//
//nolint:gocritic // Use WriteString for consistency
package markdown

import (
	"fmt"
	"strings"
)

// writeElement writes a list element with ID, label, and optional description
func writeElement(sb *strings.Builder, id, label string, desc *string) {
	sb.WriteString(fmt.Sprintf("- **%s**: %s", id, label))
	if desc != nil {
		sb.WriteString(fmt.Sprintf(" - %s", *desc))
	}
	sb.WriteString("\n")
}

// escapeMermaidLabel escapes quotes in labels for Mermaid diagrams
func escapeMermaidLabel(s string) string {
	return strings.ReplaceAll(s, `"`, `\"`)
}

// escapeMarkdown escapes markdown special characters
func escapeMarkdown(s string) string {
	// Escape backticks, asterisks, underscores, brackets
	result := strings.ReplaceAll(s, "\\", "\\\\")
	result = strings.ReplaceAll(result, "`", "\\`")
	result = strings.ReplaceAll(result, "*", "\\*")
	result = strings.ReplaceAll(result, "_", "\\_")
	result = strings.ReplaceAll(result, "[", "\\[")
	result = strings.ReplaceAll(result, "]", "\\]")
	return result
}

// generateAnchor generates a markdown anchor from a heading text
func generateAnchor(text string) string {
	// Convert to lowercase, replace spaces and special chars with hyphens
	result := strings.ToLower(text)
	result = strings.ReplaceAll(result, " ", "-")
	result = strings.ReplaceAll(result, "(", "")
	result = strings.ReplaceAll(result, ")", "")
	result = strings.ReplaceAll(result, ".", "")
	return result
}
