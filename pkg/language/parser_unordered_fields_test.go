package language

import (
	"testing"
)

func TestParser_UnorderedFields(t *testing.T) {
	// Input with out-of-order tags and status
	input := `model {
		adr ADR001 "Test" {
			tags ["tag1", "tag2"]
			status "Accepted"
			decision "We decided X"
			context "Context Y"
		}
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

	// Find ADR in Model
	var adr *ADR
	for _, item := range program.Model.Items {
		if item.ADR != nil {
			adr = item.ADR
			break
		}
	}
	if adr == nil {
		t.Fatal("Expected ADR")
	}
	if *adr.Body.Status != "Accepted" {
		t.Errorf("Expected Status 'Accepted', got %v", adr.Body.Status)
	}
	if len(adr.Body.Tags) != 2 {
		t.Errorf("Expected 2 tags, got %d", len(adr.Body.Tags))
	}
}
