// Package language provides DSL parsing and AST structures.
package language

import (
	"fmt"
	"sync"

	"github.com/alecthomas/participle/v2"
	"github.com/alecthomas/participle/v2/lexer"
	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

// Cached parser instance for reuse (thread-safe)
var (
	cachedParser     *Parser
	cachedParserOnce sync.Once
	cachedParserErr  error
)

// DefaultParser returns a cached parser instance (thread-safe).
// This avoids the overhead of rebuilding the parser for each parse operation.
// The parser is built once and reused for all subsequent calls.
//
// Panics if parser initialization fails, which should never happen in production
// as the parser grammar is statically defined. If this panics, it indicates a
// critical bug in the parser definition.
//
// Example:
//
//	parser := language.DefaultParser()
//	program, diags, err := parser.Parse("example.sruja", dslText)
func DefaultParser() *Parser {
	cachedParserOnce.Do(func() {
		cachedParser, cachedParserErr = NewParser()
	})
	if cachedParserErr != nil {
		// Parser initialization failure indicates a critical bug in the grammar definition.
		// This should never happen in production, but we panic with a clear error message
		// to aid debugging during development.
		panic(fmt.Sprintf("failed to initialize parser (this indicates a bug in parser grammar): %v", cachedParserErr))
	}
	return cachedParser
}

// Parser parses Sruja DSL text into an AST (Abstract Syntax Tree).
type Parser struct {
	parser *participle.Parser[File] // The participle parser instance
}

// NewParser creates a new parser instance.
func NewParser() (*Parser, error) {
	// Define lexer for Sruja DSL (LikeC4 compatible)
	// This tokenizes the input into keywords, strings, operators, etc.
	srujaLexer := lexer.MustSimple([]lexer.SimpleRule{
		{Name: "Comment", Pattern: `//.*|/\*.*?\*/`},
		// LikeC4: Support both double and single quoted strings
		{Name: "String", Pattern: `"(\\"|[^"])*"|'(\\'|[^'])*'`},
		{Name: "Int", Pattern: `\d+`},
		{Name: "Number", Pattern: `\d+(?:\.\d+)?`},
		// LikeC4: Tag references like #deprecated
		{Name: "TagRef", Pattern: `#[a-zA-Z_][a-zA-Z0-9_-]*`},
		{Name: "Story", Pattern: `\bstory\b`},
		{Name: "Scenario", Pattern: `\bscenario\b`},
		{Name: "Flow", Pattern: `\bflow\b`},
		{Name: "Policy", Pattern: `\bpolicy\b`},
		{Name: "Import", Pattern: `\bimport\b`},
		{Name: "From", Pattern: `\bfrom\b`},
		{Name: "Wildcard", Pattern: `\*`}, // For view expressions: include *
		{Name: "Ident", Pattern: `[a-zA-Z_][a-zA-Z0-9_-]*`},
		{Name: "Dot", Pattern: `\.`},
		// LikeC4: Support bidirectional and back arrows
		{Name: "BiArrow", Pattern: `<->`},
		{Name: "BackArrow", Pattern: `<-`},
		{Name: "Arrow", Pattern: `->`},
		{Name: "Assign", Pattern: `=`},
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
		participle.UseLookahead(5), // Increased to handle ambiguity
	)
	if err != nil {
		return nil, fmt.Errorf("failed to build parser: %w", err)
	}

	return &Parser{parser: parser}, nil
}

// Parse parses DSL text into an AST.
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

	// Collect items from TopLevelItems
	for _, item := range file.TopLevelItems {
		// LikeC4 format
		if item.Specification != nil {
			prog.Specification = item.Specification
		}
		if item.Model != nil {
			prog.Model = item.Model
		}
		if item.Views != nil {
			prog.Views = item.Views
		}
	}
	// If LikeC4 format was used, return it directly (no conversion needed)
	if prog.Specification != nil || prog.Model != nil || prog.Views != nil {
		prog.PostProcess()
		return prog, nil, nil
	}

	// No valid TopLevelItems found - return empty program
	return prog, nil, nil
}
