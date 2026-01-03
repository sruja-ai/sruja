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
	Navigation  *NavigationBlock  `parser:"( @@ )?"`
	Styles      *ExtendedStyles   `parser:"( @@ )?"`
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
//	    spacing { x: 150, y: 120 }
//	    preset "grid"
//	    direction "TB"
//	}
type LayoutBlock struct {
	Pos       lexer.Position
	LBrace    string           `parser:"'layout' '{'"`
	Direction *string          `parser:"( 'direction' @('TB' | 'LR' | 'BT' | 'RL') )?"`
	Preset    *string          `parser:"( 'preset' @('auto' | 'hierarchical' | 'grid' | 'radial' | 'force') )?"`
	Spacing   *SpacingCfg      `parser:"( 'spacing' @@ )?"`
	RankSep   *int             `parser:"( 'ranksep' @Number )?"`
	NodeSep   *int             `parser:"( 'nodesep' @Number )?"`
	Elements  []*ElementLayout `parser:"@@*"`
	RBrace    string           `parser:"'}'"`
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

// ============================================================================
// Enhanced View Features (Spacing, Navigation, Extended Styling)
// ============================================================================

// LayoutConfig represents enhanced layout configuration for a view.
//
// Example DSL:
//
//	layout {
//	    spacing { x: 150, y: 120 }
//	    ranksep 180
//	    nodesep 100
//	    direction "LR"
//	    preset "grid"
//	}
type LayoutConfig struct {
	Pos       lexer.Position
	LBrace    string           `parser:"'layout' '{'"`
	Direction *string          `parser:"| 'direction' @('TB' | 'LR' | 'BT' | 'RL')"`
	Preset    *string          `parser:"| 'preset' @('auto' | 'hierarchical' | 'grid' | 'radial' | 'force')"`
	Spacing   *SpacingCfg      `parser:"| 'spacing' @@"`
	RankSep   *int             `parser:"| 'ranksep' @Number"`
	NodeSep   *int             `parser:"| 'nodesep' @Number"`
	Elements  []*ElementLayout `parser:"@@*"`
	RBrace    string           `parser:"'}'"`
}

// SpacingCfg represents spacing configuration.
type SpacingCfg struct {
	Pos    lexer.Position
	LBrace string `parser:"'spacing' '{'"`
	XVal   string `parser:"'x' ':' @Number"`
	YVal   string `parser:"',' 'y' ':' @Number"`
	RBrace string `parser:"'}'"`
}

// Spacing returns the X and Y spacing values.
func (s *SpacingCfg) Spacing() (float64, float64) {
	x, _ := strconv.ParseFloat(s.XVal, 64)
	y, _ := strconv.ParseFloat(s.YVal, 64)
	return x, y
}

// NavigationBlock represents navigation links between views.
//
// Example DSL:
//
//	navigation {
//	    up "index"
//	    down "containers" of Shop
//	    related "deployment" of Prod
//	    sidebar [
//	        "components" of API,
//	        "deployment" of Prod
//	    ]
//	}
type NavigationBlock struct {
	Pos    lexer.Position
	LBrace string            `parser:"'navigation' '{'"`
	Links  []*NavigationLink `parser:"@@*"`
	RBrace string            `parser:"'}'"`
}

// NavigationLink represents a navigation link.
type NavigationLink struct {
	Pos       lexer.Position
	Direction string            `parser:"@('up' | 'down' | 'related' | 'sidebar')"`
	ViewRefs  []*QualifiedIdent `parser:"@@+"`
	Comma     string            `parser:"( ',' )?"`
}

// ViewRef returns the first view reference.
func (n *NavigationLink) ViewRef() *QualifiedIdent {
	if len(n.ViewRefs) > 0 {
		return n.ViewRefs[0]
	}
	return nil
}

// ExtendedStyles represents extended styling for views.
//
// Example DSL:
//
//	styles {
//	    element "Database" {
//	        shape cylinder
//	        color "#ff0000"
//	        strokeWidth 2
//	        opacity 0.8
//	    }
//
//	    element [tag=critical] {
//	        color "#ff0000"
//	        strokeWidth 3
//	    }
//
//	    edge [tag=async] {
//	        style dashed
//	        color "#666666"
//	    }
//	}
type ExtendedStyles struct {
	Pos    lexer.Position
	LBrace string               `parser:"'styles' '{'"`
	Rules  []*ExtendedStyleRule `parser:"@@*"`
	RBrace string               `parser:"'}'"`
}

// ExtendedStyleRule represents an extended style rule with tag support.
type ExtendedStyleRule struct {
	Pos    lexer.Position
	Target string       `parser:"@('element' | 'relationship' | 'edge' | 'person' | 'system' | 'container' | 'component')"`
	Tag    *TagFilter   `parser:"( '[' @@ ']' )?"`
	LBrace string       `parser:"'{'"`
	Props  []*StyleProp `parser:"@@*"`
	RBrace string       `parser:"'}'"`
}

// TagFilter represents a tag-based filter.
type TagFilter struct {
	Pos   lexer.Position
	Key   string `parser:"@Ident"`
	Equal string `parser:"'='"`
	Value string `parser:"@String"`
}

// StyleProp represents a style property.
type StyleProp struct {
	Pos   lexer.Position
	Key   string  `parser:"@Ident"`
	Value *string `parser:"( @String | @Number | @Color )?"`
}

// Color represents a color value.
type Color string

// ============================================================================
// View Metadata
// ============================================================================

// ViewMeta represents metadata for a view.
//
// Example DSL:
//
//	view index {
//	    title "System Context Diagram"
//	    description "Shows the overall system structure and external users"
//	    link "https://docs.example.com/architecture"
//	    tags ["context", "overview"]
//	}
type ViewMeta struct {
	Pos         lexer.Position
	LBrace      string    `parser:"'meta' '{'"`
	Title       *string   `parser:"| 'title' @String"`
	Description *string   `parser:"| 'description' @String"`
	Link        *string   `parser:"| 'link' @String"`
	Tags        []*string `parser:"| 'tags' '[' ( @String ','? )* ']'"`
	RBrace      string    `parser:"'}'"`
}
