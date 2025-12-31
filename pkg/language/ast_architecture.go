package language

import (
	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// File (Physical Root)
// ============================================================================

// File represents the physical structure of a parsed Sruja DSL file.
type File struct {
	Pos lexer.Position
	// Allow multiple top-level blocks
	TopLevelItems []TopLevelItem `parser:"@@*"`
}

// TopLevelItem is a union type for items that can appear at the file top level.
// TopLevelItem is a union type for items that can appear at the file top level.
type TopLevelItem struct {
	// Top-level declarations
	KindDef     *ElementKindDef   `parser:"@@"`
	TagDef      *TagDef           `parser:"| @@"`
	ElementDef  *ElementDef       `parser:"| @@"`
	ViewDef     *ViewDef          `parser:"| @@"`
	Relation    *Relation         `parser:"| @@"`
	Import      *ImportStatement  `parser:"| @@"`
	Overview    *OverviewBlock    `parser:"| @@"`
	Deployment  *DeploymentNode   `parser:"| @@"`
	Constraints *ConstraintsBlock `parser:"| @@"`
	Conventions *ConventionsBlock `parser:"| @@"`
	Extend      *ExtendElement    `parser:"| @@"`
	Styles      *StyleDecl        `parser:"| @@"`
}

func (f *File) Location() SourceLocation {
	return SourceLocation{File: f.Pos.Filename, Line: f.Pos.Line, Column: f.Pos.Column, Offset: f.Pos.Offset}
}

type Program struct {
	Items []TopLevelItem `parser:"@@*"`

	// These are populated during PostProcess
	Specification *Specification
	Model         *Model
	Views         *Views
}

func (p *Program) PostProcess() {
	// Single pass: distribute top-level items into logical containers
	for _, item := range p.Items {
		if item.KindDef != nil {
			p.ensureSpecification()
			p.Specification.Items = append(p.Specification.Items, SpecificationItem{Element: item.KindDef})
		}
		if item.TagDef != nil {
			p.ensureSpecification()
			p.Specification.Items = append(p.Specification.Items, SpecificationItem{Tag: item.TagDef})
		}
		if item.ElementDef != nil {
			p.ensureModel()
			p.Model.Items = append(p.Model.Items, ModelItem{ElementDef: item.ElementDef})
		}
		if item.Relation != nil {
			p.ensureModel()
			p.Model.Items = append(p.Model.Items, ModelItem{Relation: item.Relation})
		}
		if item.Import != nil {
			p.ensureModel()
			p.Model.Items = append(p.Model.Items, ModelItem{Import: item.Import})
		}
		if item.Overview != nil {
			p.ensureModel()
			p.Model.Items = append(p.Model.Items, ModelItem{Overview: item.Overview})
		}
		if item.Deployment != nil {
			p.ensureModel()
			p.Model.Items = append(p.Model.Items, ModelItem{DeploymentNode: item.Deployment})
		}
		if item.Constraints != nil {
			p.ensureModel()
			p.Model.Items = append(p.Model.Items, ModelItem{ConstraintsBlock: item.Constraints})
		}
		if item.Conventions != nil {
			p.ensureModel()
			p.Model.Items = append(p.Model.Items, ModelItem{ConventionsBlock: item.Conventions})
		}
		if item.Extend != nil {
			p.ensureModel()
			p.Model.Items = append(p.Model.Items, ModelItem{Extend: item.Extend})
		}
		if item.Styles != nil {
			// Styles can belong to Model or Views?
			// Previously ModelBlock had Styles, ViewsBlock had Styles (via ViewsItem)
			// ViewsItem has `Styles *StyleDecl`.
			// ModelItem has `Styles *StyleDecl`.
			// Since we don't know intent, maybe add to Model if it appears before views?
			// Or maybe just duplicate references?
			// For now, let's assume global Styles go to Model.
			// But wait, Views also had global styles?
			// Let's verify `ViewsItem` struct in `ast_model.go`.
			// It has `Styles *StyleDecl`.
			// If we put it in Model, it affects Elements.
			// If we put it in Views, it affects Views.
			// Ideally, we can put it in BOTH or check context.
			// But flat syntax implies global scope.
			// Let's put it in Model for now as that's where most styles are.
			// Styles historically belonged to Views block.
			// Even at top level, they are often view-related configurations.
			// Store in Views to satisfy existing consumers/tests.
			p.ensureViews()
			p.Views.Items = append(p.Views.Items, &ViewsItem{Styles: item.Styles})
		}

		if item.ViewDef != nil {
			p.ensureViews()
			p.Views.Items = append(p.Views.Items, &ViewsItem{View: item.ViewDef})
		}
	}

	if p.Model != nil {
		p.Model.PostProcess()
	}
	if p.Views != nil {
		p.Views.PostProcess()
	}
}

func (p *Program) ensureSpecification() {
	if p.Specification == nil {
		p.Specification = &Specification{
			Items: []SpecificationItem{},
		}
	}
}

func (p *Program) ensureModel() {
	if p.Model == nil {
		p.Model = &Model{
			Items: []ModelItem{},
		}
	}
}

func (p *Program) ensureViews() {
	if p.Views == nil {
		p.Views = &Views{
			Items: []*ViewsItem{},
		}
	}
}

func (m *Model) PostProcess() {
	for _, item := range m.Items {
		item.PostProcess()
	}
	m.InferImpliedRelationships()
}

func (m *Model) InferImpliedRelationships() {
	var inferred []*Relation
	for _, item := range m.Items {
		if item.Relation != nil {
			rel := item.Relation
			// Infer parents of From
			for i := 1; i < len(rel.From.Parts); i++ {
				parentFrom := QualifiedIdent{Parts: rel.From.Parts[:i]}
				if !isRelParentChild(parentFrom, rel.To) && !isRelParentChild(rel.To, parentFrom) {
					inferred = append(inferred, &Relation{
						From:  parentFrom,
						Arrow: "->",
						To:    rel.To,
						Verb:  rel.Verb,
						Label: rel.Label,
					})
				}
			}
			// Infer parents of To
			for i := 1; i < len(rel.To.Parts); i++ {
				parentTo := QualifiedIdent{Parts: rel.To.Parts[:i]}
				if !isRelParentChild(rel.From, parentTo) && !isRelParentChild(parentTo, rel.From) {
					inferred = append(inferred, &Relation{
						From:  rel.From,
						Arrow: "->",
						To:    parentTo,
						Verb:  rel.Verb,
						Label: rel.Label,
					})
				}
			}
			// And both parents
			for i := 1; i < len(rel.From.Parts); i++ {
				for j := 1; j < len(rel.To.Parts); j++ {
					parentFrom := QualifiedIdent{Parts: rel.From.Parts[:i]}
					parentTo := QualifiedIdent{Parts: rel.To.Parts[:j]}
					fromStr := parentFrom.String()
					toStr := parentTo.String()
					if fromStr != toStr && !isRelParentChild(parentFrom, parentTo) && !isRelParentChild(parentTo, parentFrom) {
						inferred = append(inferred, &Relation{
							From:  parentFrom,
							Arrow: "->",
							To:    parentTo,
							Verb:  rel.Verb,
							Label: rel.Label,
						})
					}
				}
			}
		}
	}

	// Add inferred relations (avoiding duplicates and self-references)
	for _, inf := range inferred {
		fromStr := inf.From.String()
		toStr := inf.To.String()
		if fromStr != toStr && !m.hasRelation(fromStr, toStr) {
			m.Items = append(m.Items, ModelItem{Relation: inf})
		}
	}
}

func (m *Model) hasRelation(from, to string) bool {
	for _, item := range m.Items {
		if item.Relation != nil {
			if item.Relation.From.String() == from && item.Relation.To.String() == to {
				return true
			}
		}
	}
	return false
}

func isRelParentChild(parent, child QualifiedIdent) bool {
	if len(parent.Parts) >= len(child.Parts) {
		return false
	}
	for i := range parent.Parts {
		if parent.Parts[i] != child.Parts[i] {
			return false
		}
	}
	return true
}

func (m *ModelItem) PostProcess() {
	if m.Import != nil {
		m.Import.PostProcess()
	}
	if m.DeploymentNode != nil {
		m.DeploymentNode.PostProcess()
	}
	if m.Overview != nil {
		m.Overview.PostProcess()
	}
	if m.Relation != nil {
		normalizeRelation(m.Relation)
	}
	if m.ElementDef != nil {
		m.ElementDef.PostProcess()
	}
}

func (v *Views) PostProcess() {
	for _, item := range v.Items {
		if item.View != nil {
			item.View.PostProcess()
		}
	}
}
