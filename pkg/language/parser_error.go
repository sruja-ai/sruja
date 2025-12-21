package language

import (
	"errors"
	"fmt"
	"strings"

	"github.com/alecthomas/participle/v2"
	"github.com/alecthomas/participle/v2/lexer"
	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

func (p *Parser) convertErrorToDiagnostics(err error, filename, text string) []diagnostics.Diagnostic {
	// Pre-allocate diagnostics slice
	diags := make([]diagnostics.Diagnostic, 0, 1)

	// Try to cast to participle.Error
	var parseErr participle.Error
	if errors.As(err, &parseErr) {
		pos := parseErr.Position()
		msg := parseErr.Message()

		// Extract context lines (error line + surrounding lines for better context)
		var context []string

		// Find line boundaries without splitting the entire file
		currentLine := 1
		start := 0
		foundTarget := false
		for i := 0; i < len(text); i++ {
			if text[i] == '\n' {
				if currentLine >= pos.Line-1 && currentLine <= pos.Line+1 {
					context = append(context, text[start:i])
				}
				if currentLine == pos.Line {
					foundTarget = true
				}
				if currentLine > pos.Line+1 {
					break
				}
				start = i + 1
				currentLine++
			}
		}
		// Handle last line if no newline at end
		if !foundTarget && currentLine == pos.Line && start < len(text) {
			context = append(context, text[start:])
		} else if currentLine == pos.Line+1 && start < len(text) {
			// If we found the error line but not the next line yet
			context = append(context, text[start:])
		}

		if len(context) > 0 {
			// Find the error line within the context
			errorLineIdx := 1
			if pos.Line == 1 {
				errorLineIdx = 0
			}
			if len(context) > errorLineIdx {
				lineContent := context[errorLineIdx]
				// Add pointer line with precise column indication
				if pos.Column > 0 {
					// Calculate pointer position accounting for tabs
					pointerCol := 0
					for i := 0; i < len(lineContent) && i < pos.Column-1; i++ {
						if lineContent[i] == '\t' {
							pointerCol += 4 // Tab width
						} else {
							pointerCol++
						}
					}
					pointer := strings.Repeat(" ", pointerCol) + "^"
					// Add underline for multi-character tokens
					tokenLen := p.estimateTokenLength(lineContent, pos.Column-1)
					if tokenLen > 1 {
						underline := strings.Repeat("~", tokenLen-1)
						pointer += underline
					}
					// Insert pointer line after the error line in context
					newContext := make([]string, 0, len(context)+1)
					newContext = append(newContext, context[:errorLineIdx+1]...)
					newContext = append(newContext, pointer)
					if errorLineIdx+1 < len(context) {
						newContext = append(newContext, context[errorLineIdx+1:]...)
					}
					context = newContext
				}
			}
		}

		// Generate enhanced error message and suggestions
		enhancedMsg, suggestions := p.enhanceErrorMessage(msg, pos)

		// Determine error code based on message content
		code := p.determineErrorCode(msg)

		diags = append(diags, diagnostics.Diagnostic{
			Code:     code,
			Severity: diagnostics.SeverityError,
			Message:  enhancedMsg,
			Location: diagnostics.SourceLocation{
				File:   filename,
				Line:   pos.Line,
				Column: pos.Column,
			},
			Context:     context,
			Suggestions: suggestions,
		})
	} else {
		// Fallback for generic errors
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeSyntaxError,
			Severity: diagnostics.SeverityError,
			Message:  err.Error(),
			Location: diagnostics.SourceLocation{File: filename},
		})
	}

	return diags
}

// estimateTokenLength estimates the length of the token at the given position
func (p *Parser) estimateTokenLength(line string, col int) int {
	if col >= len(line) {
		return 1
	}
	// Simple heuristic: check if it's an identifier, string, or operator
	start := col
	for start > 0 && (isIdentChar(line[start-1]) || line[start-1] == '.') {
		start--
	}
	end := col
	for end < len(line) && (isIdentChar(line[end]) || line[end] == '.') {
		end++
	}
	length := end - start
	if length == 0 {
		return 1
	}
	return length
}

// isIdentChar checks if a character is part of an identifier
func isIdentChar(ch byte) bool {
	return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || (ch >= '0' && ch <= '9') || ch == '_'
}

// determineErrorCode maps error messages to specific error codes
func (p *Parser) determineErrorCode(msg string) string {
	msgLower := strings.ToLower(msg)
	switch {
	case strings.Contains(msgLower, "unexpected token") && strings.Contains(msgLower, "expected"):
		return diagnostics.CodeUnexpectedToken
	case strings.Contains(msgLower, "missing") && strings.Contains(msgLower, "brace"):
		return diagnostics.CodeMissingBrace
	case strings.Contains(msgLower, "string") && (strings.Contains(msgLower, "invalid") || strings.Contains(msgLower, "unterminated")):
		return diagnostics.CodeInvalidString
	default:
		return diagnostics.CodeSyntaxError
	}
}

// enhanceErrorMessage improves error messages with better context and suggestions
func (p *Parser) enhanceErrorMessage(msg string, pos lexer.Position) (string, []string) {
	var suggestions []string
	enhancedMsg := msg

	// Extract token from error message
	token := ""
	if strings.Contains(msg, "unexpected token") {
		if parts := strings.Split(msg, "\""); len(parts) >= 2 {
			token = parts[1]
		}
	}

	// Common DSL patterns and their fixes
	knownKeywords := map[string]string{
		"tags":         "Tags should come after the element definition",
		"status":       "Status should come after the element ID and label",
		"context":      "Context should come after the element definition",
		"decision":     "Decision should come after the ADR title",
		"consequences": "Consequences should come after the decision field",
		"description":  "Description should come after the element ID and label",
		"type":         "Type should come after the element definition",
		"category":     "Category should come after the policy ID",
		"enforcement":  "Enforcement should come after the policy category",
		"technology":   "Technology should come after the container/component definition",
	}

	// Check for specific error patterns
	if strings.Contains(msg, "unexpected token") && token != "" {
		if hint, ok := knownKeywords[token]; ok {
			enhancedMsg = fmt.Sprintf("Unexpected '%s' at this position. %s", token, hint)
			suggestions = append(suggestions, fmt.Sprintf("Move '%s' to the correct position in the element definition", token))
			suggestions = append(suggestions, "Check the DSL syntax guide for the correct field order")
		} else {
			switch token {
			case "{":
				enhancedMsg = "Unexpected block start '{'. Missing required fields before the block."
				suggestions = append(suggestions, "Ensure you have defined the element ID and label before the block")
				suggestions = append(suggestions, "Example: system MySystem \"My System\" { ... }")
			case "}":
				enhancedMsg = "Unexpected block end '}'. Missing required fields inside the block."
				suggestions = append(suggestions, "Check that all required fields are defined inside the block")
			default:
				// Try to find similar keywords for typo detection
				similar := p.findSimilarKeywords(token)
				if len(similar) > 0 {
					enhancedMsg = fmt.Sprintf("Unexpected token '%s'. Did you mean one of: %s?", token, strings.Join(similar, ", "))
					suggestions = append(suggestions, fmt.Sprintf("Check for typos. Did you mean '%s'?", similar[0]))
				} else {
					enhancedMsg = fmt.Sprintf("Unexpected token '%s' at line %d, column %d", token, pos.Line, pos.Column)
					suggestions = append(suggestions, "Check the DSL syntax for valid keywords and operators")
				}
			}
		}
	} else if strings.Contains(msg, "expected") {
		// Extract what was expected
		if strings.Contains(msg, "expected") {
			enhancedMsg = fmt.Sprintf("Syntax error: %s", msg)
			suggestions = append(suggestions, "Check the DSL syntax guide for the correct structure")
		}
	}

	// Add general suggestions if none were added
	if len(suggestions) == 0 {
		suggestions = append(suggestions, "Check the DSL syntax documentation")
		suggestions = append(suggestions, "Ensure all braces, quotes, and operators are properly matched")
	}

	return enhancedMsg, suggestions
}

// findSimilarKeywords finds keywords similar to the given token (for typo detection)
func (p *Parser) findSimilarKeywords(token string) []string {
	allKeywords := []string{
		"system", "container", "component", "datastore", "queue", "person",
		"relation", "metadata", "properties", "style",
		"requirement", "adr", "policy", "flow", "scenario",
		"tags", "status", "description", "technology", "type", "category",
		"enforcement", "decision", "consequences", "context",
		"specification", "model", "views", "view", "element", "include",
	}

	var similar []string
	tokenLower := strings.ToLower(token)

	for _, keyword := range allKeywords {
		if p.isSimilar(tokenLower, keyword) {
			similar = append(similar, keyword)
		}
	}

	// Limit to top 3 most similar
	if len(similar) > 3 {
		similar = similar[:3]
	}

	return similar
}

// isSimilar checks if two strings are similar (simple Levenshtein-like check)
func (p *Parser) isSimilar(s1, s2 string) bool {
	if len(s1) == 0 || len(s2) == 0 {
		return false
	}
	// Simple similarity: same length and at least 50% character match
	if len(s1) == len(s2) {
		matches := 0
		for i := 0; i < len(s1); i++ {
			if s1[i] == s2[i] {
				matches++
			}
		}
		return float64(matches)/float64(len(s1)) >= 0.5
	}
	// Check if one contains the other (for partial matches)
	return strings.Contains(s1, s2) || strings.Contains(s2, s1)
}
