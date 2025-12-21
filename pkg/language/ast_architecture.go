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
	// Allow multiple top-level blocks (LikeC4 pattern)
	TopLevelItems []TopLevelItem `parser:"@@*"`
}

// TopLevelItem is a union type for items that can appear at the file top level.
type TopLevelItem struct {
	Specification *SpecificationBlock `parser:"@@"`
	Model         *ModelBlock         `parser:"| @@"`
	Views         *LikeC4ViewsBlock   `parser:"| @@"`
}

func (f *File) Location() SourceLocation {
	return SourceLocation{File: f.Pos.Filename, Line: f.Pos.Line, Column: f.Pos.Column, Offset: f.Pos.Offset}
}

type Program struct {
	Specification *SpecificationBlock
	Model         *ModelBlock
	Views         *LikeC4ViewsBlock
}

func (p *Program) PostProcess() {
	if p.Model != nil {
		p.Model.PostProcess()
	}
	if p.Views != nil {
		p.Views.PostProcess()
	}
}

func (m *ModelBlock) PostProcess() {
	for _, item := range m.Items {
		item.PostProcess()
	}
	m.InferImpliedRelationships()
}

func (m *ModelBlock) InferImpliedRelationships() {
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

func (m *ModelBlock) hasRelation(from, to string) bool {
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
	if m.Requirement != nil {
		m.Requirement.PostProcess()
	}
	if m.ADR != nil {
		m.ADR.PostProcess()
	}
	if m.Policy != nil {
		m.Policy.PostProcess()
	}
	if m.Scenario != nil {
		m.Scenario.PostProcess()
	}
	if m.Flow != nil {
		m.Flow.PostProcess()
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
