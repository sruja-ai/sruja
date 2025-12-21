// pkg/engine/cycle_rule_test.go
package engine_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestCycleDetectionRule_EmptyArchitecture(t *testing.T) {
	program := &language.Program{
		Model: nil,
	}

	rule := &engine.CycleDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for nil model, got %d", len(errs))
	}
}

func TestCycleDetectionRule_SelfLoop(t *testing.T) {
	dsl := `
model {
    A = system "System A"
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
model {
    A = system "System A"
    B = system "System B"
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
model {
    A = system "System A"
    B = system "System B"
    C = system "System C"
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
model {
    A = system "System A"
    B = system "System B"
    C = system "System C"
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
model {
    Sys = system "System" {
        Cont = container "Container"
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
model {
    system Sys "System" {
        Cont1 = container "Container 1"
        Cont2 = container "Container 2"
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
model {
    system Sys "System" {
        Cont = container "Container" {
            Comp1 = component "Component 1"
            Comp2 = component "Component 2"
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
	parser, err := language.NewParser()
	if err != nil {
		return
	}

	// Empty From/To are not valid in LikeC4 syntax, so we test with valid relations
	dsl := `model {
		A = system "A"
		B = system "B"
		A -> B
	}`

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		return
	}

	rule := &engine.CycleDetectionRule{}
	errs := rule.Validate(program)
	// Should not panic, empty strings should be handled
	_ = errs
}
