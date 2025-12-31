package engine_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func parse2(t *testing.T, dsl string) *language.Program {
	p, err := language.NewParser()
	if err != nil {
		t.Fatalf("parser: %v", err)
	}
	program, _, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	return program
}

func TestValidReferences_NewElements(t *testing.T) {
	dsl := `
    App = system "App" {
      Store = database "DB"
      Bus = queue "Queue"
      Pay = container "Payments"
    }
    User = person "User"
    User -> Store "reads"
    App -> Bus "publishes"
    App -> Pay "depends"
`
	prog := parse2(t, dsl)
	errs := (&engine.ValidReferenceRule{}).Validate(prog)
	if len(errs) != 0 {
		t.Fatalf("expected 0 reference errors, got %d", len(errs))
	}
}

func TestOrphanDetection_NewElementsUsage(t *testing.T) {
	dsl := `
    App = system "App" {
      Store = database "DB"
    }
    User = person "User"
    User -> Store "reads"
`
	prog := parse2(t, dsl)
	errs := (&engine.OrphanDetectionRule{}).Validate(prog)
	for _, e := range errs {
		if e.Message == "Orphan element 'App' is defined but never used in any relation." {
			t.Fatalf("App should be marked used when Store is used")
		}
	}
}
