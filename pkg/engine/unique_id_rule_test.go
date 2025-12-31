// pkg/engine/unique_id_rule_test.go
package engine_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestUniqueIDRule_EmptyArchitecture(t *testing.T) {
	program := &language.Program{
		Model: nil,
	}

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for nil model, got %d", len(errs))
	}
}

func TestUniqueIDRule_EmptyID(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	// Empty ID is not valid in Sruja syntax, so we test with a valid model
	dsl := `
		Sys = system "System"
	`

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	// Should have no errors for valid model
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for valid model, got %d", len(errs))
	}
}

func TestUniqueIDRule_DuplicateSystems(t *testing.T) {
	dsl := `
		A = system "System A"
		A = system "System B"
	`
	program := parse(t, dsl)

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected duplicate ID error for systems")
	}
}

func TestUniqueIDRule_DuplicateComponents(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Cont = container "Container" {
				X = component "X"
				X = component "X Duplicate"
			}
		}
	`
	program := parse(t, dsl)

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected duplicate ID error for components")
	}
}

func TestUniqueIDRule_DuplicateDataStores(t *testing.T) {
	dsl := `
		Sys = system "System" {
			DB = datastore "Database"
			DB = datastore "Database Duplicate"
		}
	`
	program := parse(t, dsl)

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected duplicate ID error for datastores")
	}
}

func TestUniqueIDRule_DuplicateQueues(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Q = queue "Queue"
			Q = queue "Queue Duplicate"
		}
	`
	program := parse(t, dsl)

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected duplicate ID error for queues")
	}
}

func TestUniqueIDRule_DuplicatePersons(t *testing.T) {
	dsl := `
		User = person "User"
		User = person "User Duplicate"
	`
	program := parse(t, dsl)

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected duplicate ID error for persons")
	}
}

func TestUniqueIDRule_DuplicateRequirements(t *testing.T) {
	dsl := `
		R1 = Requirement performance "Requirement 1"
		R1 = Requirement performance "Requirement 2"
	`
	program := parse(t, dsl)

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected duplicate ID error for requirements")
	}
}

func TestUniqueIDRule_DuplicateADRs(t *testing.T) {
	dsl := `
		ADR001 = Adr "ADR 1"
		ADR001 = Adr "ADR 2"
	`
	program := parse(t, dsl)

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected duplicate ID error for ADRs")
	}
}

func TestUniqueIDRule_CrossLevelDuplicates(t *testing.T) {
	dsl := `
		X = system "System X"
		Y = system "System Y" {
			X = container "Container X"
		}
	`
	program := parse(t, dsl)

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected duplicate ID error for cross-level duplicates")
	}
}
