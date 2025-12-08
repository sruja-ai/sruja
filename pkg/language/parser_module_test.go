//go:build legacy

// pkg/language/parser_module_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_ModuleMode_WithModelBlock(t *testing.T) {
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
	if len(prog.Architecture.Systems) == 0 {
		t.Fatalf("expected systems in architecture")
	}
}

func TestParser_ModuleMode_BareStatements(t *testing.T) {
	dsl := `
system A "A"
system B "B"
A -> B uses "Uses"
`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	if prog.Architecture == nil {
		t.Fatalf("expected auto-wrapped architecture")
	}
	if len(prog.Architecture.Systems) < 2 {
		t.Fatalf("expected at least 2 systems, got %d", len(prog.Architecture.Systems))
	}
	if len(prog.Architecture.Relations) < 1 {
		t.Fatalf("expected at least 1 relation, got %d", len(prog.Architecture.Relations))
	}
}
