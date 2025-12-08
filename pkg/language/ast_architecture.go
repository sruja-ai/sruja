package language

import (
	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// File (Physical Root)
// ============================================================================

// File represents the physical structure of a parsed Sruja DSL file.
//
// It contains a list of items that can appear at the top level.
// Files can contain elements at any level (system, container, component) without
// requiring an architecture wrapper. These elements can be imported and reused
// in other Sruja files.
//
// Example DSL (without architecture block):
//
//	system API "API Service" { ... }
//	container WebApp "Web Application" { ... }
//
// Example DSL (with architecture block):
//
//	architecture "My System" {
//	  system API "API Service" { ... }
//	}
type File struct {
	Pos          lexer.Position
	Architecture *Architecture      `parser:"@@"`
	Change       *ChangeBlock       `parser:"| @@"`
	Items        []ArchitectureItem `parser:"| @@*"`
}

func (f *File) Location() SourceLocation {
	return SourceLocation{File: f.Pos.Filename, Line: f.Pos.Line, Column: f.Pos.Column, Offset: f.Pos.Offset}
}

// Program represents the complete parsed program
type Program struct {
	Architecture *Architecture
	Change       *ChangeBlock
}

// ============================================================================
// Architecture (Top-Level)
// ============================================================================

// Architecture is the top-level root of a document.
//
// An architecture contains all the elements that make up the system architecture:
// systems, persons, relations, requirements, ADRs, shared artifacts, libraries, and scenarios.
//
// Example DSL:
//
//	architecture "My System" {
//	  import "shared.sruja"
//	  system API "API Service" { ... }
//	  person User "End User"
//	}
type Architecture struct {
	Pos    lexer.Position
	Name   string             `parser:"'architecture' @String"`
	LBrace string             `parser:"'{'"`
	Items  []ArchitectureItem `parser:"@@*"`
	RBrace string             `parser:"'}'"`

	// Post-processed fields
	// Post-processed fields
	Imports         []*ImportSpec
	ResolvedImports []*ImportedArchitecture
	Systems         []*System
	Containers      []*Container
	Components      []*Component
	DataStores      []*DataStore
	Queues          []*Queue
	Persons         []*Person
	Relations       []*Relation
	Requirements    []*Requirement
	ADRs            []*ADR
	SharedArtifacts []*SharedArtifact
	Libraries       []*Library
	Metadata        []*MetaEntry // Metadata from metadata blocks
	Contracts       []*Contract
	Constraints     []*ConstraintEntry
	Conventions     []*ConventionEntry
	DeploymentNodes []*DeploymentNode
	Scenarios       []*Scenario
	Policies        []*Policy
	Flows           []*Flow
	Views           *ViewBlock // Optional views block for customization
	Properties      map[string]string
	Style           map[string]string
	Description     *string
}

func (a *Architecture) Location() SourceLocation {
	return SourceLocation{File: a.Pos.Filename, Line: a.Pos.Line, Column: a.Pos.Column, Offset: a.Pos.Offset}
}

// ArchitectureItem is a union type for items that can appear at the architecture level.
type ArchitectureItem struct {
	Import           *ImportSpec       `parser:"@@"`
	Container        *Container        `parser:"| @@"`
	System           *System           `parser:"| @@"`
	Component        *Component        `parser:"| @@"`
	DataStore        *DataStore        `parser:"| @@"`
	Queue            *Queue            `parser:"| @@"`
	Person           *Person           `parser:"| @@"`
	Relation         *Relation         `parser:"| @@"`
	Requirement      *Requirement      `parser:"| @@"`
	ADR              *ADR              `parser:"| @@"`
	SharedArtifact   *SharedArtifact   `parser:"| @@"`
	Library          *Library          `parser:"| @@"`
	Metadata         *MetadataBlock    `parser:"| @@"`
	ContractsBlock   *ContractsBlock   `parser:"| 'contracts' '{' @@ '}'"`
	ConstraintsBlock *ConstraintsBlock `parser:"| 'constraints' '{' @@ '}'"`
	ConventionsBlock *ConventionsBlock `parser:"| 'conventions' '{' @@ '}'"`
	DeploymentNode   *DeploymentNode   `parser:"| @@"`
	Scenario         *Scenario         `parser:"| @@"`
	Policy           *Policy           `parser:"| @@"`
	Flow             *Flow             `parser:"| @@"`
	Views            *ViewBlock        `parser:"| @@"`
	Properties       *PropertiesBlock  `parser:"| @@"`
	Style            *StyleBlock       `parser:"| @@"`
	Description      *string           `parser:"| 'description' @String"`
}

// ============================================================================
// Imports & Multi-Architecture Composition
// ============================================================================

// ImportSpec represents an import statement.
//
// Example DSL:
//
//	import "shared.sruja"
//	import "common.sruja" as common
type ImportSpec struct {
	Pos   lexer.Position
	Path  string  `parser:"'import' @String"`
	Alias *string `parser:"( 'as' @Ident )?"`
}

func (i *ImportSpec) Location() SourceLocation {
	return SourceLocation{File: i.Pos.Filename, Line: i.Pos.Line, Column: i.Pos.Column, Offset: i.Pos.Offset}
}

// ImportedArchitecture represents a resolved imported architecture.
type ImportedArchitecture struct {
	Alias        string
	Architecture *Architecture
}

// ============================================================================
// Change Block
// ============================================================================

// ChangeBlock represents a change block
type ChangeBlock struct {
	Pos         lexer.Position
	ID          string             `parser:"'change' @String '{'"`
	Status      *string            `parser:"( 'status' @String )?"`
	Version     *string            `parser:"( 'version' @String )?"`
	Requirement *string            `parser:"( 'requirement' @String )?"`
	ADR         *string            `parser:"( 'adr' @String )?"`
	Metadata    *MetadataBlock     `parser:"( @@ )?"`
	Add         *ArchitectureBlock `parser:"( 'add' '{' @@ '}' )?"`
	Modify      *ArchitectureBlock `parser:"( 'modify' '{' @@ '}' )?"`
	Remove      *ArchitectureBlock `parser:"( 'remove' '{' @@ '}' )?"`
	RBrace      string             `parser:"'}'"`
}

func (c *ChangeBlock) Location() SourceLocation {
	return SourceLocation{File: c.Pos.Filename, Line: c.Pos.Line, Column: c.Pos.Column, Offset: c.Pos.Offset}
}

// ArchitectureBlock contains architecture elements (for add/modify/remove)
type ArchitectureBlock struct {
	Items []ArchitectureItem `parser:"@@*"`
}

// ============================================================================
// Snapshot Block
// ============================================================================

// SnapshotBlock represents a snapshot block
type SnapshotBlock struct {
	Pos          lexer.Position
	Name         string        `parser:"'snapshot' @String '{'"`
	Version      *string       `parser:"( 'version' @String )?"`
	Description  *string       `parser:"( 'description' @String )?"`
	Timestamp    *string       `parser:"( 'timestamp' @String )?"`
	Preview      *bool         `parser:"( 'preview' @('true'|'false') )?"`
	Changes      []string      `parser:"( 'changes' '[' @String ( ',' @String )* ']' )?"`
	ArchName     *string       `parser:"( 'architecture' @String )?"`
	Architecture *Architecture `parser:"( '{' @@ '}' )?"`
	RBrace       string        `parser:"'}'"`
}

func (s *SnapshotBlock) Location() SourceLocation {
	return SourceLocation{File: s.Pos.Filename, Line: s.Pos.Line, Column: s.Pos.Column, Offset: s.Pos.Offset}
}

// MetaString returns a metadata value as a string.
func (a *Architecture) MetaString(key string) (string, bool) {
	return metaString(a.Metadata, key)
}

// HasMeta checks if a metadata key exists.
func (a *Architecture) HasMeta(key string) bool {
	_, ok := a.MetaString(key)
	return ok
}

// AllMetadata returns all metadata as a map.
func (a *Architecture) AllMetadata() map[string]string {
	return metaAll(a.Metadata)
}
