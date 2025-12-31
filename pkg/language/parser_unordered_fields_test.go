package language

import (
	"testing"
)

func TestParser_UnorderedFields(t *testing.T) {
	// Input with new unified syntax for ADR
	input := `adr001 = adr "Test" {
			status "Accepted"
			decision "We decided X"
			context "Context Y"
		}`

	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, diags, err := parser.Parse("test.sruja", input)
	if err != nil {
		t.Fatalf("Failed to parse: %v", err)
	}
	if len(diags) > 0 {
		t.Fatalf("Parser returned diagnostics: %v", diags)
	}

	// Find ADR via ElementDef in Model
	var found bool
	for _, item := range program.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			a := item.ElementDef.Assignment
			if a.Kind == "adr" && a.Name == "adr001" {
				found = true
				if a.Title == nil || *a.Title != "Test" {
					t.Errorf("Expected Title 'Test', got %v", a.Title)
				}
				break
			}
		}
	}
	if !found {
		t.Fatal("Expected ADR element 'adr001' not found")
	}
}
