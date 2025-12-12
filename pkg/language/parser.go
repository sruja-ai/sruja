// Package language provides DSL parsing and AST structures.
package language

import (
	"errors"
	"fmt"
	"os"
	"strings"

	"github.com/alecthomas/participle/v2"
	"github.com/alecthomas/participle/v2/lexer"
	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

// Parser parses Sruja DSL text into an AST (Abstract Syntax Tree).
//
// The parser uses participle (a Go parser library) to parse the DSL grammar.
// It handles:
//   - Lexical analysis (tokenizing the input)
//   - Syntax analysis (parsing tokens into AST)
//   - Error reporting (with source locations)
//
// Example usage:
//
//	parser, err := language.NewParser()
//	if err != nil {
//	    log.Fatal(err)
//	}
//
//	program, diags, err := parser.Parse("example.sruja", dslText)
//	if err != nil {
//	    // handle critical error
//	}
//	if len(diags) > 0 {
//	    // handle diagnostics (errors/warnings)
//	}
//
//	workspace := program.Workspace
type Parser struct {
	parser *participle.Parser[File] // The participle parser instance
}

// NewParser creates a new parser instance.
//
// This builds the parser with the Sruja DSL grammar. The parser is configured with:
//   - Custom lexer for Sruja tokens (keywords, strings, arrows, etc.)
//   - Lookahead for ambiguous grammar
//   - String unquoting (removes quotes from string literals)
//
// Returns an error if the parser cannot be built (e.g., invalid grammar).
func NewParser() (*Parser, error) {
	// Define lexer for Sruja DSL
	// This tokenizes the input into keywords, strings, operators, etc.
	srujaLexer := lexer.MustSimple([]lexer.SimpleRule{
		{Name: "Comment", Pattern: `//.*|/\*.*?\*/`},
		{Name: "String", Pattern: `"(\\"|[^"])*"`},
		{Name: "Int", Pattern: `\d+`},
		{Name: "Number", Pattern: `\d+(?:\.\d+)?`},
		{Name: "Library", Pattern: `library`},
		{Name: "Story", Pattern: `story`},
		{Name: "Scenario", Pattern: `scenario`},
		{Name: "Flow", Pattern: `flow`},
		{Name: "Policy", Pattern: `policy`},
		{Name: "Wildcard", Pattern: `\*`}, // For view expressions: include *
		{Name: "Ident", Pattern: `[a-zA-Z_][a-zA-Z0-9_]*`},
		{Name: "Dot", Pattern: `\.`},
		{Name: "Arrow", Pattern: `->`},
		{Name: "Colon", Pattern: `:`},
		{Name: "Comma", Pattern: `,`},
		{Name: "Less", Pattern: `<`},
		{Name: "Greater", Pattern: `>`},
		{Name: "Question", Pattern: `\?`},
		{Name: "LBracket", Pattern: `\[`},
		{Name: "RBracket", Pattern: `\]`},
		{Name: "LBrace", Pattern: `\{`},
		{Name: "RBrace", Pattern: `\}`},
		{Name: "Whitespace", Pattern: `\s+`},
	})

	parser, err := participle.Build[File](
		participle.Lexer(srujaLexer),
		participle.Unquote("String"),
		participle.Elide("Whitespace"),
		participle.Elide("Comment"),
		participle.UseLookahead(5), // Increased for qualified identifiers in scenarios/flows
	)
	if err != nil {
		return nil, fmt.Errorf("failed to build parser: %w", err)
	}

	return &Parser{parser: parser}, nil
}

// Parse parses DSL text into an AST.
//
// Parameters:
//   - filename: Name of the file (used for error reporting)
//   - text: The DSL text to parse
//
// Returns:
//   - *Program: The parsed program (root of AST)
//   - []diagnostics.Diagnostic: List of diagnostics (errors/warnings)
//   - error: Critical error if parsing cannot proceed
//
// The parser supports flexible file structures:
//   - Files can contain top-level elements (system, container, component) without architecture wrapper
//   - Files can contain an explicit architecture block
//   - Top-level elements are automatically wrapped in an Architecture for compatibility
//
// Example (without architecture):
//
//	parser, _ := language.NewParser()
//	program, diags, err := parser.Parse("api.sruja", `system API "API Service" { ... }`)
//
// Example (with architecture):
//
//	parser, _ := language.NewParser()
//	program, diags, err := parser.Parse("system.sruja", `architecture "My System" { system API "API Service" }`)
func (p *Parser) Parse(filename, text string) (prog *Program, diags []diagnostics.Diagnostic, err error) {
	defer func() {
		if r := recover(); r != nil {
			// Convert panic to diagnostic
			diags = append(diags, diagnostics.Diagnostic{
				Code:     diagnostics.CodeSyntaxError,
				Severity: diagnostics.SeverityError,
				Message:  fmt.Sprintf("Internal parser panic: %v", r),
				Location: diagnostics.SourceLocation{File: filename},
			})
			// Also return error to indicate critical failure
			err = fmt.Errorf("internal parser panic: %v", r)
		}
	}()

	file, err := p.parser.ParseString(filename, text)
	if err != nil {
		// Convert participle error to diagnostics
		diags = p.convertErrorToDiagnostics(err, filename, text)
		return nil, diags, err // Return diagnostics and error
	}

	// Convert File to Program (Logical Model)
	prog = &Program{}

	if file.Change != nil {
		prog.Change = file.Change
		return prog, nil, nil
	}

	arch := file.Architecture
	if arch == nil {
		// If no architecture block, create a default one
		arch = &Architecture{
			Name:  baseName(filename),
			Items: file.Items,
		}
	} else if arch.Name == "" {
		arch.Name = baseName(filename)
	}

	// Populate convenience fields
	arch.PostProcess()
	prog.Architecture = arch

	return prog, nil, nil
}

// ParseFile parses a DSL file into an AST.
//
// This reads the file from disk and parses it. Use this for parsing files,
// or use Parse() if you already have the text in memory.
//
// Parameters:
//   - filename: Path to the .sruja file
//
// Returns:
//   - *Program: The parsed program (root of AST)
//   - []diagnostics.Diagnostic: List of diagnostics
//   - error: Critical error if file cannot be read
//
// Example:
//
//	parser, _ := language.NewParser()
//	program, diags, err := parser.ParseFile("example.sruja")
//	if err != nil {
//	    log.Fatal(err)
//	}
func (p *Parser) ParseFile(filename string) (*Program, []diagnostics.Diagnostic, error) {
	data, err := os.ReadFile(filename) // #nosec G304 // user defined path
	if err != nil {
		return nil, nil, fmt.Errorf("read error: %w", err)
	}
	return p.Parse(filename, string(data))
}

func (p *Parser) convertErrorToDiagnostics(err error, filename, text string) []diagnostics.Diagnostic {
	// Pre-allocate diagnostics slice
	diags := make([]diagnostics.Diagnostic, 0, 1)

	// Try to cast to participle.Error
	var parseErr participle.Error
	if errors.As(err, &parseErr) {
		pos := parseErr.Position()
		msg := parseErr.Message()

		// Extract context lines (error line + surrounding lines for better context)
		lineIdx := pos.Line - 1
		var context []string

		// Find line boundaries
		lines := strings.Split(text, "\n")
		if lineIdx >= 0 && lineIdx < len(lines) {
			// Add previous line for context (if available)
			if lineIdx > 0 {
				context = append(context, lines[lineIdx-1])
			}
			// Add error line
			lineContent := lines[lineIdx]
			context = append(context, lineContent)
			// Add next line for context (if available)
			if lineIdx+1 < len(lines) {
				context = append(context, lines[lineIdx+1])
			}
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
				context = append(context, pointer)
			}
		}

		// Generate enhanced error message and suggestions
		enhancedMsg, suggestions := p.enhanceErrorMessage(msg, pos, text, lineIdx)

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
func (p *Parser) enhanceErrorMessage(msg string, pos lexer.Position, text string, lineIdx int) (string, []string) {
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
	if strings.Contains(msg, "unexpected token") {
		if token != "" {
			if hint, ok := knownKeywords[token]; ok {
				enhancedMsg = fmt.Sprintf("Unexpected '%s' at this position. %s", token, hint)
				suggestions = append(suggestions, fmt.Sprintf("Move '%s' to the correct position in the element definition", token))
				suggestions = append(suggestions, "Check the DSL syntax guide for the correct field order")
			} else if token == "{" {
				enhancedMsg = "Unexpected block start '{'. Missing required fields before the block."
				suggestions = append(suggestions, "Ensure you have defined the element ID and label before the block")
				suggestions = append(suggestions, "Example: system MySystem \"My System\" { ... }")
			} else if token == "}" {
				enhancedMsg = "Unexpected block end '}'. Missing required fields inside the block."
				suggestions = append(suggestions, "Check that all required fields are defined inside the block")
			} else {
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
		"relation", "architecture", "metadata", "properties", "style",
		"requirement", "adr", "policy", "flow", "scenario", "library",
		"tags", "status", "description", "technology", "type", "category",
		"enforcement", "decision", "consequences", "context",
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

func baseName(filename string) string {
	// Extract base name without directories; if empty, use "Untitled"
	idx := strings.LastIndex(filename, "/")
	name := filename
	if idx != -1 && idx+1 < len(filename) {
		name = filename[idx+1:]
	}
	if name == "" {
		name = "Untitled"
	}
	// strip extension
	if dot := strings.LastIndex(name, "."); dot > 0 {
		name = name[:dot]
	}
	return name
}

// extractLocation extracts a SourceLocation from a participle lexer position.
//
// This helper converts participle's Position type to our SourceLocation type.
// Used internally when building AST nodes from parsed tokens.
