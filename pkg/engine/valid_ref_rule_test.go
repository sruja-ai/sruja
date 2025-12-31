// pkg/engine/valid_ref_rule_test.go
package engine_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestValidReferenceRule_EmptyArchitecture(t *testing.T) {
	program := &language.Program{
		Model: nil,
	}

	rule := &engine.ValidReferenceRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for nil model, got %d", len(errs))
	}
}

func TestValidReferenceRule_NilRelation(_ *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		return
	}

	dsl := `
		Sys = system "System"
	`

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		return
	}

	rule := &engine.ValidReferenceRule{}
	errs := rule.Validate(program)
	// Should not panic, may or may not return errors
	_ = errs
}

func TestValidReferenceRule_UndefinedFrom(t *testing.T) {
	dsl := `
		Sys = system "System"
		Undefined -> Sys "Uses"
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
		Sys = system "System"
		Sys -> Undefined "Uses"
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
		Sys = system "System" {
			Cont = container "Container"
			Sys -> Cont "Uses"
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
		Sys = system "System" {
			Cont = container "Container" {
				Comp = component "Component"
				Cont -> Comp "Uses"
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
		Sys = system "System" {
			Cont = container "Container" {
				Comp1 = component "Component 1"
				Comp2 = component "Component 2"
				Comp1 -> Comp2 "Uses"
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
