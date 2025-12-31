package language

import (
	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// Model Types (Element Definitions, Views, Layouts)
// ============================================================================

// Logical container for specification items
type Specification struct {
	Items []SpecificationItem
}

type SpecificationItem struct {
	Element *ElementKindDef
	Tag     *TagDef
}

type ElementKindDef struct {
	Pos   lexer.Position
	Name  string              `parser:"@Ident '=' 'kind'"`
	Title *string             `parser:"( @String )?"`
	Body  *ElementKindDefBody `parser:"( '{' @@ '}' )?"`
}

type ElementKindDefBody struct {
	Title       *string     `parser:"( 'title' @String )?"`
	Description *string     `parser:"( 'description' @String )?"`
	Technology  *string     `parser:"( 'technology' | 'tech' ) @String?"`
	Style       *StyleBlock `parser:"( ( 'style' | 'styles' ) '{' @@ '}' )?"`
}

type TagDef struct {
	Pos   lexer.Position
	Name  string  `parser:"@Ident '=' 'tag'"`
	Title *string `parser:"( @String )?"`
}

// Logical container for model items
type Model struct {
	Items []ModelItem
}

type ModelItem struct {
	Import           *ImportStatement  `parser:"@@ |"`
	Extend           *ExtendElement    `parser:"@@ |"`
	DeploymentNode   *DeploymentNode   `parser:"@@ |"`
	Overview         *OverviewBlock    `parser:"@@ |"`
	ConstraintsBlock *ConstraintsBlock `parser:"@@ |"`
	ConventionsBlock *ConventionsBlock `parser:"@@ |"`
	Styles           *StyleDecl        `parser:"@@ |"`
	ElementDef       *ElementDef       `parser:"@@ |"`
	Relation         *Relation         `parser:"@@"`
}

// ImportStatement represents an import statement.
//
// Example DSL:
//
//	import { serviceA, serviceB } from 'projectA'
type ImportStatement struct {
	Pos      lexer.Position
	Elements []string `parser:"'import' '{' ( @Ident | @Wildcard ) ( ',' ( @Ident | @Wildcard ) )* '}'"`
	From     string   `parser:"'from' @String"`
}

func (i *ImportStatement) Location() SourceLocation {
	return SourceLocation{File: i.Pos.Filename, Line: i.Pos.Line, Column: i.Pos.Column, Offset: i.Pos.Offset}
}

type ExtendElement struct {
	Pos  lexer.Position
	ID   QualifiedIdent  `parser:"'extend' @@"`
	Body *ElementDefBody `parser:"'{' @@ '}'"`
}

// ElementDef represents an element definition in the model.
// This is the core type for defining systems, containers, components, etc.
type ElementDef struct {
	Pos        lexer.Position
	Assignment *ElementAssignment `parser:"@@"`
}

// ElementAssignment represents the assignment part of an element definition.
type ElementAssignment struct {
	Pos     lexer.Position
	Name    string          `parser:"@Ident '='"`
	Kind    string          `parser:"@( 'person' | 'system' | 'container' | 'component' | 'database' | 'queue' | 'policy' | 'requirement' | 'adr' | 'flow' | 'scenario' | 'story' | Ident )"`
	SubKind *string         `parser:"( @Ident )?"` // For requirement type: functional, security, etc.
	Title   *string         `parser:"( @String )?"`
	TagRefs []string        `parser:"@TagRef*"`
	Body    *ElementDefBody `parser:"( '{' @@ '}' )?"`
}

func (e *ElementDef) GetID() string {
	if e.Assignment != nil {
		return e.Assignment.Name
	}
	return ""
}

func (e *ElementDef) GetKind() string {
	if e.Assignment != nil {
		return e.Assignment.Kind
	}
	return ""
}

func (e *ElementDef) GetTitle() *string {
	if e.Assignment != nil {
		return e.Assignment.Title
	}
	return nil
}

func (e *ElementDef) GetBody() *ElementDefBody {
	if e.Assignment != nil {
		return e.Assignment.Body
	}
	return nil
}

func (e *ElementDef) GetTagRefs() []string {
	if e.Assignment != nil {
		return e.Assignment.TagRefs
	}
	return nil
}

func (e *ElementDef) Location() SourceLocation {
	return SourceLocation{File: e.Pos.Filename, Line: e.Pos.Line, Column: e.Pos.Column, Offset: e.Pos.Offset}
}

// ElementDefBody represents the body of an element definition.
type ElementDefBody struct {
	Items []*BodyItem `parser:"@@*"`
}

// BodyItem represents items that can appear inside an element body.
type BodyItem struct {
	// Metadata fields (tried first)
	Description *string          `parser:"'description' ':'? @String |"`
	Technology  *string          `parser:"( 'technology' | 'tech' ) ':'? @String |"`
	Tags        []string         `parser:"'tags' ( '[' @String ( ',' @String )* ']' | @String ) |"`
	Version     *string          `parser:"'version' @String |"`
	Metadata    *MetadataBlock   `parser:"@@ |"`
	Properties  *PropertiesBlock `parser:"@@ |"`
	Styles      *StyleDecl       `parser:"@@ |"`
	SLO         *SLOBlock        `parser:"@@ |"`
	Scale       *ScaleBlock      `parser:"@@ |"`

	// ADR body fields
	Status       *string `parser:"'status' ':'? @String |"`
	Context      *string `parser:"'context' ':'? @String |"`
	Decision     *string `parser:"'decision' ':'? @String |"`
	Consequences *string `parser:"'consequences' ':'? @String |"`

	// Policy body fields
	Category    *string `parser:"'category' ':'? @String |"`
	Enforcement *string `parser:"'enforcement' ':'? @String |"`

	// Scenario/Flow step (simplified)
	Step *ScenarioStep `parser:"'step' @@ |"`

	// Link/External
	External *string `parser:"'external' @String |"`

	// Children/Relations
	Relation *Relation `parser:"@@ |"`
	// TagRefs moved to end to avoid obscuring error messages
	TagRefs []string    `parser:"@TagRef+ |"`
	Element *ElementDef `parser:"@@"`
}

func (b *BodyItem) PostProcess() {
	if b.Element != nil {
		b.Element.PostProcess()
	}
	if b.SLO != nil {
		b.SLO.PostProcess()
	}
	if b.Scale != nil {
		b.Scale.PostProcess()
	}
	if b.Relation != nil {
		normalizeRelation(b.Relation)
	}
}

func (e *ElementDef) PostProcess() {
	body := e.GetBody()
	if body != nil {
		for _, item := range body.Items {
			item.PostProcess()
		}
	}
}

// Logical container for views items
type Views struct {
	Items []*ViewsItem
}

// ViewsItem represents items that can appear inside a views block.
type ViewsItem struct {
	Styles *StyleDecl
	View   *ViewDef
}

// ViewDef represents a view definition.
type ViewDef struct {
	Pos     lexer.Position
	Name    *string         `parser:"'view' ( @Ident )?"`
	Extends *string         `parser:"( 'extends' @Ident )?"`
	Of      *QualifiedIdent `parser:"( 'of' @@ )?"`
	Title   *string         `parser:"( 'title' @String )?"`
	Body    *ViewBody       `parser:"( '{' @@ '}' )?"`
}

func (v *ViewDef) PostProcess() {
	// Post-process view body if needed
}

// Note: LayoutBlock, ElementLayout, and PositionHint are defined in ast_views.go

// ViewBody represents the body of a view definition.
type ViewBody struct {
	Items []*ViewItem `parser:"@@*"`
}

// ViewItem represents items that can appear inside a view body.
type ViewItem struct {
	Include *IncludePredicate `parser:"@@"`
	Exclude *ExcludePredicate `parser:"| @@"`
	Title   *string           `parser:"| 'title' @String"`
	Style   *ViewStyle        `parser:"| @@"`
	Layout  *LayoutBlock      `parser:"| @@"`
}

type IncludePredicate struct {
	Expressions []ViewExpr `parser:"'include' @@ ( ','? @@ )*"`
}

type ExcludePredicate struct {
	Expressions []ViewExpr `parser:"'exclude' @@ ( ','? @@ )*"`
}

type ViewExpr struct {
	Wildcard  bool            `parser:"( @Wildcard"`
	Recursive bool            `parser:"| @( Wildcard Wildcard )"`
	Selector  *string         `parser:"| @( Ident | 'element' | 'person' | 'system' | 'container' | 'component' | 'database' | 'queue' | 'style' | 'styles' ) )"`
	Sub       *ViewExprSuffix `parser:"@@?"`
}

func (v ViewExpr) String() string {
	res := ""
	switch {
	case v.Recursive:
		res = "**"
	case v.Wildcard:
		res = "*"
	case v.Selector != nil:
		res = *v.Selector
	}
	if v.Sub != nil {
		switch {
		case v.Sub.Recursive:
			res += ".**"
		case v.Sub.Wildcard:
			res += ".*"
		case v.Sub.Ident != nil:
			res += "." + *v.Sub.Ident
		}
	}
	return res
}

type ViewExprSuffix struct {
	Recursive bool    `parser:"'.' ( @( Wildcard Wildcard )"`
	Wildcard  bool    `parser:"| @Wildcard"`
	Ident     *string `parser:"| @Ident )"`
}

type ViewStyle struct {
	Selector string      `parser:"( 'style' | 'styles' ) @Ident"`
	Props    *StyleBlock `parser:"@@"`
}

func (v *ViewDef) Location() SourceLocation {
	return SourceLocation{File: v.Pos.Filename, Line: v.Pos.Line, Column: v.Pos.Column, Offset: v.Pos.Offset}
}
