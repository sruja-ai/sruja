//go:build legacy

package engine_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExternalBestPractice_InvalidDirection(t *testing.T) {
	dsl := `
architecture "Test" {
  model {
    system App "App" {
      container API "API"
      external Pay "Payments"
    }
    Pay -> API depends
  }
}`
	p, _ := language.NewParser()
	prog, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	rule := &engine.ExternalBestPracticeRule{}
	errs := rule.Validate(prog)
	if len(errs) == 0 {
		t.Fatalf("expected best-practice violation, got none")
	}
}

func TestExternalBestPractice_ValidDirection(t *testing.T) {
	dsl := `
architecture "Test" {
  model {
    system App "App" {
      container API "API"
      external Pay "Payments"
    }
    App -> Pay depends
  }
}`
	p, _ := language.NewParser()
	prog, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	rule := &engine.ExternalBestPracticeRule{}
	errs := rule.Validate(prog)
	if len(errs) != 0 {
		t.Fatalf("expected no violations, got %d", len(errs))
	}
}
