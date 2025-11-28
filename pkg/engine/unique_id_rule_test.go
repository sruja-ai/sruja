// pkg/engine/unique_id_rule_test.go
package engine_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestUniqueIDRule_EmptyArchitecture(t *testing.T) {
	program := &language.Program{
		Architecture: nil,
	}

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for nil architecture, got %d", len(errs))
	}
}

func TestUniqueIDRule_EmptyID(t *testing.T) {
	program := &language.Program{
		Architecture: &language.Architecture{
			Systems: []*language.System{
				{ID: ""},
			},
		},
	}

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for empty ID, got %d", len(errs))
	}
}

func TestUniqueIDRule_DuplicateSystems(t *testing.T) {
	dsl := `
architecture "Test" {
    system A "System A" {}
    system A "System B" {}
}
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
architecture "Test" {
    system Sys "System" {
        container Cont "Container" {
            component X "X" {}
            component X "X Duplicate" {}
        }
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
architecture "Test" {
    system Sys "System" {
        datastore DB "Database" {}
        datastore DB "Database Duplicate" {}
    }
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
architecture "Test" {
    system Sys "System" {
        queue Q "Queue" {}
        queue Q "Queue Duplicate" {}
    }
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
architecture "Test" {
    person User "User" {}
    person User "User Duplicate" {}
}
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
architecture "Test" {
    requirement R1 performance "Requirement 1"
    requirement R1 performance "Requirement 2"
}
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
architecture "Test" {
    adr ADR001 "ADR 1"
    adr ADR001 "ADR 2"
}
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
architecture "Test" {
    system X "System X" {}
    system Y "System Y" {
        container X "Container X" {}
    }
}
`
	program := parse(t, dsl)

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected duplicate ID error for cross-level duplicates")
	}
}
