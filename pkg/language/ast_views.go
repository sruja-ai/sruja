// pkg/language/ast_views.go
// Package language provides DSL parsing and AST structures for views.
package language

import (
	"strconv"

	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// Views Block
// ============================================================================

// ViewBlock represents an optional views block for customizing diagram views.
//
// Views are optional - if not specified, standard C4 views are automatically
// generated from the model. Views block is only needed for customization.
//
// Example DSL:
//
//	views {
//	    container Shop "API Focus" {
//	        include Shop.API Shop.DB
//	        exclude Shop.WebApp
//	    }
//
//	    styles {
//	        element "Database" {
//	            shape cylinder
//	            color "#ff0000"
//	        }
//	    }
//	}
type ViewBlock struct {
	Pos    lexer.Position
	LBrace string       `parser:"'views' '{'"`
	Views  []*View      `parser:"@@*"`
	Styles *StylesBlock `parser:"( @@ )?"`
	RBrace string       `parser:"'}'"`
}

func (v *ViewBlock) Location() SourceLocation {
	return SourceLocation{File: v.Pos.Filename, Line: v.Pos.Line, Column: v.Pos.Column, Offset: v.Pos.Offset}
}

// View represents a single view definition.
//
// Supports two syntaxes:
// 1. Simple: view <name> { ... }
// 2. Typed: view <type> of <scope> { ... }
// 3. Legacy: <type> <scope> "<name>" { ... }
type View struct {
	Pos         lexer.Position
	ViewKeyword string `parser:"'view'"`
	// Support both simple name and typed form
	TypeOrName  string            `parser:"@Ident"`
	OfKeyword   *string           `parser:"( 'of' )?"`
	Scope       *QualifiedIdent   `parser:"( @@ )?"`
	Name        *string           `parser:"( @String )?"`
	LBrace      string            `parser:"'{'"`
	Expressions []*ViewExpression `parser:"@@*"`
	Autolayout  *string           `parser:"( 'autolayout' @Ident )?"`
	Layout      *LayoutBlock      `parser:"( @@ )?"`
	RBrace      string            `parser:"'}'"`
}

func (v *View) Location() SourceLocation {
	return SourceLocation{File: v.Pos.Filename, Line: v.Pos.Line, Column: v.Pos.Column, Offset: v.Pos.Offset}
}

// ViewExpression represents an include or exclude expression in a view.
//
// Examples:
//   - include *
//   - include Element1 Element2
//   - include "->Element->"
//   - exclude Element1
type ViewExpression struct {
	Pos      lexer.Position
	Type     string           `parser:"@('include' | 'exclude')"`
	Wildcard *string          `parser:"( @Wildcard )?"`
	Elements []QualifiedIdent `parser:"( @@* )?"`
	Pattern  *string          `parser:"( @String )?"` // For patterns like "->Element->"
}

func (v *ViewExpression) Location() SourceLocation {
	return SourceLocation{File: v.Pos.Filename, Line: v.Pos.Line, Column: v.Pos.Column, Offset: v.Pos.Offset}
}

// StylesBlock represents styling definitions for views.
//
// Example DSL:
//
//	styles {
//	    element "Database" {
//	        shape cylinder
//	        color "#ff0000"
//	        stroke "#cc0000"
//	        strokeWidth 2
//	    }
//
//	    relationship "Relationship" {
//	        thickness 3
//	        color "#666666"
//	    }
//	}
type StylesBlock struct {
	Pos    lexer.Position
	LBrace string          `parser:"'styles' '{'"`
	Styles []*ElementStyle `parser:"@@*"`
	RBrace string          `parser:"'}'"`
}

func (s *StylesBlock) Location() SourceLocation {
	return SourceLocation{File: s.Pos.Filename, Line: s.Pos.Line, Column: s.Pos.Column, Offset: s.Pos.Offset}
}

// ElementStyle represents styling for elements or relationships by tag.
type ElementStyle struct {
	Pos        lexer.Position
	Target     string           `parser:"@('element' | 'relationship')"`
	Tag        string           `parser:"@String"`
	LBrace     string           `parser:"'{'"`
	Properties []*StyleProperty `parser:"@@*"`
	RBrace     string           `parser:"'}'"`
}

func (e *ElementStyle) Location() SourceLocation {
	return SourceLocation{File: e.Pos.Filename, Line: e.Pos.Line, Column: e.Pos.Column, Offset: e.Pos.Offset}
}

// StyleProperty represents a single style property.
type StyleProperty struct {
	Key   string  `parser:"@Ident"`
	Value *string `parser:"@String"` // Style values are strings (colors, shapes, etc.)
}

// ============================================================================
// Layout Block (Manual Editing Support)
// ============================================================================

// LayoutBlock represents layout configuration for a view, including manual position hints.
//
// Example DSL:
//
//	layout {
//	    element user {
//	        position { x: 200, y: 50 }
//	    }
//	    element system {
//	        position { x: 400, y: 200 }
//	    }
//	}
type LayoutBlock struct {
	Pos      lexer.Position
	LBrace   string           `parser:"'layout' '{'"`
	Elements []*ElementLayout `parser:"@@*"`
	RBrace   string           `parser:"'}'"`
}

func (l *LayoutBlock) Location() SourceLocation {
	return SourceLocation{File: l.Pos.Filename, Line: l.Pos.Line, Column: l.Pos.Column, Offset: l.Pos.Offset}
}

// ElementLayout represents layout configuration for a specific element.
type ElementLayout struct {
	Pos      lexer.Position
	Element  QualifiedIdent `parser:"'element' @@"`
	LBrace   string         `parser:"'{'"`
	Position *PositionHint  `parser:"( @@ )?"`
	RBrace   string         `parser:"'}'"`
}

func (e *ElementLayout) Location() SourceLocation {
	return SourceLocation{File: e.Pos.Filename, Line: e.Pos.Line, Column: e.Pos.Column, Offset: e.Pos.Offset}
}

// PositionHint represents a position hint for an element.
type PositionHint struct {
	Pos    lexer.Position
	LBrace string `parser:"'position' '{'"`
	XVal   string `parser:"'x' ':' @Number"`
	YVal   string `parser:"',' 'y' ':' @Number"`
	RBrace string `parser:"'}'"`
}

// X returns the X coordinate as float64.
func (p *PositionHint) X() float64 {
	if p.XVal != "" {
		v, err := strconv.ParseFloat(p.XVal, 64)
		if err == nil {
			return v
		}
	}
	return 0
}

// Y returns the Y coordinate as float64.
func (p *PositionHint) Y() float64 {
	if p.YVal != "" {
		v, err := strconv.ParseFloat(p.YVal, 64)
		if err == nil {
			return v
		}
	}
	return 0
}

func (p *PositionHint) Location() SourceLocation {
	return SourceLocation{File: p.Pos.Filename, Line: p.Pos.Line, Column: p.Pos.Column, Offset: p.Pos.Offset}
}
