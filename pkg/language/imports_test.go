//go:build legacy

// pkg/language/imports_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_ImportsList(t *testing.T) {
	dsl := `
architecture "Test" {
  import "shared.sruja"
  import "other.sruja"
  system A "A"
}`

	p, err := language.NewParser()
	if err != nil {
		t.Fatalf("parser: %v", err)
	}
	prog, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}

	if prog.Architecture == nil {
		t.Fatalf("architecture nil")
	}
	if len(prog.Architecture.Imports) != 2 {
		t.Fatalf("expected 2 imports, got %d", len(prog.Architecture.Imports))
	}
	if prog.Architecture.Imports[0].Path != "shared.sruja" {
		t.Fatalf("unexpected import path: %s", prog.Architecture.Imports[0].Path)
	}
	if prog.Architecture.Imports[1].Path != "other.sruja" {
		t.Fatalf("unexpected import path: %s", prog.Architecture.Imports[1].Path)
	}
}
