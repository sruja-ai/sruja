//go:build legacy

// pkg/language/parser_architecture_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_ArchitectureWrapper(t *testing.T) {
	dsl := `
architecture "Test" {
  system A "A"
}`
	p, err := language.NewParser()
	if err != nil {
		t.Fatalf("parser: %v", err)
	}
	prog, _, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	if prog.Architecture == nil {
		t.Fatalf("expected architecture to be parsed")
	}
	if len(prog.Architecture.Systems) != 1 {
		t.Fatalf("expected 1 system, got %d", len(prog.Architecture.Systems))
	}
}
