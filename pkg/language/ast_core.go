package language

import (
	"fmt"
	"strings"

	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// Shared Base Types
// ============================================================================

// MetaEntry represents a single metadata key-value pair.
//
// Metadata is freeform key-value pairs that allow infinite extension
// without modifying the core DSL grammar. Plugins can consume metadata
// for validation, code generation, diagram customization, etc.
//
// Example DSL:
//
//	metadata {
//	    team: "Payments"
//	    tier: "critical"
//	    rate_limit: "100/s"
//	    tags: ["backend", "critical"]
//	}
type MetaEntry struct {
	Pos   lexer.Position
	Key   string   `parser:"@Ident"`
	Value *string  `parser:"( @String"`
	Array []string `parser:"| '[' @String ( ',' @String )* ']' )"`
}

func (m *MetaEntry) Location() SourceLocation {
	return SourceLocation{File: m.Pos.Filename, Line: m.Pos.Line, Column: m.Pos.Column, Offset: m.Pos.Offset}
}

// SourceLocation represents the position of a node in the source file.
//
// This is used for:
//   - Error reporting (showing where errors occurred)
//   - IDE features (go-to-definition, hover, etc.)
//   - Validation (reporting which element has issues)
//
// All positions are 1-based (line 1, column 1 is the first character).
type SourceLocation struct {
	File   string // File path where this node appears
	Line   int    // 1-based line number
	Column int    // 1-based column number
	Offset int    // Byte offset in file (0-based)
	Length int    // Length of the node in bytes
}

// String returns a human-readable location string in the format "file:line:column".
//
// Example: "example.sruja:5:12"
func (l SourceLocation) String() string {
	// Build string efficiently using strings.Builder
	var sb strings.Builder
	sb.Grow(len(l.File) + 20) // Estimate: file name + ":line:column"
	sb.WriteString(l.File)
	sb.WriteString(":")
	sb.WriteString(fmt.Sprintf("%d", l.Line))
	sb.WriteString(":")
	sb.WriteString(fmt.Sprintf("%d", l.Column))
	return sb.String()
}

// ASTNode is the base interface for all AST nodes.
//
// Every node in the AST implements this interface, which provides:
//   - Source location information (for error reporting)
//
// All concrete AST types (Architecture, System, Container, etc.) implement this interface.
type ASTNode interface {
	Location() SourceLocation
}

// MetadataBlock represents a metadata block.
//
// Example DSL:
//
//	metadata {
//	    team: "Payments"
//	    tier: "critical"
//	}
type MetadataBlock struct {
	Pos     lexer.Position
	LBrace  string       `parser:"'metadata' '{'"`
	Entries []*MetaEntry `parser:"@@*"`
	RBrace  string       `parser:"'}'"`
}

func (m *MetadataBlock) Location() SourceLocation {
	return SourceLocation{File: m.Pos.Filename, Line: m.Pos.Line, Column: m.Pos.Column, Offset: m.Pos.Offset}
}

// PropertiesBlock represents a generic key-value properties block.
//
// Example DSL:
//
//	properties {
//	    "qps": "1000"
//	    "latency": "50ms"
//	}
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

// StyleBlock represents a style block for customizing appearance.
//
// Example DSL:
//
//	style {
//	    shape: "cylinder"
//	    color: "#ff0000"
//	}
type StyleBlock struct {
	Pos     lexer.Position
	LBrace  string        `parser:"'style' '{'"`
	Entries []*StyleEntry `parser:"@@*"`
	RBrace  string        `parser:"'}'"`
}

type StyleEntry struct {
	Key   string `parser:"@Ident"`
	Sep   string `parser:"( ':' )?"`
	Value string `parser:"@String"`
}

func (s *StyleBlock) Location() SourceLocation {
	return SourceLocation{File: s.Pos.Filename, Line: s.Pos.Line, Column: s.Pos.Column, Offset: s.Pos.Offset}
}

// OverviewBlock represents an architecture overview block for high-level introduction.
//
// This provides a structured way to document architecture-level information
// for reviewers and stakeholders. Uses the same MetaEntry pattern as metadata blocks.
//
// Example DSL:
//
//	overview {
//	    summary "High-performance e-commerce platform supporting 10M+ users"
//	    audience "Engineering teams, architects, stakeholders"
//	    scope "Covers ordering, payments, inventory, and analytics"
//	    goals ["Scale to 50M users", "Sub-200ms p95 latency", "99.99% uptime"]
//	    nonGoals ["Real-time inventory sync", "Multi-currency support"]
//	    risks ["Payment Gateway SPOF", "Database bottleneck"]
//	}
type OverviewBlock struct {
	Pos     lexer.Position
	LBrace  string       `parser:"'overview' '{'"`
	Entries []*MetaEntry `parser:"@@*"`
	RBrace  string       `parser:"'}'"`

	// Post-processed convenience fields
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

// PostProcess extracts well-known keys from MetaEntry items to convenience fields
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

// ScaleBlock represents a scalability block.

// Example DSL:
//
//	scale {
//	    min 3
//	    max 10
//	    metric "cpu > 80%"
//	}
type ScaleBlock struct {
	Pos    lexer.Position
	LBrace string       `parser:"'scale' '{'"`
	Items  []*ScaleItem `parser:"@@*"`
	RBrace string       `parser:"'}'"`

	// Post-processed
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
	Val int `parser:"'min' @Int"`
}

type ScaleMax struct {
	Val int `parser:"'max' @Int"`
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

// SLOBlock represents a Service Level Objectives block.
//
// Example DSL:
//
//	slo {
//	    availability {
//	        target "99.9%"
//	        window "30 days"
//	        current "99.95%"
//	    }
//	    latency {
//	        p95 "200ms"
//	        p99 "500ms"
//	    }
//	}
type SLOBlock struct {
	Pos    lexer.Position
	LBrace string     `parser:"'slo' '{'"`
	Items  []*SLOItem `parser:"@@*"`
	RBrace string     `parser:"'}'"`

	// Post-processed
	Availability *SLOAvailability
	Latency      *SLOLatency
	ErrorRate    *SLOErrorRate
	Throughput   *SLOThroughput
}

type SLOItem struct {
	Availability *SLOAvailability `parser:"@@"`
	Latency      *SLOLatency      `parser:"| @@"`
	ErrorRate    *SLOErrorRate    `parser:"| @@"`
	Throughput   *SLOThroughput   `parser:"| @@"`
}

func (s *SLOBlock) Location() SourceLocation {
	return SourceLocation{File: s.Pos.Filename, Line: s.Pos.Line, Column: s.Pos.Column, Offset: s.Pos.Offset}
}

// SLOAvailability represents availability SLO configuration.
type SLOAvailability struct {
	LBrace  string  `parser:"'availability' '{'"`
	Target  *string `parser:"( 'target' @String )?"`
	Window  *string `parser:"( 'window' @String )?"`
	Current *string `parser:"( 'current' @String )?"`
	RBrace  string  `parser:"'}'"`
}

// SLOLatency represents latency SLO configuration.
type SLOLatency struct {
	LBrace  string      `parser:"'latency' '{'"`
	P95     *string     `parser:"( 'p95' @String )?"`
	P99     *string     `parser:"( 'p99' @String )?"`
	Window  *string     `parser:"( 'window' @String )?"`
	Current *SLOCurrent `parser:"( 'current' '{' @@ '}' )?"`
	RBrace  string      `parser:"'}'"`
}

// SLOCurrent represents current latency metrics.
type SLOCurrent struct {
	P95 *string `parser:"( 'p95' @String )?"`
	P99 *string `parser:"( 'p99' @String )?"`
}

// SLOErrorRate represents error rate SLO configuration.
type SLOErrorRate struct {
	LBrace  string  `parser:"'errorRate' '{'"`
	Target  *string `parser:"( 'target' @String )?"`
	Window  *string `parser:"( 'window' @String )?"`
	Current *string `parser:"( 'current' @String )?"`
	RBrace  string  `parser:"'}'"`
}

// SLOThroughput represents throughput SLO configuration.
type SLOThroughput struct {
	LBrace  string  `parser:"'throughput' '{'"`
	Target  *string `parser:"( 'target' @String )?"`
	Window  *string `parser:"( 'window' @String )?"`
	Current *string `parser:"( 'current' @String )?"`
	RBrace  string  `parser:"'}'"`
}

// ============================================================================
// Contracts, Constraints, Conventions
// ============================================================================

type ContractsBlock struct {
	Contracts []*Contract `parser:"@@*"`
}

type Contract struct {
	Pos  lexer.Position
	Kind string        `parser:"@( 'api' | 'event' | 'data' )"`
	ID   string        `parser:"@Ident"`
	L    string        `parser:"'{'"`
	Body *ContractBody `parser:"@@"`
	R    string        `parser:"'}'"`
}

func (c *Contract) Location() SourceLocation {
	return SourceLocation{File: c.Pos.Filename, Line: c.Pos.Line, Column: c.Pos.Column, Offset: c.Pos.Offset}
}

type ContractBody struct {
	Version       *string             `parser:"( 'version' @String )?"`
	Status        *string             `parser:"( 'status' @String )?"`
	Endpoint      *string             `parser:"( 'endpoint' @String )?"`
	Method        *string             `parser:"( 'method' @String )?"`
	Request       *SchemaBlock        `parser:"( 'request' '{' @@ '}' )?"`
	Response      *SchemaBlock        `parser:"( 'response' '{' @@ '}' )?"`
	Errors        []string            `parser:"( 'errors' '[' @String ( ',' @String )* ']' )?"`
	Schema        *SchemaBlock        `parser:"( 'schema' '{' @@ '}' )?"`
	Retention     *string             `parser:"( 'retention' @String )?"`
	RequestMap    *string             `parser:"( 'request_map' @String )?"`
	ResponseMap   *string             `parser:"( 'response_map' @String )?"`
	ErrorMap      []string            `parser:"( 'error_map' '[' @String ( ',' @String )* ']' )?"`
	EmitsSchema   *string             `parser:"( 'emits_schema' @String )?"`
	WritesSchema  *string             `parser:"( 'writes_schema' @String )?"`
	Deprecation   *DeprecationBlock   `parser:"( 'deprecation' '{' @@ '}' )?"`
	Compatibility *CompatibilityBlock `parser:"( 'compatibility' '{' @@ '}' )?"`
	Guarantees    *GuaranteesBlock    `parser:"( 'guarantees' '{' @@ '}' )?"`
}

type SchemaBlock struct {
	Pos     lexer.Position
	Entries []*SchemaEntry `parser:"@@*"`
}

type SchemaEntry struct {
	Key  string    `parser:"@Ident ':'"`
	Type *TypeSpec `parser:"@@"`
}

func (s *SchemaBlock) Location() SourceLocation {
	return SourceLocation{File: s.Pos.Filename, Line: s.Pos.Line, Column: s.Pos.Column, Offset: s.Pos.Offset}
}

type ConstraintsBlock struct {
	Entries []*ConstraintEntry `parser:"@@*"`
}

type ConstraintEntry struct {
	Pos   lexer.Position
	Key   string `parser:"@Ident"`
	Value string `parser:"@String"`
}

func (c *ConstraintEntry) Location() SourceLocation {
	return SourceLocation{File: c.Pos.Filename, Line: c.Pos.Line, Column: c.Pos.Column, Offset: c.Pos.Offset}
}

type ConventionsBlock struct {
	Entries []*ConventionEntry `parser:"@@*"`
}

type ConventionEntry struct {
	Pos   lexer.Position
	Key   string `parser:"@Ident"`
	Value string `parser:"@String"`
}

func (c *ConventionEntry) Location() SourceLocation {
	return SourceLocation{File: c.Pos.Filename, Line: c.Pos.Line, Column: c.Pos.Column, Offset: c.Pos.Offset}
}

type TypeSpec struct {
	Name     string   `parser:"@Ident"`
	Generics []string `parser:"( '<' @Ident ( ',' @Ident )* '>' )?"`
	Optional string   `parser:"( @'?' )?"`
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

// metaString is a helper function that searches metadata for a key.
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

// metaAll is a helper function that converts metadata to a map.
func metaAll(metadata []*MetaEntry) map[string]string {
	result := make(map[string]string, len(metadata))
	for _, meta := range metadata {
		if meta.Value != nil {
			result[meta.Key] = *meta.Value
		}
	}
	return result
}

// metaMap is a helper function that returns metadata entries with a given prefix.
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

// PostProcess extracts items into typed SLO fields
func (s *SLOBlock) PostProcess() {
	for _, it := range s.Items {
		if it.Availability != nil {
			s.Availability = it.Availability
		}
		if it.Latency != nil {
			s.Latency = it.Latency
		}
		if it.ErrorRate != nil {
			s.ErrorRate = it.ErrorRate
		}
		if it.Throughput != nil {
			s.Throughput = it.Throughput
		}
	}
}
