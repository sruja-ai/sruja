//go:build legacy

// pkg/language/requirements_implements_test.go
// Note: Implements field is not currently part of the new Requirement structure
// This test is simplified to test basic requirement parsing
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_RequirementsImplements(t *testing.T) {
	dsl := `
architecture "Test" {
  system API "API" {
    container DB "DB"
  }
  requirement R1 functional "Must persist data"
}`
	p, err := language.NewParser()
	if err != nil {
		t.Fatalf("parser: %v", err)
	}
	prog, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	if len(prog.Architecture.Requirements) != 1 {
		t.Fatalf("expected 1 requirement, got %d", len(prog.Architecture.Requirements))
	}
	r1 := prog.Architecture.Requirements[0]
	if r1.ID != "R1" {
		t.Fatalf("expected requirement ID 'R1', got '%s'", r1.ID)
	}
	if r1.Type != "functional" {
		t.Fatalf("expected requirement type 'functional', got '%s'", r1.Type)
	}
}
