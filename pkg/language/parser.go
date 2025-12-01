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
//   - text: The DSL text to parse
//
// Returns:
//   - *Program: The parsed program (root of AST)
//   - error: Parse error if syntax is invalid
//
// The parser supports flexible file structures:
//   - Files can contain top-level elements (system, container, component) without architecture wrapper
//   - Files can contain an explicit architecture block
//   - Top-level elements are automatically wrapped in an Architecture for compatibility
//
// Example (without architecture):
//
//	parser, _ := language.NewParser()
//	program, err := parser.Parse("api.sruja", `system API "API Service" { ... }`)
//
// Example (with architecture):
//
//	parser, _ := language.NewParser()
//	program, err := parser.Parse("system.sruja", `architecture "My System" { system API "API Service" }`)
func (p *Parser) Parse(filename, text string) (*Program, error) {
	file, err := p.parser.ParseString(filename, text)
	if err != nil {
		return nil, fmt.Errorf("parse error: %w", err)
	}

	// Convert File to Program (Logical Model)
	// Top-level elements are automatically wrapped in an Architecture for compatibility
	arch := &Architecture{
		Name: baseName(filename), // Default name from filename
	}

	// Merge items from File into Architecture
	for _, item := range file.Items {
		if item.Architecture != nil {
			// Merge explicit architecture block
			if item.Architecture.Name != "" {
				arch.Name = item.Architecture.Name
			}
			if len(item.Architecture.Follows) > 0 {
				arch.Follows = append(arch.Follows, item.Architecture.Follows...)
			}
			// Add items from explicit architecture block
			arch.Items = append(arch.Items, item.Architecture.Items...)
		} else {
			// Convert top-level FileItem (system, container, component, etc.) to ArchitectureItem
			// This allows files to define elements at any level without requiring architecture wrapper
			archItem := convertFileItemToArchitectureItem(item)
			if archItem != nil {
				arch.Items = append(arch.Items, *archItem)
			}
		}
	}

	// Populate convenience fields
	arch.PostProcess()

	return &Program{Architecture: arch}, nil
}

func convertFileItemToArchitectureItem(item FileItem) *ArchitectureItem {
	if item.Import != nil {
		return &ArchitectureItem{Import: item.Import}
	}
	if item.System != nil {
		return &ArchitectureItem{System: item.System}
	}
	if item.Person != nil {
		return &ArchitectureItem{Person: item.Person}
	}
	if item.Relation != nil {
		return &ArchitectureItem{Relation: item.Relation}
	}
	if item.Requirement != nil {
		return &ArchitectureItem{Requirement: item.Requirement}
	}
	if item.Policy != nil {
		return &ArchitectureItem{Policy: item.Policy}
	}
	if item.ADR != nil {
		return &ArchitectureItem{ADR: item.ADR}
	}
	if item.SharedArtifact != nil {
		return &ArchitectureItem{SharedArtifact: item.SharedArtifact}
	}
	if item.Library != nil {
		return &ArchitectureItem{Library: item.Library}
	}
	if item.Metadata != nil {
		return &ArchitectureItem{Metadata: item.Metadata}
	}
	if item.ContractsBlock != nil {
		return &ArchitectureItem{ContractsBlock: item.ContractsBlock}
	}
	if item.ConstraintsBlock != nil {
		return &ArchitectureItem{ConstraintsBlock: item.ConstraintsBlock}
	}
	if item.ConventionsBlock != nil {
		return &ArchitectureItem{ConventionsBlock: item.ConventionsBlock}
	}
	if item.Context != nil {
		return &ArchitectureItem{Context: item.Context}
	}
	if item.Domain != nil {
		return &ArchitectureItem{Domain: item.Domain}
	}
	if item.DeploymentNode != nil {
		return &ArchitectureItem{DeploymentNode: item.DeploymentNode}
	}
	if item.Scenario != nil {
		return &ArchitectureItem{Scenario: item.Scenario}
	}
	if item.Container != nil {
		return &ArchitectureItem{Container: item.Container}
	}
	if item.Component != nil {
		return &ArchitectureItem{Component: item.Component}
	}
	if item.DataStore != nil {
		return &ArchitectureItem{DataStore: item.DataStore}
	}
	if item.Queue != nil {
		return &ArchitectureItem{Queue: item.Queue}
	}
	if item.Properties != nil {
		return &ArchitectureItem{Properties: item.Properties}
	}
	if item.Style != nil {
		return &ArchitectureItem{Style: item.Style}
	}
	if item.Description != nil {
		return &ArchitectureItem{Description: item.Description}
	}

	return nil
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
