// pkg/engine/validator_test.go
package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestNewValidator(t *testing.T) {
	v := NewValidator()
	if v == nil {
		t.Fatal("NewValidator should not return nil")
	}
	if len(v.Rules) != 0 {
		t.Error("New validator should have no rules")
	}
}

func TestValidator_RegisterRule(t *testing.T) {
	v := NewValidator()
	rule := &UniqueIDRule{}
	v.RegisterRule(rule)
	if len(v.Rules) != 1 {
		t.Errorf("Expected 1 rule, got %d", len(v.Rules))
	}
}

func TestValidator_Validate(t *testing.T) {
	v := NewValidator()
	rule := &UniqueIDRule{}
	v.RegisterRule(rule)

	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{ID: "Sys1", Label: "System 1"},
				{ID: "Sys2", Label: "System 2"},
			},
		},
	}

	errs := v.Validate(prog)
	// Should have no errors for unique IDs
	if len(errs) != 0 {
		t.Errorf("Expected no errors, got %d", len(errs))
	}
}

func TestValidator_Validate_MultipleRules(_ *testing.T) {
	v := NewValidator()
	v.RegisterRule(&UniqueIDRule{})
	v.RegisterRule(&ValidReferenceRule{})

	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{ID: "Sys1", Label: "System 1"},
			},
		},
	}

	errs := v.Validate(prog)
	// Should run all rules
	_ = errs
}

func TestCycleDetectionRule_Name(t *testing.T) {
	rule := &CycleDetectionRule{}
	if rule.Name() != "CycleDetection" {
		t.Errorf("Expected 'CycleDetection', got '%s'", rule.Name())
	}
}

func TestCycleDetectionRule_Validate_NoCycle(t *testing.T) {
	rule := &CycleDetectionRule{}
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Relations: []*language.Relation{
				{From: language.QualifiedIdent{Parts: []string{"A"}}, To: language.QualifiedIdent{Parts: []string{"B"}}},
				{From: language.QualifiedIdent{Parts: []string{"B"}}, To: language.QualifiedIdent{Parts: []string{"C"}}},
			},
		},
	}

	errs := rule.Validate(prog)
	if len(errs) != 0 {
		t.Errorf("Expected no errors, got %d", len(errs))
	}
}

func TestCycleDetectionRule_Validate_Cycle(t *testing.T) {
	rule := &CycleDetectionRule{}
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Relations: []*language.Relation{
				{From: language.QualifiedIdent{Parts: []string{"A"}}, To: language.QualifiedIdent{Parts: []string{"B"}}},
				{From: language.QualifiedIdent{Parts: []string{"B"}}, To: language.QualifiedIdent{Parts: []string{"A"}}},
			},
		},
	}

	errs := rule.Validate(prog)
	if len(errs) == 0 {
		t.Error("Expected cycle detection error")
	}
}

func TestCycleDetectionRule_Validate_NilArchitecture(t *testing.T) {
	rule := &CycleDetectionRule{}
	prog := &language.Program{}

	errs := rule.Validate(prog)
	if len(errs) != 0 {
		t.Errorf("Expected no errors for nil architecture, got %d", len(errs))
	}
}

func TestCycleDetectionRule_Validate_SystemRelations(t *testing.T) {
	rule := &CycleDetectionRule{}
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID: "Sys",
					Relations: []*language.Relation{
						{From: language.QualifiedIdent{Parts: []string{"A"}}, To: language.QualifiedIdent{Parts: []string{"B"}}},
						{From: language.QualifiedIdent{Parts: []string{"B"}}, To: language.QualifiedIdent{Parts: []string{"A"}}},
					},
				},
			},
		},
	}

	errs := rule.Validate(prog)
	if len(errs) == 0 {
		t.Error("Expected cycle detection error in system relations")
	}
}

func TestUniqueIDRule_Name(t *testing.T) {
	rule := &UniqueIDRule{}
	if rule.Name() == "" {
		t.Error("Name() should return rule name")
	}
}

func TestValidReferenceRule_Name(t *testing.T) {
	rule := &ValidReferenceRule{}
	if rule.Name() == "" {
		t.Error("Name() should return rule name")
	}
}

func TestOrphanDetectionRule_Name(t *testing.T) {
	rule := &OrphanDetectionRule{}
	if rule.Name() == "" {
		t.Error("Name() should return rule name")
	}
}
