// pkg/dx/errors.go
package dx

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
)

// EnhancedError provides rich error information with suggestions.
type EnhancedError struct {
	Message     string
	Line        int
	Column      int
	File        string
	Suggestions []string
	Context     string
	QuickFix    string
	RuleName    string
}

// Format formats an enhanced error with color and suggestions.
func (e *EnhancedError) Format(useColor bool) string {
	var sb strings.Builder

	// Location
	location := fmt.Sprintf("%s:%d:%d", e.File, e.Line, e.Column)
	if e.File == "" {
		location = fmt.Sprintf("line %d, column %d", e.Line, e.Column)
	}

	if useColor {
		sb.WriteString(fmt.Sprintf("\033[31m✗ Error\033[0m: %s\n", e.Message))
		sb.WriteString(fmt.Sprintf("\033[90m  At: %s\033[0m\n", location))
	} else {
		sb.WriteString(fmt.Sprintf("✗ Error: %s\n", e.Message))
		sb.WriteString(fmt.Sprintf("  At: %s\n", location))
	}

	// Context
	if e.Context != "" {
		if useColor {
			sb.WriteString("\033[90m  Context:\033[0m\n")
		} else {
			sb.WriteString("  Context:\n")
		}
		lines := strings.Split(e.Context, "\n")
		for _, line := range lines {
			if useColor {
				sb.WriteString(fmt.Sprintf("\033[90m    %s\033[0m\n", line))
			} else {
				sb.WriteString(fmt.Sprintf("    %s\n", line))
			}
		}
	}

	// Suggestions
	if len(e.Suggestions) > 0 {
		if useColor {
			sb.WriteString("\033[33m  Suggestions:\033[0m\n")
		} else {
			sb.WriteString("  Suggestions:\n")
		}
		for i, suggestion := range e.Suggestions {
			if useColor {
				sb.WriteString(fmt.Sprintf("\033[33m  → %s\033[0m\n", suggestion))
			} else {
				sb.WriteString(fmt.Sprintf("  → %s\n", suggestion))
			}
			// Limit to 3 suggestions
			if i >= 2 {
				break
			}
		}
	}

	// Quick fix
	if e.QuickFix != "" {
		if useColor {
			sb.WriteString(fmt.Sprintf("\033[32m  Quick fix:\033[0m %s\n", e.QuickFix))
		} else {
			sb.WriteString(fmt.Sprintf("  Quick fix: %s\n", e.QuickFix))
		}
	}

	return sb.String()
}

// ErrorEnhancer enhances basic validation errors with suggestions and context.
type ErrorEnhancer struct {
	program   interface{} // *language.Program
	fileLines []string
	fileName  string
}

// NewErrorEnhancer creates a new error enhancer.
func NewErrorEnhancer(fileName string, fileLines []string, program interface{}) *ErrorEnhancer {
	return &ErrorEnhancer{
		fileName:  fileName,
		fileLines: fileLines,
		program:   program,
	}
}

// Enhance converts a basic validation error into an enhanced error with suggestions.
func (e *ErrorEnhancer) Enhance(err engine.ValidationError) *EnhancedError {
	enhanced := &EnhancedError{
		Message:  err.Message,
		Line:     err.Line,
		Column:   err.Column,
		File:     e.fileName,
		RuleName: extractRuleName(err.Message),
	}

	// Extract context from source code
	enhanced.Context = e.extractContext(err.Line, err.Column)

	// Generate suggestions based on error type
	enhanced.Suggestions = e.generateSuggestions(err)

	return enhanced
}

// extractContext extracts surrounding code context for an error.
func (e *ErrorEnhancer) extractContext(line, column int) string {
	if len(e.fileLines) == 0 {
		return ""
	}

	var context strings.Builder
	lineIndex := line - 1 // Convert to 0-based index

	// Show 2 lines before and after
	start := lineIndex - 2
	if start < 0 {
		start = 0
	}
	end := lineIndex + 2
	if end > len(e.fileLines) {
		end = len(e.fileLines)
	}

	for i := start; i < end; i++ {
		if i == lineIndex {
			// Highlight the error line
			lineText := e.fileLines[i]
			if column > 0 && column <= len(lineText) {
				context.WriteString(fmt.Sprintf("%s\n", lineText))
				// Add caret pointing to column
				indent := strings.Repeat(" ", column-1)
				context.WriteString(fmt.Sprintf("%s^\n", indent))
			} else {
				context.WriteString(fmt.Sprintf("%s ← Error here\n", lineText))
			}
		} else {
			context.WriteString(fmt.Sprintf("%s\n", e.fileLines[i]))
		}
	}

	return strings.TrimSpace(context.String())
}

// generateSuggestions generates helpful suggestions based on error message.
func (e *ErrorEnhancer) generateSuggestions(err engine.ValidationError) []string {
	suggestions := []string{}
	msg := strings.ToLower(err.Message)

	// Unknown reference errors
	if strings.Contains(msg, "unknown") || strings.Contains(msg, "not found") || strings.Contains(msg, "invalid reference") {
		suggestions = append(suggestions, "Check if the element ID is spelled correctly")
		suggestions = append(suggestions, "Verify that the element is defined in the same file or imported")
		suggestions = append(suggestions, "Use 'sruja list systems' or 'sruja list containers' to see available elements")

		// Try to suggest similar element names
		if suggested := e.suggestSimilarElement(err.Message); suggested != "" {
			suggestions = append(suggestions, fmt.Sprintf("Did you mean '%s'?", suggested))
		}
	}

	// Duplicate ID errors
	if strings.Contains(msg, "duplicate") || strings.Contains(msg, "already defined") {
		suggestions = append(suggestions, "Rename one of the elements to have a unique ID")
		suggestions = append(suggestions, "Use a more specific name that reflects the element's purpose")
	}

	// Cycle detection errors
	if strings.Contains(msg, "cycle") || strings.Contains(msg, "circular") {
		suggestions = append(suggestions, "Review the dependency relationships")
		suggestions = append(suggestions, "Consider breaking the cycle by introducing an intermediate element")
		suggestions = append(suggestions, "Use a queue or event bus to decouple the dependencies")
	}

	// Missing metadata errors
	if strings.Contains(msg, "missing metadata") || strings.Contains(msg, "required metadata") {
		suggestions = append(suggestions, "Add a metadata block with the required keys")
		suggestions = append(suggestions, "Check available metadata keys with LSP autocomplete")
	}

	// Import errors
	if strings.Contains(msg, "import") || strings.Contains(msg, "cannot resolve") {
		suggestions = append(suggestions, "Check that the import path is correct")
		suggestions = append(suggestions, "Verify that the imported file exists")
		suggestions = append(suggestions, "Ensure the imported file has valid DSL syntax")
	}

	return suggestions
}

// suggestSimilarElement tries to suggest a similar element name from the error message.
func (e *ErrorEnhancer) suggestSimilarElement(errorMsg string) string {
	// Extract element ID from error message
	// Simple heuristic: look for quoted strings or identifiers
	// This is a basic implementation - can be enhanced with fuzzy matching

	// For now, return empty string - can be enhanced later with actual element lookups
	return ""
}

// extractRuleName extracts the rule name from an error message.
func extractRuleName(msg string) string {
	// Try to extract rule name from common patterns
	if strings.Contains(msg, "duplicate") {
		return "Unique IDs"
	}
	if strings.Contains(msg, "unknown") || strings.Contains(msg, "invalid reference") {
		return "Valid References"
	}
	if strings.Contains(msg, "cycle") || strings.Contains(msg, "circular") {
		return "Cycle Detection"
	}
	if strings.Contains(msg, "orphan") {
		return "Orphan Detection"
	}
	return "Validation"
}

// FormatErrors formats a slice of enhanced errors with color support.
func FormatErrors(errors []*EnhancedError, useColor bool) string {
	if len(errors) == 0 {
		return ""
	}

	var sb strings.Builder

	if useColor {
		sb.WriteString(fmt.Sprintf("\033[31m✗ Found %d error(s)\033[0m\n\n", len(errors)))
	} else {
		sb.WriteString(fmt.Sprintf("✗ Found %d error(s)\n\n", len(errors)))
	}

	for i, err := range errors {
		sb.WriteString(err.Format(useColor))
		if i < len(errors)-1 {
			sb.WriteString("\n")
		}
	}

	return sb.String()
}
