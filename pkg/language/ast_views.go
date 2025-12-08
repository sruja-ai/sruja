// pkg/language/ast_views.go
// Package language provides DSL parsing and AST structures for views.
package language

import (
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
// Views can be of different types: systemContext, container, component, deployment.
// Each view can have include/exclude expressions to filter elements.
type View struct {
	Pos         lexer.Position
	Type        string            `parser:"@('systemContext' | 'container' | 'component' | 'deployment')"`
	Scope       QualifiedIdent    `parser:"@@"`
	Name        string            `parser:"@String"`
	LBrace      string            `parser:"'{'"`
	Expressions []*ViewExpression `parser:"@@*"`
	Autolayout  *string           `parser:"( 'autolayout' @Ident )?"`
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
