package views

import (
	"testing"
)

func TestExtractAllElements(t *testing.T) {
	dsl := `
		System = system "My System" {
			API = container "API" {
				Auth = component "Auth"
			}
			DB = database "Database"
		}
		User = person "User"
	`
	prog := parseDSL(t, dsl)
	elements := ExtractAllElements(prog)

	expected := map[string]string{
		"System":          "system",
		"System.API":      "container",
		"System.API.Auth": "component",
		"System.DB":       "datastore", // Kind mapping: database -> datastore
		"User":            "person",
	}

	if len(elements) != len(expected) {
		t.Errorf("Expected %d elements, got %d", len(expected), len(elements))
	}

	for _, elem := range elements {
		kind, ok := expected[elem.ID]
		if !ok {
			t.Errorf("Unexpected element ID: %s", elem.ID)
			continue
		}
		if elem.Kind != kind {
			t.Errorf("Element %s: expected kind %s, got %s", elem.ID, kind, elem.Kind)
		}
	}
}

func TestExtractRelationsFromModel(t *testing.T) {
	dsl := `
		A = system "A" {
			C1 = container "C1"
		}
		B = system "B"
		A.C1 -> B "Uses" [gRPC]
	`
	prog := parseDSL(t, dsl)
	relations := ExtractRelationsFromModel(prog)

	if len(relations) < 1 {
		t.Fatalf("Expected at least 1 relation, got %d", len(relations))
	}

	rel := relations[0]
	if rel.From != "A.C1" {
		t.Errorf("Expected From A.C1, got %s", rel.From)
	}
	if rel.To != "B" {
		t.Errorf("Expected To B, got %s", rel.To)
	}
	if rel.Label != "Uses [gRPC]" {
		t.Errorf("Expected Label 'Uses [gRPC]', got '%s'", rel.Label)
	}
}
