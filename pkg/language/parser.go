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
	var diags []diagnostics.Diagnostic

	// Try to cast to participle.Error
	var parseErr participle.Error
	if errors.As(err, &parseErr) {
		pos := parseErr.Position()
		msg := parseErr.Message()

		// Extract context line efficiently
		lineIdx := pos.Line - 1
		var context []string

		// Find the start and end of the target line
		start := 0
		currentLine := 0
		for i := 0; i < len(text); i++ {
			if currentLine == lineIdx {
				start = i
				break
			}
			if text[i] == '\n' {
				currentLine++
			}
		}

		if currentLine == lineIdx {
			end := start
			for i := start; i < len(text); i++ {
				if text[i] == '\n' {
					break
				}
				end++
			}

			lineContent := text[start:end]
			context = append(context, lineContent)
			// Add pointer line
			pointer := strings.Repeat(" ", pos.Column-1) + "^"
			context = append(context, pointer)
		}

		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeSyntaxError,
			Severity: diagnostics.SeverityError,
			Message:  msg,
			Location: diagnostics.SourceLocation{
				File:   filename,
				Line:   pos.Line,
				Column: pos.Column,
			},
			Context: context,
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
