//go:build legacy

// pkg/language/parser_adr_test.go
// Package language_test provides tests for ADR parsing.
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_ADRs(t *testing.T) {
	dsl := `
architecture "Test" {
    system API "API Service"
    adr ADR001 "Use microservices architecture for scalability"
    adr ADR002 "Use PostgreSQL for data persistence"
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse ADRs: %v", err)
	}

	if len(program.Architecture.ADRs) != 2 {
		t.Fatalf("Expected 2 ADRs, got %d", len(program.Architecture.ADRs))
	}

	adr1 := program.Architecture.ADRs[0]
	if adr1.ID != "ADR001" {
		t.Errorf("Expected ADR ID 'ADR001', got '%s'", adr1.ID)
	}
	if adr1.Title != "Use microservices architecture for scalability" {
		t.Errorf("Expected title 'Use microservices architecture for scalability', got '%s'", adr1.Title)
	}
}
