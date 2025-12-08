// pkg/engine/cycle_rule_test.go
package engine_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestCycleDetectionRule_EmptyArchitecture(t *testing.T) {
	program := &language.Program{
		Architecture: nil,
	}

	rule := &engine.CycleDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for nil architecture, got %d", len(errs))
	}
}

func TestCycleDetectionRule_SelfLoop(t *testing.T) {
	dsl := `
architecture "Test" {
    system A "System A" {}
    A -> A "Self reference"
}
`
	program := parse(t, dsl)

	rule := &engine.CycleDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected cycle error for self-loop")
	}
}

func TestCycleDetectionRule_SimpleCycle(t *testing.T) {
	dsl := `
architecture "Test" {
    system A "System A" {}
    system B "System B" {}
    A -> B "Uses"
    B -> A "Uses back"
}
`
	program := parse(t, dsl)

	rule := &engine.CycleDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected cycle error for A -> B -> A cycle")
	}
}

func TestCycleDetectionRule_ThreeNodeCycle(t *testing.T) {
	dsl := `
architecture "Test" {
    system A "System A" {}
    system B "System B" {}
    system C "System C" {}
    A -> B "Uses"
    B -> C "Uses"
    C -> A "Uses"
}
`
	program := parse(t, dsl)

	rule := &engine.CycleDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected cycle error for A -> B -> C -> A cycle")
	}
}

func TestCycleDetectionRule_NoCycle(t *testing.T) {
	dsl := `
architecture "Test" {
    system A "System A" {}
    system B "System B" {}
    system C "System C" {}
    A -> B "Uses"
    B -> C "Uses"
}
`
	program := parse(t, dsl)

	rule := &engine.CycleDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for acyclic graph, got %d", len(errs))
	}
}

func TestCycleDetectionRule_SystemLevelRelations(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        container Cont "Container" {}
        Sys -> Cont "Uses"
        Cont -> Sys "Uses back"
    }
}
`
	program := parse(t, dsl)

	rule := &engine.CycleDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected cycle error for system-level cycle")
	}
}

func TestCycleDetectionRule_ContainerLevelRelations(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        container Cont1 "Container 1" {}
        container Cont2 "Container 2" {}
        Cont1 -> Cont2 "Uses"
        Cont2 -> Cont1 "Uses back"
    }
}
`
	program := parse(t, dsl)

	rule := &engine.CycleDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected cycle error for container-level cycle")
	}
}

func TestCycleDetectionRule_ComponentLevelRelations(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        container Cont "Container" {
            component Comp1 "Component 1" {}
            component Comp2 "Component 2" {}
            Comp1 -> Comp2 "Uses"
            Comp2 -> Comp1 "Uses back"
        }
    }
}
`
	program := parse(t, dsl)

	rule := &engine.CycleDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected cycle error for component-level cycle")
	}
}

func TestCycleDetectionRule_EmptyFromTo(_ *testing.T) {
	program := &language.Program{
		Architecture: &language.Architecture{
			Relations: []*language.Relation{
				{From: language.QualifiedIdent{Parts: []string{}}, To: language.QualifiedIdent{Parts: []string{"B"}}},
				{From: language.QualifiedIdent{Parts: []string{"A"}}, To: language.QualifiedIdent{Parts: []string{}}},
			},
		},
	}

	rule := &engine.CycleDetectionRule{}
	errs := rule.Validate(program)
	// Should not panic, empty strings should be handled
	_ = errs
}
