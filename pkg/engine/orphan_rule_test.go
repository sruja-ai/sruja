// pkg/engine/orphan_rule_test.go
package engine_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestOrphanDetectionRule_EmptyArchitecture(t *testing.T) {
	program := &language.Program{
		Model: nil,
	}

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for nil model, got %d", len(errs))
	}
}

func TestOrphanDetectionRule_EmptySystems(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	dsl := ``

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for empty model, got %d", len(errs))
	}
}

func TestOrphanDetectionRule_OrphanSystem(t *testing.T) {
	dsl := `
		Orphan = system "Orphan System"
	`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected orphan error for unused system")
	}
	found := false
	for _, e := range errs {
		// Updated message format includes additional context
		if strings.Contains(e.Message, "Orphan element") && strings.Contains(e.Message, "never used") {
			found = true
			break
		}
	}
	if !found {
		t.Error("Expected orphan error message for 'Orphan' system")
	}
}

func TestOrphanDetectionRule_OrphanContainer(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Orphan = container "Orphan Container"
		}
	`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	foundOrphan := false
	for _, e := range errs {
		// Updated message format includes additional context
		if strings.Contains(e.Message, "Orphan element") && strings.Contains(e.Message, "never used") {
			foundOrphan = true
			break
		}
	}
	if !foundOrphan {
		t.Error("Expected orphan error for unused container")
	}
}

func TestOrphanDetectionRule_OrphanComponent(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Cont = container "Container" {
				Orphan = component "Orphan Component"
			}
		}
	`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	foundOrphan := false
	for _, e := range errs {
		// Updated message format includes additional context
		if strings.Contains(e.Message, "Orphan element") && strings.Contains(e.Message, "never used") {
			foundOrphan = true
			break
		}
	}
	if !foundOrphan {
		t.Error("Expected orphan error for unused component")
	}
}

func TestOrphanDetectionRule_OrphanDataStore(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Orphan = datastore "Orphan DB"
		}
	`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	foundOrphan := false
	for _, e := range errs {
		// Updated message format includes additional context
		if strings.Contains(e.Message, "Orphan element") && strings.Contains(e.Message, "never used") {
			foundOrphan = true
			break
		}
	}
	if !foundOrphan {
		t.Error("Expected orphan error for unused datastore")
	}
}

func TestOrphanDetectionRule_OrphanQueue(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Orphan = queue "Orphan Queue"
		}
	`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	foundOrphan := false
	for _, e := range errs {
		// Updated message format includes additional context
		if strings.Contains(e.Message, "Orphan element 'Orphan'") && strings.Contains(e.Message, "never used") {
			foundOrphan = true
			break
		}
	}
	if !foundOrphan {
		t.Error("Expected orphan error for unused queue")
	}
}

func TestOrphanDetectionRule_OrphanPerson(t *testing.T) {
	dsl := `
		Orphan = person "Orphan Person"
	`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	foundOrphan := false
	for _, e := range errs {
		// Updated message format includes additional context
		if strings.Contains(e.Message, "Orphan element") && strings.Contains(e.Message, "never used") {
			foundOrphan = true
			break
		}
	}
	if !foundOrphan {
		t.Error("Expected orphan error for unused person")
	}
}

func TestOrphanDetectionRule_SystemLevelRelations(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Cont = container "Container"
			Sys -> Cont "Uses"
		}
	`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	for _, e := range errs {
		if e.Message == "Orphan element 'Sys' is defined but never used in any relation." {
			t.Error("System should not be orphan when it has relations")
		}
	}
}

func TestOrphanDetectionRule_ContainerLevelRelations(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Cont = container "Container" {
				Comp = component "Component"
				Cont -> Comp "Uses"
			}
		}
	`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	for _, e := range errs {
		if e.Message == "Orphan element 'Cont' is defined but never used in any relation." {
			t.Error("Container should not be orphan when it has relations")
		}
	}
}

func TestOrphanDetectionRule_ComponentLevelRelations(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Cont = container "Container" {
				Comp1 = component "Component 1"
				Comp2 = component "Component 2"
				Comp1 -> Comp2 "Uses"
			}
		}
	`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	for _, e := range errs {
		if e.Message == "Orphan element 'Comp1' is defined but never used in any relation." {
			t.Error("Component should not be orphan when it has relations")
		}
	}
}
