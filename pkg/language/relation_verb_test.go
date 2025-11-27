//go:build legacy

// pkg/language/relation_verb_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_RelationWithVerb(t *testing.T) {
	dsl := `
architecture "Test" {
  system User "User"
  system DB "Database"
  User -> DB reads "Reads data"
}`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("parser: %v", err)
	}
	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}

	if len(program.Architecture.Relations) < 1 {
		t.Fatalf("expected at least 1 relation")
	}
	rel := program.Architecture.Relations[0]
	if rel == nil {
		t.Fatalf("expected relation")
	}
	if rel.Verb == nil {
		t.Fatalf("expected verb to be present")
	}
	if *rel.Verb != "reads" {
		t.Fatalf("expected verb 'reads', got '%s'", *rel.Verb)
	}
}
