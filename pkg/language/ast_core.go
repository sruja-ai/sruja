package language

import (
	"strconv"

	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// Shared Base Types
// ============================================================================

type MetaEntry struct {
	Pos   lexer.Position
	Key   string   `parser:"@Ident"`
	Value *string  `parser:"( @String"`
	Array []string `parser:"| '[' @String ( ',' @String )* ']' )"`
}

func (m *MetaEntry) Location() SourceLocation {
	return SourceLocation{File: m.Pos.Filename, Line: m.Pos.Line, Column: m.Pos.Column, Offset: m.Pos.Offset}
}

type SourceLocation struct {
	File   string
	Line   int
	Column int
	Offset int
	Length int
}

func (l SourceLocation) String() string {
	return l.File + ":" + strconv.Itoa(l.Line) + ":" + strconv.Itoa(l.Column)
}

type ASTNode interface {
	Location() SourceLocation
}

type MetadataBlock struct {
	Pos     lexer.Position
	LBrace  string       `parser:"'metadata' '{'"`
	Entries []*MetaEntry `parser:"@@*"`
	RBrace  string       `parser:"'}'"`
}

func (m *MetadataBlock) Location() SourceLocation {
	return SourceLocation{File: m.Pos.Filename, Line: m.Pos.Line, Column: m.Pos.Column, Offset: m.Pos.Offset}
}

type PropertiesBlock struct {
	Pos     lexer.Position
	LBrace  string           `parser:"'properties' '{'"`
	Entries []*PropertyEntry `parser:"@@*"`
	RBrace  string           `parser:"'}'"`
}

type PropertyEntry struct {
	Key   string `parser:"( @Ident | @String )"`
	Sep   string `parser:"( ':' )?"`
	Value string `parser:"@String"`
}

func (p *PropertiesBlock) Location() SourceLocation {
	return SourceLocation{File: p.Pos.Filename, Line: p.Pos.Line, Column: p.Pos.Column, Offset: p.Pos.Offset}
}

type StyleDecl struct {
	Pos     lexer.Position
	Keyword string      `parser:"@( 'style' | 'styles' )"`
	Body    *StyleBlock `parser:"@@"`
}

type StyleBlock struct {
	Pos     lexer.Position
	LBrace  string        `parser:"'{'"`
	Entries []*StyleEntry `parser:"@@*"`
	RBrace  string        `parser:"'}'"`
}

type StyleEntry struct {
	Key   string      `parser:"@( Ident | 'element' | 'person' | 'system' | 'container' | 'component' | 'database' | 'queue' | 'style' | 'styles' | TagRef | ( '@' Ident ) )"`
	Value *string     `parser:"( @String | @Ident | @Number | @( 'true' | 'false' ) | @TagRef )?"`
	Body  *StyleBlock `parser:"( @@ )?"`
}

func (s *StyleDecl) Location() SourceLocation {
	return SourceLocation{File: s.Pos.Filename, Line: s.Pos.Line, Column: s.Pos.Column, Offset: s.Pos.Offset}
}

func (s *StyleBlock) Location() SourceLocation {
	return SourceLocation{File: s.Pos.Filename, Line: s.Pos.Line, Column: s.Pos.Column, Offset: s.Pos.Offset}
}

type OverviewBlock struct {
	Pos     lexer.Position
	LBrace  string       `parser:"'overview' '{'"`
	Entries []*MetaEntry `parser:"@@*"`
	RBrace  string       `parser:"'}'"`

	Summary  *string
	Audience *string
	Scope    *string
	Goals    []string
	NonGoals []string
	Risks    []string
}

func (o *OverviewBlock) Location() SourceLocation {
	return SourceLocation{File: o.Pos.Filename, Line: o.Pos.Line, Column: o.Pos.Column, Offset: o.Pos.Offset}
}

func (o *OverviewBlock) PostProcess() {
	for _, entry := range o.Entries {
		switch entry.Key {
		case "summary":
			o.Summary = entry.Value
		case "audience":
			o.Audience = entry.Value
		case "scope":
			o.Scope = entry.Value
		case "goals":
			o.Goals = entry.Array
		case "nonGoals":
			o.NonGoals = entry.Array
		case "risks":
			o.Risks = entry.Array
		}
	}
}

type ScaleBlock struct {
	Pos    lexer.Position
	LBrace string       `parser:"'scale' '{'"`
	Items  []*ScaleItem `parser:"@@*"`
	RBrace string       `parser:"'}'"`

	Min    *int
	Max    *int
	Metric *string
}

type ScaleItem struct {
	Min    *ScaleMin    `parser:"@@"`
	Max    *ScaleMax    `parser:"| @@"`
	Metric *ScaleMetric `parser:"| @@"`
}

type ScaleMin struct {
	Val int `parser:"'min' @Number"`
}

type ScaleMax struct {
	Val int `parser:"'max' @Number"`
}

type ScaleMetric struct {
	Val string `parser:"'metric' @String"`
}

func (s *ScaleBlock) Location() SourceLocation {
	return SourceLocation{File: s.Pos.Filename, Line: s.Pos.Line, Column: s.Pos.Column, Offset: s.Pos.Offset}
}

// ============================================================================
// SLO (Service Level Objectives)
// ============================================================================

type SLOBlock struct {
	Pos    lexer.Position
	LBrace string     `parser:"'slo' '{'"`
	Items  []*SLOItem `parser:"@@*"`
	RBrace string     `parser:"'}'"`

	Availability *SLOAvailability
	Latency      *SLOLatency
	ErrorRate    *SLOErrorRate
	Throughput   *SLOThroughput
	Cost         *SLOCost
}

type SLOItem struct {
	Availability *SLOAvailability `parser:"@@"`
	Latency      *SLOLatency      `parser:"| @@"`
	ErrorRate    *SLOErrorRate    `parser:"| @@"`
	Throughput   *SLOThroughput   `parser:"| @@"`
	Cost         *SLOCost         `parser:"| @@"`
}

type SLOCost struct {
	LBrace string  `parser:"'cost' '{'"`
	Target *string `parser:"( 'target' @String )?"`
	Window *string `parser:"( 'window' @String )?"`
	RBrace string  `parser:"'}'"`
}

func (s *SLOBlock) Location() SourceLocation {
	return SourceLocation{File: s.Pos.Filename, Line: s.Pos.Line, Column: s.Pos.Column, Offset: s.Pos.Offset}
}

type SLOAvailability struct {
	LBrace  string  `parser:"'availability' '{'"`
	Target  *string `parser:"( 'target' @String )?"`
	Window  *string `parser:"( 'window' @String )?"`
	Current *string `parser:"( 'current' @String )?"`
	RBrace  string  `parser:"'}'"`
}

type SLOLatency struct {
	LBrace  string      `parser:"'latency' '{'"`
	P95     *string     `parser:"( 'p95' @String )?"`
	P99     *string     `parser:"( 'p99' @String )?"`
	Window  *string     `parser:"( 'window' @String )?"`
	Current *SLOCurrent `parser:"( 'current' '{' @@ '}' )?"`
	RBrace  string      `parser:"'}'"`
}

type SLOCurrent struct {
	P95 *string `parser:"( 'p95' @String )?"`
	P99 *string `parser:"( 'p99' @String )?"`
}

type SLOErrorRate struct {
	LBrace  string  `parser:"'errorRate' '{'"`
	Target  *string `parser:"( 'target' @String )?"`
	Window  *string `parser:"( 'window' @String )?"`
	Current *string `parser:"( 'current' @String )?"`
	RBrace  string  `parser:"'}'"`
}

type SLOThroughput struct {
	LBrace  string  `parser:"'throughput' '{'"`
	Target  *string `parser:"( 'target' @String )?"`
	Window  *string `parser:"( 'window' @String )?"`
	Current *string `parser:"( 'current' @String )?"`
	RBrace  string  `parser:"'}'"`
}

// ============================================================================
// Constraints, Conventions
// ============================================================================

type ConstraintsBlock struct {
	Pos     lexer.Position
	LBrace  string             `parser:"'constraints' '{'"`
	Entries []*ConstraintEntry `parser:"@@*"`
	RBrace  string             `parser:"'}'"`
}

type ConstraintEntry struct {
	Pos   lexer.Position
	Key   string `parser:"( @Ident )?"`
	Value string `parser:"@String"`
}

func (c *ConstraintEntry) Location() SourceLocation {
	return SourceLocation{File: c.Pos.Filename, Line: c.Pos.Line, Column: c.Pos.Column, Offset: c.Pos.Offset}
}

func (c *ConstraintsBlock) Location() SourceLocation {
	return SourceLocation{File: c.Pos.Filename, Line: c.Pos.Line, Column: c.Pos.Column, Offset: c.Pos.Offset}
}

type ConventionsBlock struct {
	Pos     lexer.Position
	LBrace  string             `parser:"'conventions' '{'"`
	Entries []*ConventionEntry `parser:"@@*"`
	RBrace  string             `parser:"'}'"`
}

type ConventionEntry struct {
	Pos   lexer.Position
	Key   string `parser:"( @Ident )?"`
	Value string `parser:"@String"`
}

func (c *ConventionEntry) Location() SourceLocation {
	return SourceLocation{File: c.Pos.Filename, Line: c.Pos.Line, Column: c.Pos.Column, Offset: c.Pos.Offset}
}

func (c *ConventionsBlock) Location() SourceLocation {
	return SourceLocation{File: c.Pos.Filename, Line: c.Pos.Line, Column: c.Pos.Column, Offset: c.Pos.Offset}
}

type DeprecationBlock struct {
	Reason      *string `parser:"( 'reason' @String )?"`
	Sunset      *string `parser:"( 'sunset' @String )?"`
	Replacement *string `parser:"( 'replacement' @String )?"`
}

type CompatibilityBlock struct {
	BackwardsWith     *string `parser:"( 'backwards_with' @String )?"`
	ForwardsWith      *string `parser:"( 'forwards_with' @String )?"`
	BreakingChange    *string `parser:"( 'breaking_change' @String )?"`
	RequiresMigration *string `parser:"( 'requires_migration' @String )?"`
	Deprecates        *string `parser:"( 'deprecates' @String )?"`
}

type GuaranteesBlock struct {
	Entries []*GuaranteeEntry `parser:"@@*"`
}

type GuaranteeEntry struct {
	Key   string `parser:"@Ident"`
	Value string `parser:"@String"`
}

type BehaviorBlock struct {
	Entries []*BehaviorEntry `parser:"@@*"`
}

type BehaviorEntry struct {
	Key   string `parser:"@Ident"`
	Value string `parser:"@String"`
}

// ============================================================================
// Metadata Helper Methods
// ============================================================================

func metaString(metadata []*MetaEntry, key string) (string, bool) {
	for _, meta := range metadata {
		if meta.Key == key {
			if meta.Value != nil {
				return *meta.Value, true
			}
		}
	}
	return "", false
}

func metaAll(metadata []*MetaEntry) map[string]string {
	result := make(map[string]string, len(metadata))
	for _, meta := range metadata {
		if meta.Value != nil {
			result[meta.Key] = *meta.Value
		}
	}
	return result
}

func metaMap(metadata []*MetaEntry, prefix string) map[string]string {
	result := make(map[string]string)
	for _, meta := range metadata {
		if len(meta.Key) >= len(prefix) && meta.Key[:len(prefix)] == prefix {
			if meta.Value != nil {
				result[meta.Key] = *meta.Value
			}
		}
	}
	return result
}
