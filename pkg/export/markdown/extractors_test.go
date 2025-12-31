package markdown

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExtractSystemsFromModel_Nil(t *testing.T) {
	if extractSystemsFromModel(nil) != nil {
		t.Error("expected nil for nil program")
	}
	if extractSystemsFromModel(&language.Program{}) != nil {
		t.Error("expected nil for empty program")
	}
}

func TestExtractPersonsFromModel_Nil(t *testing.T) {
	if extractPersonsFromModel(nil) != nil {
		t.Error("expected nil for nil program")
	}
}

func TestExtractPersonsFromModel(t *testing.T) {
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind:  "person",
							Name:  "User1",
							Title: mkStrMD("User One"),
						},
					},
				},
			},
		},
	}

	persons := extractPersonsFromModel(prog)
	if len(persons) != 1 {
		t.Errorf("expected 1 person, got %d", len(persons))
	}
	if persons[0].ID != "User1" {
		t.Errorf("expected ID 'User1', got %s", persons[0].ID)
	}
}

func TestExtractSystemFromElement(t *testing.T) {
	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind:  "system",
			Name:  "Sys1",
			Title: mkStrMD("System One"),
		},
	}
	sys := extractSystemFromElement(elem)
	if sys == nil || sys.ID != "Sys1" {
		t.Error("expected system with ID 'Sys1'")
	}
}

func TestExtractSystemFromElement_Nil(t *testing.T) {
	if extractSystemFromElement(nil) != nil {
		t.Error("expected nil")
	}

	// Wrong kind
	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind: "container",
			Name: "Cont",
		},
	}
	if extractSystemFromElement(elem) != nil {
		t.Error("expected nil for non-system")
	}
}

func TestExtractContainerFromElement(t *testing.T) {
	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind:  "container",
			Name:  "Cont1",
			Title: mkStrMD("Container One"),
		},
	}
	cont := extractContainerFromElement(elem)
	if cont == nil || cont.ID != "Cont1" {
		t.Error("expected container with ID 'Cont1'")
	}
}

func TestExtractContainerFromElement_Nil(t *testing.T) {
	if extractContainerFromElement(nil) != nil {
		t.Error("expected nil")
	}
}

func TestExtractComponentFromElement(t *testing.T) {
	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind:  "component",
			Name:  "Comp1",
			Title: mkStrMD("Component One"),
		},
	}
	comp := extractComponentFromElement(elem)
	if comp == nil || comp.ID != "Comp1" {
		t.Error("expected component with ID 'Comp1'")
	}
}

func TestExtractComponentFromElement_Nil(t *testing.T) {
	if extractComponentFromElement(nil) != nil {
		t.Error("expected nil")
	}

	// Wrong kind
	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind: "system",
			Name: "Sys",
		},
	}
	if extractComponentFromElement(elem) != nil {
		t.Error("expected nil for non-component")
	}
}

func TestExtractDataStoreFromElement(t *testing.T) {
	// Test with 'database' kind
	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind:  "database",
			Name:  "DB1",
			Title: mkStrMD("Database One"),
		},
	}
	ds := extractDataStoreFromElement(elem)
	if ds == nil || ds.ID != "DB1" {
		t.Error("expected datastore with ID 'DB1'")
	}
}

func TestExtractDataStoreFromElement_Nil(t *testing.T) {
	if extractDataStoreFromElement(nil) != nil {
		t.Error("expected nil")
	}
}

func TestExtractQueueFromElement(t *testing.T) {
	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind:  "queue",
			Name:  "Q1",
			Title: mkStrMD("Queue One"),
		},
	}
	q := extractQueueFromElement(elem)
	if q == nil || q.ID != "Q1" {
		t.Error("expected queue with ID 'Q1'")
	}
}

func TestExtractQueueFromElement_Nil(t *testing.T) {
	if extractQueueFromElement(nil) != nil {
		t.Error("expected nil")
	}

	// Wrong kind
	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind: "container",
			Name: "Cont",
		},
	}
	if extractQueueFromElement(elem) != nil {
		t.Error("expected nil for non-queue")
	}
}

func TestExtractRequirementsFromModel_Nil(t *testing.T) {
	if extractRequirementsFromModel(nil) != nil {
		t.Error("expected nil")
	}
}

func TestExtractRequirementsFromModel(t *testing.T) {
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind:  "requirement",
							Name:  "REQ001",
							Title: mkStrMD("First Requirement"),
						},
					},
				},
			},
		},
	}

	reqs := extractRequirementsFromModel(prog)
	if len(reqs) != 1 {
		t.Errorf("expected 1 requirement, got %d", len(reqs))
	}
}

func TestExtractADRsFromModel_Nil(t *testing.T) {
	if extractADRsFromModel(nil) != nil {
		t.Error("expected nil")
	}
}

func TestExtractADRsFromModel(t *testing.T) {
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind:  "adr",
							Name:  "ADR001",
							Title: mkStrMD("First ADR"),
						},
					},
				},
			},
		},
	}

	adrs := extractADRsFromModel(prog)
	if len(adrs) != 1 {
		t.Errorf("expected 1 ADR, got %d", len(adrs))
	}
}

func TestExtractRelationsFromModel_Nil(t *testing.T) {
	if extractRelationsFromModel(nil) != nil {
		t.Error("expected nil")
	}
}

func TestGetString(t *testing.T) {
	if getString(nil) != "" {
		t.Error("expected empty string for nil")
	}
	s := "test"
	if getString(&s) != "test" {
		t.Error("expected 'test'")
	}
}

func TestSanitizeIDForMermaid(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"simple", "simple"},
		{"with-dash", "with_dash"},
		{"with.dot", "with_dot"},
	}

	for _, tt := range tests {
		got := sanitizeIDForMermaid(tt.input)
		if got != tt.expected {
			t.Errorf("sanitizeIDForMermaid(%q) = %q, want %q", tt.input, got, tt.expected)
		}
	}
}

func TestEscapeQuotesForMermaid(t *testing.T) {
	if escapeQuotesForMermaid(`he said "hi"`) != `he said #quot;hi#quot;` {
		t.Error("expected quotes escaped")
	}
}

func TestPrioritizeRelationships(t *testing.T) {
	label := "uses"
	relations := []*language.Relation{
		{From: language.QualifiedIdent{Parts: []string{"A"}}, To: language.QualifiedIdent{Parts: []string{"B"}}},
		{From: language.QualifiedIdent{Parts: []string{"C"}}, To: language.QualifiedIdent{Parts: []string{"D"}}, Label: &label},
	}

	sorted := prioritizeRelationships(relations)
	if len(sorted) != 2 {
		t.Errorf("expected 2 relations, got %d", len(sorted))
	}
	// The one with label should be first (higher score)
	if sorted[0].Label == nil {
		t.Error("expected relation with label to be first")
	}
}

func TestRelationshipScore(t *testing.T) {
	// Simple relation
	rel := &language.Relation{
		From: language.QualifiedIdent{Parts: []string{"A"}},
		To:   language.QualifiedIdent{Parts: []string{"B"}},
	}
	score := relationshipScore(rel)
	if score < 0 {
		t.Error("expected non-negative score")
	}

	// Relation with label should have higher score
	label := "uses"
	relWithLabel := &language.Relation{
		From:  language.QualifiedIdent{Parts: []string{"A"}},
		To:    language.QualifiedIdent{Parts: []string{"B"}},
		Label: &label,
	}
	scoreWithLabel := relationshipScore(relWithLabel)
	if scoreWithLabel <= score {
		t.Error("expected relation with label to have higher score")
	}
}

func mkStrMD(s string) *string {
	return &s
}
