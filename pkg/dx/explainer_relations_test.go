// pkg/dx/explainer_relations_test.go
package dx

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestFindRelations(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID: "Sys1",
					Relations: []*language.Relation{
						{
							From: language.QualifiedIdent{Parts: []string{"Sys1"}},
							To:   language.QualifiedIdent{Parts: []string{"Sys2"}},
							Verb: stringPtr("calls"),
						},
					},
				},
			},
			Relations: []*language.Relation{
				{
					From:  language.QualifiedIdent{Parts: []string{"Sys2"}},
					To:    language.QualifiedIdent{Parts: []string{"Sys1"}},
					Label: stringPtr("HTTP"),
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Sys1")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if len(explanation.Relations.Outgoing) == 0 {
		t.Error("Should find outgoing relations")
	}
	if len(explanation.Relations.Incoming) == 0 {
		t.Error("Should find incoming relations")
	}
}

func TestFindRelations_ContainerRelations(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID: "Sys",
					Containers: []*language.Container{
						{
							ID: "Cont",
							Relations: []*language.Relation{
								{
									From: language.QualifiedIdent{Parts: []string{"Cont"}},
									To:   language.QualifiedIdent{Parts: []string{"Other"}},
									Verb: stringPtr("calls"),
								},
							},
						},
					},
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Cont")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if len(explanation.Relations.Outgoing) == 0 {
		t.Error("Should find outgoing relations from container")
	}
}

func TestFindRelations_ComponentRelations(t *testing.T) {
	// findRelations only checks components directly in system, not inside containers
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID:    "Sys",
					Label: "System",
					Components: []*language.Component{
						{
							ID:    "Comp",
							Label: "Component",
							Relations: []*language.Relation{
								{
									From: language.QualifiedIdent{Parts: []string{"Comp"}},
									To:   language.QualifiedIdent{Parts: []string{"Other"}},
								},
							},
						},
					},
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Comp")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if len(explanation.Relations.Outgoing) == 0 {
		t.Error("Should find outgoing relations from component")
	}
}

func TestFindRelatedADRs(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			ADRs: []*language.ADR{
				{
					ID:    "ADR001",
					Title: stringPtr("Use API for authentication"),
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("API")
	if err != nil {
		// Element doesn't exist, but we can still test ADR finding
		// Let's create a system first
		prog.Architecture.Systems = []*language.System{
			{ID: "API", Label: "API"},
		}
		explanation, err = explainer.ExplainElement("API")
		if err != nil {
			t.Fatalf("ExplainElement failed: %v", err)
		}
	}
	if len(explanation.ADRs) == 0 {
		t.Error("Should find related ADRs")
	}
}

func TestFindDependencies(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID: "Sys1",
					Relations: []*language.Relation{
						{
							From: language.QualifiedIdent{Parts: []string{"Sys1"}},
							To:   language.QualifiedIdent{Parts: []string{"Sys2"}},
						},
						{
							From: language.QualifiedIdent{Parts: []string{"Sys1"}},
							To:   language.QualifiedIdent{Parts: []string{"Sys3"}},
						},
					},
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Sys1")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if len(explanation.Dependencies) != 2 {
		t.Errorf("Expected 2 dependencies, got %d", len(explanation.Dependencies))
	}
}

func TestExtractRelationInfo_WithBoth(t *testing.T) {
	rel := &language.Relation{
		From:  language.QualifiedIdent{Parts: []string{"A"}},
		To:    language.QualifiedIdent{Parts: []string{"B"}},
		Verb:  stringPtr("calls"),
		Label: stringPtr("HTTP"),
	}
	label, verb := extractRelationInfo(rel)
	if label != "HTTP" {
		t.Errorf("Expected label 'HTTP', got '%s'", label)
	}
	if verb != "calls" {
		t.Errorf("Expected verb 'calls', got '%s'", verb)
	}
}

func TestExtractRelationInfo_WithNeither(t *testing.T) {
	rel := &language.Relation{
		From: language.QualifiedIdent{Parts: []string{"A"}},
		To:   language.QualifiedIdent{Parts: []string{"B"}},
	}
	label, verb := extractRelationInfo(rel)
	if label != "" {
		t.Errorf("Expected empty label, got '%s'", label)
	}
	if verb != "" {
		t.Errorf("Expected empty verb, got '%s'", verb)
	}
}
