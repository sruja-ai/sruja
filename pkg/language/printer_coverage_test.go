package language

import (
	"strings"
	"testing"
)

func TestPrinter_Coverage(t *testing.T) {
	// Create a comprehensive program model
	program := &Program{
		Specification: &SpecificationBlock{
			Items: []SpecificationItem{
				{Element: &ElementKindDef{Kind: "component", Body: &ElementKindDefBody{Title: sPtr("Component"), Description: sPtr("Desc")}}},
				{Tag: &TagDef{Kind: "mytag"}},
			},
		},
		Model: &ModelBlock{
			Items: []ModelItem{
				{
					Import: &ImportStatement{
						Elements: []string{"A", "B"},
						From:     "lib",
					},
				},
				{
					Requirement: &Requirement{
						ID:          "REQ-1",
						Description: sPtr("Must go fast"),
						Type:        sPtr("performance"),
					},
				},
				{
					ADR: &ADR{
						ID:    "ADR-1",
						Title: sPtr("Use Go"),
						Body: &ADRBody{
							Status: sPtr("accepted"),
						},
					},
				},
				{
					ElementDef: &LikeC4ElementDef{
						Definition: &LikeC4Definition{
							Kind:  "system",
							Name:  sPtr("sys"),
							Title: sPtr("System"),
							Body: &LikeC4ElementDefBody{
								Items: []*LikeC4BodyItem{
									{Description: sPtr("A system")},
									{Relation: &Relation{From: QualifiedIdent{Parts: []string{"sys"}}, To: QualifiedIdent{Parts: []string{"other"}}, Label: sPtr("calls")}},
								},
							},
						},
					},
				},
				{
					Scenario: &Scenario{
						ID:    "sc1",
						Steps: []*ScenarioStep{{FromParts: []string{"a"}, ToParts: []string{"b"}, Arrow: "->"}},
					},
				},
			},
		},
		Views: &LikeC4ViewsBlock{
			Items: []*LikeC4ViewsItem{
				{
					View: &LikeC4ViewDef{
						Name: sPtr("index"),
						Body: &LikeC4ViewBody{
							Items: []*LikeC4ViewItem{
								{Include: &IncludePredicate{Expressions: []ViewExpr{{Wildcard: true}}}},
								{Exclude: &ExcludePredicate{Expressions: []ViewExpr{{Selector: sPtr("sys")}}}},
								{Title: sPtr("Index View")},
							},
						},
					},
				},
			},
		},
	}

	p := NewPrinter()

	// Testing exposed Print method
	output := p.Print(program)

	// Verify key parts are present
	checks := []string{
		"import { A, B } from \"lib\"",
		"requirement REQ-1",
		"adr ADR-1",
		"system sys",
		"calls",
		"scenario sc1",
		"views {",
		"view index",
		"include *",
		"exclude sys",
	}

	for _, check := range checks {
		if !strings.Contains(output, check) {
			t.Errorf("Output missing %q", check)
		}
	}
}

func sPtr(s string) *string {
	return &s
}
