// Package language provides DSL parsing and AST structures.
package language

import (
	"fmt"
	"os"
	"strings"

	"github.com/alecthomas/participle/v2"
	"github.com/alecthomas/participle/v2/lexer"
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
//	program, err := parser.Parse("example.sruja", dslText)
//	if err != nil {
//	    log.Fatal(err)
//	}
//
//	workspace := program.Workspace
type Parser struct {
	parser *participle.Parser[Architecture] // The participle parser instance
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
		{Name: "Number", Pattern: `\d+(?:\.\d+)?`},
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

	parser, err := participle.Build[Architecture](
		participle.Lexer(srujaLexer),
		participle.Unquote("String"),
		participle.Elide("Whitespace"),
		participle.Elide("Comment"),
		participle.UseLookahead(2),
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
//   - text: The DSL text to parse (must start with "architecture {")
//
// Returns:
//   - *Program: The parsed program (root of AST)
//   - error: Parse error if syntax is invalid
//
// Example:
//
//	parser, _ := language.NewParser()
//	program, err := parser.Parse("example.sruja", `architecture "My System" { system API "API Service" }`)
//	if err != nil {
//	    log.Fatal(err)
//	}
func (p *Parser) Parse(filename, text string) (*Program, error) {
	// Allow files without explicit 'architecture' wrapper
	src := strings.TrimSpace(text)
	if !strings.HasPrefix(src, "workspace") && !strings.HasPrefix(src, "architecture") {
		// Wrap bare statements into architecture block
		src = "architecture \"" + baseName(filename) + "\" { " + src + " }"
	} else {
		// Root present: if header name missing between keyword and first '{', inject default
		// Find keyword and first '{'
		// Accept forms: "architecture \"Name\" {" or "workspace \"Name\" {"
		// If no quoted string before '{', add one.
		prefix := "architecture"
		if strings.HasPrefix(src, "workspace") {
			prefix = "workspace"
			// Replace workspace with architecture for compatibility
			src = strings.Replace(src, "workspace", "architecture", 1)
		}
		// index of first '{'
		braceIdx := strings.Index(src, "{")
		if braceIdx > len(prefix) {
			header := strings.TrimSpace(src[len(prefix):braceIdx])
			hasHeaderName := strings.Contains(header, "\"")
			if !hasHeaderName {
				src = prefix + " \"" + baseName(filename) + "\" " + src[braceIdx:]
			}
		}
	}

	arch, err := p.parser.ParseString(filename, src)
	if err != nil {
		return nil, fmt.Errorf("parse error: %w", err)
	}

	// Populate convenience fields
	arch.PostProcess()

	return &Program{Architecture: arch}, nil
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
//   - error: Parse error if syntax is invalid or file cannot be read
//
// Example:
//
//	parser, _ := language.NewParser()
//	program, err := parser.ParseFile("example.sruja")
//	if err != nil {
//	    log.Fatal(err)
//	}
func (p *Parser) ParseFile(filename string) (*Program, error) {
	data, err := os.ReadFile(filename)
	if err != nil {
		return nil, fmt.Errorf("read error: %w", err)
	}
	return p.Parse(filename, string(data))
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
func extractLocation(pos lexer.Position) SourceLocation {
	return SourceLocation{
		File:   pos.Filename,
		Line:   pos.Line,
		Column: pos.Column,
		Offset: pos.Offset,
		Length: 0, // Will be calculated during parsing
	}
}

// trimQuotes removes surrounding quotes from a string literal.
//
// Participle can be configured to unquote strings automatically, but this helper
// is available if manual unquoting is needed.
func trimQuotes(s string) string {
	s = strings.TrimSpace(s)
	if len(s) >= 2 && s[0] == '"' && s[len(s)-1] == '"' {
		return s[1 : len(s)-1]
	}
	return s
}
