//go:build legacy

package engine_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestValidReferences_RequirementsImplements(t *testing.T) {
	dsl := `
architecture "Test" {
  model {
    system API "API" {
      container DB "DB"
    }
  }
  requirements {
    R1: functional "Must persist data" implements [API, DB]
    R2: constraint "Encrypted at rest" implements [DB]
  }
}`
	p, _ := language.NewParser()
	prog, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	errs := (&engine.ValidReferenceRule{}).Validate(prog)
	if len(errs) != 0 {
		t.Fatalf("expected 0 implement reference errors, got %d", len(errs))
	}
}

func TestValidReferences_RequirementsImplementsUnknown(t *testing.T) {
	dsl := `
architecture "Test" {
  model { system API "API" }
  requirements { R1: functional "X" implements [Unknown] }
}`
	p, _ := language.NewParser()
	prog, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	errs := (&engine.ValidReferenceRule{}).Validate(prog)
	if len(errs) == 0 {
		t.Fatalf("expected implement reference error, got none")
	}
}
