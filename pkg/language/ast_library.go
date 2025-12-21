package language

import (
	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// Policy
// ============================================================================

// Policy represents a policy declaration.
//
// A policy is an architectural rule or constraint that must be followed.
// Policies define how the system should behave or what standards it must meet.
//
// Example DSL:
//
//	policy SecurityPolicy "Enforce TLS 1.3 for all external communications"
//	policy DataRetentionPolicy "Retain order data for 7 years for tax compliance" {
//	  category "compliance"
//	  enforcement "required"
//	}
type Policy struct {
	Pos         lexer.Position
	ID          string `parser:"'policy' @Ident"`
	Description string `parser:"@String"`

	// Inline fields (for backward compatibility)
	InlineCategory    *string `parser:"( 'category' @String )?"`
	InlineEnforcement *string `parser:"( 'enforcement' @String )?"`

	Body *PolicyBody `parser:"( '{' @@ '}' )?"`

	// Post-processed (extracted from Body or Inline)
	Category    *string
	Enforcement *string
	Metadata    []*MetaEntry
}

type PolicyBody struct {
	Properties []PolicyProperty `parser:"@@*"`

	// Post-processed (extracted from Body properties)
	Category    *string
	Enforcement *string
	Description *string
	Tags        []string
	Metadata    *MetadataBlock
}

type PolicyProperty struct {
	Category    *string        `parser:"  'category' @String"`
	Enforcement *string        `parser:"| 'enforcement' @String"`
	Description *string        `parser:"| 'description' @String"`
	Tags        []string       `parser:"| 'tags' '[' @String ( ',' @String )* ']'"`
	Metadata    *MetadataBlock `parser:"| @@"`
}

func (p *Policy) Location() SourceLocation {
	return SourceLocation{File: p.Pos.Filename, Line: p.Pos.Line, Column: p.Pos.Column, Offset: p.Pos.Offset}
}
