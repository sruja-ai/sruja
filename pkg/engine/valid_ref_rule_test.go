// pkg/engine/valid_ref_rule_test.go
package engine_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestValidReferenceRule_EmptyArchitecture(t *testing.T) {
	program := &language.Program{
		Architecture: nil,
	}

	rule := &engine.ValidReferenceRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for nil architecture, got %d", len(errs))
	}
}

func TestValidReferenceRule_NilRelation(_ *testing.T) {
	program := &language.Program{
		Architecture: &language.Architecture{
			Relations: []*language.Relation{nil},
		},
	}

	rule := &engine.ValidReferenceRule{}
	errs := rule.Validate(program)
	// Should not panic, may or may not return errors
	_ = errs
}

func TestValidReferenceRule_UndefinedFrom(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {}
    Undefined -> Sys "Uses"
}
`
	program := parse(t, dsl)

	rule := &engine.ValidReferenceRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected error for undefined 'From' reference")
	}
	found := false
	for _, e := range errs {
		if e.Message == "Reference to undefined element 'Undefined'" {
			found = true
			break
		}
	}
	if !found {
		t.Error("Expected error message for undefined 'From' element")
	}
}

func TestValidReferenceRule_UndefinedTo(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {}
    Sys -> Undefined "Uses"
}
`
	program := parse(t, dsl)

	rule := &engine.ValidReferenceRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected error for undefined 'To' reference")
	}
	found := false
	for _, e := range errs {
		if e.Message == "Reference to undefined element 'Undefined'" {
			found = true
			break
		}
	}
	if !found {
		t.Error("Expected error message for undefined 'To' element")
	}
}

func TestValidReferenceRule_SystemLevelRelations(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        container Cont "Container" {}
        Sys -> Cont "Uses"
    }
}
`
	program := parse(t, dsl)

	rule := &engine.ValidReferenceRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for valid system-level relations, got %d", len(errs))
	}
}

func TestValidReferenceRule_ContainerLevelRelations(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        container Cont "Container" {
            component Comp "Component" {}
            Cont -> Comp "Uses"
        }
    }
}
`
	program := parse(t, dsl)

	rule := &engine.ValidReferenceRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for valid container-level relations, got %d", len(errs))
	}
}

func TestValidReferenceRule_ComponentLevelRelations(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        container Cont "Container" {
            component Comp1 "Component 1" {}
            component Comp2 "Component 2" {}
            Comp1 -> Comp2 "Uses"
        }
    }
}
`
	program := parse(t, dsl)

	rule := &engine.ValidReferenceRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for valid component-level relations, got %d", len(errs))
	}
}
