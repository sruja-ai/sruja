//go:build legacy

// pkg/language/person_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_PersonAndRelations(t *testing.T) {
	dsl := `
architecture "Test" {
  person Customer "Customer"
  system Order "Order System" {
    container API "Order API" {
      component Checkout "Checkout"
    }
  }
  Customer -> API "Uses"
  API -> Checkout "Calls"
}`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	if program.Architecture == nil {
		t.Fatalf("Expected architecture to be parsed")
	}

	// Expect: 1 person, 1 system, 2 relations
	if len(program.Architecture.Persons) != 1 {
		t.Fatalf("Expected 1 person, got %d", len(program.Architecture.Persons))
	}
	if len(program.Architecture.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(program.Architecture.Systems))
	}
	if len(program.Architecture.Relations) != 2 {
		t.Fatalf("Expected 2 relations, got %d", len(program.Architecture.Relations))
	}

	// Verify person
	if program.Architecture.Persons[0].ID != "Customer" {
		t.Errorf("Expected person ID 'Customer', got '%s'", program.Architecture.Persons[0].ID)
	}
}
