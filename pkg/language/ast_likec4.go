package language

import (
	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// LikeC4 Types
// ============================================================================

type SpecificationBlock struct {
	Pos    lexer.Position
	LBrace string              `parser:"'specification' '{'"`
	Items  []SpecificationItem `parser:"@@*"`
	RBrace string              `parser:"'}'"`
}

type SpecificationItem struct {
	Element *ElementKindDef `parser:"@@"`
	Tag     *TagDef         `parser:"| @@"`
}

type ElementKindDef struct {
	Pos  lexer.Position
	Kind string              `parser:"'element' @Ident"`
	Body *ElementKindDefBody `parser:"( '{' @@ '}' )?"`
}

type ElementKindDefBody struct {
	Title       *string     `parser:"( 'title' @String )?"`
	Description *string     `parser:"( 'description' @String )?"`
	Technology  *string     `parser:"( 'technology' | 'tech' ) @String?"`
	Style       *StyleBlock `parser:"( ( 'style' | 'styles' ) '{' @@ '}' )?"`
}

type TagDef struct {
	Pos  lexer.Position
	Kind string `parser:"'tag' @Ident"`
}

type ModelBlock struct {
	Pos    lexer.Position
	LBrace string      `parser:"'model' '{'"`
	Items  []ModelItem `parser:"@@*"`
	RBrace string      `parser:"'}'"`
}

type ModelItem struct {
	// 1. Keyword-based items (Tried FIRST to avoid greedily matching Relation)
	Import           *ImportStatement  `parser:"@@"`
	Extend           *ExtendElement    `parser:"| @@"`
	Requirement      *Requirement      `parser:"| @@"`
	ADR              *ADR              `parser:"| @@"`
	Policy           *Policy           `parser:"| @@"`
	Scenario         *Scenario         `parser:"| @@"`
	Flow             *Flow             `parser:"| @@"`
	DeploymentNode   *DeploymentNode   `parser:"| @@"`
	Overview         *OverviewBlock    `parser:"| @@"`
	ConstraintsBlock *ConstraintsBlock `parser:"| 'constraints' '{' @@ '}'"`
	ConventionsBlock *ConventionsBlock `parser:"| 'conventions' '{' @@ '}'"`
	ContractsBlock   *ContractsBlock   `parser:"| 'contracts' '{' @@ '}'"`
	Styles           *StyleDecl        `parser:"| @@"`

	// 2. Structural items
	Relation *Relation `parser:"| @@"`

	// 3. Fallback: Element Definition
	ElementDef *LikeC4ElementDef `parser:"| @@"`
}

// ImportStatement represents a LikeC4 import statement.
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
	ID   QualifiedIdent        `parser:"'extend' @@"`
	Body *LikeC4ElementDefBody `parser:"'{' @@ '}'"`
}

type LikeC4ElementDef struct {
	Pos        lexer.Position
	Assignment *LikeC4Assignment `parser:"( @@"`
	Definition *LikeC4Definition `parser:"| @@ )"`
}

type LikeC4Assignment struct {
	Name    string                `parser:"@Ident '='"`
	Kind    string                `parser:"@( 'person' | 'system' | 'container' | 'component' | 'database' | 'queue' | Ident )"`
	Title   *string               `parser:"( @String )?"`
	TagRefs []string              `parser:"@TagRef*"`
	Body    *LikeC4ElementDefBody `parser:"( '{' @@ '}' )?"`
}

type LikeC4Definition struct {
	Kind    string                `parser:"@( 'person' | 'system' | 'container' | 'component' | 'database' | 'queue' | Ident )"`
	Name    *string               `parser:"( @Ident )?"`
	Title   *string               `parser:"( @String )?"`
	TagRefs []string              `parser:"@TagRef*"`
	Body    *LikeC4ElementDefBody `parser:"( '{' @@ '}' )?"`
}

func (e *LikeC4ElementDef) GetID() string {
	if e.Assignment != nil {
		return e.Assignment.Name
	}
	if e.Definition != nil && e.Definition.Name != nil {
		return *e.Definition.Name
	}
	return ""
}

func (e *LikeC4ElementDef) GetKind() string {
	if e.Assignment != nil {
		return e.Assignment.Kind
	}
	if e.Definition != nil {
		return e.Definition.Kind
	}
	return ""
}

func (e *LikeC4ElementDef) GetTitle() *string {
	if e.Assignment != nil {
		return e.Assignment.Title
	}
	if e.Definition != nil {
		return e.Definition.Title
	}
	return nil
}

func (e *LikeC4ElementDef) GetBody() *LikeC4ElementDefBody {
	if e.Assignment != nil {
		return e.Assignment.Body
	}
	if e.Definition != nil {
		return e.Definition.Body
	}
	return nil
}

func (e *LikeC4ElementDef) GetTagRefs() []string {
	if e.Assignment != nil {
		return e.Assignment.TagRefs
	}
	if e.Definition != nil {
		return e.Definition.TagRefs
	}
	return nil
}

func (e *LikeC4ElementDef) Location() SourceLocation {
	return SourceLocation{File: e.Pos.Filename, Line: e.Pos.Line, Column: e.Pos.Column, Offset: e.Pos.Offset}
}

type LikeC4ElementDefBody struct {
	Items []*LikeC4BodyItem `parser:"@@*"`
}

type LikeC4BodyItem struct {
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

	// Link/External
	External *string `parser:"'external' @String |"`

	// Children/Relations
	Relation *Relation `parser:"@@ |"`
	// TagRefs moved to end to avoid obscuring error messages
	TagRefs []string          `parser:"@TagRef+ |"`
	Element *LikeC4ElementDef `parser:"@@"`
}

func (b *LikeC4BodyItem) PostProcess() {
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

func (e *LikeC4ElementDef) PostProcess() {
	body := e.GetBody()
	if body != nil {
		for _, item := range body.Items {
			item.PostProcess()
		}
	}
}

type LikeC4ViewsBlock struct {
	Pos    lexer.Position
	Folder *string            `parser:"'views' ( @String )?"`
	Items  []*LikeC4ViewsItem `parser:"'{' @@* '}'"`
}

type LikeC4ViewsItem struct {
	Styles *StyleDecl     `parser:"@@"`
	View   *LikeC4ViewDef `parser:"| @@"`
}

func (v *LikeC4ViewsBlock) Location() SourceLocation {
	return SourceLocation{File: v.Pos.Filename, Line: v.Pos.Line, Column: v.Pos.Column, Offset: v.Pos.Offset}
}

func (v *LikeC4ViewsBlock) PostProcess() {
	for _, item := range v.Items {
		if item.View != nil {
			item.View.PostProcess()
		}
	}
}

type LikeC4ViewDef struct {
	Pos     lexer.Position
	Name    *string         `parser:"'view' ( @Ident )?"`
	Extends *string         `parser:"( 'extends' @Ident )?"`
	Of      *QualifiedIdent `parser:"( 'of' @@ )?"`
	Title   *string         `parser:"( 'title' @String )?"`
	Body    *LikeC4ViewBody `parser:"( '{' @@ '}' )?"`
}

func (v *LikeC4ViewDef) PostProcess() {
	// Post-process view body if needed
}

type LikeC4ViewBody struct {
	Items []*LikeC4ViewItem `parser:"@@*"`
}

type LikeC4ViewItem struct {
	Include *IncludePredicate `parser:"@@"`
	Exclude *ExcludePredicate `parser:"| @@"`
	Title   *string           `parser:"| 'title' @String"`
	Style   *ViewStyle        `parser:"| @@"`
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

func (v *LikeC4ViewDef) Location() SourceLocation {
	return SourceLocation{File: v.Pos.Filename, Line: v.Pos.Line, Column: v.Pos.Column, Offset: v.Pos.Offset}
}
