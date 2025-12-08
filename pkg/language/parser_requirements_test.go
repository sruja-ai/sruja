//go:build legacy

// pkg/language/parser_requirements_test.go
// Package language_test provides tests for requirements parsing.
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_Requirements(t *testing.T) {
	dsl := `
architecture "Test" {
    system API "API Service"
    requirement R1 functional "Must handle 10k concurrent users"
    requirement R2 constraint "Must use PostgreSQL"
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse requirements: %v", err)
	}

	if len(program.Architecture.Requirements) != 2 {
		t.Fatalf("Expected 2 requirements, got %d", len(program.Architecture.Requirements))
	}

	r1 := program.Architecture.Requirements[0]
	if r1.ID != "R1" {
		t.Errorf("Expected requirement ID 'R1', got '%s'", r1.ID)
	}
	if r1.Type != "functional" {
		t.Errorf("Expected requirement type 'functional', got '%s'", r1.Type)
	}
	if r1.Description != "Must handle 10k concurrent users" {
		t.Errorf("Expected description 'Must handle 10k concurrent users', got '%s'", r1.Description)
	}

	r2 := program.Architecture.Requirements[1]
	if r2.Type != "constraint" {
		t.Errorf("Expected requirement type 'constraint', got '%s'", r2.Type)
	}
}
