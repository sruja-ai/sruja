package language

import (
	"strings"
	"testing"
)

func TestPrinter_Coverage(t *testing.T) {
	// Create a comprehensive program model using the new unified syntax structures
	program := &Program{
		Specification: &Specification{
			Items: []SpecificationItem{
				{Element: &ElementKindDef{Name: "component", Title: sPtr("Component")}},
				{Tag: &TagDef{Name: "mytag", Title: sPtr("My Tag")}},
			},
		},
		Model: &Model{
			Items: []ModelItem{
				{
					Import: &ImportStatement{
						Elements: []string{"A", "B"},
						From:     "lib",
					},
				},
				{
					// Using Assignment syntax for elements
					ElementDef: &ElementDef{
						Assignment: &ElementAssignment{
							Name:  "sys",
							Kind:  "system",
							Title: sPtr("System"),
							Body: &ElementDefBody{
								Items: []*BodyItem{
									{Description: sPtr("A system")},
									{Relation: &Relation{From: QualifiedIdent{Parts: []string{"sys"}}, To: QualifiedIdent{Parts: []string{"other"}}, Label: sPtr("calls")}},
								},
							},
						},
					},
				},
				{
					// ADR using new syntax
					ElementDef: &ElementDef{
						Assignment: &ElementAssignment{
							Name:  "ADR1",
							Kind:  "adr",
							Title: sPtr("Use Go"),
						},
					},
				},
				{
					// Scenario using new syntax
					ElementDef: &ElementDef{
						Assignment: &ElementAssignment{
							Name:  "sc1",
							Kind:  "scenario",
							Title: sPtr("User Login"),
						},
					},
				},
			},
		},
		Views: &Views{
			Items: []*ViewsItem{
				{
					View: &ViewDef{
						Name: sPtr("index"),
						Body: &ViewBody{
							Items: []*ViewItem{
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
		"sys = system",
		"calls",
		"view index",
		"include *",
		"exclude sys",
	}

	for _, check := range checks {
		if !strings.Contains(output, check) {
			t.Errorf("Output missing %q\nFull output:\n%s", check, output)
		}
	}
}

func sPtr(s string) *string {
	return &s
}
