//go:build legacy

package compiler_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/compiler"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestTransformer_TagsPropagation(t *testing.T) {
	dsl := `
workspace {
  model {
    system App "Application" {
      tags ["core", "backend"]
      container API "API" { tags ["http", "public"] }
      datastore DB "PostgreSQL" { tags ["storage"] }
      queue MQ "Events" { tags ["async"] }
      external Pay "Stripe" { tags ["third-party"] }
    }
  }
}`
	p, _ := language.NewParser()
	prog, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	tr := compiler.NewTransformer()
	m, err := tr.Transform(prog)
	if err != nil {
		t.Fatalf("transform: %v", err)
	}
	found := 0
	for _, e := range m.Architecture.Elements {
		if e.ID == "App" && len(e.Tags) == 2 {
			found++
		}
		if e.ID == "API" && len(e.Tags) == 2 {
			found++
		}
		if e.ID == "DB" && len(e.Tags) == 1 {
			found++
		}
		if e.ID == "MQ" && len(e.Tags) == 1 {
			found++
		}
		if e.ID == "Pay" && len(e.Tags) == 1 {
			found++
		}
	}
	if found != 5 {
		t.Fatalf("expected tags on 5 elements, got %d", found)
	}
}
