//go:build legacy

package compiler_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/compiler"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestCompile_WithImportedModelMerged(t *testing.T) {
	shared := `
workspace {
  model { system Shared "Shared" }
}`

	main := `
workspace {
  import "shared.sruja"
  model {
    system App "App" { container API "API" }
    App -> Shared uses
    API -> Shared reads
  }
}`

	p, _ := language.NewParser()
	sharedProg, err := p.Parse("shared.sruja", shared)
	if err != nil {
		t.Fatalf("shared parse: %v", err)
	}
	mainProg, err := p.Parse("main.sruja", main)
	if err != nil {
		t.Fatalf("main parse: %v", err)
	}

	// Simulate CLI import merge
	if sharedProg.Architecture != nil && sharedProg.Architecture.Model != nil {
		if mainProg.Architecture.Model == nil {
			mainProg.Architecture.Model = &language.Model{Statements: []*language.ModelStatement{}}
		}
		mainProg.Architecture.Model.Statements = append(mainProg.Architecture.Model.Statements, sharedProg.Architecture.Model.Statements...)
	}

	c := compiler.NewMermaidCompiler()
	out, err := c.Compile(mainProg)
	if err != nil {
		t.Fatalf("compile: %v", err)
	}

	expects := []string{
		"System(App, \"App\")",
		"Container(API, \"API\")",
		"System(Shared, \"Shared\")",
		"Rel(App, Shared, \"Uses\")",
		"Rel(API, Shared, \"Reads\")",
	}
	for _, e := range expects {
		if !strings.Contains(out, e) {
			t.Fatalf("expected output to contain: %s\nGot:\n%s", e, out)
		}
	}
}
