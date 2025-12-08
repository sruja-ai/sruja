package language

import (
	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// Requirements & ADRs
// ============================================================================

// Requirement represents a requirement declaration.
//
// A requirement describes what the system must do or constraints it must satisfy.
// Types:
//   - "functional": What the system must do
//   - "constraint": Constraints on the system
//   - "performance": Performance requirements
//   - "security": Security requirements
//
// Example DSL:
//
//	requirement R1 functional "Must handle 10k concurrent users"
type Requirement struct {
	Pos         lexer.Position
	ID          string           `parser:"'requirement' @Ident"`
	Type        *string          `parser:"( @Ident )?"`
	Description *string          `parser:"( @String )?"`
	Body        *RequirementBody `parser:"( '{' @@ '}' )?"`

	// Post-processed
	Metadata []*MetaEntry
}

type RequirementBody struct {
	Type        *string        `parser:"( 'type' @String )?"`
	Description *string        `parser:"( 'description' @String )?"`
	Metadata    *MetadataBlock `parser:"@@?"`
}

func (r *Requirement) Location() SourceLocation {
	return SourceLocation{File: r.Pos.Filename, Line: r.Pos.Line, Column: r.Pos.Column, Offset: r.Pos.Offset}
}

// ADR represents an Architecture Decision Record.
//
// An ADR documents an important architectural decision.
//
// Example DSL:
//
//	adr ADR001 "Use microservices architecture for scalability"
type ADR struct {
	Pos   lexer.Position
	ID    string   `parser:"'adr' @Ident"`
	Title *string  `parser:"( @String )?"`
	Body  *ADRBody `parser:"( '{' @@ '}' )?"`
}

type ADRRef struct {
	ID string `parser:"'adr' @Ident"`
}

type ADRBody struct {
	Status       *string `parser:"( 'status' @String )?"`
	Context      *string `parser:"( 'context' @String )?"`
	Decision     *string `parser:"( 'decision' @String )?"`
	Consequences *string `parser:"( 'consequences' @String )?"`
}

func (a *ADR) Location() SourceLocation {
	return SourceLocation{File: a.Pos.Filename, Line: a.Pos.Line, Column: a.Pos.Column, Offset: a.Pos.Offset}
}
