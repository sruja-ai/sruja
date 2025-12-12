package language

import (
	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// Shared Artifacts & Libraries
// ============================================================================

// SharedArtifact represents a shared artifact declaration.
//
// A shared artifact is a reusable component or service shared across systems.
//
// Example DSL:
//
//	sharedArtifact Auth "Authentication Service" version "1.0.0" owner "platform-team"
type SharedArtifact struct {
	Pos     lexer.Position
	ID      string                `parser:"'sharedArtifact' ( @Ident | @String )"`
	Label   string                `parser:"( @String )?"`
	Version *string               `parser:"( 'version' @String )?"`
	Owner   *string               `parser:"( 'owner' @String )?"`
	Items   []*SharedArtifactItem `parser:"( '{' @@* '}' )?"`

	// Post-processed
	Description *string
	URL         *string
}

type SharedArtifactItem struct {
	Description *string `parser:"'description' @String"`
	Version     *string `parser:"| 'version' @String"`
	URL         *string `parser:"| 'url' @String"`
}

func (a *SharedArtifact) Location() SourceLocation {
	return SourceLocation{File: a.Pos.Filename, Line: a.Pos.Line, Column: a.Pos.Column, Offset: a.Pos.Offset}
}

// Library represents a library declaration.
//
// A library is a reusable code library or framework.
//
// Example DSL:
//
//	library React "React Framework" version "18.0.0" owner "facebook"
type Library struct {
	Pos     lexer.Position
	ID      string         `parser:"'library' @Ident"`
	Label   string         `parser:"( @String )?"`
	Version *string        `parser:"( 'version' @String )?"`
	Owner   *string        `parser:"( 'owner' @String )?"`
	LBrace  string         `parser:"( '{'"`
	Items   []*LibraryItem `parser:"@@*"`
	RBrace  string         `parser:"'}' )?"`

	// Post-processed
	Description  *string
	Policies     []*Policy
	Requirements []*Requirement
	Metadata     []*MetaEntry
}

func (l *Library) Location() SourceLocation {
	return SourceLocation{File: l.Pos.Filename, Line: l.Pos.Line, Column: l.Pos.Column, Offset: l.Pos.Offset}
}

type LibraryItem struct {
	Description    *string         `parser:"'description' @String"`
	SharedArtifact *SharedArtifact `parser:"| @@"`
	Metadata       *MetadataBlock  `parser:"| @@"`
	Policy         *Policy         `parser:"| @@"`
	Requirement    *Requirement    `parser:"| @@"`
}

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
	Tags        []string       `parser:"| 'tags' @String ( ',' @String )*"`
	Metadata    *MetadataBlock `parser:"| @@"`
}

func (p *Policy) Location() SourceLocation {
	return SourceLocation{File: p.Pos.Filename, Line: p.Pos.Line, Column: p.Pos.Column, Offset: p.Pos.Offset}
}
