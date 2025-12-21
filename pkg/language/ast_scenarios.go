package language

import (
	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// Scenario (D2 Scenario Mapping)
// ============================================================================

// Scenario represents a D2 scenario (layer).
//
// Example DSL:
//
//	scenario "Login Flow" {
//	   User -> WebApp "Credentials"
//	   WebApp -> DB "Verify"
//	}
//
//	scenario AuthFlow "Authentication" "Handles OAuth2" { ... }
type Scenario struct {
	Pos         lexer.Position
	ID          string          `parser:"( 'scenario' | 'story' ) ( @Ident )?"`
	Title       *string         `parser:"( @String )?"`
	Description *string         `parser:"( @String )?"`
	Items       []*ScenarioItem `parser:"( '{' @@* '}' )?"`

	Steps []*ScenarioStep
}

func (s *Scenario) Location() SourceLocation {
	return SourceLocation{File: s.Pos.Filename, Line: s.Pos.Line, Column: s.Pos.Column, Offset: s.Pos.Offset}
}

type ScenarioItem struct {
	Description *string       `parser:"( 'description' @String )"`
	Step        *ScenarioStep `parser:"| @@"`
}

type ScenarioStep struct {
	Pos lexer.Position
	// Optional 'step' keyword
	StepKeyword *string  `parser:"( 'step' )?"`
	FromParts   []string `parser:"@Ident ( '.' @Ident )*"` // Source parts (supports qualified refs like System.Container)
	Arrow       string   `parser:"'->'"`                   // Arrow operator
	ToParts     []string `parser:"@Ident ( '.' @Ident )*"` // Target parts (supports qualified refs like System.Container)
	Description *string  `parser:"( @String )?"`
	Tags        []string `parser:"( '[' @Ident ( ',' @Ident )* ']' )?"`
	Order       *string  `parser:"( 'order' @String )?"`

	// Post-processed
	From QualifiedIdent
	To   QualifiedIdent
}

func (s *ScenarioStep) Location() SourceLocation {
	return SourceLocation{File: s.Pos.Filename, Line: s.Pos.Line, Column: s.Pos.Column, Offset: s.Pos.Offset}
}

// ============================================================================
// Flow
// ============================================================================

// Flow represents a Data Flow Diagram (DFD) style flow declaration.
// Flow is an alias to Scenario - they use the same syntax and structure.
//
// A flow describes how data moves between components in the system (DFD-style).
// Flows support relations between components to show data flows, just like scenarios.
//
// Example DSL:
//
//	flow OrderProcess "Order Processing" {
//	  Customer -> Shop "Order Details"
//	  Shop -> Database "Save Order"
//	  Database -> Shop "Confirmation"
//	}
type Flow struct {
	Pos         lexer.Position
	ID          string          `parser:"'flow' ( @Ident )?"`
	Title       *string         `parser:"( @String )?"`
	Description *string         `parser:"( @String )?"`
	Items       []*ScenarioItem `parser:"( '{' @@* '}' )?"` // Same as Scenario - uses ScenarioItem

	Steps []*ScenarioStep // DFD-style data flows (same as Scenario)
}

func (f *Flow) Location() SourceLocation {
	return SourceLocation{File: f.Pos.Filename, Line: f.Pos.Line, Column: f.Pos.Column, Offset: f.Pos.Offset}
}

// Flow uses ScenarioItem and ScenarioStep - it's an alias to Scenario
// No separate FlowItem, FlowRelation, FlowStep needed
